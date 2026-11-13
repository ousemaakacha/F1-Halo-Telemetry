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
    let mut bytes = [0u8; 1460];
    for seconds in [0.0, 4.0, 8.0, 9.99, 10.0, 12.0, 16.0, 18.0, 1028.0] {
        for packet in demo::sample(seconds, 720) {
            let n = demo::encode(packet, &mut bytes);
            assert_eq!(parse(&bytes[..n]).unwrap(), packet);
        }
    }
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
