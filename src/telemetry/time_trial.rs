use super::{LapData, LapHistory, LapTiming, state::FAST_TTL};
use std::time::Instant;

const MAX_SAMPLES: usize = 65_536;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeTrialReading {
    pub current_ms: u32,
    pub invalid: bool,
    pub sectors_ms: [Option<u32>; 3],
    pub sector_deltas_ms: [Option<i64>; 3],
    pub delta_ms: Option<i64>,
    pub reference_ms: Option<u32>,
    pub session_best_ms: Option<u32>,
}

#[derive(Clone, Copy)]
struct Sample {
    distance_m: f32,
    millis: u32,
}

#[derive(Clone, Default)]
struct RecordedLap {
    number: u8,
    samples: Vec<Sample>,
    complete: bool,
    invalid: bool,
    total_ms: u32,
    sectors_ms: [Option<u32>; 3],
}

impl RecordedLap {
    fn time_at(&self, distance: f32) -> Option<i64> {
        let upper = self.samples.partition_point(|p| p.distance_m < distance);
        let right = self.samples.get(upper)?;
        if right.distance_m == distance {
            return Some(i64::from(right.millis));
        }
        let left = self.samples.get(upper.checked_sub(1)?)?;
        let fraction =
            f64::from(distance - left.distance_m) / f64::from(right.distance_m - left.distance_m);
        Some(
            (f64::from(left.millis) + fraction * f64::from(right.millis - left.millis)).round()
                as i64,
        )
    }
}

#[derive(Default)]
pub(crate) struct TimeTrialTracker {
    current: RecordedLap,
    best: Option<RecordedLap>,
    previous_best: Option<RecordedLap>,
    reference: Option<RecordedLap>,
    session_best_ms: Option<u32>,
    previous: Option<(u8, LapTiming, Instant)>,
}

impl TimeTrialTracker {
    pub fn interrupt(&mut self) {
        // TT can keep the UDP lap number fixed; preserve its history index across interruptions.
        self.current = RecordedLap {
            number: self.current.number,
            ..RecordedLap::default()
        };
        self.reference = None;
        self.previous = None;
    }

    pub fn history(&mut self, history: LapHistory) {
        let history_best = [history.best_before_last_ms, history.last_lap_ms]
            .into_iter()
            .flatten()
            .min();
        // History can invalidate a finish whose final invalidation packet was lost.
        let contradicted = |lap: &RecordedLap| {
            lap.number <= history.completed_laps
                && ((lap.number == history.completed_laps
                    && history.last_lap_ms != Some(lap.total_ms))
                    || history_best.is_none_or(|best| lap.total_ms < best))
        };
        if self.best.as_ref().is_some_and(contradicted) {
            self.best = self.previous_best.take().filter(|lap| !contradicted(lap));
        }
        if self.reference.as_ref().is_some_and(contradicted) {
            // Do not substitute another lap in the middle of the current lap.
            self.reference = None;
        }
        self.session_best_ms = [history_best, self.best.as_ref().map(|lap| lap.total_ms)]
            .into_iter()
            .flatten()
            .min();
    }

    pub fn update(
        &mut self,
        lap: LapData,
        track_length_m: u16,
        now: Instant,
    ) -> Option<TimeTrialReading> {
        let Some(timing) = lap.timing else {
            self.interrupt();
            return None;
        };
        if lap.current_lap == 0 || lap.result_status != 2 || !(1..=4).contains(&lap.driver_status) {
            self.interrupt();
            return None;
        }
        if self
            .previous
            .is_some_and(|(_, _, received)| now.saturating_duration_since(received) > FAST_TTL)
        {
            self.interrupt();
        }
        let crossed_line = self.previous.is_some_and(|(number, old, _)| {
            old.sector == 2
                && timing.sector == 0
                && timing.current_ms < old.current_ms
                && timing.distance_m < old.distance_m
                && timing.last_ms >= old.current_ms
                && (lap.current_lap == number || lap.current_lap == number.saturating_add(1))
        });
        let started = self.previous.is_some_and(|(_, old, _)| {
            old.distance_m < 0.0 && timing.distance_m >= 0.0 && timing.sector == 0
        });
        let new_lap = self.previous.is_none_or(|(number, old, _)| {
            number != lap.current_lap
                || timing.current_ms < old.current_ms
                || started
                || crossed_line
        });
        if new_lap {
            let number = if crossed_line {
                self.current.number.saturating_add(1).max(lap.current_lap)
            } else {
                self.current.number.max(lap.current_lap)
            };
            if crossed_line {
                self.finish(timing.last_ms, track_length_m);
            }
            self.current = RecordedLap {
                number,
                complete: crossed_line
                    || started
                    || (timing.current_ms == 0 && timing.distance_m == 0.0),
                ..RecordedLap::default()
            };
            if self.current.complete {
                // Crossing the timing line defines distance zero and elapsed time zero.
                self.current.samples.push(Sample {
                    distance_m: 0.0,
                    millis: 0,
                });
            }
            // Freeze the identity and trace until the next lap, including late history updates.
            self.reference = self.best.clone().filter(|best| {
                self.session_best_ms
                    .is_none_or(|time| best.total_ms <= time)
            });
        } else if self.previous.is_some_and(|(_, old, _)| {
            timing.distance_m < old.distance_m || timing.sector < old.sector
        }) {
            self.current.complete = false;
        }
        self.current.invalid |= timing.invalid;
        self.current.sectors_ms[..2].copy_from_slice(&timing.sectors_ms);
        if timing.distance_m >= 0.0 {
            if self.current.samples.len() >= MAX_SAMPLES {
                self.current.complete = false;
            } else if self.current.samples.last().is_none_or(|last| {
                timing.distance_m >= last.distance_m
                    && timing.current_ms >= last.millis
                    && (timing.distance_m != last.distance_m || timing.current_ms != last.millis)
            }) {
                self.current.samples.push(Sample {
                    distance_m: timing.distance_m,
                    millis: timing.current_ms,
                });
            }
        }
        self.previous = Some((lap.current_lap, timing, now));
        let sectors_ms = live_sectors(timing);
        let reference = self.reference.as_ref();
        Some(TimeTrialReading {
            current_ms: timing.current_ms,
            invalid: self.current.invalid,
            sectors_ms,
            sector_deltas_ms: std::array::from_fn(|i| {
                // Only compare completed splits, not an unfinished sector with a full sector.
                if i >= usize::from(timing.sector) {
                    return None;
                }
                Some(i64::from(sectors_ms[i]?) - i64::from(reference?.sectors_ms[i]?))
            }),
            delta_ms: reference
                .and_then(|lap| lap.time_at(timing.distance_m))
                .map(|time| i64::from(timing.current_ms) - time),
            reference_ms: reference.map(|lap| lap.total_ms),
            session_best_ms: self.session_best_ms,
        })
    }

    fn finish(&mut self, total_ms: u32, track_length_m: u16) {
        if !self.current.complete || self.current.invalid || total_ms == 0 || track_length_m == 0 {
            return;
        }
        let Some(last) = self.current.samples.last() else {
            return;
        };
        if last.distance_m > f32::from(track_length_m) || last.millis > total_ms {
            return;
        }
        let [Some(s1), Some(s2), _] = self.current.sectors_ms else {
            return;
        };
        let Some(s3) = total_ms.checked_sub(s1 + s2).filter(|&time| time > 0) else {
            return;
        };
        self.current.samples.push(Sample {
            distance_m: f32::from(track_length_m),
            millis: total_ms,
        });
        self.current.total_ms = total_ms;
        self.current.sectors_ms[2] = Some(s3);
        self.session_best_ms = Some(
            self.session_best_ms
                .map_or(total_ms, |old| old.min(total_ms)),
        );
        if self
            .best
            .as_ref()
            .is_none_or(|best| total_ms < best.total_ms)
        {
            self.previous_best = self.best.take();
            self.best = Some(std::mem::take(&mut self.current));
        }
    }
}

fn live_sectors(timing: LapTiming) -> [Option<u32>; 3] {
    let [s1, s2] = timing.sectors_ms;
    match timing.sector {
        0 => [Some(timing.current_ms), None, None],
        1 => [
            s1,
            s1.and_then(|s1| timing.current_ms.checked_sub(s1)),
            None,
        ],
        2 => [
            s1,
            s2,
            s1.zip(s2)
                .and_then(|(s1, s2)| timing.current_ms.checked_sub(s1 + s2)),
        ],
        _ => [None; 3],
    }
}
