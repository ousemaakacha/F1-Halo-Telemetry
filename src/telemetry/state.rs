use super::{Aero, CarStatus, CarTelemetry, Damage, ErsCapacity, Header, Packet};
use std::time::{Duration, Instant};

pub const FAST_TTL: Duration = Duration::from_millis(500);
pub const DAMAGE_TTL: Duration = Duration::from_secs(2);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Connection {
    #[default]
    Waiting,
    Listening,
    Live,
    Demo,
    Stale,
    BindError,
    Unsupported,
    Inactive,
    Paused,
}

impl Connection {
    pub fn label(self) -> &'static str {
        match self {
            Self::Waiting | Self::Listening => "WAITING FOR UDP",
            Self::Live => "LIVE",
            Self::Demo => "DEMO · SIMULATED DATA",
            Self::Stale => "STALE DATA",
            Self::BindError => "UDP PORT ERROR",
            Self::Unsupported => "UNSUPPORTED FORMAT",
            Self::Inactive => "CAR INACTIVE",
            Self::Paused => "PAUSED",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Timed<T: Copy> {
    pub value: T,
    pub received: Instant,
    pub overall_frame: u32,
}

impl<T: Copy> Timed<T> {
    pub fn fresh(&self, now: Instant, ttl: Duration) -> bool {
        now.saturating_duration_since(self.received) <= ttl
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NetworkCounters {
    pub received: u64,
    pub accepted: u64,
    pub malformed: u64,
    pub unsupported: u64,
    pub reordered: u64,
    pub duplicates: u64,
    pub sessions: u64,
    pub flashbacks: u64,
    pub restarts: u64,
    pub per_packet: [u64; 17],
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    pub source_epoch: u64,
    pub ers_capacity: Option<ErsCapacity>,
    pub header: Option<Header>,
    pub telemetry: Option<Timed<CarTelemetry>>,
    pub status: Option<Timed<CarStatus>>,
    pub damage: Option<Timed<Damage>>,
    pub aero: Option<Timed<Aero>>,
    pub lap_status: Option<Timed<u8>>,
    pub session_paused: Option<Timed<bool>>,
    pub formula: Option<u8>,
    pub participants: Option<u8>,
    pub source: Connection,
    pub counters: NetworkCounters,
    pub last_packet: Option<Instant>,
    pub socket_error: i32,
}

impl Snapshot {
    pub fn connection(&self, now: Instant) -> Connection {
        if matches!(self.source, Connection::BindError | Connection::Unsupported) {
            return self.source;
        }
        let Some(t) = self.telemetry else {
            return self.source;
        };
        if !t.fresh(now, FAST_TTL) {
            return Connection::Stale;
        }
        if self
            .lap_status
            .is_some_and(|s| s.fresh(now, FAST_TTL) && s.value <= 1)
        {
            return Connection::Inactive;
        }
        if self
            .session_paused
            .is_some_and(|s| s.fresh(now, DAMAGE_TTL) && s.value)
            || self
                .status
                .is_some_and(|s| s.fresh(now, FAST_TTL) && s.value.paused)
        {
            return Connection::Paused;
        }
        if self.source == Connection::Demo {
            Connection::Demo
        } else {
            Connection::Live
        }
    }

    pub fn driving(&self, now: Instant) -> bool {
        matches!(self.connection(now), Connection::Live | Connection::Demo)
    }

    pub fn freshness_mask(&self, now: Instant) -> u16 {
        (self.driving(now) as u16)
            | ((self.status.is_some_and(|x| x.fresh(now, FAST_TTL)) as u16) << 1)
            | ((self.damage.is_some_and(|x| x.fresh(now, DAMAGE_TTL)) as u16) << 2)
            | ((self.aero.is_some_and(|x| x.fresh(now, FAST_TTL)) as u16) << 3)
            | ((self.connection(now) as u16) << 4)
    }
}

pub fn serial_newer(new: u32, old: u32) -> bool {
    let distance = new.wrapping_sub(old);
    distance != 0 && distance < 0x8000_0000
}

#[derive(Default)]
pub struct TelemetryState {
    pub snapshot: Snapshot,
    sequence: [Option<u32>; 17],
    high: Option<Header>,
    boundary: Option<u32>,
    retired: [Option<u64>; 8],
    retired_next: usize,
    ended: bool,
}

impl TelemetryState {
    fn clear_values(&mut self) {
        self.snapshot.ers_capacity = None;
        self.snapshot.telemetry = None;
        self.snapshot.status = None;
        self.snapshot.damage = None;
        self.snapshot.aero = None;
        self.snapshot.lap_status = None;
        self.snapshot.session_paused = None;
        self.snapshot.formula = None;
        self.snapshot.participants = None;
        self.sequence = [None; 17];
    }

    pub fn apply(&mut self, packet: Packet, now: Instant) -> bool {
        if matches!(packet, Packet::Ignored(_)) {
            return false;
        }
        let h = packet.header();
        let old = self.high;
        if matches!(packet, Packet::Event(_, code) if &code == b"SSTA" || &code == b"SEND")
            && old.is_some_and(|v| {
                v.session_uid == h.session_uid && serial_newer(v.overall_frame, h.overall_frame)
            })
        {
            self.snapshot.counters.reordered += 1;
            return false;
        }
        let session_changed =
            old.is_none_or(|v| v.session_uid != h.session_uid || v.player != h.player);
        if session_changed {
            if self.retired.contains(&Some(h.session_uid)) {
                self.snapshot.counters.reordered += 1;
                return false;
            }
            if let Some(old) = old.filter(|v| v.session_uid != h.session_uid) {
                self.retired[self.retired_next] = Some(old.session_uid);
                self.retired_next = (self.retired_next + 1) % self.retired.len();
            }
            self.clear_values();
            self.boundary = None;
            self.high = None;
            self.ended = false;
            self.snapshot.counters.sessions += 1;
        } else if let Some(old) = old {
            let restarted = self
                .snapshot
                .last_packet
                .is_some_and(|t| now.saturating_duration_since(t) > DAMAGE_TTL)
                && h.overall_frame < old.overall_frame
                && h.session_time < old.session_time;
            if restarted {
                self.clear_values();
                self.high = None;
                self.boundary = None;
                self.ended = false;
                self.snapshot.counters.restarts += 1;
            } else if serial_newer(h.overall_frame, old.overall_frame)
                && h.session_time + 0.001 < old.session_time
                && h.frame < old.frame
            {
                self.clear_values();
                self.boundary = Some(h.overall_frame);
                self.snapshot.counters.flashbacks += 1;
            }
        }
        if self
            .boundary
            .is_some_and(|b| h.overall_frame != b && !serial_newer(h.overall_frame, b))
        {
            self.snapshot.counters.reordered += 1;
            return false;
        }
        let id = h.packet_id as usize;
        if let Some(previous) = self.sequence[id] {
            if h.overall_frame == previous {
                self.snapshot.counters.duplicates += 1;
                return false;
            }
            if !serial_newer(h.overall_frame, previous) {
                self.snapshot.counters.reordered += 1;
                return false;
            }
        }
        self.sequence[id] = Some(h.overall_frame);
        if self
            .high
            .is_none_or(|v| serial_newer(h.overall_frame, v.overall_frame))
        {
            self.high = Some(h);
        }
        self.snapshot.header = self.high;
        self.snapshot.last_packet = Some(now);
        self.snapshot.counters.accepted += 1;
        self.snapshot.counters.per_packet[id] += 1;
        self.snapshot.source = Connection::Listening;
        macro_rules! timed {
            ($v:expr) => {
                Some(Timed {
                    value: $v,
                    received: now,
                    overall_frame: h.overall_frame,
                })
            };
        }
        match packet {
            Packet::Telemetry(_, v) if !self.ended => self.snapshot.telemetry = timed!(v),
            Packet::Status(_, v) => self.snapshot.status = timed!(v),
            Packet::Damage(_, v) => self.snapshot.damage = timed!(v),
            Packet::Aero(_, v) => self.snapshot.aero = timed!(v),
            Packet::Session(_, paused, formula) => {
                self.snapshot.session_paused = timed!(paused);
                self.snapshot.formula = Some(formula);
            }
            Packet::Lap(_, v) => self.snapshot.lap_status = timed!(v),
            Packet::Participants(_, n) => self.snapshot.participants = Some(n),
            Packet::Event(_, code) => match &code {
                b"SEND" => {
                    self.clear_values();
                    self.sequence[id] = Some(h.overall_frame);
                    self.ended = true;
                }
                b"SSTA" => {
                    self.clear_values();
                    self.sequence[id] = Some(h.overall_frame);
                    self.ended = false;
                }
                // Header rollback is authoritative; a delayed FLBK event must not clear new data twice.
                _ => (),
            },
            _ => (),
        }
        true
    }
}
