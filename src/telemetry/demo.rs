//! Synthetic data is opt-in and never used as a fallback for missing UDP values.
use super::protocol::*;

pub const DEMO_ERS_CAPACITY: f32 = 4_000_000.0;
pub const DEMO_DEPLOY_LIMIT: f32 = 13_000_000.0;

pub fn sample(seconds: f32, frame: u32) -> [Packet; 8] {
    let phase = (seconds * 0.33).sin();
    let zone_seconds = seconds.rem_euclid(18.0);
    let speed = (190.0 + phase * 125.0).max(0.0);
    let gear = ((speed / 43.0) as i8 + 1).clamp(1, 8);
    let rpm = (6500.0 + (speed % 43.0) * 155.0) as u16;
    let gap_behind_ms = (1_800.0 + phase * 1_200.0).round() as u32;
    let lights = ((rpm as f32 - 7500.0) / 350.0).clamp(0.0, 15.0) as u32;
    let h = Header {
        game_year: 25,
        major: 1,
        minor: 0,
        packet_id: 6,
        session_uid: 0xDE00_2026,
        session_time: seconds,
        frame,
        overall_frame: frame,
        player: 23,
    };
    let tel = CarTelemetry {
        speed_kmh: speed as u16,
        throttle: (phase + 0.6).clamp(0.0, 1.0),
        brake: (-phase - 0.25).clamp(0.0, 1.0),
        gear,
        rpm,
        rev_percent: (lights * 100 / 15) as u8,
        rev_bits: ((1u32 << lights) - 1) as u16,
        surface_c: [98, 102, 94, 97].map(|v| (v as f32 + phase * 9.0) as u8),
        inner_c: [94, 96, 90, 92],
    };
    [
        Packet::Telemetry(h, tel),
        Packet::Status(
            Header { packet_id: 7, ..h },
            CarStatus {
                visual_compounds: std::array::from_fn(|car| match car {
                    0 => 17,
                    1 => 16,
                    23 => 18,
                    _ => 0,
                }),
                fuel_remaining_laps: 0.75 + phase * 1.55,
                max_rpm: 15000,
                ers_joules: DEMO_ERS_CAPACITY * (0.58 + phase * 0.27),
                deploy_mode: (seconds / 4.0).rem_euclid(4.0) as u8,
                harvested_mguk_joules: 8_500_000.0 * 0.4,
                harvest_limit_joules: 8_500_000.0,
                deployed_joules: DEMO_DEPLOY_LIMIT * 0.55,
                paused: false,
                fia_flag: None,
            },
        ),
        Packet::Damage(
            Header { packet_id: 10, ..h },
            Damage {
                wear: [18.0, 22.0, 13.0, 15.0],
            },
        ),
        Packet::Aero(
            Header { packet_id: 16, ..h },
            Aero {
                straight: (12.0..16.0).contains(&zone_seconds),
                available: (10.0..16.0).contains(&zone_seconds),
                activation_distance_m: if zone_seconds < 10.0 {
                    ((10.0 - zone_seconds) * 70.0).ceil() as u16
                } else {
                    0
                },
                overtake_available: phase > 0.0,
                overtake_active: phase > 0.75,
                overtake_distance_m: 0,
                regulations_2026: true,
            },
        ),
        Packet::History(
            Header { packet_id: 11, ..h },
            LapHistory {
                completed_laps: 2,
                last_lap_ms: Some(match (seconds / 6.0) as u32 % 3 {
                    0 => 90_230,
                    1 => 89_880,
                    _ => 90_000,
                }),
                best_before_last_ms: Some(90_000),
            },
        ),
        Packet::Lap(
            Header { packet_id: 2, ..h },
            LapData {
                position: 2,
                result_status: 2,
                driver_status: 4,
                current_lap: 3,
                gap_behind_ms: Some(gap_behind_ms),
                race_gaps: [
                    Some(CarGap {
                        car_index: 1,
                        millis: (1_500.0 + phase * 700.0).round() as u32,
                    }),
                    Some(CarGap {
                        car_index: 0,
                        millis: gap_behind_ms,
                    }),
                ],
                race_control: RaceControl::default(),
                timing: None,
            },
        ),
        Packet::Session(
            Header { packet_id: 1, ..h },
            SessionData {
                total_laps: 18,
                formula: 13,
                session_type: 18,
                ..SessionData::default()
            },
        ),
        Packet::Participants(
            Header { packet_id: 4, ..h },
            Participants {
                active_cars: 3,
                names: std::array::from_fn(|i| {
                    let mut name = [0; 32];
                    let text: &[u8] = match i {
                        23 => b"NORRIS",
                        0 => b"STROLL",
                        1 => b"PIASTRI",
                        _ => b"",
                    };
                    name[..text.len()].copy_from_slice(text);
                    name
                }),
            },
        ),
    ]
}

/// Encodes only the documented fields used by this application. Intended for
/// the explicit external UDP simulator and deterministic test fixtures.
pub fn encode(packet: Packet, buffer: &mut [u8]) -> usize {
    let h = packet.header();
    let size = match packet {
        Packet::Telemetry(..) => TELEMETRY_SIZE,
        Packet::Status(..) => STATUS_SIZE,
        Packet::Damage(..) => DAMAGE_SIZE,
        Packet::Aero(..) => AERO_SIZE,
        Packet::History(..) => HISTORY_SIZE,
        Packet::Lap(..) => LAP_SIZE,
        Packet::Session(..) => 926,
        Packet::Participants(..) => PARTICIPANTS_SIZE,
        _ => return 0,
    };
    if buffer.len() < size {
        return 0;
    }
    let b = &mut buffer[..size];
    b.fill(0);
    b[0..2].copy_from_slice(&2026u16.to_le_bytes());
    b[2] = h.game_year;
    b[3] = h.major;
    b[4] = h.minor;
    b[5] = 1;
    b[6] = h.packet_id;
    b[7..15].copy_from_slice(&h.session_uid.to_le_bytes());
    b[15..19].copy_from_slice(&h.session_time.to_le_bytes());
    b[19..23].copy_from_slice(&h.frame.to_le_bytes());
    b[23..27].copy_from_slice(&h.overall_frame.to_le_bytes());
    b[27] = h.player;
    b[28] = 255;
    if let Packet::Participants(_, participants) = packet {
        b[29] = participants.active_cars;
        for (index, name) in participants.names.iter().enumerate() {
            let offset = 30 + 60 * index + 10;
            b[offset..offset + 32].copy_from_slice(name);
        }
        return size;
    }
    if let Packet::Session(_, session) = packet {
        b[HEADER_SIZE + 3] = session.total_laps;
        b[HEADER_SIZE + 4..HEADER_SIZE + 6].copy_from_slice(&session.track_length_m.to_le_bytes());
        b[HEADER_SIZE + 6] = session.session_type;
        b[HEADER_SIZE + 8] = session.formula;
        b[HEADER_SIZE + 14] = session.paused as u8;
        b[HEADER_SIZE + 15] = session.spectating as u8;
        b[HEADER_SIZE + 124] = session.safety_car.unwrap_or(255);
        return size;
    }
    if let Packet::Lap(_, lap) = packet {
        let offset = HEADER_SIZE + 57 * h.player as usize;
        let position = if lap.position == 0 {
            if lap.race_gaps[0].is_some() { 2 } else { 1 }
        } else {
            lap.position
        };
        b[offset + 32] = position;
        b[offset + 33] = lap.current_lap;
        b[offset + 44] = lap.driver_status;
        b[offset + 45] = lap.result_status;
        b[offset + 38] = lap.race_control.penalty_seconds;
        b[offset + 39] = lap.race_control.warnings;
        b[offset + 41] = lap.race_control.drive_through;
        b[offset + 42] = lap.race_control.stop_go;
        if let Some(timing) = lap.timing {
            b[offset..offset + 4].copy_from_slice(&timing.last_ms.to_le_bytes());
            b[offset + 4..offset + 8].copy_from_slice(&timing.current_ms.to_le_bytes());
            b[offset + 20..offset + 24].copy_from_slice(&timing.distance_m.to_le_bytes());
            b[offset + 36] = timing.sector;
            b[offset + 37] = u8::from(timing.invalid);
            for (offset, time) in [offset + 8, offset + 11].into_iter().zip(timing.sectors_ms) {
                let time = time.unwrap_or(0);
                if time / 60_000 > u32::from(u8::MAX) {
                    return 0;
                }
                b[offset..offset + 2].copy_from_slice(&((time % 60_000) as u16).to_le_bytes());
                b[offset + 2] = (time / 60_000) as u8;
            }
        } else {
            // Mark absent synthetic timing explicitly, rather than encode a valid zero lap.
            b[offset + 36] = 255;
        }
        if let Some(ahead) = lap.race_gaps[0] {
            if ahead.car_index as usize >= CAR_COUNT
                || ahead.car_index == h.player
                || ahead.millis / 60_000 > u8::MAX as u32
            {
                return 0;
            }
            b[offset + 14..offset + 16]
                .copy_from_slice(&((ahead.millis % 60_000) as u16).to_le_bytes());
            b[offset + 16] = (ahead.millis / 60_000) as u8;
            let other = HEADER_SIZE + 57 * ahead.car_index as usize;
            b[other + 32] = position - 1;
            b[other + 44] = 4;
            b[other + 45] = 2;
        }
        if let Some(gap) = lap.gap_behind_ms {
            if gap / 60_000 > u8::MAX as u32 {
                return 0;
            }
            let index = lap.race_gaps[1]
                .map_or(((h.player as usize + 1) % CAR_COUNT) as u8, |gap| {
                    gap.car_index
                });
            if index as usize >= CAR_COUNT
                || index == h.player
                || lap.race_gaps[0].is_some_and(|ahead| ahead.car_index == index)
            {
                return 0;
            }
            let behind = HEADER_SIZE + 57 * index as usize;
            b[behind + 32] = position + 1;
            b[behind + 44] = 4;
            b[behind + 45] = 2;
            b[behind + 14..behind + 16].copy_from_slice(&((gap % 60_000) as u16).to_le_bytes());
            b[behind + 16] = (gap / 60_000) as u8;
        }
        return size;
    }
    if let Packet::History(_, history) = packet {
        if history.completed_laps > 99 {
            return 0;
        }
        b[29] = h.player;
        b[30] = history.completed_laps + 1;
        if let Some(best) = history
            .best_before_last_ms
            .filter(|_| history.completed_laps > 1)
        {
            b[32] = 1;
            b[36..40].copy_from_slice(&best.to_le_bytes());
            b[49] = 0x0f;
        }
        if let Some(last) = history.last_lap_ms.filter(|_| history.completed_laps > 0) {
            let offset = 36 + (history.completed_laps as usize - 1) * 14;
            b[offset..offset + 4].copy_from_slice(&last.to_le_bytes());
            b[offset + 13] = 0x0f;
        }
        return size;
    }
    let stride = match packet {
        Packet::Damage(..) => 46,
        Packet::Aero(..) => 10,
        _ => 59,
    };
    let c = &mut b[HEADER_SIZE + stride * h.player as usize..];
    match packet {
        Packet::Telemetry(_, t) => {
            c[0..2].copy_from_slice(&t.speed_kmh.to_le_bytes());
            c[2..6].copy_from_slice(&t.throttle.to_le_bytes());
            c[10..14].copy_from_slice(&t.brake.to_le_bytes());
            c[15] = t.gear as u8;
            c[16..18].copy_from_slice(&t.rpm.to_le_bytes());
            c[19] = t.rev_percent;
            c[20..22].copy_from_slice(&t.rev_bits.to_le_bytes());
            c[30..34].copy_from_slice(&t.surface_c);
            c[34..38].copy_from_slice(&t.inner_c);
        }
        Packet::Status(_, s) => {
            c[13..17].copy_from_slice(&s.fuel_remaining_laps.to_le_bytes());
            c[17..19].copy_from_slice(&s.max_rpm.to_le_bytes());
            c[37..41].copy_from_slice(&s.ers_joules.to_le_bytes());
            c[41] = s.deploy_mode;
            c[42..46].copy_from_slice(&s.harvested_mguk_joules.to_le_bytes());
            c[50..54].copy_from_slice(&s.harvest_limit_joules.to_le_bytes());
            c[54..58].copy_from_slice(&s.deployed_joules.to_le_bytes());
            c[58] = s.paused as u8;
            c[28] = s.fia_flag.unwrap_or(255);
            for (index, compound) in s.visual_compounds.into_iter().enumerate() {
                b[HEADER_SIZE + 59 * index + 26] = compound;
            }
        }
        Packet::Damage(_, d) => {
            for (i, v) in d.wear.iter().enumerate() {
                c[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        Packet::Aero(_, a) => {
            c[0] = a.straight as u8;
            c[1] = a.available as u8;
            c[2..4].copy_from_slice(&a.activation_distance_m.to_le_bytes());
            c[4] = a.overtake_available as u8;
            c[5] = a.overtake_active as u8;
            c[6..8].copy_from_slice(&a.overtake_distance_m.to_le_bytes());
            c[8] = a.regulations_2026 as u8;
        }
        _ => (),
    }
    size
}
