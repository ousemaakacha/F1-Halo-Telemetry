pub mod demo;
pub mod protocol;
pub mod state;

pub use protocol::{
    Aero, CarStatus, CarTelemetry, Damage, Header, LapData, LapHistory, Packet, ParseError, parse,
};
pub use state::{Connection, Snapshot, TelemetryState, Timed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapacitySource {
    GameFiles,
    Demo,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ErsCapacity {
    pub joules: f32,
    pub year: u16,
    pub source: CapacitySource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Wheel {
    RearLeft = 0,
    RearRight = 1,
    FrontLeft = 2,
    FrontRight = 3,
}

impl Wheel {
    pub const VISUAL_ORDER: [Self; 4] = [
        Self::FrontLeft,
        Self::FrontRight,
        Self::RearLeft,
        Self::RearRight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::RearLeft => "RL",
            Self::RearRight => "RR",
            Self::FrontLeft => "FL",
            Self::FrontRight => "FR",
        }
    }
}

pub fn pedal_percent(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0) * 100.0
    } else {
        0.0
    }
}

pub fn ers_percent(joules: f32, capacity: Option<f32>) -> Option<f32> {
    let capacity = capacity?;
    (joules.is_finite() && joules >= 0.0 && capacity.is_finite() && capacity > 0.0)
        .then(|| (joules / capacity * 100.0).clamp(0.0, 100.0))
}

pub fn gear_label(gear: i8) -> &'static str {
    match gear {
        -1 => "R",
        0 => "N",
        1 => "1",
        2 => "2",
        3 => "3",
        4 => "4",
        5 => "5",
        6 => "6",
        7 => "7",
        8 => "8",
        _ => "--",
    }
}
