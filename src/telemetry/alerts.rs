use super::{
    Header, RaceControl, Snapshot, Timed,
    state::{DAMAGE_TTL, FAST_TTL, serial_newer},
};
use std::time::{Duration, Instant};

pub const ALERT_TTL: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaceAlert {
    Flag(u8),
    SafetyCar(u8),
    TimePenalty(u8),
    DriveThrough,
    StopGo,
    Warning(u8),
    RedFlag,
    ChequeredFlag,
}

impl RaceAlert {
    pub fn code(self) -> u8 {
        match self {
            Self::Flag(1) => 1,
            Self::Flag(2) => 2,
            Self::Flag(3) => 3,
            Self::SafetyCar(1) => 4,
            Self::SafetyCar(2) => 5,
            Self::TimePenalty(_) => 6,
            Self::DriveThrough => 7,
            Self::StopGo => 8,
            Self::Warning(_) => 9,
            Self::RedFlag => 10,
            Self::ChequeredFlag => 11,
            _ => 0,
        }
    }

    fn priority(self) -> u8 {
        match self {
            Self::RedFlag => 100,
            Self::SafetyCar(1 | 2) => 90,
            Self::Flag(3) => 80,
            Self::DriveThrough | Self::StopGo | Self::TimePenalty(_) => 70,
            Self::Warning(_) => 60,
            Self::Flag(2) => 50,
            Self::ChequeredFlag => 40,
            Self::Flag(1) => 30,
            _ => 0,
        }
    }
}

impl Snapshot {
    pub fn race_alert(&self, now: Instant) -> Option<Timed<RaceAlert>> {
        if !self.on_track(now) {
            return None;
        }
        self.race_alerts
            .iter()
            .flatten()
            .copied()
            .filter(|alert| {
                alert.value.code() != 0
                    && alert.fresh(now, ALERT_TTL)
                    && match alert.value {
                        RaceAlert::Flag(flag) => self.status.is_some_and(|s| {
                            s.fresh(now, FAST_TTL) && s.value.fia_flag == Some(flag)
                        }),
                        RaceAlert::SafetyCar(mode) => self.session.is_some_and(|s| {
                            s.fresh(now, DAMAGE_TTL) && s.value.safety_car == Some(mode)
                        }),
                        _ => true,
                    }
            })
            .max_by_key(|alert| (alert.value.priority(), alert.received))
    }
}

type Slots = [Option<Timed<RaceAlert>>; 5];

#[derive(Default)]
pub(super) struct AlertTracker {
    flag: Option<(Option<u8>, Instant)>,
    safety: Option<(Option<u8>, Instant)>,
    control: Option<(RaceControl, Instant)>,
    events: [Option<u32>; 2],
}

fn offer(slots: &mut Slots, slot: usize, value: RaceAlert, now: Instant, frame: u32) {
    if slots[slot]
        .is_none_or(|old| !old.fresh(now, ALERT_TTL) || value.priority() >= old.value.priority())
    {
        slots[slot] = Some(Timed {
            value,
            received: now,
            overall_frame: frame,
        });
    }
}

impl AlertTracker {
    pub fn inactive(&mut self, slots: &mut Slots) {
        slots.fill(None);
        self.flag = None;
        self.safety = None;
        self.control = None;
    }

    pub fn flag(&mut self, flag: Option<u8>, now: Instant, frame: u32, slots: &mut Slots) {
        if self
            .flag
            .is_none_or(|(old, at)| old != flag || now.saturating_duration_since(at) > FAST_TTL)
        {
            slots[0] = None;
            if let Some(flag @ 1..=3) = flag {
                offer(slots, 0, RaceAlert::Flag(flag), now, frame);
            }
        }
        self.flag = Some((flag, now));
        if flag == Some(1)
            && slots[4].is_some_and(|a| {
                a.value == RaceAlert::RedFlag && serial_newer(frame, a.overall_frame)
            })
        {
            slots[4] = None;
        }
    }

    pub fn safety(&mut self, safety: Option<u8>, now: Instant, frame: u32, slots: &mut Slots) {
        if self
            .safety
            .is_none_or(|(old, at)| old != safety || now.saturating_duration_since(at) > DAMAGE_TTL)
        {
            slots[1] = None;
            if let Some(mode @ 1..=2) = safety {
                offer(slots, 1, RaceAlert::SafetyCar(mode), now, frame);
            }
        }
        self.safety = Some((safety, now));
    }

    pub fn control(&mut self, control: RaceControl, now: Instant, frame: u32, slots: &mut Slots) {
        let previous = self.control.replace((control, now));
        let Some((old, _)) =
            previous.filter(|(_, at)| now.saturating_duration_since(*at) <= FAST_TTL)
        else {
            slots[2] = None;
            slots[3] = None;
            return;
        };
        if control.penalty_seconds < old.penalty_seconds
            || control.drive_through < old.drive_through
            || control.stop_go < old.stop_go
            || control.warnings < old.warnings
        {
            slots[2] = None;
            slots[3] = None;
        }
        if control.drive_through > old.drive_through {
            offer(slots, 2, RaceAlert::DriveThrough, now, frame);
        } else if control.stop_go > old.stop_go {
            offer(slots, 2, RaceAlert::StopGo, now, frame);
        } else if control.penalty_seconds > old.penalty_seconds {
            offer(
                slots,
                2,
                RaceAlert::TimePenalty(control.penalty_seconds - old.penalty_seconds),
                now,
                frame,
            );
        }
        if control.warnings > old.warnings {
            offer(
                slots,
                3,
                RaceAlert::Warning(control.warnings - old.warnings),
                now,
                frame,
            );
        }
    }

    pub fn event(&mut self, code: [u8; 4], h: Header, now: Instant, slots: &mut Slots) -> bool {
        let (index, kind) = match &code {
            b"RDFL" => (0, RaceAlert::RedFlag),
            b"CHQF" => (1, RaceAlert::ChequeredFlag),
            _ => return false,
        };
        if self.events[index] == Some(h.overall_frame) {
            return false;
        }
        self.events[index] = Some(h.overall_frame);
        offer(slots, 4, kind, now, h.overall_frame);
        true
    }

    pub fn invalidate(&mut self, id: u8, slots: &mut Slots) {
        match id {
            7 => {
                self.flag = None;
                slots[0] = None;
            }
            1 => {
                self.safety = None;
                slots[1] = None;
            }
            2 => {
                self.control = None;
                slots[2] = None;
                slots[3] = None;
            }
            3 => {
                slots[4] = None;
            }
            _ => (),
        }
    }
}
