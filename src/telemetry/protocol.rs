//! EA 2026 Season Pack v1.2, packet version 1. Explicit little-endian reads avoid
//! packed references, alignment assumptions and unsafe deserialization.

use std::fmt;

pub const HEADER_SIZE: usize = 29;
pub const CAR_COUNT: usize = 24;
pub const TELEMETRY_SIZE: usize = 1448;
pub const STATUS_SIZE: usize = 1445;
pub const DAMAGE_SIZE: usize = 1133;
pub const AERO_SIZE: usize = 269;
pub const HISTORY_SIZE: usize = 1460;
pub const LAP_SIZE: usize = 1399;
pub const PARTICIPANTS_SIZE: usize = 1470;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarGap {
    pub car_index: u8,
    pub millis: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LapData {
    pub position: u8,
    pub result_status: u8,
    pub driver_status: u8,
    pub current_lap: u8,
    pub gap_behind_ms: Option<u32>,
    /// Active race neighbours, ahead then behind, with identity for trend tracking.
    pub race_gaps: [Option<CarGap>; 2],
    pub race_control: RaceControl,
    pub timing: Option<LapTiming>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LapTiming {
    pub last_ms: u32,
    pub current_ms: u32,
    pub distance_m: f32,
    pub sector: u8,
    pub sectors_ms: [Option<u32>; 2],
    pub invalid: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Participants {
    pub active_cars: u8,
    pub names: [[u8; 32]; CAR_COUNT],
}

impl Participants {
    pub fn name(&self, index: u8) -> Option<&str> {
        let bytes = self.names.get(usize::from(index))?;
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
        std::str::from_utf8(&bytes[..end])
            .ok()
            .filter(|s| !s.chars().any(char::is_control))
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RaceControl {
    pub penalty_seconds: u8,
    pub warnings: u8,
    pub drive_through: u8,
    pub stop_go: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SessionData {
    pub total_laps: u8,
    pub track_length_m: u16,
    pub paused: bool,
    pub formula: u8,
    pub session_type: u8,
    pub spectating: bool,
    pub safety_car: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LapHistory {
    pub completed_laps: u8,
    pub last_lap_ms: Option<u32>,
    pub best_before_last_ms: Option<u32>,
}

impl LapHistory {
    pub fn delta_ms(self) -> Option<i64> {
        Some(i64::from(self.last_lap_ms?) - i64::from(self.best_before_last_ms?))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Header {
    pub game_year: u8,
    pub major: u8,
    pub minor: u8,
    pub packet_id: u8,
    pub session_uid: u64,
    pub session_time: f32,
    pub frame: u32,
    pub overall_frame: u32,
    pub player: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarTelemetry {
    pub speed_kmh: u16,
    pub throttle: f32,
    pub brake: f32,
    pub gear: i8,
    pub rpm: u16,
    pub rev_percent: u8,
    pub rev_bits: u16,
    pub surface_c: [u8; 4],
    pub inner_c: [u8; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarStatus {
    /// Visual compound IDs for every car, indexed by its slot in the status packet.
    pub visual_compounds: [u8; CAR_COUNT],
    pub fuel_remaining_laps: f32,
    pub max_rpm: u16,
    pub ers_joules: f32,
    pub deploy_mode: u8,
    pub harvested_mguk_joules: f32,
    pub harvest_limit_joules: f32,
    pub deployed_joules: f32,
    pub paused: bool,
    pub fia_flag: Option<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Damage {
    pub wear: [f32; 4],
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Aero {
    pub straight: bool,
    pub available: bool,
    pub activation_distance_m: u16,
    pub overtake_available: bool,
    pub overtake_active: bool,
    pub overtake_distance_m: u16,
    pub regulations_2026: bool,
}

// Fixed-size wire data keeps parsing and snapshot publication allocation-free.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Packet {
    Telemetry(Header, CarTelemetry),
    Status(Header, CarStatus),
    Damage(Header, Damage),
    Aero(Header, Aero),
    Session(Header, SessionData),
    Lap(Header, LapData),
    History(Header, LapHistory),
    Participants(Header, Participants),
    Event(Header, [u8; 4]),
    Ignored(Header),
}

impl Packet {
    pub fn header(&self) -> Header {
        match *self {
            Self::Telemetry(h, _)
            | Self::Status(h, _)
            | Self::Damage(h, _)
            | Self::Aero(h, _)
            | Self::Session(h, _)
            | Self::Lap(h, _)
            | Self::History(h, _)
            | Self::Participants(h, _)
            | Self::Event(h, _)
            | Self::Ignored(h) => h,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    Truncated,
    Format(u16),
    Version(u8),
    Player(u8),
    Length {
        id: u8,
        expected: usize,
        actual: usize,
    },
    InvalidValue(&'static str),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, "incomplete UDP header"),
            Self::Format(v) => write!(f, "unsupported UDP format {v}; select 2026"),
            Self::Version(v) => write!(f, "unsupported packet version {v}; expected 1"),
            Self::Player(v) => write!(f, "invalid player index {v}"),
            Self::Length {
                id,
                expected,
                actual,
            } => write!(f, "packet {id}: {actual} bytes, expected {expected}"),
            Self::InvalidValue(name) => write!(f, "invalid UDP value: {name}"),
        }
    }
}

impl std::error::Error for ParseError {}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i..i + 4].try_into().unwrap())
}
fn f32_at(b: &[u8], i: usize) -> f32 {
    f32::from_bits(u32_at(b, i))
}

fn finite_range(value: f32, min: f32, max: f32, field: &'static str) -> Result<f32, ParseError> {
    if value.is_finite() && (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(ParseError::InvalidValue(field))
    }
}

fn boolean(value: u8, field: &'static str) -> Result<bool, ParseError> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(ParseError::InvalidValue(field)),
    }
}

pub fn parse(b: &[u8]) -> Result<Packet, ParseError> {
    if b.len() < HEADER_SIZE {
        return Err(ParseError::Truncated);
    }
    let format = u16_at(b, 0);
    if format != 2026 {
        return Err(ParseError::Format(format));
    }
    let id = b[6];
    let expected = match id {
        1 => 926,
        2 => LAP_SIZE,
        3 => 45,
        4 => PARTICIPANTS_SIZE,
        6 => TELEMETRY_SIZE,
        7 => STATUS_SIZE,
        10 => DAMAGE_SIZE,
        11 => HISTORY_SIZE,
        16 => AERO_SIZE,
        _ => 0,
    };
    if expected != 0 {
        if b[5] != 1 {
            return Err(ParseError::Version(b[5]));
        }
        if b.len() != expected {
            return Err(ParseError::Length {
                id,
                expected,
                actual: b.len(),
            });
        }
    }
    let player = b[27];
    // 255 is used outside a local driving session. No array is indexed in that case.
    if player as usize >= CAR_COUNT && player != 255 {
        return Err(ParseError::Player(player));
    }
    let h = Header {
        game_year: b[2],
        major: b[3],
        minor: b[4],
        packet_id: id,
        session_uid: u64::from_le_bytes(b[7..15].try_into().unwrap()),
        session_time: finite_range(f32_at(b, 15), 0.0, f32::MAX, "sessionTime")?,
        frame: u32_at(b, 19),
        overall_frame: u32_at(b, 23),
        player,
    };
    if player == 255 || expected == 0 {
        return Ok(Packet::Ignored(h));
    }
    let index = player as usize;
    Ok(match id {
        6 => {
            let c = &b[HEADER_SIZE + 59 * index..][..59];
            let gear = c[15] as i8;
            if !(-1..=8).contains(&gear) {
                return Err(ParseError::InvalidValue("gear"));
            }
            if c[19] > 100 || u16_at(c, 20) & 0x8000 != 0 {
                return Err(ParseError::InvalidValue("revLights"));
            }
            Packet::Telemetry(
                h,
                CarTelemetry {
                    speed_kmh: u16_at(c, 0),
                    throttle: finite_range(f32_at(c, 2), 0.0, 1.0, "throttle")?,
                    brake: finite_range(f32_at(c, 10), 0.0, 1.0, "brake")?,
                    gear,
                    rpm: u16_at(c, 16),
                    rev_percent: c[19],
                    rev_bits: u16_at(c, 20),
                    surface_c: c[30..34].try_into().unwrap(),
                    inner_c: c[34..38].try_into().unwrap(),
                },
            )
        }
        7 => {
            let c = &b[HEADER_SIZE + 59 * index..][..59];
            if c[41] > 3 {
                return Err(ParseError::InvalidValue("ersDeployMode"));
            }
            Packet::Status(
                h,
                CarStatus {
                    visual_compounds: std::array::from_fn(|car| b[HEADER_SIZE + 59 * car + 26]),
                    fuel_remaining_laps: finite_range(
                        f32_at(c, 13),
                        -f32::MAX,
                        f32::MAX,
                        "fuelRemainingLaps",
                    )?,
                    max_rpm: u16_at(c, 17),
                    ers_joules: finite_range(f32_at(c, 37), 0.0, f32::MAX, "ersStoreEnergy")?,
                    deploy_mode: c[41],
                    harvested_mguk_joules: finite_range(
                        f32_at(c, 42),
                        0.0,
                        f32::MAX,
                        "ersHarvestedThisLapMGUK",
                    )?,
                    harvest_limit_joules: finite_range(
                        f32_at(c, 50),
                        0.0,
                        f32::MAX,
                        "ersHarvestLimitPerLap",
                    )?,
                    deployed_joules: finite_range(
                        f32_at(c, 54),
                        0.0,
                        f32::MAX,
                        "ersDeployedThisLap",
                    )?,
                    paused: boolean(c[58], "networkPaused")?,
                    fia_flag: (c[28] <= 3).then_some(c[28]),
                },
            )
        }
        10 => {
            let c = &b[HEADER_SIZE + 46 * index..][..46];
            let mut wear = [0.0; 4];
            for (i, v) in wear.iter_mut().enumerate() {
                *v = finite_range(f32_at(c, i * 4), 0.0, 100.0, "tyresWear")?;
            }
            Packet::Damage(h, Damage { wear })
        }
        16 => {
            let c = &b[HEADER_SIZE + 10 * index..][..10];
            Packet::Aero(
                h,
                Aero {
                    straight: boolean(c[0], "activeAeroMode")?,
                    available: boolean(c[1], "activeAeroAvailable")?,
                    activation_distance_m: u16_at(c, 2),
                    overtake_available: boolean(c[4], "overtakeAvailable")?,
                    overtake_active: boolean(c[5], "overtakeActive")?,
                    overtake_distance_m: u16_at(c, 6),
                    regulations_2026: boolean(c[8], "2026Regulations")?,
                },
            )
        }
        1 => Packet::Session(
            h,
            SessionData {
                total_laps: b[HEADER_SIZE + 3],
                track_length_m: u16_at(b, HEADER_SIZE + 4),
                paused: boolean(b[HEADER_SIZE + 14], "gamePaused")?,
                formula: b[HEADER_SIZE + 8],
                session_type: b[HEADER_SIZE + 6],
                spectating: boolean(b[HEADER_SIZE + 15], "isSpectating")?,
                safety_car: (b[HEADER_SIZE + 124] <= 3).then_some(b[HEADER_SIZE + 124]),
            },
        ),
        2 => {
            let cars = &b[HEADER_SIZE..HEADER_SIZE + 57 * CAR_COUNT];
            let car = &cars[57 * index..][..57];
            let result = car[45];
            let driver_status = car[44];
            if driver_status > 4 {
                return Err(ParseError::InvalidValue("driverStatus"));
            }
            if result > 7 {
                return Err(ParseError::InvalidValue("resultStatus"));
            }
            let position = car[32];
            if position as usize > CAR_COUNT {
                return Err(ParseError::InvalidValue("carPosition"));
            }
            let mut gap_behind_ms = None;
            if result == 2 && (1..CAR_COUNT as u8).contains(&position) {
                // The following car's interval to its race predecessor is our rear gap.
                if let Some(behind) = cars
                    .chunks_exact(57)
                    .find(|other| other[32] == position + 1 && other[45] == 2)
                {
                    let millis = u16_at(behind, 14);
                    if millis >= 60_000 {
                        return Err(ParseError::InvalidValue("deltaToCarInFrontMSPart"));
                    }
                    gap_behind_ms = Some(u32::from(behind[16]) * 60_000 + u32::from(millis));
                }
            }
            let neighbour = |target: u8, interval: Option<u32>| {
                let mut matches = cars
                    .chunks_exact(57)
                    .enumerate()
                    .filter(|(_, other)| other[32] == target && other[45] == 2);
                let (car_index, other) = matches.next()?;
                if matches.next().is_some() || !(1..=4).contains(&other[44]) {
                    return None;
                }
                Some(CarGap {
                    car_index: car_index as u8,
                    millis: interval?,
                })
            };
            let mut race_gaps = [None; 2];
            if result == 2 && (1..=4).contains(&driver_status) {
                if position > 1 {
                    let millis = u16_at(car, 14);
                    // A bad front interval must not discard the existing rear-gap data.
                    let interval = (millis < 60_000)
                        .then_some(u32::from(car[16]) * 60_000 + u32::from(millis));
                    race_gaps[0] = neighbour(position - 1, interval);
                }
                if (1..CAR_COUNT as u8).contains(&position) {
                    race_gaps[1] = neighbour(position + 1, gap_behind_ms);
                }
            }
            Packet::Lap(
                h,
                LapData {
                    position,
                    result_status: result,
                    driver_status,
                    current_lap: car[33],
                    gap_behind_ms,
                    race_gaps,
                    race_control: RaceControl {
                        penalty_seconds: car[38],
                        warnings: car[39],
                        drive_through: car[41],
                        stop_go: car[42],
                    },
                    // Bad timing must not discard the existing race standings.
                    timing: lap_timing(car),
                },
            )
        }
        11 => {
            let car = b[29];
            if car as usize >= CAR_COUNT {
                return Err(ParseError::InvalidValue("historyCarIndex"));
            }
            if car != player {
                return Ok(Packet::Ignored(h));
            }
            let laps = b[30] as usize;
            if laps > 100 || b[31] > 8 || b[32..36].iter().any(|&lap| lap as usize > laps) {
                return Err(ParseError::InvalidValue("historyCounts"));
            }
            let mut history = LapHistory {
                // The final history record is the current, incomplete lap.
                completed_laps: laps.saturating_sub(1) as u8,
                ..LapHistory::default()
            };
            for lap in 0..laps {
                let offset = 36 + lap * 14;
                let valid = b[offset + 13];
                if valid & !0x0f != 0 {
                    return Err(ParseError::InvalidValue("lapValidBitFlags"));
                }
                if lap >= history.completed_laps as usize {
                    continue;
                }
                let time = u32_at(b, offset);
                let time = (valid & 1 != 0 && time > 0).then_some(time);
                if lap + 1 == history.completed_laps as usize {
                    history.last_lap_ms = time;
                } else if let Some(time) = time {
                    history.best_before_last_ms = Some(
                        history
                            .best_before_last_ms
                            .map_or(time, |best| best.min(time)),
                    );
                }
            }
            Packet::History(h, history)
        }
        4 => {
            if b[29] > 24 {
                return Err(ParseError::InvalidValue("numActiveCars"));
            }
            Packet::Participants(
                h,
                Participants {
                    active_cars: b[29],
                    names: std::array::from_fn(|i| {
                        b[30 + 60 * i + 10..30 + 60 * i + 42].try_into().unwrap()
                    }),
                },
            )
        }
        3 => Packet::Event(h, b[29..33].try_into().unwrap()),
        _ => Packet::Ignored(h),
    })
}

fn lap_timing(car: &[u8]) -> Option<LapTiming> {
    if car[36] > 2 || car[37] > 1 || u16_at(car, 8) >= 60_000 || u16_at(car, 11) >= 60_000 {
        return None;
    }
    let distance_m = f32_at(car, 20);
    if !distance_m.is_finite() {
        return None;
    }
    Some(LapTiming {
        last_ms: u32_at(car, 0),
        current_ms: u32_at(car, 4),
        distance_m,
        sector: car[36],
        sectors_ms: [8, 11].map(|offset| {
            let millis = u32::from(car[offset + 2]) * 60_000 + u32::from(u16_at(car, offset));
            (millis > 0).then_some(millis)
        }),
        invalid: car[37] == 1,
    })
}
