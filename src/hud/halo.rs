mod classic;

use crate::{
    graphics::{Font::*, *},
    settings::{HaloModel, Settings},
    telemetry::{state::FAST_TTL, *},
};
use std::time::Instant;
use windows::{
    Win32::Graphics::Direct2D::{Common::*, *},
    core::Result,
};
use windows_numerics::Vector2;

pub const WIDTH: f32 = 1000.0;
pub const HEIGHT: f32 = 300.0;
pub const CONTENT_PADDING: f32 = 80.0;
const CENTER_X: f32 = 420.0;
const HALO_RADIUS: f32 = 800.0;
const EDGE: [u8; 3] = [66, 86, 108];
const INK: [u8; 3] = [14, 20, 29];

const OVERTAKE_BLUE: [u8; 3] = [54, 193, 255];

fn current_aero(snapshot: &Snapshot, now: Instant) -> Option<Aero> {
    snapshot
        .aero
        .filter(|a| snapshot.driving(now) && a.fresh(now, FAST_TTL))
        .map(|a| a.value)
}

fn ers_colour(aero: Option<Aero>) -> [u8; 3] {
    if aero.is_some_and(|a| a.regulations_2026 && a.overtake_available) {
        OVERTAKE_BLUE
    } else {
        WHITE
    }
}

fn straight_mode_label(aero: Option<Aero>) -> &'static str {
    match aero {
        Some(a) if !a.regulations_2026 => "SM N/A",
        Some(a) if a.straight => "SM ACTIVE",
        Some(_) => "SM IN-ACT",
        None => "SM --",
    }
}

fn aero_state(aero: Option<Aero>, settings: &Settings) -> (&'static str, [u8; 3]) {
    match aero {
        Some(a) if a.regulations_2026 && a.straight => ("STRAIGHT", settings.colours.throttle),
        Some(a) if a.regulations_2026 => ("CORNER", MUTED),
        Some(_) => ("N/A", MUTED),
        None => ("--", MUTED),
    }
}

fn deploy_label(status: Option<CarStatus>) -> &'static str {
    match status.map(|s| s.deploy_mode) {
        Some(0) => "ERS / NONE",
        Some(1) => "ERS / MEDIUM",
        Some(2) => "ERS / HOTLAP",
        Some(3) => "ERS / BOOST",
        _ => "ERS / --",
    }
}

fn rpm_fraction(tel: Option<CarTelemetry>, status: Option<CarStatus>) -> Option<f32> {
    tel.zip(status)
        .filter(|(_, s)| s.max_rpm > 0)
        .map(|(t, s)| (t.rpm as f32 / s.max_rpm as f32).clamp(0.0, 1.0))
}

// The cockpit's inner edge is flatter at the centre and steeper near its ends.
fn halo_edge(x: f32) -> f32 {
    153.0 + HALO_RADIUS - (HALO_RADIUS.powi(2) - (x - CENTER_X).powi(2)).sqrt()
}

fn halo_angle(x: f32) -> f32 {
    halo_band_angle(x, 0.0)
}

fn halo_band_edge(x: f32, offset: f32) -> f32 {
    if offset == 0.0 {
        return halo_edge(x);
    }
    let shoulder = ((x - 550.0) / 325.0).clamp(0.0, 1.0);
    let flare = (-offset / 82.0).max(0.0) * 18.0 * shoulder.powi(4);
    let radius = HALO_RADIUS - offset + flare;
    153.0 + HALO_RADIUS - (radius.powi(2) - (x - CENTER_X).powi(2)).sqrt()
}

fn halo_band_angle(x: f32, offset: f32) -> f32 {
    ((halo_band_edge(x + 0.25, offset) - halo_band_edge(x - 0.25, offset)) / 0.5)
        .atan()
        .to_degrees()
}

pub struct Halo {
    classic: classic::Classic,
    ribbon: ID2D1PathGeometry,
    brow: ID2D1PathGeometry,
    lower_edge: ID2D1PathGeometry,
    straight_band: ID2D1PathGeometry,
    aero_band: ID2D1PathGeometry,
    gear_band: ID2D1PathGeometry,
    rpm_segments: [((f32, f32), (f32, f32)); 48],
    dial: ID2D1PathGeometry,
    rev_segments: Vec<ID2D1PathGeometry>,
    ribs: Vec<ID2D1PathGeometry>,
}

impl Halo {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        Ok(Self {
            classic: classic::Classic::new(factory)?,
            ribbon: halo_path(factory, 0.0, 875.0, -82.0, Some((20.0, 825.0, -3.0)))?,
            brow: halo_path(factory, 0.0, 875.0, -82.0, None)?,
            lower_edge: halo_path(factory, 20.0, 825.0, 0.0, None)?,
            straight_band: halo_path(factory, 0.0, 224.0, -68.0, Some((6.0, 224.0, -42.0)))?,
            aero_band: halo_path(factory, 10.0, 224.0, -38.0, Some((16.0, 224.0, -12.0)))?,
            gear_band: halo_path(factory, 550.0, 875.0, -72.0, Some((550.0, 825.0, -2.0)))?,
            rpm_segments: std::array::from_fn(|i| {
                let x = 555.0 + i as f32 * 260.0 / 48.0;
                let next = 555.0 + (i + 1) as f32 * 260.0 / 48.0;
                (
                    (x, halo_band_edge(x, -19.0)),
                    (next, halo_band_edge(next, -19.0)),
                )
            }),
            dial: dial_arc(factory, 67.0, 135.0, 405.0)?,
            rev_segments: (0..15)
                .map(|i| {
                    dial_arc(
                        factory,
                        61.0,
                        136.0 + i as f32 * 18.0,
                        152.0 + i as f32 * 18.0,
                    )
                })
                .collect::<Result<_>>()?,
            ribs: [79.0, 85.0, 91.0]
                .into_iter()
                .flat_map(|radius| [(radius, 153.0, 207.0), (radius, -27.0, 27.0)])
                .map(|(radius, start, end)| dial_arc(factory, radius, start, end))
                .collect::<Result<_>>()?,
        })
    }

    pub fn draw(
        &self,
        p: &Painter<'_>,
        snapshot: &Snapshot,
        settings: &Settings,
        edit: bool,
        now: Instant,
    ) {
        p.translated((CONTENT_PADDING, 0.0), |p| {
            self.draw_content(p, snapshot, settings, now);
        });
        if edit {
            super::edit_frame(p, WIDTH, HEIGHT, settings.colours.accent);
        }
    }

    fn draw_content(
        &self,
        p: &Painter<'_>,
        snapshot: &Snapshot,
        settings: &Settings,
        now: Instant,
    ) {
        let active = snapshot.driving(now);
        let tel = snapshot.telemetry.filter(|_| active).map(|t| t.value);
        let status = snapshot
            .status
            .filter(|t| active && t.fresh(now, FAST_TTL))
            .map(|t| t.value);
        let aero = current_aero(snapshot, now);
        let accent = settings.colours.accent;

        let is_classic = settings.halo_model == HaloModel::Classic;
        if is_classic {
            self.classic.draw_frame(p, accent);
        } else {
            p.geometry(&self.ribbon, INK, 0.84);
            p.stroke_geometry(&self.brow, EDGE, 0.9, 0.65);

            for (width, alpha) in [(20.0, 0.015), (12.0, 0.035), (5.0, 0.1), (1.6, 0.95)] {
                p.stroke_geometry(&self.lower_edge, accent, width, alpha);
            }
        }

        for rib in &self.ribs {
            p.stroke_geometry(rib, EDGE, 0.9, 0.8);
        }
        // The ring retains the game's 15 rev-light bits, from left to right.
        p.circle(420.0, 94.0, 56.0, INK, 0.94);
        p.stroke_geometry(&self.dial, accent, 8.0, 0.06);
        p.stroke_geometry(&self.dial, accent, 2.2, 1.0);
        for (i, segment) in self.rev_segments.iter().enumerate() {
            let on = tel.is_some_and(|t| t.rev_bits & (1 << i) != 0);
            p.stroke_geometry(
                segment,
                if on { WHITE } else { EDGE },
                3.8,
                if on { 1.0 } else { 0.65 },
            );
        }
        p.text("KM/H", rect(383.0, 49.0, 74.0, 15.0), HudLabel, WHITE);
        p.line((390.0, 103.0), (450.0, 103.0), EDGE, 0.7, 0.6);
        if let Some(t) = tel {
            p.textf(
                format_args!("{}", t.speed_kmh),
                rect(360.0, 62.0, 120.0, 41.0),
                HudSpeed,
                WHITE,
            );
        } else {
            p.text("--", rect(360.0, 62.0, 120.0, 41.0), HudSpeed, MUTED);
        }
        let percentage =
            status.and_then(|s| ers_percent(s.ers_joules, snapshot.ers_capacity.map(|c| c.joules)));
        let energy_colour = ers_colour(aero);
        if let Some(percent) = percentage {
            p.textf(
                format_args!("{percent:.0}%"),
                rect(365.0, 105.0, 110.0, 29.0),
                HudSecondary,
                energy_colour,
            );
        } else {
            p.text("--%", rect(365.0, 105.0, 110.0, 29.0), HudSecondary, MUTED);
        }
        if let Some(s) = status.filter(|_| percentage.is_none()) {
            p.textf(
                format_args!("ERS / {:.2} MJ", s.ers_joules / 1_000_000.0),
                rect(361.0, 132.0, 118.0, 13.0),
                HudLabel,
                energy_colour,
            );
        } else {
            p.text(
                "ERS",
                rect(389.0, 132.0, 62.0, 13.0),
                HudLabel,
                energy_colour,
            );
        }

        for (x, label, value, rgb) in [
            (294.0, "BRAKE", tel.map(|t| t.brake), settings.colours.brake),
            (
                508.0,
                "THROTTLE",
                tel.map(|t| t.throttle),
                settings.colours.throttle,
            ),
        ] {
            let angle = if is_classic {
                if x < CENTER_X { -9.0 } else { 9.0 }
            } else {
                halo_angle(x + 19.0)
            };
            p.rotated((x + 19.0, 107.0), angle, |p| {
                for i in 0..10 {
                    let y = 136.0 - i as f32 * 7.2;
                    p.fill(rect(x, y, 38.0, 5.4), EDGE, 0.55);
                    if let Some(value) = value {
                        let level = (pedal_percent(value) / 10.0 - i as f32).clamp(0.0, 1.0);
                        if level > 0.0 {
                            p.fill(
                                rect(x, y + 5.4 * (1.0 - level), 38.0, 5.4 * level),
                                rgb,
                                1.0,
                            );
                        }
                    }
                }
                p.text(label, rect(x - 9.0, 141.0, 56.0, 13.0), HudLabel, WHITE);
                if value.is_none() {
                    p.text("--", rect(x, 115.0, 38.0, 22.0), HudNumber, MUTED);
                }
            });
        }

        // No corner-number feed is available; this pod shows the actual gear.
        p.rotated(
            (260.0, 123.0),
            if is_classic { -13.0 } else { halo_angle(260.0) },
            |p| {
                let pod = rect(230.0, 87.0, 59.0, 74.0);
                p.rounded(pod, 10.0, INK, 0.9);
                p.rounded_outline(pod, 10.0, EDGE, 0.9, 0.9);
                p.text("GEAR", rect(230.0, 96.0, 59.0, 17.0), HudLabel, MUTED);
                p.text(
                    tel.map(|t| gear_label(t.gear)).unwrap_or("--"),
                    rect(230.0, 112.0, 59.0, 43.0),
                    HudSpeed,
                    if active { WHITE } else { MUTED },
                );
            },
        );

        if is_classic {
            self.classic.draw_wings(p, tel, status, aero, settings);
        } else {
            self.draw_wide_wings(p, tel, status, aero, settings);
        }

        let connection = snapshot.connection(now);
        if connection != Connection::Live {
            p.text(
                connection.label(),
                rect(287.0, 252.0, 266.0, 18.0),
                HudLabel,
                if connection == Connection::Demo {
                    MUTED
                } else {
                    [241, 181, 84]
                },
            );
        }
    }
    fn draw_wide_wings(
        &self,
        p: &Painter<'_>,
        tel: Option<CarTelemetry>,
        status: Option<CarStatus>,
        aero: Option<Aero>,
        settings: &Settings,
    ) {
        let accent = settings.colours.accent;
        let sm_active = aero.is_some_and(|a| a.regulations_2026 && a.straight);
        p.geometry(
            &self.straight_band,
            if sm_active {
                settings.colours.throttle
            } else {
                INK
            },
            0.95,
        );
        p.stroke_geometry(
            &self.straight_band,
            if sm_active {
                settings.colours.throttle
            } else {
                EDGE
            },
            0.8,
            0.9,
        );
        let sm_y = halo_band_edge(124.0, -55.0);
        p.rotated((124.0, sm_y), halo_band_angle(124.0, -55.0), |p| {
            p.text(
                straight_mode_label(aero),
                rect(43.0, sm_y - 13.0, 162.0, 26.0),
                HudNumber,
                if sm_active {
                    INK
                } else if aero.is_some() {
                    WHITE
                } else {
                    MUTED
                },
            );
        });
        p.geometry(&self.aero_band, INK, 0.9);
        p.stroke_geometry(&self.aero_band, EDGE, 0.8, 0.9);
        let (mode, rgb) = aero_state(aero, settings);
        for (x, width, label, font) in [
            (78.0, 74.0, "AERO", HudNumber),
            (175.0, 77.0, mode, HudLabel),
        ] {
            let y = halo_band_edge(x, -25.0);
            p.rotated((x, y), halo_band_angle(x, -25.0), |p| {
                p.text(
                    label,
                    rect(x - width / 2.0, y - 13.0, width, 26.0),
                    font,
                    rgb,
                );
            });
        }
        let deploy = deploy_label(status);
        let deploy_y = halo_band_edge(124.0, 3.0);
        p.rotated((124.0, deploy_y), halo_band_angle(124.0, 3.0), |p| {
            p.text(
                deploy,
                rect(51.5, deploy_y - 7.5, 145.0, 15.0),
                HudLabel,
                MUTED,
            );
        });

        p.geometry(&self.gear_band, INK, 0.9);
        p.stroke_geometry(&self.gear_band, EDGE, 0.9, 0.85);
        let heading_y = halo_band_edge(690.0, -61.0);
        p.rotated((690.0, heading_y), halo_band_angle(690.0, -61.0), |p| {
            p.text(
                "GEARS",
                rect(645.0, heading_y - 8.0, 90.0, 16.0),
                HudLabel,
                WHITE,
            );
        });
        for i in 0..9 {
            let gear = if i == 0 && tel.is_some_and(|t| t.gear == -1) {
                -1
            } else {
                i
            };
            let selected = tel.is_some_and(|t| t.gear == gear);
            let x = 565.0 + i as f32 * 31.0;
            let y = halo_band_edge(x, -41.0);
            p.rotated((x, y), halo_band_angle(x, -41.0), |p| {
                if selected {
                    p.rounded(rect(x - 11.5, y - 13.0, 23.0, 27.0), 3.0, accent, 0.22);
                    p.fill(rect(x - 6.5, y + 12.0, 13.0, 2.0), accent, 1.0);
                }
                p.text(
                    gear_label(gear),
                    rect(x - 11.5, y - 13.0, 23.0, 26.0),
                    HudNumber,
                    if selected { WHITE } else { MUTED },
                );
            });
        }
        let rpm_ratio = rpm_fraction(tel, status);
        for (i, &(a, b)) in self.rpm_segments.iter().enumerate() {
            p.line(a, b, EDGE, 3.0, 0.8);
            if let Some(ratio) = rpm_ratio {
                let level = (ratio * self.rpm_segments.len() as f32 - i as f32).clamp(0.0, 1.0);
                if level > 0.0 {
                    p.line(
                        a,
                        (a.0 + (b.0 - a.0) * level, a.1 + (b.1 - a.1) * level),
                        accent,
                        3.0,
                        1.0,
                    );
                }
            }
        }
        for (x, width) in [(555.0, 19.0), (690.0, 161.0), (815.0, 31.0)] {
            let y = halo_band_edge(x, -7.0);
            p.rotated((x, y), halo_band_angle(x, -7.0), |p| {
                let r = rect(x - width / 2.0, y - 7.0, width, 14.0);
                if x == 555.0 {
                    p.text("0", r, HudLabel, WHITE);
                } else if x == 690.0 {
                    if let Some(t) = tel {
                        p.textf(
                            format_args!("{:.1} RPM x1000", t.rpm as f32 / 1000.0),
                            r,
                            HudLabel,
                            WHITE,
                        );
                    } else {
                        p.text("-- RPM x1000", r, HudLabel, MUTED);
                    }
                } else if let Some(s) = status.filter(|s| s.max_rpm > 0) {
                    p.textf(
                        format_args!("{:.0}", s.max_rpm as f32 / 1000.0),
                        r,
                        HudLabel,
                        WHITE,
                    );
                } else {
                    p.text("--", r, HudLabel, MUTED);
                }
            });
        }
    }
}

fn point(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}

fn halo_path(
    factory: &ID2D1Factory1,
    start: f32,
    end: f32,
    top: f32,
    bottom: Option<(f32, f32, f32)>,
) -> Result<ID2D1PathGeometry> {
    let edge = |start: f32, end: f32, offset: f32| -> Vec<Vector2> {
        (0..=96)
            .map(|i| {
                let x = start + (end - start) * i as f32 / 96.0;
                point(x, halo_band_edge(x, offset))
            })
            .collect()
    };
    let upper = edge(start, end, top);
    // SAFETY: the sampled contours are cached once and remain on the UI thread.
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(
            upper[0],
            if bottom.is_some() {
                D2D1_FIGURE_BEGIN_FILLED
            } else {
                D2D1_FIGURE_BEGIN_HOLLOW
            },
        );
        sink.AddLines(&upper[1..]);
        if let Some((start, end, offset)) = bottom {
            let mut lower = edge(start, end, offset);
            lower.reverse();
            sink.AddLines(&lower);
        }
        sink.EndFigure(if bottom.is_some() {
            D2D1_FIGURE_END_CLOSED
        } else {
            D2D1_FIGURE_END_OPEN
        });
        sink.Close()?;
        Ok(path.into())
    }
}

fn dial_arc(
    factory: &ID2D1Factory1,
    radius: f32,
    start: f32,
    end: f32,
) -> Result<ID2D1PathGeometry> {
    let position = |angle: f32| {
        point(
            420.0 + radius * angle.to_radians().cos(),
            94.0 + radius * angle.to_radians().sin(),
        )
    };
    // SAFETY: each arc is constructed and retained on the same UI thread.
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(position(start), D2D1_FIGURE_BEGIN_HOLLOW);
        sink.AddArc(&D2D1_ARC_SEGMENT {
            point: position(end),
            size: D2D_SIZE_F {
                width: radius,
                height: radius,
            },
            rotationAngle: 0.0,
            sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
            arcSize: if end - start > 180.0 {
                D2D1_ARC_SIZE_LARGE
            } else {
                D2D1_ARC_SIZE_SMALL
            },
        });
        sink.EndFigure(D2D1_FIGURE_END_OPEN);
        sink.Close()?;
        Ok(path.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn both_models_share_rpm_limits_and_missing_data_behaviour() {
        let tel = CarTelemetry {
            rpm: 7500,
            ..CarTelemetry::default()
        };
        let status = CarStatus {
            max_rpm: 15000,
            ..CarStatus::default()
        };
        assert_eq!(rpm_fraction(Some(tel), Some(status)), Some(0.5));
        assert_eq!(
            rpm_fraction(Some(CarTelemetry { rpm: 20000, ..tel }), Some(status)),
            Some(1.0)
        );
        assert_eq!(
            rpm_fraction(
                Some(tel),
                Some(CarStatus {
                    max_rpm: 0,
                    ..status
                })
            ),
            None
        );
        assert_eq!(rpm_fraction(Some(tel), None), None);
        assert_eq!(rpm_fraction(None, Some(status)), None);
    }

    fn driving_snapshot(now: Instant) -> Snapshot {
        Snapshot {
            source: Connection::Listening,
            telemetry: Some(Timed {
                value: CarTelemetry::default(),
                received: now,
                overall_frame: 1,
            }),
            aero: Some(Timed {
                value: Aero {
                    regulations_2026: true,
                    available: true,
                    ..Aero::default()
                },
                received: now,
                overall_frame: 1,
            }),
            ..Snapshot::default()
        }
    }

    #[test]
    fn ers_turns_blue_on_zone_availability_before_boost_activation() {
        for regulations_2026 in [false, true] {
            for overtake_available in [false, true] {
                for overtake_active in [false, true] {
                    let aero = Aero {
                        regulations_2026,
                        overtake_available,
                        overtake_active,
                        ..Aero::default()
                    };
                    assert_eq!(
                        ers_colour(Some(aero)),
                        if regulations_2026 && overtake_available {
                            OVERTAKE_BLUE
                        } else {
                            WHITE
                        }
                    );
                }
            }
        }
        assert_eq!(ers_colour(None), WHITE);
    }

    #[test]
    fn straight_mode_text_uses_engagement_instead_of_availability() {
        for available in [false, true] {
            for straight in [false, true] {
                let aero = Aero {
                    regulations_2026: true,
                    available,
                    straight,
                    ..Aero::default()
                };
                assert_eq!(
                    straight_mode_label(Some(aero)),
                    if straight { "SM ACTIVE" } else { "SM IN-ACT" }
                );
                assert_eq!(
                    straight_mode_label(Some(Aero {
                        regulations_2026: false,
                        ..aero
                    })),
                    "SM N/A"
                );
            }
        }
        assert_eq!(straight_mode_label(None), "SM --");
    }

    #[test]
    fn aero_indicators_expire_and_clear_when_driving_stops() {
        let now = Instant::now();
        let snapshot = driving_snapshot(now);
        assert!(current_aero(&snapshot, now + FAST_TTL).is_some());
        let expired = now + FAST_TTL + Duration::from_millis(1);
        let mut changed = snapshot;
        changed.telemetry.as_mut().unwrap().received = expired;
        assert!(current_aero(&changed, expired).is_none());
        changed = snapshot;
        changed.aero.as_mut().unwrap().received = expired;
        assert!(current_aero(&changed, expired).is_none());
        assert!(current_aero(&Snapshot::default(), now).is_none());
        changed = snapshot;
        changed.aero = None;
        assert!(current_aero(&changed, now).is_none());
        for source in [Connection::BindError, Connection::Unsupported] {
            changed = snapshot;
            changed.source = source;
            assert!(current_aero(&changed, now).is_none());
        }
        changed = snapshot;
        changed.session_paused = Some(Timed {
            value: true,
            received: now,
            overall_frame: 1,
        });
        assert!(current_aero(&changed, now).is_none());
        changed = snapshot;
        changed.lap_status = Some(Timed {
            value: 1,
            received: now,
            overall_frame: 1,
        });
        assert!(current_aero(&changed, now).is_none());
        assert_eq!(straight_mode_label(current_aero(&changed, now)), "SM --");
        assert_eq!(ers_colour(current_aero(&changed, now)), WHITE);
    }

    #[test]
    fn cockpit_curve_is_symmetric_and_text_follows_its_tangent() {
        assert_eq!(CONTENT_PADDING + CENTER_X, WIDTH / 2.0);
        assert_eq!(halo_angle(CENTER_X), 0.0);
        let mut previous = halo_edge(CENTER_X);
        for x in 420..=825 {
            let x = x as f32;
            let y = halo_edge(x);
            assert!(y >= previous && y < HEIGHT - 22.0);
            assert!((y - halo_edge(2.0 * CENTER_X - x)).abs() < 0.001);
            let derivative = (halo_edge(x + 0.1) - halo_edge(x - 0.1)) / 0.2;
            assert!((derivative.atan().to_degrees() - halo_angle(x)).abs() < 0.04);
            previous = y;
        }
        assert!(halo_angle(825.0) < 31.0);
        assert!(halo_angle(527.0) < 10.0);
    }

    #[test]
    fn wide_arc_follows_the_annotated_upper_and_lower_guides() {
        let scale = 1.036_904_8;
        let guides = [
            (552.0, 623.5, -82.0),
            (652.0, 581.0, -82.0),
            (727.0, 550.5, -82.0),
            (1152.0, 544.5, -82.0),
            (1252.0, 572.5, -82.0),
            (1327.0, 598.5, -82.0),
            (1427.0, 631.5, -82.0),
            (552.0, 714.5, 0.0),
            (652.0, 668.0, 0.0),
            (752.0, 633.0, 0.0),
            (1102.0, 621.5, 0.0),
            (1252.0, 663.0, 0.0),
            (1377.0, 721.0, 0.0),
        ];
        for (screen_x, expected_y, offset) in guides {
            let x = (screen_x - 523.0) / scale;
            let actual_y = 450.0 + halo_band_edge(x, offset) * scale;
            assert!(
                (actual_y - expected_y).abs() < 8.0,
                "guide at {screen_x}: expected {expected_y}, got {actual_y}"
            );
        }
    }
}
