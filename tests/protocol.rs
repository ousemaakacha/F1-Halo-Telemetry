use f1_halo_telemetry::{
    settings::*,
    telemetry::{state::*, *},
};
use std::time::{Duration, Instant};

fn wire(id: u8, player: u8, frame: u32) -> Vec<u8> {
    let size = match id {
        1 => 926,
        2 => 1399,
        3 => 45,
        4 => 1470,
        6 => 1448,
        7 => 1445,
        10 => 1133,
        11 => 1460,
        16 => 269,
        _ => 29,
    };
    let mut b = vec![0; size];
    b[0..2].copy_from_slice(&2026u16.to_le_bytes());
    b[2] = 25;
    b[5] = 1;
    b[6] = id;
    b[7..15].copy_from_slice(&42u64.to_le_bytes());
    b[15..19].copy_from_slice(&(frame as f32 / 60.0).to_le_bytes());
    b[19..23].copy_from_slice(&frame.to_le_bytes());
    b[23..27].copy_from_slice(&frame.to_le_bytes());
    b[27] = player;
    b[28] = 255;
    b
}

fn history_wire(player: u8, frame: u32, laps: &[(u32, u8)]) -> Vec<u8> {
    assert!(laps.len() <= 100);
    let mut bytes = wire(11, player, frame);
    bytes[29] = player;
    bytes[30] = laps.len() as u8;
    for (index, &(time, valid)) in laps.iter().enumerate() {
        let offset = 36 + index * 14;
        bytes[offset..offset + 4].copy_from_slice(&time.to_le_bytes());
        bytes[offset + 13] = valid;
    }
    bytes
}

fn lap_wire(player: u8, frame: u32, position: u8, behind: Option<(u8, u32)>) -> Vec<u8> {
    let mut bytes = wire(2, player, frame);
    let offset = 29 + 57 * player as usize;
    bytes[offset + 32] = position;
    bytes[offset + 33] = 3;
    bytes[offset + 45] = 2;
    bytes[offset + 14..offset + 16].copy_from_slice(&999u16.to_le_bytes());
    if let Some((index, gap)) = behind {
        assert_ne!(index, player);
        let offset = 29 + 57 * index as usize;
        bytes[offset + 32] = position + 1;
        bytes[offset + 45] = 2;
        bytes[offset + 14..offset + 16].copy_from_slice(&((gap % 60_000) as u16).to_le_bytes());
        bytes[offset + 16] = (gap / 60_000) as u8;
    }
    bytes
}

fn race_lap_wire(
    player: u8,
    frame: u32,
    ahead: Option<(u8, u32)>,
    behind: Option<(u8, u32)>,
) -> Vec<u8> {
    let mut bytes = lap_wire(player, frame, 6, behind);
    let local = 29 + 57 * player as usize;
    bytes[local + 44] = 4;
    if let Some((index, millis)) = ahead {
        assert_ne!(index, player);
        let offset = 29 + 57 * index as usize;
        bytes[offset + 32] = 5;
        bytes[offset + 44] = 4;
        bytes[offset + 45] = 2;
        bytes[local + 14..local + 16].copy_from_slice(&((millis % 60_000) as u16).to_le_bytes());
        bytes[local + 16] = (millis / 60_000) as u8;
    }
    if let Some((index, _)) = behind {
        bytes[29 + 57 * index as usize + 44] = 4;
    }
    bytes
}

fn race_sample(
    state: &mut TelemetryState,
    start: Instant,
    tick: u32,
    ahead: Option<(u8, u32)>,
    behind: Option<(u8, u32)>,
) -> Instant {
    let now = start + Duration::from_millis(u64::from(tick) * 100);
    let frame = 600 + tick * 6;
    let mut session = wire(1, 23, frame);
    session[29 + 6] = 10;
    for bytes in [
        session,
        wire(6, 23, frame),
        race_lap_wire(23, frame, ahead, behind),
    ] {
        assert!(state.apply(parse(&bytes).unwrap(), now));
    }
    now
}

fn track_state(now: Instant, player: u8, session_type: u8, driver_status: u8) -> TelemetryState {
    let mut session = wire(1, player, 60);
    session[29 + 6] = session_type;
    session[29 + 8] = 13;
    let mut lap = lap_wire(player, 60, 1, None);
    lap[29 + 57 * player as usize + 44] = driver_status;
    let mut state = TelemetryState::default();
    for bytes in [session, lap, wire(6, player, 60)] {
        assert!(state.apply(parse(&bytes).unwrap(), now));
    }
    state
}

#[test]
fn race_gaps_read_both_neighbours_with_their_identity_and_millisecond_precision() {
    for (player, ahead, behind) in [(0, 15, 23), (23, 0, 4), (10, 20, 0)] {
        for millis in [0, 1_234, 61_567, 15_359_999] {
            let bytes = race_lap_wire(player, 600, Some((ahead, millis)), Some((behind, 2_345)));
            let Packet::Lap(_, lap) = parse(&bytes).unwrap() else {
                panic!()
            };
            assert_eq!(
                lap.race_gaps,
                [
                    Some(CarGap {
                        car_index: ahead,
                        millis
                    }),
                    Some(CarGap {
                        car_index: behind,
                        millis: 2_345
                    }),
                ]
            );
            assert_eq!(lap.gap_behind_ms, Some(2_345));
        }
    }
}

#[test]
fn race_gaps_reject_inactive_ambiguous_and_invalid_neighbours_without_changing_legacy_rear_data() {
    let base = race_lap_wire(23, 600, Some((0, 1_234)), Some((4, 2_345)));
    for side in 0..2 {
        let index = if side == 0 { 0 } else { 4 };
        for field in [44, 45] {
            let mut bytes = base.clone();
            bytes[29 + 57 * index + field] = 0;
            let Packet::Lap(_, lap) = parse(&bytes).unwrap() else {
                panic!()
            };
            assert_eq!(lap.race_gaps[side], None);
            assert!(lap.race_gaps[1 - side].is_some());
        }
        let mut bytes = base.clone();
        let duplicate = 29 + 57 * 11;
        bytes[duplicate + 32] = if side == 0 { 5 } else { 7 };
        bytes[duplicate + 44] = 4;
        bytes[duplicate + 45] = 2;
        let Packet::Lap(_, lap) = parse(&bytes).unwrap() else {
            panic!()
        };
        assert_eq!(lap.race_gaps[side], None);
        assert_eq!(lap.gap_behind_ms, Some(2_345));
    }
    let mut bytes = base.clone();
    let local = 29 + 57 * 23;
    bytes[local + 14..local + 16].copy_from_slice(&60_000u16.to_le_bytes());
    let Packet::Lap(_, lap) = parse(&bytes).unwrap() else {
        panic!()
    };
    assert_eq!(lap.race_gaps[0], None);
    assert_eq!(lap.race_gaps[1].unwrap().millis, 2_345);
    assert_eq!(lap.gap_behind_ms, Some(2_345));
    for position in [0, 1, 24] {
        let mut bytes = base.clone();
        bytes[local + 32] = position;
        let Packet::Lap(_, lap) = parse(&bytes).unwrap() else {
            panic!()
        };
        assert_eq!(lap.race_gaps, [None; 2]);
    }
}

#[test]
fn race_gap_trends_need_a_full_second_and_ignore_small_variations() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    for tick in 0..=24 {
        let ahead = if tick <= 12 {
            3_000 - tick * 20
        } else {
            2_800 + tick % 2 * 10
        };
        let behind = if tick <= 12 {
            2_000 + tick * 20
        } else {
            2_200 + tick % 2 * 10
        };
        let now = race_sample(&mut state, start, tick, Some((0, ahead)), Some((4, behind)));
        let gaps = state.snapshot.race_gaps(now).map(Option::unwrap);
        assert_eq!([gaps[0].millis, gaps[1].millis], [ahead, behind]);
        if tick < 10 || tick == 24 {
            assert_eq!(gaps.map(|gap| gap.trend), [GapTrend::Neutral; 2]);
        } else if tick == 12 {
            assert_eq!(gaps[0].trend, GapTrend::Shrinking);
            assert_eq!(gaps[1].trend, GapTrend::Growing);
        }
    }
}

#[test]
fn race_gap_history_resets_after_missing_data_opponent_changes_pause_and_silence() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    for tick in 0..=12 {
        race_sample(
            &mut state,
            start,
            tick,
            Some((0, 3_000 - tick * 20)),
            Some((4, 2_000)),
        );
    }
    for (tick, ahead) in [(13, Some((1, 900))), (14, None), (15, Some((1, 700)))] {
        let now = race_sample(&mut state, start, tick, ahead, Some((4, 2_000)));
        assert_eq!(
            state.snapshot.race_gaps(now)[0].map(|gap| gap.trend),
            ahead.map(|_| GapTrend::Neutral)
        );
    }
    let mut paused = wire(1, 23, 696);
    paused[29 + 6] = 10;
    paused[29 + 14] = 1;
    let now = start + Duration::from_millis(1_600);
    assert!(state.apply(parse(&paused).unwrap(), now));
    assert_eq!(state.snapshot.race_gaps(now), [None; 2]);
    for tick in [17, 23] {
        let now = race_sample(&mut state, start, tick, Some((1, 400)), Some((4, 2_000)));
        assert_eq!(
            state.snapshot.race_gaps(now).map(|gap| gap.unwrap().trend),
            [GapTrend::Neutral; 2]
        );
    }
    let expired = start + Duration::from_millis(2_801);
    assert_eq!(state.snapshot.race_gaps(expired), [None; 2]);
    assert_eq!(state.snapshot.freshness_mask(expired) & (1 << 10), 0);
}

#[test]
fn race_gap_trends_do_not_use_duplicate_frames_frozen_time_or_old_timelines() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    for tick in 0..=12 {
        race_sample(
            &mut state,
            start,
            tick,
            Some((0, 3_000 - tick * 20)),
            Some((4, 2_000)),
        );
    }
    let now = start + Duration::from_millis(1_300);
    for frame in [672, 671] {
        let bytes = race_lap_wire(23, frame, Some((0, 9_000)), None);
        assert!(!state.apply(parse(&bytes).unwrap(), now));
        assert_eq!(state.snapshot.race_gaps(now)[0].unwrap().millis, 2_760);
    }
    let mut frozen = race_lap_wire(23, 678, Some((0, 2_000)), Some((4, 2_000)));
    frozen[15..19].copy_from_slice(&(672.0f32 / 60.0).to_le_bytes());
    assert!(state.apply(parse(&frozen).unwrap(), now));
    assert_eq!(
        state.snapshot.race_gaps(now)[0].unwrap().trend,
        GapTrend::Neutral
    );
    let mut rollback = wire(6, 23, 600);
    rollback[23..27].copy_from_slice(&679u32.to_le_bytes());
    assert!(state.apply(parse(&rollback).unwrap(), now));
    assert!(state.snapshot.race_gaps.is_none());
    let now = race_sample(&mut state, start, 14, Some((0, 900)), Some((4, 2_000)));
    assert_eq!(
        state.snapshot.race_gaps(now)[0].unwrap().trend,
        GapTrend::Neutral
    );
    state.invalidate_race_gaps();
    assert_eq!(state.snapshot.race_gaps(now), [None; 2]);
    let now = race_sample(&mut state, start, 15, Some((0, 500)), Some((4, 2_000)));
    assert_eq!(
        state.snapshot.race_gaps(now)[0].unwrap().trend,
        GapTrend::Neutral
    );
}

#[test]
fn visibility_reads_session_and_local_driver_fields_and_validates_booleans() {
    for player in [0, 23] {
        let mut session = wire(1, player, 60);
        session[35] = 18;
        session[37] = 13;
        session[43] = 1;
        session[44] = 1;
        let Packet::Session(_, data) = parse(&session).unwrap() else {
            panic!()
        };
        assert_eq!(
            data,
            SessionData {
                paused: true,
                formula: 13,
                session_type: 18,
                spectating: true,
                safety_car: Some(0),
            }
        );
        for (offset, field) in [(43, "gamePaused"), (44, "isSpectating")] {
            let mut invalid = session.clone();
            invalid[offset] = 2;
            assert_eq!(parse(&invalid), Err(ParseError::InvalidValue(field)));
        }
        let mut lap = lap_wire(player, 60, 1, None);
        lap[29 + 57 * ((player as usize + 1) % 24) + 44] = 255;
        lap[29 + 57 * player as usize + 44] = 3;
        let Packet::Lap(_, data) = parse(&lap).unwrap() else {
            panic!()
        };
        assert_eq!(data.driver_status, 3);
        lap[29 + 57 * player as usize + 44] = 5;
        assert_eq!(parse(&lap), Err(ParseError::InvalidValue("driverStatus")));
    }
}

#[test]
fn automatic_visibility_accepts_practice_qualifying_sprints_races_and_time_trial() {
    let now = Instant::now();
    for player in [0, 23] {
        for session in 1..=18 {
            for driver in 1..=4 {
                assert!(
                    track_state(now, player, session, driver)
                        .snapshot
                        .on_track(now)
                );
            }
        }
    }
    assert!(!Snapshot::default().on_track(now));
    for session in [0, 19, 255] {
        assert!(!track_state(now, 23, session, 4).snapshot.on_track(now));
    }
    assert!(!track_state(now, 23, 18, 0).snapshot.on_track(now));
}

#[test]
fn automatic_visibility_hides_for_pause_spectating_garage_results_and_missing_packets() {
    let now = Instant::now();
    let snapshot = track_state(now, 23, 15, 4).snapshot;
    let mut variants = Vec::new();
    let mut changed = snapshot;
    changed.session.as_mut().unwrap().value.paused = true;
    variants.push(changed);
    changed = snapshot;
    changed.session.as_mut().unwrap().value.spectating = true;
    variants.push(changed);
    changed = snapshot;
    changed.driver_status.as_mut().unwrap().value = 0;
    variants.push(changed);
    changed = snapshot;
    changed.status = Some(Timed {
        value: CarStatus {
            paused: true,
            ..CarStatus::default()
        },
        received: now,
        overall_frame: 60,
    });
    variants.push(changed);
    for result in [0, 1, 3, 4, 5, 6, 7] {
        changed = snapshot;
        changed.lap_status.as_mut().unwrap().value = result;
        variants.push(changed);
    }
    for missing in 0..4 {
        changed = snapshot;
        match missing {
            0 => changed.telemetry = None,
            1 => changed.session = None,
            2 => changed.lap_status = None,
            _ => changed.driver_status = None,
        }
        variants.push(changed);
    }
    for source in [Connection::Unsupported, Connection::BindError] {
        changed = snapshot;
        changed.source = source;
        variants.push(changed);
    }
    for snapshot in variants {
        assert!(!snapshot.on_track(now), "{snapshot:?}");
    }
}

#[test]
fn automatic_visibility_expires_packet_families_and_recovers_after_menus_and_session_end() {
    let now = Instant::now();
    let mut state = track_state(now, 23, 18, 4);
    assert!(
        !state
            .snapshot
            .on_track(now + FAST_TTL + Duration::from_millis(1))
    );
    let later = now + Duration::from_millis(600);
    state.apply(parse(&wire(6, 23, 100)).unwrap(), later);
    assert!(!state.snapshot.on_track(later));
    let mut lap = lap_wire(23, 100, 1, None);
    lap[29 + 57 * 23 + 44] = 4;
    state.apply(parse(&lap).unwrap(), later);
    assert!(
        state.snapshot.on_track(later),
        "2 Hz Session stays fresh between telemetry packets"
    );
    let later = now + DAMAGE_TTL + Duration::from_millis(1);
    state.apply(parse(&wire(6, 23, 200)).unwrap(), later);
    lap[19..23].copy_from_slice(&200u32.to_le_bytes());
    lap[23..27].copy_from_slice(&200u32.to_le_bytes());
    lap[15..19].copy_from_slice(&(200f32 / 60.0).to_le_bytes());
    state.apply(parse(&lap).unwrap(), later);
    assert!(
        !state.snapshot.on_track(later),
        "Fresh telemetry cannot keep stale session metadata visible"
    );
    let mut session = wire(1, 23, 200);
    session[35] = 18;
    state.apply(parse(&session).unwrap(), later);
    assert!(state.snapshot.on_track(later));
    let mut event = wire(3, 23, 201);
    event[29..33].copy_from_slice(b"SEND");
    state.apply(parse(&event).unwrap(), later);
    assert!(!state.snapshot.on_track(later));
    assert!(state.snapshot.session.is_none());
    assert!(state.snapshot.driver_status.is_none());
    event = wire(3, 23, 202);
    event[29..33].copy_from_slice(b"SSTA");
    state.apply(parse(&event).unwrap(), later);
    assert!(!state.snapshot.on_track(later));
    for mut bytes in [
        wire(1, 23, 203),
        lap_wire(23, 203, 1, None),
        wire(6, 23, 203),
    ] {
        match bytes[6] {
            1 => bytes[35] = 18,
            2 => bytes[29 + 57 * 23 + 44] = 4,
            _ => (),
        }
        state.apply(parse(&bytes).unwrap(), later);
    }
    assert!(state.snapshot.on_track(later));
    let mut changed_session = wire(6, 23, 1);
    changed_session[7..15].copy_from_slice(&43u64.to_le_bytes());
    state.apply(parse(&changed_session).unwrap(), later);
    assert!(!state.snapshot.on_track(later));
}

#[test]
fn rear_gap_reads_the_following_race_position_not_array_order_or_our_front_gap() {
    for (player, behind) in [(0, 23), (23, 0), (10, 4)] {
        for gap in [0, 1_234, 61_567, 15_359_999] {
            let mut bytes = lap_wire(player, 600, 6, Some((behind, gap)));
            bytes[29 + 57 * player as usize + 37] = 1;
            let Packet::Lap(header, lap) = parse(&bytes).unwrap() else {
                panic!()
            };
            assert_eq!(header.player, player);
            assert_eq!(lap.result_status, 2);
            assert_eq!(lap.current_lap, 3);
            assert_eq!(lap.gap_behind_ms, Some(gap));
        }
    }
}

#[test]
fn rear_gap_is_unavailable_without_an_active_following_car_and_valid_position() {
    let base = lap_wire(23, 600, 6, Some((4, 1_234)));
    let gap = |bytes: &[u8]| {
        let Packet::Lap(_, lap) = parse(bytes).unwrap() else {
            panic!()
        };
        lap.gap_behind_ms
    };
    for result in [0, 1, 3, 4, 5, 6, 7] {
        let mut bytes = base.clone();
        bytes[29 + 57 * 4 + 45] = result;
        assert_eq!(gap(&bytes), None);
        let mut bytes = base.clone();
        bytes[29 + 57 * 23 + 45] = result;
        assert_eq!(gap(&bytes), None);
    }
    for position in [0, 24] {
        let mut bytes = base.clone();
        bytes[29 + 57 * 23 + 32] = position;
        assert_eq!(gap(&bytes), None);
    }
    let mut bytes = base.clone();
    bytes[29 + 57 * 4 + 32] = 8;
    assert_eq!(gap(&bytes), None);
    assert_eq!(gap(&lap_wire(23, 600, 6, None)), None);
    let mut bytes = base.clone();
    bytes[29 + 57 * 23 + 32] = 25;
    assert_eq!(parse(&bytes), Err(ParseError::InvalidValue("carPosition")));
    for invalid in [60_000u16, u16::MAX] {
        let mut bytes = base.clone();
        let offset = 29 + 57 * 4 + 14;
        bytes[offset..offset + 2].copy_from_slice(&invalid.to_le_bytes());
        assert_eq!(
            parse(&bytes),
            Err(ParseError::InvalidValue("deltaToCarInFrontMSPart"))
        );
    }
}

#[test]
fn rear_gap_updates_live_and_clears_on_expiry_missing_car_pause_or_timeline_reset() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(6, 23, 100)).unwrap(), now);
    state.apply(parse(&lap_wire(23, 100, 6, Some((4, 1_800)))).unwrap(), now);
    assert_eq!(state.snapshot.gap_behind_ms(now), Some(1_800));
    state.apply(parse(&lap_wire(23, 101, 6, Some((4, 900)))).unwrap(), now);
    assert_eq!(state.snapshot.gap_behind_ms(now), Some(900));
    assert!(!state.apply(parse(&lap_wire(23, 100, 6, Some((4, 1_800)))).unwrap(), now));
    assert_eq!(state.snapshot.gap_behind_ms(now), Some(900));
    state.apply(parse(&wire(6, 23, 102)).unwrap(), now + FAST_TTL);
    assert_eq!(state.snapshot.gap_behind_ms(now + FAST_TTL), Some(900));
    let expired = now + FAST_TTL + Duration::from_millis(1);
    assert_eq!(state.snapshot.gap_behind_ms(expired), None);
    assert_eq!(state.snapshot.freshness_mask(expired) & (1 << 9), 0);
    state.apply(parse(&lap_wire(23, 102, 6, None)).unwrap(), expired);
    assert!(state.snapshot.gap_behind.is_none());
    state.apply(
        parse(&lap_wire(23, 103, 6, Some((0, 2_100)))).unwrap(),
        expired,
    );
    assert_eq!(state.snapshot.gap_behind_ms(expired), Some(2_100));
    let mut paused = wire(1, 23, 103);
    paused[29 + 14] = 1;
    state.apply(parse(&paused).unwrap(), expired);
    assert_eq!(state.snapshot.gap_behind_ms(expired), None);
    let mut rollback = wire(6, 23, 50);
    rollback[23..27].copy_from_slice(&104u32.to_le_bytes());
    state.apply(parse(&rollback).unwrap(), expired);
    assert!(state.snapshot.gap_behind.is_none());
    state.apply(
        parse(&lap_wire(23, 105, 6, Some((0, 700)))).unwrap(),
        expired,
    );
    assert_eq!(state.snapshot.gap_behind_ms(expired), Some(700));
    let mut new_session = wire(6, 23, 1);
    new_session[7..15].copy_from_slice(&43u64.to_le_bytes());
    state.apply(parse(&new_session).unwrap(), expired);
    assert!(state.snapshot.gap_behind.is_none());
}

#[test]
fn history_compares_last_completed_lap_with_previous_valid_personal_best() {
    for player in [0, 23] {
        for (last, delta) in [(90_230, 230), (89_880, -120), (90_000, 0)] {
            let bytes = history_wire(player, 600, &[(90_000, 15), (last, 15), (1_000, 15)]);
            let Packet::History(header, history) = parse(&bytes).unwrap() else {
                panic!()
            };
            assert_eq!(header.player, player);
            assert_eq!(history.completed_laps, 2);
            assert_eq!(history.last_lap_ms, Some(last));
            assert_eq!(history.best_before_last_ms, Some(90_000));
            assert_eq!(history.delta_ms(), Some(delta));
        }
        for laps in [
            vec![],
            vec![(10_000, 15)],
            vec![(90_000, 15), (10_000, 15)],
            vec![(90_000, 15), (89_880, 14), (10_000, 15)],
            vec![(90_000, 15), (0, 15), (10_000, 15)],
        ] {
            let Packet::History(_, history) = parse(&history_wire(player, 600, &laps)).unwrap()
            else {
                panic!()
            };
            assert_eq!(history.delta_ms(), None);
        }
        let bytes = history_wire(
            player,
            600,
            &[(80_000, 14), (90_000, 15), (90_230, 15), (1_000, 15)],
        );
        let Packet::History(_, history) = parse(&bytes).unwrap() else {
            panic!()
        };
        assert_eq!(history.delta_ms(), Some(230));
    }
    let mut laps = [(0, 0); 100];
    laps[0] = (90_000, 15);
    laps[98] = (90_230, 15);
    laps[99] = (1_000, 15);
    let Packet::History(_, history) = parse(&history_wire(23, 600, &laps)).unwrap() else {
        panic!()
    };
    assert_eq!(history.completed_laps, 99);
    assert_eq!(history.delta_ms(), Some(230));
}

#[test]
fn history_ignores_other_cars_and_rejects_invalid_counts_and_flags() {
    let bytes = history_wire(23, 600, &[(90_000, 15), (90_230, 15), (0, 0)]);
    let mut other = bytes.clone();
    other[29] = 22;
    assert!(matches!(parse(&other), Ok(Packet::Ignored(_))));
    for (offset, invalid) in [
        (29, 24),
        (30, 101),
        (31, 9),
        (32, 4),
        (35, 4),
        (49, 16),
        (77, 16),
    ] {
        let mut malformed = bytes.clone();
        malformed[offset] = invalid;
        assert!(
            parse(&malformed).is_err(),
            "Accepted history byte {offset}={invalid}"
        );
    }
}

#[test]
fn lap_delta_waits_for_the_new_completed_lap_and_clears_when_unavailable() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(6, 23, 100)).unwrap(), now);
    state.apply(
        parse(&history_wire(
            23,
            100,
            &[(90_000, 15), (90_230, 15), (1_000, 15)],
        ))
        .unwrap(),
        now,
    );
    assert_eq!(state.snapshot.lap_delta_ms(now), Some(230));
    let mut lap = wire(2, 23, 101);
    let offset = 29 + 57 * 23;
    lap[offset + 45] = 2;
    lap[offset + 33] = 3;
    state.apply(parse(&lap).unwrap(), now);
    assert_eq!(state.snapshot.lap_delta_ms(now), Some(230));
    lap[19..23].copy_from_slice(&102u32.to_le_bytes());
    lap[23..27].copy_from_slice(&102u32.to_le_bytes());
    lap[offset + 33] = 4;
    state.apply(parse(&lap).unwrap(), now);
    assert_eq!(state.snapshot.lap_delta_ms(now), None);
    state.apply(
        parse(&history_wire(
            23,
            103,
            &[(90_000, 15), (90_230, 15), (89_880, 15), (1_000, 15)],
        ))
        .unwrap(),
        now,
    );
    assert_eq!(state.snapshot.lap_delta_ms(now), Some(-120));
    state.apply(parse(&wire(6, 23, 104)).unwrap(), now + HISTORY_TTL);
    assert_eq!(state.snapshot.lap_delta_ms(now + HISTORY_TTL), Some(-120));
    let expired = now + HISTORY_TTL + Duration::from_millis(1);
    assert_eq!(state.snapshot.lap_delta_ms(expired), None);
    assert_eq!(state.snapshot.freshness_mask(expired) & (1 << 8), 0);
    let mut new_session = wire(6, 23, 1);
    new_session[7..15].copy_from_slice(&43u64.to_le_bytes());
    state.apply(parse(&new_session).unwrap(), expired);
    assert!(state.snapshot.lap_history.is_none());
    assert!(state.snapshot.current_lap.is_none());
    assert_eq!(state.snapshot.lap_delta_ms(expired), None);
}

#[test]
fn reads_independent_last_car_fixture_with_exact_2026_offsets() {
    let mut b = wire(6, 23, 600);
    let o = 29 + 59 * 23;
    b[o..o + 2].copy_from_slice(&327u16.to_le_bytes());
    b[o + 2..o + 6].copy_from_slice(&0.75f32.to_le_bytes());
    b[o + 10..o + 14].copy_from_slice(&0.25f32.to_le_bytes());
    b[o + 15] = 255;
    b[o + 16..o + 18].copy_from_slice(&11800u16.to_le_bytes());
    b[o + 20..o + 22].copy_from_slice(&0x4001u16.to_le_bytes());
    b[o + 30..o + 34].copy_from_slice(&[91, 92, 93, 94]);
    b[o + 34..o + 38].copy_from_slice(&[81, 82, 83, 84]);
    let Packet::Telemetry(h, c) = parse(&b).unwrap() else {
        panic!()
    };
    assert_eq!((h.player, h.session_uid, h.overall_frame), (23, 42, 600));
    assert_eq!(
        (c.speed_kmh, c.gear, c.rpm, c.rev_bits),
        (327, -1, 11800, 0x4001)
    );
    assert_eq!((c.throttle, c.brake), (0.75, 0.25));
    assert_eq!(c.surface_c, [91, 92, 93, 94]);
    assert_eq!(c.inner_c, [81, 82, 83, 84]);
}

#[test]
fn status_offsets_include_new_harvest_limit() {
    let mut b = wire(7, 23, 99);
    let o = 29 + 59 * 23;
    b[o + 13..o + 17].copy_from_slice(&2.3f32.to_le_bytes());
    b[o + 17..o + 19].copy_from_slice(&15000u16.to_le_bytes());
    b[o + 37..o + 41].copy_from_slice(&2_000_000f32.to_le_bytes());
    b[o + 41] = 3;
    b[o + 50..o + 54].copy_from_slice(&8_500_000f32.to_le_bytes());
    let Packet::Status(_, s) = parse(&b).unwrap() else {
        panic!()
    };
    assert_eq!(s.max_rpm, 15000);
    assert_eq!(s.ers_joules, 2_000_000.0);
    assert_eq!(s.harvest_limit_joules, 8_500_000.0);
    assert_eq!(s.deploy_mode, 3);
    assert_eq!(s.fuel_remaining_laps, 2.3);
    b[o + 13..o + 17].copy_from_slice(&(-0.8f32).to_le_bytes());
    let Packet::Status(_, s) = parse(&b).unwrap() else {
        panic!()
    };
    assert_eq!(s.fuel_remaining_laps, -0.8);
}

#[test]
fn lap_energy_counters_use_mguk_and_deployed_offsets_and_reject_invalid_values() {
    let mut bytes = wire(7, 23, 600);
    let offset = 29 + 59 * 23;
    for (field, value) in [
        (37, 1_400_000.0_f32),
        (42, 3_000_000.0),
        (46, 99_000_000.0),
        (50, 8_500_000.0),
        (54, 6_000_000.0),
    ] {
        bytes[offset + field..offset + field + 4].copy_from_slice(&value.to_le_bytes());
    }
    let Packet::Status(_, status) = parse(&bytes).unwrap() else {
        panic!()
    };
    assert_eq!(status.ers_joules, 1_400_000.0);
    assert_eq!(status.harvested_mguk_joules, 3_000_000.0);
    assert_eq!(status.harvest_limit_joules, 8_500_000.0);
    assert_eq!(status.deployed_joules, 6_000_000.0);
    for field in [42, 54] {
        for invalid in [-1.0_f32, f32::NAN, f32::INFINITY] {
            let mut invalid_bytes = bytes.clone();
            invalid_bytes[offset + field..offset + field + 4]
                .copy_from_slice(&invalid.to_le_bytes());
            assert!(parse(&invalid_bytes).is_err());
        }
    }
    let mut state = TelemetryState::default();
    let now = Instant::now();
    state.apply(parse(&bytes).unwrap(), now);
    state.snapshot.ers_deploy_limit_joules = Some(13_000_000.0);
    let mut new_session = wire(6, 23, 1);
    new_session[7..15].copy_from_slice(&43u64.to_le_bytes());
    state.apply(parse(&new_session).unwrap(), now);
    assert!(state.snapshot.ers_deploy_limit_joules.is_none());
    assert!(state.snapshot.status.is_none());
}

#[test]
fn fuel_margin_accepts_signed_values_but_rejects_non_finite_udp_data() {
    for player in [0, 23] {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut b = wire(7, player, 1);
            let o = 29 + 59 * player as usize + 13;
            b[o..o + 4].copy_from_slice(&value.to_le_bytes());
            assert_eq!(
                parse(&b),
                Err(ParseError::InvalidValue("fuelRemainingLaps"))
            );
        }
    }
}

#[test]
fn aero_is_independent_of_legacy_drs() {
    let mut b = wire(16, 23, 20);
    let o = 29 + 10 * 23;
    b[o] = 1;
    b[o + 1] = 1;
    b[o + 2..o + 4].copy_from_slice(&350u16.to_le_bytes());
    b[o + 4] = 1;
    b[o + 5] = 1;
    b[o + 8] = 1;
    b[o + 6..o + 8].copy_from_slice(&123u16.to_le_bytes());
    let Packet::Aero(_, a) = parse(&b).unwrap() else {
        panic!()
    };
    assert!(
        a.straight
            && a.available
            && a.overtake_active
            && a.overtake_available
            && a.regulations_2026
    );
    assert_eq!(a.overtake_distance_m, 123);
    assert_eq!(a.activation_distance_m, 350);
}

#[test]
fn damage_wear_is_not_remaining_life() {
    let mut b = wire(10, 0, 1);
    for (i, wear) in [0f32, 25.0, 50.0, 100.0].iter().enumerate() {
        b[29 + i * 4..33 + i * 4].copy_from_slice(&wear.to_le_bytes());
    }
    let Packet::Damage(_, d) = parse(&b).unwrap() else {
        panic!()
    };
    assert_eq!(d.wear, [0.0, 25.0, 50.0, 100.0]);
}

#[test]
fn wheel_mapping_front_then_rear_is_explicit() {
    let wire = [11, 22, 33, 44];
    assert_eq!(
        Wheel::VISUAL_ORDER.map(|w| wire[w as usize]),
        [33, 44, 11, 22]
    );
    assert_eq!(
        Wheel::VISUAL_ORDER.map(Wheel::label),
        ["FL", "FR", "RL", "RR"]
    );
}

#[test]
fn conversions_and_ers_require_known_capacity() {
    assert_eq!(pedal_percent(0.75), 75.0);
    assert_eq!(gear_label(-1), "R");
    assert_eq!(gear_label(0), "N");
    assert_eq!(gear_label(8), "8");
    assert_eq!(ers_percent(2_000_000.0, Some(4_000_000.0)), Some(50.0));
    assert_eq!(ers_percent(0.0, Some(4_000_000.0)), Some(0.0));
    assert_eq!(ers_percent(2_000_000.0, None), None);
    assert_eq!(ers_percent(1.0, Some(0.0)), None);
    assert_eq!(ers_percent(f32::NAN, Some(4_000_000.0)), None);
}

#[test]
fn rejects_every_truncation_and_trailing_bytes() {
    for id in [1, 2, 3, 4, 6, 7, 10, 11, 16] {
        let b = wire(id, 0, 1);
        for n in 0..b.len() {
            assert!(parse(&b[..n]).is_err(), "id={id}, n={n}");
        }
        let mut extra = b.clone();
        extra.push(0);
        assert!(parse(&extra).is_err());
        assert!(parse(&b).is_ok());
    }
}

#[test]
fn rejects_wrong_format_version_and_player() {
    let mut b = wire(6, 0, 1);
    b[0..2].copy_from_slice(&2025u16.to_le_bytes());
    assert_eq!(parse(&b), Err(ParseError::Format(2025)));
    b[0..2].copy_from_slice(&2026u16.to_le_bytes());
    b[5] = 2;
    assert_eq!(parse(&b), Err(ParseError::Version(2)));
    b[5] = 1;
    b[27] = 24;
    assert_eq!(parse(&b), Err(ParseError::Player(24)));
    b[27] = 255;
    assert!(matches!(parse(&b), Ok(Packet::Ignored(_))));
}

#[test]
fn rejects_non_finite_and_out_of_range_values() {
    for value in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
        let mut b = wire(6, 0, 1);
        b[31..35].copy_from_slice(&value.to_le_bytes());
        assert!(matches!(
            parse(&b),
            Err(ParseError::InvalidValue("throttle"))
        ));
    }
    let mut b = wire(16, 0, 1);
    b[29] = 2;
    assert!(parse(&b).is_err());
    let mut b = wire(10, 0, 1);
    b[29..33].copy_from_slice(&101f32.to_le_bytes());
    assert!(parse(&b).is_err());
}

#[test]
fn arbitrary_datagrams_never_panic() {
    let mut seed = 0x12345678u32;
    let mut bytes = [0u8; 1500];
    for round in 0..4000 {
        for b in &mut bytes {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            *b = seed as u8;
        }
        if round % 2 == 0 {
            bytes[0..2].copy_from_slice(&2026u16.to_le_bytes());
        }
        let _ = parse(&bytes[..(seed as usize % 1501)]);
    }
}

#[test]
fn demo_packets_round_trip_without_changing_data() {
    let mut bytes = [0u8; protocol::PARTICIPANTS_SIZE];
    for seconds in [0.0, 4.0, 8.0, 9.99, 10.0, 12.0, 16.0, 18.0, 1028.0] {
        for packet in demo::sample(seconds, 720) {
            let n = demo::encode(packet, &mut bytes);
            assert_eq!(parse(&bytes[..n]).unwrap(), packet);
        }
    }
}

#[test]
fn participant_names_use_the_documented_stride_and_preserve_utf8_without_trusting_bad_text() {
    let mut bytes = wire(4, 23, 600);
    bytes[29] = 2;
    let local = 30 + 60 * 23 + 10;
    bytes[local..local + 6].copy_from_slice(b"NORRIS");
    let other = 30 + 60 * 4 + 10;
    let name = "LÉCLERC".as_bytes();
    bytes[other..other + name.len()].copy_from_slice(name);
    let Packet::Participants(_, participants) = parse(&bytes).unwrap() else {
        panic!()
    };
    assert_eq!(participants.active_cars, 2);
    assert_eq!(participants.name(23), Some("NORRIS"));
    assert_eq!(participants.name(4), Some("LÉCLERC"));
    assert_eq!(participants.name(0), None);
    assert_eq!(participants.name(24), None);
    for invalid in [0xff, b'\n'] {
        bytes[other] = invalid;
        let Packet::Participants(_, participants) = parse(&bytes).unwrap() else {
            panic!()
        };
        assert_eq!(participants.name(4), None);
        assert_eq!(participants.name(23), Some("NORRIS"));
    }
    bytes[local..local + 32].fill(b'A');
    let Packet::Participants(_, participants) = parse(&bytes).unwrap() else {
        panic!()
    };
    assert_eq!(
        participants.name(23),
        Some("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
    );
    assert!(parse(&bytes[..bytes.len() - 1]).is_err());
    bytes[29] = 25;
    assert!(parse(&bytes).is_err());
}

#[test]
fn replica_standings_follow_race_order_and_names_expire_and_clear_on_session_change() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    let now = race_sample(&mut state, start, 0, Some((0, 500)), Some((4, 2000)));
    let mut bytes = wire(4, 23, 600);
    bytes[29] = 3;
    for (index, name) in [(23, "NORRIS"), (4, "STROLL"), (7, "LECLERC")] {
        let offset = 30 + 60 * index + 10;
        bytes[offset..offset + name.len()].copy_from_slice(name.as_bytes());
    }
    state.apply(parse(&bytes).unwrap(), now);
    assert_eq!(state.snapshot.race_position.unwrap().value, 6);
    assert_eq!(state.snapshot.following_car.unwrap().value, 4);
    assert_eq!(state.snapshot.participant_name(23, now), Some("NORRIS"));
    assert_eq!(state.snapshot.participant_name(4, now), Some("STROLL"));
    let now = race_sample(&mut state, start, 1, Some((0, 500)), Some((7, 1000)));
    assert_eq!(state.snapshot.following_car.unwrap().value, 7);
    assert_eq!(state.snapshot.participant_name(7, now), Some("LECLERC"));
    assert_eq!(
        state
            .snapshot
            .participant_name(23, now + FAST_TTL + Duration::from_millis(1)),
        None
    );
    let expired = start + PARTICIPANTS_TTL + Duration::from_millis(1);
    state.apply(parse(&wire(6, 23, 1300)).unwrap(), expired);
    assert_eq!(state.snapshot.participant_name(23, expired), None);
    let mut new_session = wire(6, 23, 1);
    new_session[7..15].copy_from_slice(&43u64.to_le_bytes());
    state.apply(parse(&new_session).unwrap(), expired);
    assert!(state.snapshot.driver_names.is_none());
    assert!(state.snapshot.race_position.is_none());
    assert!(state.snapshot.preceding_car.is_none());
    assert!(state.snapshot.following_car.is_none());
}

#[test]
fn f1_tv_opponents_pair_each_name_with_its_own_gap_and_clear_unavailable_rows() {
    let start = Instant::now();
    let mut state = TelemetryState::default();
    let now = race_sample(&mut state, start, 0, Some((0, 1234)), Some((4, 832)));
    let mut bytes = wire(4, 23, 600);
    bytes[29] = 4;
    for (index, name) in [(23, "NORRIS"), (0, "PIASTRI"), (4, "STROLL"), (7, "ALONSO")] {
        let offset = 30 + 60 * index + 10;
        bytes[offset..offset + name.len()].copy_from_slice(name.as_bytes());
    }
    state.apply(parse(&bytes).unwrap(), now);
    let opponents = state.snapshot.race_opponents(now);
    assert_eq!(
        opponents,
        [
            Some(CarGap {
                car_index: 0,
                millis: 1234
            }),
            Some(CarGap {
                car_index: 4,
                millis: 832
            })
        ]
    );
    assert_eq!(
        opponents.map(|opponent| state
            .snapshot
            .participant_name(opponent.unwrap().car_index, now)),
        [Some("PIASTRI"), Some("STROLL")]
    );
    assert_eq!(
        state
            .snapshot
            .race_opponents(now + FAST_TTL + Duration::from_millis(1)),
        [None; 2]
    );
    let mut paused = state.snapshot;
    paused.session.as_mut().unwrap().value.paused = true;
    assert_eq!(paused.race_opponents(now), [None; 2]);
    let now = race_sample(&mut state, start, 1, Some((7, 3210)), Some((4, 500)));
    let ahead = state.snapshot.race_opponents(now)[0].unwrap();
    assert_eq!(
        ahead,
        CarGap {
            car_index: 7,
            millis: 3210
        }
    );
    assert_eq!(
        state.snapshot.participant_name(ahead.car_index, now),
        Some("ALONSO")
    );
    let now = race_sample(&mut state, start, 2, None, Some((4, 0)));
    assert_eq!(
        state.snapshot.race_opponents(now),
        [
            None,
            Some(CarGap {
                car_index: 4,
                millis: 0
            })
        ]
    );
    assert!(state.snapshot.preceding_car.is_none());
    state.invalidate_race_gaps();
    assert_eq!(state.snapshot.race_opponents(now), [None; 2]);
    assert!(state.snapshot.following_car.is_none());
}

#[test]
fn frame_sequence_is_per_packet_type() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    assert!(state.apply(parse(&wire(6, 0, 100)).unwrap(), now));
    assert!(state.apply(parse(&wire(7, 0, 100)).unwrap(), now));
    assert!(!state.apply(parse(&wire(6, 0, 100)).unwrap(), now));
    assert!(!state.apply(parse(&wire(6, 0, 99)).unwrap(), now));
    assert_eq!(state.snapshot.counters.duplicates, 1);
    assert_eq!(state.snapshot.counters.reordered, 1);
    assert!(state.snapshot.telemetry.is_some() && state.snapshot.status.is_some());
}

#[test]
fn wrapping_sequence_is_supported() {
    assert!(serial_newer(0, u32::MAX));
    assert!(!serial_newer(u32::MAX, 0));
    assert!(!serial_newer(1, 1));
}

#[test]
fn session_change_clears_other_packet_families_and_rejects_retired_uid() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(7, 0, 5)).unwrap(), now);
    let mut new = wire(6, 0, 1);
    new[7..15].copy_from_slice(&43u64.to_le_bytes());
    assert!(state.apply(parse(&new).unwrap(), now));
    assert!(state.snapshot.status.is_none());
    assert!(!state.apply(parse(&wire(7, 0, 6)).unwrap(), now));
    assert_eq!(state.snapshot.header.unwrap().session_uid, 43);
}

#[test]
fn flashback_clears_slow_data_and_rejects_late_old_timeline() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(
        parse(&history_wire(0, 500, &[(90_000, 15), (90_230, 15), (0, 0)])).unwrap(),
        now,
    );
    state.apply(parse(&wire(10, 0, 500)).unwrap(), now);
    state.apply(parse(&wire(6, 0, 600)).unwrap(), now);
    let mut rollback = wire(6, 0, 300);
    rollback[23..27].copy_from_slice(&601u32.to_le_bytes());
    assert!(state.apply(parse(&rollback).unwrap(), now));
    assert!(state.snapshot.damage.is_none());
    assert!(state.snapshot.lap_history.is_none());
    assert!(!state.apply(parse(&wire(10, 0, 599)).unwrap(), now));
    assert_eq!(state.snapshot.counters.flashbacks, 1);
}

#[test]
fn restart_after_silence_accepts_reset_counter() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(6, 0, 600)).unwrap(), now);
    assert!(state.apply(parse(&wire(6, 0, 1)).unwrap(), now + Duration::from_secs(3)));
    assert_eq!(state.snapshot.counters.restarts, 1);
}

#[test]
fn freshness_is_independent_for_wear_and_temperature() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(10, 0, 1)).unwrap(), now);
    state.apply(parse(&wire(6, 0, 1)).unwrap(), now);
    assert_eq!(state.snapshot.connection(now), Connection::Live);
    let later = now + Duration::from_millis(600);
    assert_eq!(state.snapshot.connection(later), Connection::Stale);
    assert!(state.snapshot.damage.unwrap().fresh(later, DAMAGE_TTL));
    assert!(
        !state
            .snapshot
            .damage
            .unwrap()
            .fresh(now + Duration::from_secs(3), DAMAGE_TTL)
    );
}

#[test]
fn inactive_lap_result_prevents_displaying_placeholder_zeroes() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(6, 0, 1)).unwrap(), now);
    state.apply(parse(&wire(2, 0, 1)).unwrap(), now);
    assert_eq!(state.snapshot.connection(now), Connection::Inactive);
}

#[test]
fn settings_validate_and_save_atomically() {
    let mut settings = Settings::default();
    settings.validate().unwrap();
    settings.halo.scale = f32::NAN;
    assert!(settings.validate().is_err());
    settings.halo.scale = 1.2;
    let path = std::env::temp_dir().join(format!("f1-halo-test-{}.json", std::process::id()));
    settings.save(&path).unwrap();
    settings.halo.scale = 1.5;
    settings.save(&path).unwrap();
    assert_eq!(Settings::load(&path).unwrap().halo.scale, 1.5);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn session_events_clear_values_without_replaying_duplicate_start() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    let mut start = wire(3, 0, 1);
    start[29..33].copy_from_slice(b"SSTA");
    assert!(state.apply(parse(&start).unwrap(), now));
    state.apply(parse(&wire(6, 0, 1)).unwrap(), now);
    assert!(!state.apply(parse(&start).unwrap(), now));
    assert!(state.snapshot.telemetry.is_some());
    state.apply(parse(&wire(6, 0, 20)).unwrap(), now);
    let mut ended = wire(3, 0, 21);
    ended[29..33].copy_from_slice(b"SEND");
    assert!(state.apply(parse(&ended).unwrap(), now));
    assert!(state.snapshot.telemetry.is_none());
    state.apply(parse(&wire(6, 0, 22)).unwrap(), now);
    assert!(state.snapshot.telemetry.is_none());
}

#[test]
fn late_session_end_does_not_clear_newer_live_data() {
    let now = Instant::now();
    let mut state = TelemetryState::default();
    state.apply(parse(&wire(6, 0, 20)).unwrap(), now);
    let mut ended = wire(3, 0, 10);
    ended[29..33].copy_from_slice(b"SEND");
    assert!(!state.apply(parse(&ended).unwrap(), now));
    assert!(state.snapshot.telemetry.is_some());
}

#[test]
fn diagnostics_percentiles_use_actual_samples() {
    use f1_halo_telemetry::diagnostics::{Diagnostics, percentile};
    let values = std::collections::VecDeque::from([1.0, 100.0, 3.0, 2.0]);
    assert_eq!(percentile(&values, 0.95), Some(100.0));
    assert_eq!(percentile(&std::collections::VecDeque::new(), 0.95), None);
    let empty = Diagnostics::default().summary(0, 0.0);
    assert_eq!(empty.gpu_ms_p95, None);
    assert_eq!(empty.receive_to_present_submit_ms_p95, None);
}
