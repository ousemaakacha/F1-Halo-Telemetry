use f1_halo_telemetry::telemetry::{state::FAST_TTL, *};
use std::time::{Duration, Instant};

fn feed(
    state: &mut TelemetryState,
    start: Instant,
    tick: u32,
    flag: Option<u8>,
    safety: Option<u8>,
    control: RaceControl,
) -> Instant {
    let now = start + Duration::from_millis(u64::from(tick) * 100);
    for mut packet in demo::sample(tick as f32 / 10.0, 100 + tick * 6) {
        match &mut packet {
            Packet::Status(_, status) => status.fia_flag = flag,
            Packet::Session(_, session) => session.safety_car = safety,
            Packet::Lap(_, lap) => lap.race_control = control,
            _ => (),
        }
        state.apply(packet, now);
    }
    now
}

fn event(tick: u32, code: [u8; 4]) -> Packet {
    Packet::Event(
        Header {
            packet_id: 3,
            ..demo::sample(tick as f32 / 10.0, 100 + tick * 6)[0].header()
        },
        code,
    )
}

#[test]
fn documented_local_flag_safety_car_and_control_offsets_are_decoded_without_affecting_other_data() {
    let mut buffer = [0u8; 1460];
    for player in [0u8, 23] {
        let size = demo::encode(demo::sample(0.0, 100)[1], &mut buffer);
        buffer[27] = player;
        let offset = 29 + 59 * usize::from(player);
        buffer[offset + 37..offset + 41].copy_from_slice(&2_200_000f32.to_le_bytes());
        for raw in [0, 1, 2, 3, 4, 127, 255] {
            buffer[offset + 28] = raw;
            let Packet::Status(_, status) = parse(&buffer[..size]).unwrap() else {
                panic!()
            };
            assert_eq!(status.fia_flag, (raw <= 3).then_some(raw));
            assert_eq!(status.ers_joules, 2_200_000.0);
        }
        let size = demo::encode(demo::sample(0.0, 100)[6], &mut buffer);
        buffer[27] = player;
        for raw in [0, 1, 2, 3, 4, 255] {
            buffer[29 + 124] = raw;
            let Packet::Session(_, session) = parse(&buffer[..size]).unwrap() else {
                panic!()
            };
            assert_eq!(session.safety_car, (raw <= 3).then_some(raw));
        }
        let size = demo::encode(demo::sample(0.0, 100)[5], &mut buffer);
        buffer[27] = player;
        let offset = 29 + 57 * usize::from(player);
        buffer[offset + 38] = 15;
        buffer[offset + 39] = 3;
        buffer[offset + 41] = 1;
        buffer[offset + 42] = 2;
        let Packet::Lap(_, lap) = parse(&buffer[..size]).unwrap() else {
            panic!()
        };
        assert_eq!(
            lap.race_control,
            RaceControl {
                penalty_seconds: 15,
                warnings: 3,
                drive_through: 1,
                stop_go: 2
            }
        );
    }
}

#[test]
fn flags_trigger_on_change_never_extend_on_updates_and_clear_on_invalid_or_missing_source() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    feed(
        &mut state,
        start,
        0,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    let now = feed(
        &mut state,
        start,
        1,
        Some(3),
        Some(0),
        RaceControl::default(),
    );
    let first = state.snapshot.race_alert(now).unwrap();
    assert_eq!(first.value, RaceAlert::Flag(3));
    for tick in 2..=52 {
        let now = feed(
            &mut state,
            start,
            tick,
            Some(3),
            Some(0),
            RaceControl::default(),
        );
        if tick < 51 {
            assert_eq!(
                state.snapshot.race_alert(now).unwrap().received,
                first.received
            );
        }
        if tick == 52 {
            assert!(state.snapshot.race_alert(now).is_none());
        }
    }
    let now = feed(
        &mut state,
        start,
        53,
        Some(2),
        Some(0),
        RaceControl::default(),
    );
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::Flag(2)
    );
    let mut expired = state.snapshot;
    expired.status.as_mut().unwrap().received -= FAST_TTL + Duration::from_millis(1);
    assert!(expired.race_alert(now).is_none());
    let now = feed(&mut state, start, 54, None, Some(0), RaceControl::default());
    assert!(state.snapshot.race_alert(now).is_none());
    assert_eq!(state.snapshot.freshness_mask(now) >> 11, 0);
}

#[test]
fn safety_car_and_vsc_are_live_states_and_formation_is_not_a_deployment() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    feed(
        &mut state,
        start,
        0,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    let now = feed(
        &mut state,
        start,
        1,
        Some(3),
        Some(1),
        RaceControl::default(),
    );
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::SafetyCar(1)
    );
    let now = feed(
        &mut state,
        start,
        2,
        Some(0),
        Some(2),
        RaceControl::default(),
    );
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::SafetyCar(2)
    );
    for (tick, safety) in [(3, Some(3)), (4, Some(0)), (5, None)] {
        let now = feed(
            &mut state,
            start,
            tick,
            Some(0),
            safety,
            RaceControl::default(),
        );
        assert!(state.snapshot.race_alert(now).is_none());
    }
}

#[test]
fn penalties_and_official_warnings_require_a_fresh_baseline_and_increases_not_totals() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    let baseline = RaceControl {
        penalty_seconds: 5,
        warnings: 2,
        ..RaceControl::default()
    };
    for tick in 0..=1 {
        let now = feed(&mut state, start, tick, Some(0), Some(0), baseline);
        assert!(state.snapshot.race_alert(now).is_none());
    }
    let penalty = RaceControl {
        penalty_seconds: 10,
        ..baseline
    };
    let now = feed(&mut state, start, 2, Some(0), Some(0), penalty);
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::TimePenalty(5)
    );
    let first = state.snapshot.race_alert(now).unwrap().received;
    for tick in 3..=53 {
        let now = feed(&mut state, start, tick, Some(0), Some(0), penalty);
        if tick < 52 {
            assert_eq!(state.snapshot.race_alert(now).unwrap().received, first);
        }
        if tick == 53 {
            assert!(state.snapshot.race_alert(now).is_none());
        }
    }
    let warned = RaceControl {
        warnings: 3,
        ..penalty
    };
    let now = feed(&mut state, start, 54, Some(0), Some(0), warned);
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::Warning(1)
    );
    let through = RaceControl {
        drive_through: 1,
        ..warned
    };
    let now = feed(&mut state, start, 55, Some(0), Some(0), through);
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::DriveThrough
    );
    let now = feed(&mut state, start, 56, Some(0), Some(0), warned);
    assert!(state.snapshot.race_alert(now).is_none());
    let stop = RaceControl {
        stop_go: 1,
        ..warned
    };
    let now = feed(&mut state, start, 57, Some(0), Some(0), stop);
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::StopGo
    );
    let now = feed(
        &mut state,
        start,
        64,
        Some(0),
        Some(0),
        RaceControl {
            penalty_seconds: 20,
            ..stop
        },
    );
    assert!(state.snapshot.race_alert(now).is_none());
}

#[test]
fn official_flag_events_deduplicate_even_when_another_event_shares_the_frame() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    let now = feed(
        &mut state,
        start,
        0,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    assert!(state.apply(event(0, *b"BUTN"), now));
    assert!(state.apply(event(0, *b"RDFL"), now));
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::RedFlag
    );
    assert!(!state.apply(event(0, *b"RDFL"), now + Duration::from_millis(10)));
    let now = feed(
        &mut state,
        start,
        1,
        Some(1),
        Some(0),
        RaceControl::default(),
    );
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::Flag(1)
    );
    assert!(state.apply(event(1, *b"CHQF"), now));
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::ChequeredFlag
    );
    assert!(!state.apply(event(0, *b"RDFL"), now));
    let now = feed(
        &mut state,
        start,
        2,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::ChequeredFlag
    );
    for tick in 3..=52 {
        feed(
            &mut state,
            start,
            tick,
            Some(0),
            Some(0),
            RaceControl::default(),
        );
    }
    assert!(
        state
            .snapshot
            .race_alert(start + Duration::from_millis(5200))
            .is_none()
    );
}

#[test]
fn alerts_clear_for_pause_session_end_malformed_sources_and_new_sessions() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    feed(
        &mut state,
        start,
        0,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    let now = feed(
        &mut state,
        start,
        1,
        Some(3),
        Some(0),
        RaceControl::default(),
    );
    state.invalidate_race_alerts(7);
    assert!(state.snapshot.race_alert(now).is_none());
    let now = feed(
        &mut state,
        start,
        2,
        Some(3),
        Some(0),
        RaceControl::default(),
    );
    assert!(state.snapshot.race_alert(now).is_some());
    let Packet::Session(h, mut session) = demo::sample(0.3, 118)[6] else {
        panic!()
    };
    session.paused = true;
    state.apply(
        Packet::Session(h, session),
        now + Duration::from_millis(100),
    );
    assert!(
        state
            .snapshot
            .race_alert(now + Duration::from_millis(100))
            .is_none()
    );
    feed(
        &mut state,
        start,
        4,
        Some(3),
        Some(0),
        RaceControl::default(),
    );
    let now = feed(
        &mut state,
        start,
        5,
        Some(3),
        Some(0),
        RaceControl::default(),
    );
    assert!(state.snapshot.race_alert(now).is_some());
    assert!(state.apply(event(5, *b"SEND"), now));
    assert!(state.snapshot.race_alert(now).is_none());
    let Packet::Telemetry(mut h, telemetry) = demo::sample(0.0, 1)[0] else {
        panic!()
    };
    h.session_uid += 1;
    state.apply(Packet::Telemetry(h, telemetry), now);
    assert!(state.snapshot.race_alerts.iter().all(Option::is_none));
}

#[test]
fn flashbacks_restarts_and_delayed_events_cannot_replay_old_notices() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    feed(
        &mut state,
        start,
        0,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    feed(
        &mut state,
        start,
        1,
        Some(3),
        Some(0),
        RaceControl::default(),
    );
    let Packet::Status(mut h, status) = demo::sample(0.0, 1)[1] else {
        panic!()
    };
    h.overall_frame = 112;
    state.apply(
        Packet::Status(h, status),
        start + Duration::from_millis(200),
    );
    assert_eq!(state.snapshot.counters.flashbacks, 1);
    assert!(state.snapshot.race_alerts.iter().all(Option::is_none));
    feed(
        &mut state,
        start,
        3,
        Some(0),
        Some(0),
        RaceControl::default(),
    );
    let now = feed(
        &mut state,
        start,
        4,
        Some(0),
        Some(2),
        RaceControl::default(),
    );
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::SafetyCar(2)
    );
    state.apply(event(3, *b"RDFL"), now);
    assert_eq!(
        state.snapshot.race_alert(now).unwrap().value,
        RaceAlert::SafetyCar(2)
    );
    state.apply(demo::sample(0.0, 1)[0], start + Duration::from_secs(4));
    assert_eq!(state.snapshot.counters.restarts, 1);
    assert!(state.snapshot.race_alerts.iter().all(Option::is_none));
}

#[test]
fn old_game_samples_do_not_trigger_notices_just_because_they_arrived_recently() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    for tick in 0..=1 {
        feed(
            &mut state,
            start,
            tick,
            Some(0),
            Some(0),
            RaceControl::default(),
        );
    }
    let now = start + Duration::from_millis(200);
    state.apply(demo::sample(3.0, 280)[0], now);
    for mut packet in demo::sample(0.2, 112) {
        match &mut packet {
            Packet::Status(_, s) => s.fia_flag = Some(3),
            Packet::Session(_, s) => s.safety_car = Some(1),
            Packet::Lap(_, l) => l.race_control.penalty_seconds = 5,
            _ => continue,
        }
        assert!(state.apply(packet, now));
        assert!(state.snapshot.race_alert(now).is_none());
    }
}
