use super::{protocol::CarGap, state::FAST_TTL};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GapTrend {
    #[default]
    Neutral,
    Shrinking,
    Growing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GapReading {
    pub millis: u32,
    pub trend: GapTrend,
}

#[derive(Clone, Copy)]
struct Sample {
    gap: CarGap,
    session_time: f32,
    received: Instant,
}

#[derive(Default)]
pub(super) struct GapTracker {
    baseline: Option<Sample>,
    last: Option<Sample>,
    trend: GapTrend,
}

impl GapTracker {
    pub fn update(
        &mut self,
        gap: Option<CarGap>,
        session_time: f32,
        received: Instant,
    ) -> Option<GapReading> {
        let Some(gap) = gap else {
            *self = Self::default();
            return None;
        };
        let continuous = self.last.is_some_and(|last| {
            last.gap.car_index == gap.car_index
                && session_time > last.session_time
                && session_time - last.session_time <= FAST_TTL.as_secs_f32()
                && received > last.received
                && received.duration_since(last.received) <= FAST_TTL
        });
        if !continuous {
            *self = Self::default();
        }
        let current = Sample {
            gap,
            session_time,
            received,
        };
        let baseline = *self.baseline.get_or_insert(current);
        // Observe a full second in game time and receipt time; ignore small timing jitter.
        if session_time - baseline.session_time >= 1.0
            && received.duration_since(baseline.received) >= Duration::from_secs(1)
        {
            let change = i64::from(gap.millis) - i64::from(baseline.gap.millis);
            self.trend = if change <= -50 {
                GapTrend::Shrinking
            } else if change >= 50 {
                GapTrend::Growing
            } else {
                GapTrend::Neutral
            };
            self.baseline = Some(current);
        }
        self.last = Some(current);
        Some(GapReading {
            millis: gap.millis,
            trend: self.trend,
        })
    }
}
