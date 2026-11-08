//! Synthetic data is opt-in and never used as a fallback for missing UDP values.
use super::protocol::*;

pub const DEMO_ERS_CAPACITY: f32 = 4_000_000.0;

pub fn sample(seconds: f32, frame: u32) -> [Packet; 4] {
    let phase = (seconds * 0.33).sin();
    let zone_seconds = seconds.rem_euclid(18.0);
    let speed = (190.0 + phase * 125.0).max(0.0);
    let gear = ((speed / 43.0) as i8 + 1).clamp(1, 8);
    let rpm = (6500.0 + (speed % 43.0) * 155.0) as u16;
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
                fuel_remaining_laps: 0.75 + phase * 1.55,
                max_rpm: 15000,
                ers_joules: DEMO_ERS_CAPACITY * (0.58 + phase * 0.27),
                deploy_mode: (seconds / 4.0).rem_euclid(4.0) as u8,
                harvest_limit_joules: 8_500_000.0,
                paused: false,
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
            c[50..54].copy_from_slice(&s.harvest_limit_joules.to_le_bytes());
            c[58] = s.paused as u8;
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
