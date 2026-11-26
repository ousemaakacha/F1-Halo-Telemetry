use super::{
    Aero, CarGap, CarStatus, CarTelemetry, Damage, ErsCapacity, GapReading, Header, LapHistory,
    Packet, Participants, RaceAlert, SessionData, TimeTrialReading, alerts::AlertTracker,
    gaps::GapTracker, time_trial::TimeTrialTracker,
};
use std::time::{Duration, Instant};

pub const FAST_TTL: Duration = Duration::from_millis(500);
pub const DAMAGE_TTL: Duration = Duration::from_secs(2);
pub const HISTORY_TTL: Duration = Duration::from_secs(5);
pub const PARTICIPANTS_TTL: Duration = Duration::from_secs(10);

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
    pub session_generation: u64,
    pub ers_capacity: Option<ErsCapacity>,
    pub ers_deploy_limit_joules: Option<f32>,
    pub header: Option<Header>,
    pub telemetry: Option<Timed<CarTelemetry>>,
    pub status: Option<Timed<CarStatus>>,
    pub damage: Option<Timed<Damage>>,
    pub aero: Option<Timed<Aero>>,
    pub lap_status: Option<Timed<u8>>,
    pub driver_status: Option<Timed<u8>>,
    pub current_lap: Option<Timed<u8>>,
    pub lap_history: Option<Timed<LapHistory>>,
    pub time_trial: Option<Timed<TimeTrialReading>>,
    pub gap_behind: Option<Timed<u32>>,
    pub race_gaps: Option<Timed<[Option<GapReading>; 2]>>,
    pub race_alerts: [Option<Timed<RaceAlert>>; 5],
    pub session: Option<Timed<SessionData>>,
    pub formula: Option<u8>,
    pub participants: Option<u8>,
    pub driver_names: Option<Timed<Participants>>,
    pub race_position: Option<Timed<u8>>,
    pub preceding_car: Option<Timed<u8>>,
    pub following_car: Option<Timed<u8>>,
    pub source: Connection,
    pub counters: NetworkCounters,
    pub last_packet: Option<Instant>,
    pub socket_error: i32,
}

impl Snapshot {
    pub fn visual_tyre_compound(&self, index: u8, now: Instant) -> Option<u8> {
        self.status
            .as_ref()
            .filter(|status| self.driving(now) && status.fresh(now, FAST_TTL))?
            .value
            .visual_compounds
            .get(usize::from(index))
            .copied()
    }

    pub fn participant_name(&self, index: u8, now: Instant) -> Option<&str> {
        self.driver_names
            .as_ref()
            .filter(|names| self.driving(now) && names.fresh(now, PARTICIPANTS_TTL))?
            .value
            .name(index)
    }

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
            .session
            .is_some_and(|s| s.fresh(now, DAMAGE_TTL) && s.value.paused)
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

    pub fn on_track(&self, now: Instant) -> bool {
        self.driving(now)
            && self.session.is_some_and(|s| {
                s.fresh(now, DAMAGE_TTL)
                    && (1..=18).contains(&s.value.session_type)
                    && !s.value.paused
                    && !s.value.spectating
            })
            && self
                .lap_status
                .is_some_and(|s| s.fresh(now, FAST_TTL) && s.value == 2)
            && self
                .driver_status
                .is_some_and(|s| s.fresh(now, FAST_TTL) && (1..=4).contains(&s.value))
    }

    pub fn lap_counter(&self, now: Instant) -> Option<(u8, u8)> {
        if !self.on_track(now) {
            return None;
        }
        let current = self
            .current_lap
            .filter(|lap| lap.fresh(now, FAST_TTL))?
            .value;
        let total = self.session?.value.total_laps;
        (current > 0 && total > 0).then_some((current, total))
    }

    pub fn lap_delta_ms(&self, now: Instant) -> Option<i64> {
        let history = self.lap_history?;
        if !self.driving(now)
            || !history.fresh(now, HISTORY_TTL)
            || self.current_lap.is_some_and(|lap| {
                lap.value == 0 || lap.value.saturating_sub(1) != history.value.completed_laps
            })
        {
            return None;
        }
        history.value.delta_ms()
    }

    pub fn gap_behind_ms(&self, now: Instant) -> Option<u32> {
        self.gap_behind
            .filter(|gap| self.driving(now) && gap.fresh(now, FAST_TTL))
            .map(|gap| gap.value)
    }

    pub fn race_gaps(&self, now: Instant) -> [Option<GapReading>; 2] {
        self.race_gaps
            .filter(|gaps| self.on_track(now) && gaps.fresh(now, FAST_TTL))
            .map_or([None; 2], |gaps| gaps.value)
    }

    pub fn race_opponents(&self, now: Instant) -> [Option<CarGap>; 2] {
        let gaps = self.race_gaps(now);
        let cars = [self.preceding_car, self.following_car];
        std::array::from_fn(|i| {
            Some(CarGap {
                car_index: cars[i].filter(|car| car.fresh(now, FAST_TTL))?.value,
                millis: gaps[i]?.millis,
            })
        })
    }

    pub fn time_trial(&self, now: Instant) -> Option<TimeTrialReading> {
        self.time_trial
            .filter(|reading| self.on_track(now) && reading.fresh(now, FAST_TTL))
            .map(|reading| reading.value)
    }

    pub fn freshness_mask(&self, now: Instant) -> u32 {
        (self.driving(now) as u32)
            | ((self.status.is_some_and(|x| x.fresh(now, FAST_TTL)) as u32) << 1)
            | ((self.damage.is_some_and(|x| x.fresh(now, DAMAGE_TTL)) as u32) << 2)
            | ((self.aero.is_some_and(|x| x.fresh(now, FAST_TTL)) as u32) << 3)
            | ((self.connection(now) as u32) << 4)
            | ((self.lap_history.is_some_and(|x| x.fresh(now, HISTORY_TTL)) as u32) << 8)
            | ((self.gap_behind.is_some_and(|x| x.fresh(now, FAST_TTL)) as u32) << 9)
            | ((self.race_gaps.is_some_and(|x| x.fresh(now, FAST_TTL)) as u32) << 10)
            | (u32::from(self.race_alert(now).map_or(0, |alert| alert.value.code())) << 11)
            | ((self.time_trial(now).is_some() as u32) << 16)
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
    gap_trackers: [GapTracker; 2],
    alert_tracker: AlertTracker,
    time_trial_tracker: TimeTrialTracker,
}

impl TelemetryState {
    pub fn invalidate_time_trial(&mut self) {
        self.time_trial_tracker.interrupt();
        self.snapshot.time_trial = None;
    }

    pub fn invalidate_race_alerts(&mut self, packet_id: u8) {
        self.alert_tracker
            .invalidate(packet_id, &mut self.snapshot.race_alerts);
    }

    pub fn invalidate_race_gaps(&mut self) {
        self.snapshot.race_gaps = None;
        self.snapshot.preceding_car = None;
        self.snapshot.following_car = None;
        self.gap_trackers = Default::default();
    }

    fn clear_values(&mut self) {
        self.time_trial_tracker = TimeTrialTracker::default();
        self.snapshot.time_trial = None;
        self.invalidate_race_gaps();
        self.snapshot.race_alerts = [None; 5];
        self.alert_tracker = AlertTracker::default();
        self.snapshot.ers_capacity = None;
        self.snapshot.ers_deploy_limit_joules = None;
        self.snapshot.telemetry = None;
        self.snapshot.status = None;
        self.snapshot.damage = None;
        self.snapshot.aero = None;
        self.snapshot.lap_status = None;
        self.snapshot.driver_status = None;
        self.snapshot.current_lap = None;
        self.snapshot.lap_history = None;
        self.snapshot.gap_behind = None;
        self.snapshot.session = None;
        self.snapshot.formula = None;
        self.snapshot.participants = None;
        self.snapshot.driver_names = None;
        self.snapshot.race_position = None;
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
            self.snapshot.session_generation = self.snapshot.session_generation.wrapping_add(1);
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
                self.snapshot.session_generation = self.snapshot.session_generation.wrapping_add(1);
                self.high = None;
                self.boundary = None;
                self.ended = false;
                self.snapshot.counters.restarts += 1;
            } else if serial_newer(h.overall_frame, old.overall_frame)
                && h.session_time + 0.001 < old.session_time
                && h.frame < old.frame
            {
                // A rewind interrupts the active trace, not completed reference laps.
                let time_trial_tracker = std::mem::take(&mut self.time_trial_tracker);
                self.clear_values();
                self.time_trial_tracker = time_trial_tracker;
                self.invalidate_time_trial();
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
                // Different event codes can share a frame; deduplicate new notices separately.
                if let Packet::Event(_, code) = packet {
                    if !self.ended
                        && self.snapshot.on_track(now)
                        && !self
                            .high
                            .is_some_and(|v| serial_newer(v.overall_frame, h.overall_frame))
                    {
                        return self.alert_tracker.event(
                            code,
                            h,
                            now,
                            &mut self.snapshot.race_alerts,
                        );
                    }
                }
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
        let alert_lag = self
            .high
            .map_or(0.0, |v| (v.session_time - h.session_time).max(0.0));
        match packet {
            Packet::Telemetry(_, v) if !self.ended => self.snapshot.telemetry = timed!(v),
            Packet::Status(_, v) => {
                self.snapshot.status = timed!(v);
                if alert_lag <= FAST_TTL.as_secs_f32() {
                    self.alert_tracker.flag(
                        v.fia_flag,
                        now,
                        h.overall_frame,
                        &mut self.snapshot.race_alerts,
                    );
                } else {
                    self.invalidate_race_alerts(7);
                }
            }
            Packet::Damage(_, v) => self.snapshot.damage = timed!(v),
            Packet::Aero(_, v) => self.snapshot.aero = timed!(v),
            Packet::Session(_, session) => {
                self.snapshot.session = timed!(session);
                self.snapshot.formula = Some(session.formula);
                if alert_lag <= DAMAGE_TTL.as_secs_f32() {
                    self.alert_tracker.safety(
                        session.safety_car,
                        now,
                        h.overall_frame,
                        &mut self.snapshot.race_alerts,
                    );
                } else {
                    self.invalidate_race_alerts(1);
                }
            }
            Packet::Lap(_, lap) if !self.ended => {
                if let Some(session) = self.snapshot.session.filter(|session| {
                    alert_lag <= FAST_TTL.as_secs_f32()
                        && session.fresh(now, DAMAGE_TTL)
                        && !session.value.paused
                        && !session.value.spectating
                }) {
                    self.snapshot.time_trial = self
                        .time_trial_tracker
                        .update(lap, session.value.track_length_m, now)
                        .map(|value| Timed {
                            value,
                            received: now,
                            overall_frame: h.overall_frame,
                        });
                } else {
                    self.invalidate_time_trial();
                }
                self.snapshot.race_position = timed!(lap.position);
                self.snapshot.preceding_car = lap.race_gaps[0].map(|gap| Timed {
                    value: gap.car_index,
                    received: now,
                    overall_frame: h.overall_frame,
                });
                self.snapshot.following_car = lap.race_gaps[1].map(|gap| Timed {
                    value: gap.car_index,
                    received: now,
                    overall_frame: h.overall_frame,
                });
                self.snapshot.lap_status = timed!(lap.result_status);
                self.snapshot.driver_status = timed!(lap.driver_status);
                self.snapshot.current_lap = timed!(lap.current_lap);
                self.snapshot.gap_behind = lap.gap_behind_ms.map(|gap| Timed {
                    value: gap,
                    received: now,
                    overall_frame: h.overall_frame,
                });
                let readings = std::array::from_fn(|i| {
                    self.gap_trackers[i].update(lap.race_gaps[i], h.session_time, now)
                });
                self.snapshot.race_gaps = timed!(readings);
                if alert_lag <= FAST_TTL.as_secs_f32() {
                    self.alert_tracker.control(
                        lap.race_control,
                        now,
                        h.overall_frame,
                        &mut self.snapshot.race_alerts,
                    );
                } else {
                    self.invalidate_race_alerts(2);
                }
            }
            Packet::History(_, history) if !self.ended => {
                self.time_trial_tracker.history(history);
                self.snapshot.lap_history = timed!(history)
            }
            Packet::Participants(_, names) => {
                self.snapshot.participants = Some(names.active_cars);
                self.snapshot.driver_names = timed!(names);
            }
            Packet::Event(_, code) => match &code {
                b"SEND" => {
                    self.clear_values();
                    self.snapshot.session_generation =
                        self.snapshot.session_generation.wrapping_add(1);
                    self.sequence[id] = Some(h.overall_frame);
                    self.ended = true;
                }
                b"SSTA" => {
                    self.clear_values();
                    self.snapshot.session_generation =
                        self.snapshot.session_generation.wrapping_add(1);
                    self.sequence[id] = Some(h.overall_frame);
                    self.ended = false;
                }
                // Header rollback is authoritative; a delayed FLBK event must not clear new data twice.
                _ => {
                    if !self.ended
                        && self.snapshot.on_track(now)
                        && !self
                            .high
                            .is_some_and(|v| serial_newer(v.overall_frame, h.overall_frame))
                    {
                        self.alert_tracker
                            .event(code, h, now, &mut self.snapshot.race_alerts);
                    }
                }
            },
            _ => (),
        }
        if !self.snapshot.on_track(now) {
            self.invalidate_race_gaps();
            self.alert_tracker.inactive(&mut self.snapshot.race_alerts);
        }
        true
    }
}
