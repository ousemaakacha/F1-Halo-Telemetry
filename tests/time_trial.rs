use f1_halo_telemetry::telemetry::{state::FAST_TTL, *};
use std::time::{Duration, Instant};

struct Session {
    state: TelemetryState,
    start: Instant,
    frame: u32,
    rewound_frames: u32,
    lap: u8,
    last_ms: u32,
}

impl Session {
    fn new() -> Self {
        Self {
            state: TelemetryState::default(),
            start: Instant::now(),
            frame: 0,
            rewound_frames: 0,
            lap: 1,
            last_ms: 0,
        }
    }

    fn header(&self, id: u8) -> Header {
        let frame = self.frame - self.rewound_frames;
        Header {
            packet_id: id,
            session_uid: 42,
            player: 23,
            session_time: frame as f32 / 10.0,
            frame,
            overall_frame: self.frame,
            ..Header::default()
        }
    }

    fn now(&self) -> Instant {
        self.start + Duration::from_millis(u64::from(self.frame) * 100)
    }

    fn apply(&mut self, packet: Packet) {
        let mut bytes = [0; 1470];
        let len = demo::encode(packet, &mut bytes);
        assert!(len > 0);
        assert!(self.state.apply(parse(&bytes[..len]).unwrap(), self.now()));
    }

    fn rewind(&mut self) {
        self.frame += 1;
        self.rewound_frames += 2;
        self.apply(Packet::Telemetry(self.header(6), CarTelemetry::default()));
    }

    fn sample(
        &mut self,
        millis: u32,
        distance: f32,
        sector: u8,
        sectors: [Option<u32>; 2],
        invalid: bool,
    ) -> TimeTrialReading {
        self.frame += 1;
        self.apply(Packet::Session(
            self.header(1),
            SessionData {
                session_type: 18,
                track_length_m: 300,
                ..SessionData::default()
            },
        ));
        self.apply(Packet::Telemetry(self.header(6), CarTelemetry::default()));
        self.apply(Packet::Lap(
            self.header(2),
            LapData {
                current_lap: self.lap,
                position: 1,
                result_status: 2,
                driver_status: 1,
                timing: Some(LapTiming {
                    last_ms: self.last_ms,
                    current_ms: millis,
                    distance_m: distance,
                    sector,
                    sectors_ms: sectors,
                    invalid,
                }),
                ..LapData::default()
            },
        ));
        self.state.snapshot.time_trial(self.now()).unwrap()
    }

    fn finish_lap(&mut self, total: u32, invalid: bool) -> TimeTrialReading {
        let sector = total / 3;
        self.sample(0, 0.0, 0, [None; 2], false);
        self.sample(sector, 100.0, 1, [Some(sector), None], invalid);
        self.sample(2 * sector, 200.0, 2, [Some(sector); 2], false);
        self.sample(total - 100, 299.0, 2, [Some(sector); 2], false);
        self.lap += 1;
        self.last_ms = total;
        self.sample(0, 0.0, 0, [None; 2], false)
    }
}

#[test]
fn local_udp_timing_uses_milliseconds_minutes_and_keeps_race_data_when_timing_is_bad() {
    for player in [0, 23] {
        let packet = Packet::Lap(
            Header {
                packet_id: 2,
                player,
                ..Header::default()
            },
            LapData {
                current_lap: 3,
                position: 1,
                result_status: 2,
                driver_status: 1,
                timing: Some(LapTiming {
                    last_ms: 180_789,
                    current_ms: 100_123,
                    distance_m: 2345.5,
                    sector: 2,
                    sectors_ms: [Some(61_234), Some(32_567)],
                    invalid: true,
                }),
                ..LapData::default()
            },
        );
        let mut bytes = [0; 1470];
        let len = demo::encode(packet, &mut bytes);
        assert_eq!(parse(&bytes[..len]).unwrap(), packet);
        let local = 29 + 57 * usize::from(player);
        for (offset, value) in [(36, 3), (37, 2), (8, 255)] {
            let mut invalid = bytes;
            invalid[local + offset] = value;
            if offset == 8 {
                invalid[local + 9] = 255;
            }
            let Packet::Lap(_, data) = parse(&invalid[..len]).unwrap() else {
                panic!()
            };
            assert!(data.timing.is_none());
            assert_eq!(
                (data.position, data.current_lap, data.result_status),
                (1, 3, 2)
            );
        }
        for distance in [f32::NAN, f32::INFINITY] {
            bytes[local + 20..local + 24].copy_from_slice(&distance.to_le_bytes());
            let Packet::Lap(_, data) = parse(&bytes[..len]).unwrap() else {
                panic!()
            };
            assert!(data.timing.is_none());
        }
    }
}

#[test]
fn delta_waits_for_a_complete_valid_lap_and_compares_equal_distances() {
    let mut session = Session::new();
    let reading = session.sample(0, 0.0, 0, [None; 2], false);
    assert_eq!((reading.delta_ms, reading.reference_ms), (None, None));
    assert_eq!(session.finish_lap(30_000, false).reference_ms, Some(30_000));
    let reading = session.sample(4_500, 50.0, 0, [None; 2], false);
    assert_eq!(reading.delta_ms, Some(-500));
    assert_eq!(reading.current_ms, 4_500);
    assert_eq!(reading.sectors_ms, [Some(4_500), None, None]);
    let reading = session.sample(18_200, 200.0, 2, [Some(9_000), Some(9_200)], false);
    assert_eq!(reading.delta_ms, Some(-1_800));
    assert_eq!(reading.sectors_ms, [Some(9_000), Some(9_200), Some(0)]);
    assert_eq!(reading.sector_deltas_ms, [Some(-1_000), Some(-800), None]);
}

#[test]
fn reference_changes_only_at_lap_boundary_and_invalid_or_slower_laps_never_replace_it() {
    let mut session = Session::new();
    session.finish_lap(30_000, false);
    let reading = session.sample(9_000, 100.0, 1, [Some(9_000), None], false);
    assert_eq!(
        (reading.reference_ms, reading.delta_ms),
        (Some(30_000), Some(-1_000))
    );
    session.sample(18_000, 200.0, 2, [Some(9_000); 2], false);
    let reading = session.sample(26_900, 299.0, 2, [Some(9_000); 2], false);
    assert_eq!(reading.reference_ms, Some(30_000));
    session.lap += 1;
    session.last_ms = 27_000;
    assert_eq!(
        session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
        Some(27_000)
    );
    assert_eq!(session.finish_lap(33_000, false).reference_ms, Some(27_000));
    assert_eq!(session.finish_lap(24_000, true).reference_ms, Some(27_000));
}

#[test]
fn invalidity_is_latched_and_an_invalid_first_lap_leaves_delta_unavailable() {
    let mut session = Session::new();
    session.sample(0, 0.0, 0, [None; 2], false);
    assert!(
        session
            .sample(10_000, 100.0, 1, [Some(10_000), None], true)
            .invalid
    );
    assert!(
        session
            .sample(20_000, 200.0, 2, [Some(10_000); 2], false)
            .invalid
    );
    session.lap += 1;
    session.last_ms = 30_000;
    let reading = session.sample(0, 0.0, 0, [None; 2], false);
    assert!(!reading.invalid);
    assert_eq!((reading.delta_ms, reading.reference_ms), (None, None));
}

#[test]
fn history_never_supplies_a_trace_or_switches_the_reference_during_a_lap() {
    let mut session = Session::new();
    session.finish_lap(30_000, false);
    session.apply(Packet::History(
        session.header(11),
        LapHistory {
            completed_laps: 2,
            best_before_last_ms: Some(29_000),
            last_lap_ms: None,
        },
    ));
    assert_eq!(
        session
            .sample(10_100, 100.0, 1, [Some(10_100), None], false)
            .reference_ms,
        Some(30_000)
    );
    session.sample(20_200, 200.0, 2, [Some(10_100); 2], false);
    session.lap += 1;
    session.last_ms = 30_300;
    let reading = session.sample(0, 0.0, 0, [None; 2], false);
    assert_eq!((reading.delta_ms, reading.reference_ms), (None, None));
    assert_eq!(reading.session_best_ms, Some(29_000));
}

#[test]
fn official_history_rejects_a_lost_final_invalidation_without_switching_reference_mid_lap() {
    for history_best in [30_000, 26_000] {
        let mut session = Session::new();
        session.finish_lap(30_000, false);
        assert_eq!(session.finish_lap(27_000, false).reference_ms, Some(27_000));
        session.apply(Packet::History(
            session.header(11),
            LapHistory {
                completed_laps: 2,
                best_before_last_ms: Some(history_best),
                last_lap_ms: None,
            },
        ));
        let reading = session.sample(9_000, 100.0, 1, [Some(9_000), None], false);
        assert_eq!((reading.reference_ms, reading.delta_ms), (None, None));
        assert_eq!(reading.session_best_ms, Some(history_best));
        session.sample(22_000, 200.0, 2, [Some(11_000); 2], false);
        session.lap += 1;
        session.last_ms = 33_000;
        assert_eq!(
            session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
            (history_best == 30_000).then_some(30_000)
        );
    }
}

#[test]
fn time_trial_can_cross_the_line_without_advancing_the_lap_number_and_never_extrapolates() {
    let mut session = Session::new();
    session.sample(0, -10.0, 0, [None; 2], false);
    session.sample(30, 0.3, 0, [None; 2], false);
    session.sample(10_000, 100.0, 1, [Some(10_000), None], false);
    session.sample(29_900, 299.0, 2, [Some(10_000); 2], false);
    session.last_ms = 30_000;
    assert_eq!(
        session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
        Some(30_000)
    );
    let reading = session.sample(31_000, 301.0, 2, [Some(10_000); 2], false);
    assert_eq!(reading.delta_ms, None);
    assert_eq!(reading.reference_ms, Some(30_000));
}

#[test]
fn unchanged_tt_lap_number_does_not_apply_previous_laps_history_to_a_new_best() {
    let mut session = Session::new();
    session.sample(0, 0.0, 0, [None; 2], false);
    session.sample(29_900, 299.0, 2, [Some(10_000); 2], false);
    session.last_ms = 30_000;
    session.sample(0, 0.0, 0, [None; 2], false);
    session.sample(26_900, 299.0, 2, [Some(9_000); 2], false);
    session.last_ms = 27_000;
    assert_eq!(
        session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
        Some(27_000)
    );
    session.apply(Packet::History(
        session.header(11),
        LapHistory {
            completed_laps: 1,
            last_lap_ms: Some(30_000),
            best_before_last_ms: None,
        },
    ));
    assert_eq!(
        session.sample(100, 1.0, 0, [None; 2], false).reference_ms,
        Some(27_000)
    );
}

#[test]
fn duplicate_reordered_and_flashback_packets_preserve_the_completed_reference() {
    let mut session = Session::new();
    session.finish_lap(30_000, false);
    let reading = session.sample(9_500, 100.0, 1, [Some(9_500), None], false);
    let old_generation = session.state.snapshot.session_generation;
    let mut header = session.header(2);
    let packet = |header| {
        Packet::Lap(
            header,
            LapData {
                current_lap: 99,
                ..LapData::default()
            },
        )
    };
    assert!(!session.state.apply(packet(header), session.now()));
    header.overall_frame -= 1;
    assert!(!session.state.apply(packet(header), session.now()));
    assert_eq!(
        session.state.snapshot.time_trial(session.now()),
        Some(reading)
    );
    session.rewind();
    assert!(session.state.snapshot.time_trial.is_none());
    assert_eq!(session.state.snapshot.session_generation, old_generation);
    assert_eq!(session.state.snapshot.counters.flashbacks, 1);
    assert_eq!(
        session
            .sample(10_000, 100.0, 1, [Some(10_000), None], false)
            .reference_ms,
        Some(30_000)
    );
}

#[test]
fn flashback_keeps_the_reference_and_later_valid_laps_can_improve_it() {
    let mut session = Session::new();
    session.finish_lap(30_000, false);
    session.apply(Packet::History(
        session.header(11),
        LapHistory {
            completed_laps: 1,
            last_lap_ms: Some(30_000),
            best_before_last_ms: None,
        },
    ));
    session.sample(18_000, 200.0, 2, [Some(9_000); 2], false);
    session.rewind();
    assert!(session.state.snapshot.time_trial.is_none());
    session.apply(Packet::History(
        session.header(11),
        LapHistory {
            completed_laps: 1,
            last_lap_ms: Some(30_000),
            best_before_last_ms: None,
        },
    ));
    let reading = session.sample(10_500, 100.0, 1, [Some(10_500), None], false);
    assert_eq!(
        (reading.reference_ms, reading.delta_ms),
        (Some(30_000), Some(500))
    );
    assert_eq!(reading.sector_deltas_ms, [Some(500), None, None]);
    session.sample(22_000, 200.0, 2, [Some(11_000); 2], false);
    session.sample(32_900, 299.0, 2, [Some(11_000); 2], false);
    session.lap += 1;
    session.last_ms = 33_000;
    let reading = session.sample(0, 0.0, 0, [None; 2], false);
    assert_eq!(
        (reading.reference_ms, reading.delta_ms),
        (Some(30_000), Some(0))
    );
    session.apply(Packet::History(
        session.header(11),
        LapHistory {
            completed_laps: 2,
            last_lap_ms: Some(33_000),
            best_before_last_ms: Some(30_000),
        },
    ));
    assert_eq!(session.finish_lap(33_000, false).reference_ms, Some(30_000));
    assert_eq!(session.finish_lap(27_000, false).reference_ms, Some(27_000));
    let reading = session.sample(8_500, 100.0, 1, [Some(8_500), None], false);
    assert_eq!(
        (reading.delta_ms, reading.session_best_ms),
        (Some(-500), Some(27_000))
    );
}

#[test]
fn flashback_discards_the_interrupted_trace_and_can_record_a_new_reference() {
    let mut session = Session::new();
    session.sample(0, 0.0, 0, [None; 2], false);
    session.sample(18_000, 200.0, 2, [Some(9_000); 2], false);
    session.rewind();
    session.sample(9_000, 100.0, 1, [Some(9_000), None], false);
    session.sample(18_000, 200.0, 2, [Some(9_000); 2], false);
    session.sample(26_900, 299.0, 2, [Some(9_000); 2], false);
    session.lap += 1;
    session.last_ms = 27_000;
    let reading = session.sample(0, 0.0, 0, [None; 2], false);
    assert_eq!((reading.reference_ms, reading.delta_ms), (None, None));
    assert_eq!(session.finish_lap(30_000, false).reference_ms, Some(30_000));
    let reading = session.sample(9_500, 100.0, 1, [Some(9_500), None], false);
    assert_eq!(reading.delta_ms, Some(-500));
}

#[test]
fn flashback_with_unchanged_lap_number_does_not_let_late_history_reject_a_new_best() {
    let mut session = Session::new();
    session.sample(0, 0.0, 0, [None; 2], false);
    session.sample(29_900, 299.0, 2, [Some(10_000); 2], false);
    session.last_ms = 30_000;
    session.sample(0, 0.0, 0, [None; 2], false);
    session.sample(18_000, 200.0, 2, [Some(9_000); 2], false);
    session.rewind();
    session.sample(9_000, 100.0, 1, [Some(9_000), None], false);
    session.sample(32_900, 299.0, 2, [Some(11_000); 2], false);
    session.last_ms = 33_000;
    session.sample(0, 0.0, 0, [None; 2], false);
    let history = LapHistory {
        completed_laps: 2,
        last_lap_ms: Some(33_000),
        best_before_last_ms: Some(30_000),
    };
    session.apply(Packet::History(session.header(11), history));
    session.sample(26_900, 299.0, 2, [Some(9_000); 2], false);
    session.last_ms = 27_000;
    assert_eq!(
        session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
        Some(27_000)
    );
    session.apply(Packet::History(session.header(11), history));
    assert_eq!(
        session.sample(100, 1.0, 0, [None; 2], false).reference_ms,
        Some(27_000)
    );
}

#[test]
fn partial_discontinuous_missing_or_reversed_traces_cannot_become_a_reference() {
    for failure in 0..5 {
        let mut session = Session::new();
        session.sample(
            if failure == 0 { 100 } else { 0 },
            if failure == 0 { 1.0 } else { 0.0 },
            0,
            [None; 2],
            false,
        );
        session.sample(10_000, 100.0, 1, [Some(10_000), None], false);
        match failure {
            1 => session.frame += 6,
            2 => session.state.invalidate_time_trial(),
            3 => {
                session.sample(11_000, 99.0, 1, [Some(10_000), None], false);
            }
            _ => (),
        }
        session.sample(
            20_000,
            200.0,
            2,
            if failure == 4 {
                [None; 2]
            } else {
                [Some(10_000); 2]
            },
            false,
        );
        session.lap += 1;
        session.last_ms = 30_000;
        assert_eq!(
            session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
            None,
            "case {failure}"
        );
    }
}

#[test]
fn stale_paused_spectating_and_session_changes_hide_timing_and_clear_references() {
    let mut session = Session::new();
    session.finish_lap(30_000, false);
    let now = session.now();
    let snapshot = session.state.snapshot;
    let expired = now + FAST_TTL + Duration::from_millis(1);
    assert!(snapshot.time_trial(expired).is_none());
    assert_ne!(
        snapshot.freshness_mask(now),
        snapshot.freshness_mask(expired)
    );
    for spectating in [false, true] {
        let mut inactive = snapshot;
        let data = &mut inactive.session.as_mut().unwrap().value;
        data.paused = !spectating;
        data.spectating = spectating;
        assert!(inactive.time_trial(now).is_none());
    }
    let generation = snapshot.session_generation;
    session.frame += 1;
    session
        .state
        .apply(Packet::Event(session.header(3), *b"SSTA"), session.now());
    assert!(session.state.snapshot.time_trial.is_none());
    assert_ne!(generation, session.state.snapshot.session_generation);
    assert_eq!(
        session.sample(0, 0.0, 0, [None; 2], false).reference_ms,
        None
    );
    session.finish_lap(30_000, false);
    let mut next = session.header(6);
    next.session_uid = 99;
    assert!(session.state.apply(
        Packet::Telemetry(next, CarTelemetry::default()),
        session.now()
    ));
    assert!(session.state.snapshot.time_trial.is_none());
}
