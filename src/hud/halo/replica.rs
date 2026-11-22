use super::modern::left_text_curve as text_curve;
use super::*;

const CYAN: [u8; 3] = [75, 178, 220];
const MINT: [u8; 3] = [58, 224, 187];
const GLASS: [u8; 3] = [12, 26, 31];
const BORDER: [u8; 3] = [131, 167, 176];
const OFF: [u8; 3] = [75, 102, 109];
const AERO_GREEN: [u8; 3] = [70, 229, 123];
const AERO_RED: [u8; 3] = [255, 66, 72];
const DIAL: (f32, f32) = (CENTER_X, 104.0);
const DIAL_REFERENCE: Point = (404.0, 150.0);
const ORIGIN: (f32, f32) = (CONTENT_PADDING, 0.0);
const WINGS: [(f32, f32); 2] = [(-31.5, -13.0), (13.0, 31.5)];
const WING_TOP: f32 = -66.0;
const WING_BOTTOM: f32 = 14.0;
const NAME_ROWS: [f32; 2] = [WING_TOP + 20.0, WING_BOTTOM - 20.0];
const SPLIT_ROW: f32 = (WING_TOP + WING_BOTTOM) / 2.0;
const META_ROW: f32 = WING_TOP + 13.0;
const GEAR_ROW: f32 = WING_TOP + 34.0;
const RPM_ROW: f32 = WING_TOP + 53.0;
const RPM_CAPTION: f32 = WING_BOTTOM - 12.0;
const FIRST_GEAR: f32 = 19.5;
const LAST_GEAR: f32 = 30.5;
type Point = (f32, f32);

#[derive(Clone, Copy, Debug, PartialEq)]
struct LapBudget {
    joules: f32,
    fraction: f32,
}

fn remaining_budget(used: f32, limit: f32) -> Option<LapBudget> {
    if !used.is_finite() || used < 0.0 || !limit.is_finite() || limit < 0.0 {
        return None;
    }
    let joules = (limit - used).max(0.0);
    Some(LapBudget {
        joules,
        fraction: if limit > 0.0 { joules / limit } else { 0.0 },
    })
}

fn lap_budgets(status: Option<CarStatus>, deploy_limit: Option<f32>) -> [Option<LapBudget>; 2] {
    let Some(status) = status else {
        return [None; 2];
    };
    [
        remaining_budget(status.harvested_mguk_joules, status.harvest_limit_joules),
        deploy_limit.and_then(|limit| remaining_budget(status.deployed_joules, limit)),
    ]
}

fn battery_body() -> D2D_RECT_F {
    // Include the two-unit terminal when centring the complete battery outline.
    rect(DIAL_REFERENCE.0 - 32.0, 214.0, 62.0, 18.0)
}

fn aero_border(aero: Option<Aero>) -> [u8; 3] {
    match aero.filter(|a| a.regulations_2026) {
        Some(a) if a.straight => AERO_GREEN,
        Some(_) => AERO_RED,
        None => OFF,
    }
}

fn wing_point(angle: f32, offset: f32) -> Point {
    modern::band_point(angle.to_radians(), offset)
}

fn path(
    factory: &ID2D1Factory1,
    points: impl IntoIterator<Item = Point>,
    closed: bool,
) -> Result<ID2D1PathGeometry> {
    let mut points = points.into_iter();
    let first = points.next().expect("geometry has a starting point");
    // SAFETY: geometry and sink are created, closed and retained on the UI thread.
    unsafe {
        let geometry = factory.CreatePathGeometry()?;
        let sink = geometry.Open()?;
        sink.BeginFigure(
            point(first.0, first.1),
            if closed {
                D2D1_FIGURE_BEGIN_FILLED
            } else {
                D2D1_FIGURE_BEGIN_HOLLOW
            },
        );
        for (x, y) in points {
            sink.AddLine(point(x, y));
        }
        sink.EndFigure(if closed {
            D2D1_FIGURE_END_CLOSED
        } else {
            D2D1_FIGURE_END_OPEN
        });
        sink.Close()?;
        Ok(geometry.into())
    }
}

fn side_point(right: bool, radius: f32, y: f32) -> Point {
    let sign = if right { 1.0 } else { -1.0 };
    (DIAL.0 + sign * (radius * radius - y * y).sqrt(), DIAL.1 + y)
}

fn side_path(
    factory: &ID2D1Factory1,
    right: bool,
    inner: f32,
    outer: f32,
    top: f32,
    bottom: f32,
    tilt: f32,
) -> Result<ID2D1PathGeometry> {
    let edge = |radius, reverse| {
        (0..=24).map(move |i| {
            let t = if reverse {
                1.0 - i as f32 / 24.0
            } else {
                i as f32 / 24.0
            };
            side_point(
                right,
                radius,
                top + (bottom - top) * t + if radius == outer { tilt } else { 0.0 },
            )
        })
    };
    path(factory, edge(inner, false).chain(edge(outer, true)), true)
}

fn ring(factory: &ID2D1Factory1, radius: f32) -> Result<ID2D1PathGeometry> {
    path(
        factory,
        (0..=160).map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 160.0;
            (DIAL.0 + radius * a.cos(), DIAL.1 + radius * a.sin())
        }),
        true,
    )
}

fn dot_mesh(factory: &ID2D1Factory1) -> Result<ID2D1PathGeometry> {
    // SAFETY: the stipple is immutable and submitted as one cached geometry per frame.
    unsafe {
        let geometry = factory.CreatePathGeometry()?;
        let sink = geometry.Open()?;
        for y in (-72..=72).step_by(3) {
            for x in (-72..=72).step_by(3) {
                if x * x + y * y < 72 * 72 {
                    let (x, y) = (DIAL.0 + x as f32, DIAL.1 + y as f32);
                    sink.BeginFigure(point(x, y), D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(point(x + 0.45, y));
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                }
            }
        }
        sink.Close()?;
        Ok(geometry.into())
    }
}

struct Pedal {
    right: bool,
    housing: ID2D1PathGeometry,
    energy: ID2D1PathGeometry,
    bars: Vec<ID2D1PathGeometry>,
}

impl Pedal {
    fn new(factory: &ID2D1Factory1, right: bool) -> Result<Self> {
        Ok(Self {
            right,
            housing: side_path(factory, right, 116.0, 150.0, -35.0, 49.0, 10.0)?,
            energy: side_path(factory, right, 83.0, 112.0, -47.0, 41.0, 4.0)?,
            bars: (0..11)
                .map(|i| {
                    let bottom = 46.0 - i as f32 * 7.3;
                    side_path(factory, right, 119.0, 146.5, bottom - 5.0, bottom, 9.0)
                })
                .collect::<Result<_>>()?,
        })
    }

    fn draw(
        &self,
        p: &Painter<'_>,
        value: Option<f32>,
        rgb: [u8; 3],
        opacity: f32,
        budget: Option<LapBudget>,
    ) {
        p.geometry(&self.housing, GLASS, opacity * 0.8);
        p.stroke_geometry(&self.housing, BORDER, 1.0, 0.65);
        p.geometry(&self.energy, [0, 37, 53], opacity * 0.9);
        if let Some(budget) = budget.filter(|b| b.fraction > 0.0) {
            let height = 92.0 * budget.fraction;
            p.clipped(
                rect(DIAL.0 - 155.0, DIAL.1 + 45.0 - height, 310.0, height),
                |p| {
                    p.geometry(&self.energy, [0, 96, 139], 0.95);
                },
            );
        }
        p.stroke_geometry(&self.energy, CYAN, 1.0, 0.9);
        for (i, bar) in self.bars.iter().enumerate() {
            p.geometry(bar, OFF, 0.85);
            let level = value.map_or(0.0, |v| {
                (pedal_percent(v) / 100.0 * 11.0 - i as f32).clamp(0.0, 1.0)
            });
            if level > 0.0 {
                p.geometry(bar, rgb, level);
            }
        }
        let (x, y) = side_point(self.right, 97.5, -1.0);
        p.text(
            if self.right { "DPL" } else { "RCH" },
            rect(x - 14.0, y - 7.0, 28.0, 14.0),
            HudLabel,
            if budget.is_some() { WHITE } else { MUTED },
        );
        let (x, y) = side_point(self.right, 134.0, 69.0);
        p.rotated((x, y), if self.right { 15.0 } else { -15.0 }, |p| {
            p.text(
                if self.right { "THR" } else { "BRK" },
                rect(x - 36.0, y - 7.0, 72.0, 15.0),
                CarbonLabel,
                WHITE,
            );
        });
    }
}

pub(super) struct Replica {
    left: ID2D1PathGeometry,
    right: ID2D1PathGeometry,
    brow: ID2D1PathGeometry,
    dividers: [ID2D1PathGeometry; 3],
    ring: ID2D1PathGeometry,
    inner_ring: ID2D1PathGeometry,
    mesh: ID2D1PathGeometry,
    pedals: [Pedal; 2],
    names: [TextCurve; 2],
    positions: [TextCurve; 2],
    gaps: [TextCurve; 2],
    aero: [TextCurve; 2],
    gears: [TextCurve; 9],
    gear_label: TextCurve,
    rpm_label: TextCurve,
    rpm_zero: TextCurve,
    rpm_end: TextCurve,
}

impl Replica {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        Ok(Self {
            left: modern::panel(factory, WINGS[0].0, WINGS[0].1, WING_TOP, WING_BOTTOM)?,
            right: modern::panel(factory, WINGS[1].0, WINGS[1].1, WING_TOP, WING_BOTTOM)?,
            brow: modern::brow(factory)?,
            dividers: [
                path(
                    factory,
                    (0..=96).map(|i| {
                        wing_point(
                            WINGS[0].0 + (WINGS[0].1 - WINGS[0].0) * i as f32 / 96.0,
                            SPLIT_ROW,
                        )
                    }),
                    false,
                )?,
                path(
                    factory,
                    [
                        wing_point(-18.5, WING_TOP + 5.0),
                        wing_point(-18.5, WING_BOTTOM - 5.0),
                    ],
                    false,
                )?,
                path(
                    factory,
                    [
                        wing_point(18.5, WING_TOP + 5.0),
                        wing_point(18.5, WING_BOTTOM - 5.0),
                    ],
                    false,
                )?,
            ],
            ring: ring(factory, 80.0)?,
            inner_ring: ring(factory, 76.0)?,
            mesh: dot_mesh(factory)?,
            pedals: [Pedal::new(factory, false)?, Pedal::new(factory, true)?],
            names: NAME_ROWS.map(|offset| text_curve(-28.1, -18.9, offset)),
            positions: NAME_ROWS.map(|offset| text_curve(-30.9, -28.9, offset)),
            gaps: NAME_ROWS.map(|offset| text_curve(-18.0, -13.5, offset)),
            aero: [META_ROW, META_ROW + 15.0].map(|offset| text_curve(13.7, 18.0, offset)),
            gears: std::array::from_fn(|i| {
                let angle = FIRST_GEAR + i as f32 * (LAST_GEAR - FIRST_GEAR) / 8.0;
                text_curve(angle - 0.6, angle + 0.6, GEAR_ROW)
            }),
            gear_label: text_curve(FIRST_GEAR, LAST_GEAR, META_ROW),
            rpm_label: text_curve(22.0, 28.0, RPM_CAPTION),
            rpm_zero: text_curve(FIRST_GEAR - 0.6, FIRST_GEAR + 0.6, RPM_CAPTION),
            rpm_end: text_curve(LAST_GEAR - 0.7, LAST_GEAR + 0.7, RPM_CAPTION),
        })
    }

    pub fn draw(&self, p: &Painter<'_>, snapshot: &Snapshot, settings: &Settings, now: Instant) {
        p.translated(ORIGIN, |p| {
            let active = snapshot.driving(now);
            let tel = snapshot.telemetry.filter(|_| active).map(|t| t.value);
            let status = snapshot
                .status
                .filter(|s| active && s.fresh(now, FAST_TTL))
                .map(|s| s.value);
            let aero = current_aero(snapshot, now);
            let opacity = settings.halo.opacity;
            let accent = settings.colours.accent;
            // The reference has a mint throttle; explicit colour overrides still apply.
            let throttle =
                if settings.colours.throttle == crate::settings::Colours::default().throttle {
                    MINT
                } else {
                    settings.colours.throttle
                };

            p.stroke_geometry(&self.brow, [15, 25, 28], 3.0, 0.2);
            p.stroke_geometry(&self.brow, [220, 232, 231], 1.2, 0.85);
            for panel in [&self.left, &self.right] {
                p.material_geometry(panel, opacity * 0.72);
                p.stroke_geometry(panel, accent, 1.0, 0.85);
            }
            for divider in &self.dividers {
                p.stroke_geometry(divider, accent, 0.8, 0.5);
            }
            self.draw_standings(p, snapshot, now);

            p.circle(DIAL.0, DIAL.1, 79.5, GLASS, opacity * 0.9);
            p.stroke_geometry(&self.mesh, BORDER, 0.55, opacity * 0.26);
            p.stroke_geometry(&self.ring, [28, 44, 51], 5.0, 0.8);
            p.stroke_geometry(&self.ring, [142, 197, 221], 1.8, 0.95);
            p.stroke_geometry(&self.inner_ring, CYAN, 0.65, 0.32);
            p.translated(
                (DIAL.0 - DIAL_REFERENCE.0, DIAL.1 - DIAL_REFERENCE.1),
                |p| {
                    self.draw_dial(
                        p,
                        tel,
                        status,
                        aero,
                        snapshot.ers_capacity.map(|c| c.joules),
                        opacity,
                    )
                },
            );
            let budgets = lap_budgets(status, snapshot.ers_deploy_limit_joules);
            self.pedals[0].draw(
                p,
                tel.map(|t| t.brake),
                settings.colours.brake,
                opacity,
                budgets[0],
            );
            self.pedals[1].draw(p, tel.map(|t| t.throttle), throttle, opacity, budgets[1]);
            self.draw_gears(p, tel, status, aero, accent, throttle);

            let connection = snapshot.connection(now);
            if connection != Connection::Live {
                p.text(
                    connection.label(),
                    rect(DIAL.0 - 115.0, 204.0, 230.0, 15.0),
                    HudLabel,
                    MUTED,
                );
            }
        });
    }

    fn draw_standings(&self, p: &Painter<'_>, snapshot: &Snapshot, now: Instant) {
        let active = snapshot.driving(now);
        let position = snapshot
            .race_position
            .filter(|s| active && s.fresh(now, FAST_TTL) && s.value > 0)
            .map(|s| s.value);
        for (i, opponent) in snapshot.race_opponents(now).into_iter().enumerate() {
            let name =
                opponent.and_then(|opponent| snapshot.participant_name(opponent.car_index, now));
            let number = opponent.and(position).and_then(|n| {
                if i == 0 {
                    n.checked_sub(1).filter(|&n| n > 0)
                } else {
                    n.checked_add(1)
                }
            });
            if let Some(number) = number {
                p.curved_textf(
                    format_args!("{number}"),
                    &self.positions[i],
                    HudNumber,
                    MUTED,
                );
            } else {
                p.curved_text("--", &self.positions[i], HudNumber, MUTED);
            }
            let mut uppercase = [0; 32];
            let name = name.unwrap_or("--");
            uppercase[..name.len()].copy_from_slice(name.as_bytes());
            uppercase[..name.len()].make_ascii_uppercase();
            p.curved_text_ellipsized(
                std::str::from_utf8(&uppercase[..name.len()]).unwrap(),
                &self.names[i],
                HudStraight,
                if name == "--" { MUTED } else { WHITE },
            );
            if let Some(gap) = opponent {
                // Long intervals keep a readable size in the same fixed-width column.
                let decimals = if gap.millis < 100_000 {
                    3
                } else if gap.millis < 1_000_000 {
                    1
                } else {
                    0
                };
                p.curved_textf(
                    format_args!("{:.*}s", decimals, f64::from(gap.millis) / 1000.0),
                    &self.gaps[i],
                    HudFuel,
                    WHITE,
                );
            } else {
                p.curved_text("--", &self.gaps[i], HudFuel, MUTED);
            }
        }
    }

    fn draw_dial(
        &self,
        p: &Painter<'_>,
        tel: Option<CarTelemetry>,
        status: Option<CarStatus>,
        aero: Option<Aero>,
        capacity: Option<f32>,
        opacity: f32,
    ) {
        p.text("KM/H", rect(369.0, 83.0, 70.0, 17.0), HudLabel, WHITE);
        if let Some(t) = tel {
            p.stretched((404.0, 116.0), (1.22, 1.0), |p| {
                p.textf(
                    format_args!("{}", t.speed_kmh),
                    rect(350.0, 99.0, 108.0, 35.0),
                    HudSecondary,
                    WHITE,
                )
            });
            p.stretched((404.0, 180.0), (1.15, 1.0), |p| {
                p.textf(
                    format_args!("{:.0}", f64::from(t.speed_kmh) / 1.609_344),
                    rect(354.0, 168.0, 100.0, 25.0),
                    HudNumber,
                    WHITE,
                )
            });
        } else {
            p.text("--", rect(350.0, 99.0, 108.0, 35.0), HudSecondary, MUTED);
            p.text("--", rect(354.0, 168.0, 100.0, 25.0), HudNumber, MUTED);
        }
        p.text("MPH", rect(369.0, 191.0, 70.0, 15.0), HudLabel, WHITE);
        let boost = status.is_some_and(|s| s.deploy_mode == 3);
        for i in 0..24 {
            let x = 350.0 + i as f32 * 4.6;
            p.line(
                (x + 2.5, 134.0),
                (x, 145.0),
                if boost { CYAN } else { OFF },
                2.3,
                0.9,
            );
        }
        p.rounded(rect(376.0, 133.0, 55.0, 14.0), 1.5, GLASS, opacity);
        p.text(
            deploy_label(status),
            rect(376.0, 132.0, 55.0, 16.0),
            HudLabel,
            if status.is_some() { WHITE } else { MUTED },
        );
        let overtake = aero.filter(|a| a.regulations_2026);
        let available = overtake.is_some_and(|a| a.overtake_available || a.overtake_active);
        let colour = if available { [59, 209, 104] } else { OFF };
        p.rounded(
            rect(350.0, 150.0, 109.0, 13.0),
            1.0,
            colour,
            if available { 0.28 } else { opacity * 0.3 },
        );
        p.rounded_outline(rect(350.0, 150.0, 109.0, 13.0), 1.0, colour, 1.3, 0.95);
        p.text(
            "OVERTAKE",
            rect(350.0, 148.5, 109.0, 16.0),
            HudLabel,
            if available { colour } else { MUTED },
        );
        let battery = battery_body();
        p.rounded(battery, 2.0, [3, 54, 74], opacity);
        let percentage = status.and_then(|s| ers_percent(s.ers_joules, capacity));
        if let Some(percent) = percentage.filter(|&p| p > 0.0) {
            p.rounded(
                rect(
                    battery.left + 1.5,
                    battery.top + 1.5,
                    59.0 * percent / 100.0,
                    15.0,
                ),
                1.0,
                [0, 111, 157],
                0.9,
            );
        }
        p.rounded_outline(battery, 2.0, CYAN, 1.0, 0.9);
        p.rounded_outline(
            rect(battery.right, battery.top + 5.0, 2.0, 8.0),
            0.5,
            CYAN,
            1.0,
            0.9,
        );
        let caption = rect(battery.left, battery.top, 64.0, 18.0);
        if let Some(percent) = percentage {
            p.textf(format_args!("{percent:.0}%"), caption, HudFuel, WHITE);
        } else {
            p.text("--%", caption, HudFuel, MUTED);
        }
    }

    fn draw_gears(
        &self,
        p: &Painter<'_>,
        tel: Option<CarTelemetry>,
        status: Option<CarStatus>,
        aero: Option<Aero>,
        accent: [u8; 3],
        throttle: [u8; 3],
    ) {
        p.curved_text("ACTIVE", &self.aero[0], HudLabel, WHITE);
        p.curved_text("AERO", &self.aero[1], HudLabel, WHITE);
        let straight = aero.filter(|a| a.regulations_2026).map(|a| a.straight);
        for offset in [0.0, 11.0] {
            let (a, b) = if straight == Some(true) {
                (
                    wing_point(14.8, WING_TOP + 53.0 + offset),
                    wing_point(17.0, WING_TOP + 53.0 + offset),
                )
            } else {
                (
                    wing_point(15.1 + offset * 0.13, WING_TOP + 51.0),
                    wing_point(14.5 + offset * 0.13, WING_BOTTOM - 12.0),
                )
            };
            let length = (b.0 - a.0).hypot(b.1 - a.1);
            let angle = (b.1 - a.1).atan2(b.0 - a.0).to_degrees();
            p.rotated(a, angle, |p| {
                p.rounded(
                    rect(a.0 - 1.2, a.1 - 2.4, length + 2.4, 4.8),
                    2.4,
                    aero_border(aero),
                    1.0,
                );
                p.rounded(
                    rect(a.0, a.1 - 1.2, length, 2.4),
                    1.2,
                    if straight.is_some() {
                        [169, 237, 237]
                    } else {
                        OFF
                    },
                    1.0,
                );
            });
        }
        p.curved_text("GEARS", &self.gear_label, HudLabel, MUTED);
        for (i, curve) in self.gears.iter().enumerate() {
            let gear = if i == 0 && tel.is_some_and(|t| t.gear == -1) {
                -1
            } else {
                i as i8
            };
            let selected = tel.is_some_and(|t| t.gear == gear);
            p.curved_text(
                gear_label(gear),
                curve,
                if selected { HudNumber } else { HudFuel },
                if selected { WHITE } else { MUTED },
            );
        }
        let rpm = rpm_fraction(tel, status);
        for i in 0..64 {
            let start = FIRST_GEAR + i as f32 * (LAST_GEAR - FIRST_GEAR) / 64.0;
            let end = start + (LAST_GEAR - FIRST_GEAR) / 64.0;
            let a = wing_point(start, RPM_ROW);
            let b = wing_point(end, RPM_ROW);
            p.line(a, b, OFF, 3.2, 0.85);
            if let Some(rpm) = rpm {
                let amount = (rpm * 64.0 - i as f32).clamp(0.0, 1.0);
                if amount > 0.0 {
                    p.line(
                        a,
                        (a.0 + (b.0 - a.0) * amount, a.1 + (b.1 - a.1) * amount),
                        if aero.is_some_and(|a| a.overtake_active) {
                            throttle
                        } else {
                            accent
                        },
                        3.2,
                        1.0,
                    );
                }
            }
        }
        if let Some(t) = tel {
            p.curved_textf(
                format_args!("RPM {}", t.rpm),
                &self.rpm_label,
                HudLabel,
                MUTED,
            );
        } else {
            p.curved_text("RPM --", &self.rpm_label, HudLabel, MUTED);
        }
        p.curved_text("0", &self.rpm_zero, HudLabel, MUTED);
        if let Some(s) = status.filter(|s| s.max_rpm > 0) {
            p.curved_textf(
                format_args!("{:.0}", s.max_rpm as f32 / 1000.0),
                &self.rpm_end,
                HudLabel,
                MUTED,
            );
        } else {
            p.curved_text("--", &self.rpm_end, HudLabel, MUTED);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lap_budgets_use_separate_limits_and_reset_with_the_reported_counters() {
        let status = CarStatus {
            ers_joules: 1_000_000.0,
            harvested_mguk_joules: 3_400_000.0,
            harvest_limit_joules: 8_500_000.0,
            deployed_joules: 7_150_000.0,
            ..CarStatus::default()
        };
        let budgets = lap_budgets(Some(status), Some(13_000_000.0)).map(Option::unwrap);
        assert_eq!(budgets[0].joules, 5_100_000.0);
        assert!((budgets[0].fraction - 0.6).abs() < 0.0001);
        assert_eq!(budgets[1].joules, 5_850_000.0);
        assert!((budgets[1].fraction - 0.45).abs() < 0.0001);
        assert_eq!(
            lap_budgets(
                Some(CarStatus {
                    ers_joules: 4_000_000.0,
                    ..status
                }),
                Some(13_000_000.0)
            ),
            budgets.map(Some)
        );
        let reset = lap_budgets(
            Some(CarStatus {
                harvested_mguk_joules: 0.0,
                deployed_joules: 0.0,
                ..status
            }),
            Some(13_000_000.0),
        )
        .map(Option::unwrap);
        assert_eq!(
            reset[0],
            LapBudget {
                joules: 8_500_000.0,
                fraction: 1.0
            }
        );
        assert_eq!(
            reset[1],
            LapBudget {
                joules: 13_000_000.0,
                fraction: 1.0
            }
        );
        assert_eq!(lap_budgets(None, Some(13_000_000.0)), [None; 2]);
        assert_eq!(lap_budgets(Some(status), None), [Some(budgets[0]), None]);
        assert_eq!(
            remaining_budget(9_000_000.0, 8_500_000.0),
            Some(LapBudget {
                joules: 0.0,
                fraction: 0.0
            })
        );
        assert_eq!(
            remaining_budget(0.0, 0.0),
            Some(LapBudget {
                joules: 0.0,
                fraction: 0.0
            })
        );
        for invalid in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(remaining_budget(invalid, 8_500_000.0).is_none());
            assert!(remaining_budget(0.0, invalid).is_none());
        }
    }

    #[test]
    fn battery_outline_is_centred_and_aero_border_requires_actual_engagement() {
        let battery = battery_body();
        let centre = (battery.left + battery.right + 2.0) / 2.0;
        assert_eq!(centre, DIAL_REFERENCE.0);
        assert_eq!(centre + DIAL.0 - DIAL_REFERENCE.0 + ORIGIN.0, WIDTH / 2.0);
        let aero = Aero {
            regulations_2026: true,
            available: true,
            ..Aero::default()
        };
        assert_eq!(aero_border(Some(aero)), AERO_RED);
        assert_eq!(
            aero_border(Some(Aero {
                straight: true,
                ..aero
            })),
            AERO_GREEN
        );
        assert_eq!(
            aero_border(Some(Aero {
                regulations_2026: false,
                ..aero
            })),
            OFF
        );
        assert_eq!(aero_border(None), OFF);
    }

    #[test]
    fn wings_brow_and_dial_share_the_centre_and_mirrored_shift_curvature() {
        assert_eq!(DIAL.0 + ORIGIN.0, WIDTH / 2.0);
        assert_eq!(WINGS[0], (-WINGS[1].1, -WINGS[1].0));
        for i in 0..=74 {
            let angle = 13.0 + i as f32 / 4.0;
            for depth in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let offset = WING_TOP + (WING_BOTTOM - WING_TOP) * depth;
                let (x, y) = wing_point(angle, offset);
                let (left_x, left_y) = wing_point(-angle, offset);
                assert!((x + left_x - 2.0 * DIAL.0).abs() < 0.001);
                assert!((y - left_y).abs() < 0.001);
                for x in [x, left_x] {
                    assert!((0.0..WIDTH).contains(&(x + ORIGIN.0)));
                    assert!((0.0..HEIGHT).contains(&(y + ORIGIN.1)));
                }
                let radius = (x - DIAL.0).hypot(y - (153.0 + modern::RADIUS));
                assert!((radius - (modern::RADIUS - offset)).abs() < 0.001);
            }
            let upper = wing_point(angle, WING_TOP);
            let lower = wing_point(angle, WING_BOTTOM);
            assert!(((upper.0 - lower.0).hypot(upper.1 - lower.1) - 80.0).abs() < 0.001);
        }
        let mirror = |(x, y)| (2.0 * DIAL.0 - x, y);
        let right = [
            modern::BROW[3],
            mirror(modern::BROW[2]),
            mirror(modern::BROW[1]),
            mirror(modern::BROW[0]),
        ];
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            let a = classic::curve_point(&modern::BROW, t);
            let b = classic::curve_point(&right, 1.0 - t);
            assert!((a.0 + b.0 - 2.0 * DIAL.0).abs() < 0.001);
            assert!((a.1 - b.1).abs() < 0.001);
            let angle = (a.0 - DIAL.0)
                .atan2(153.0 + modern::RADIUS - a.1)
                .to_degrees();
            if (WINGS[0].0..WINGS[0].1).contains(&angle) {
                let radius = (a.0 - DIAL.0).hypot(a.1 - (153.0 + modern::RADIUS));
                assert!(
                    modern::RADIUS - radius < WING_TOP - 8.0,
                    "brow enters the wing"
                );
            }
        }
        assert_eq!(modern::BROW[3], (DIAL.0, 8.0));
        for right in [false, true] {
            for radius in [83.0, 112.0, 116.0, 150.0] {
                for y in [-47.0, 0.0, 59.0] {
                    let (x, y) = side_point(right, radius, y);
                    assert!(((x - DIAL.0).hypot(y - DIAL.1) - radius).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn text_rows_leave_space_for_names_gears_rpm_and_panel_margins() {
        for row in NAME_ROWS {
            assert!(row - WING_TOP - 11.0 >= 9.0 && WING_BOTTOM - row - 11.0 >= 9.0);
            assert!((row - SPLIT_ROW).abs() - 11.0 >= 9.0);
        }
        const {
            assert!(META_ROW - WING_TOP - 5.0 >= 8.0);
            assert!(GEAR_ROW - META_ROW - 11.0 - 5.0 >= 5.0);
            assert!(RPM_ROW - GEAR_ROW - 11.0 - 1.6 >= 6.0);
            assert!(RPM_CAPTION - RPM_ROW - 5.0 - 1.6 >= 8.0);
            assert!(WING_BOTTOM - RPM_CAPTION - 5.0 >= 7.0);
        }
        let gear_step = (modern::RADIUS - GEAR_ROW) * ((LAST_GEAR - FIRST_GEAR) / 8.0).to_radians();
        assert!(gear_step >= 17.5);
        assert!(FIRST_GEAR - 18.5 >= 1.0 && WINGS[1].1 - LAST_GEAR >= 1.0);
    }
}
