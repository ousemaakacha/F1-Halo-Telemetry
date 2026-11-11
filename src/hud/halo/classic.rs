use super::*;

const TOP: [(f32, f32); 4] = [(64.0, 188.0), (238.0, 55.0), (602.0, 55.0), (776.0, 188.0)];
const TEXT_ROWS: [f32; 2] = [10.0, 34.0];
const LOWER: [(f32, f32); 4] = [
    (102.0, 246.0),
    (274.0, 110.0),
    (566.0, 110.0),
    (738.0, 246.0),
];

fn curve_point(points: &[(f32, f32); 4], t: f32) -> (f32, f32) {
    let weights = [
        (1.0 - t).powi(3),
        3.0 * t * (1.0 - t).powi(2),
        3.0 * t * t * (1.0 - t),
        t.powi(3),
    ];
    points
        .iter()
        .zip(weights)
        .fold((0.0, 0.0), |(x, y), (&(px, py), weight)| {
            (x + px * weight, y + py * weight)
        })
}

pub(super) fn lower_angle(x: f32) -> f32 {
    let (mut start, mut end) = (0.0, 1.0);
    for _ in 0..16 {
        let t = (start + end) / 2.0;
        if curve_point(&LOWER, t).0 < x {
            start = t;
        } else {
            end = t;
        }
    }
    let t = (start + end) / 2.0;
    let a = curve_point(&LOWER, t - 0.001);
    let b = curve_point(&LOWER, t + 0.001);
    (b.1 - a.1).atan2(b.0 - a.0).to_degrees()
}

fn left_text_curve(start: f32, end: f32, offset: f32) -> TextCurve {
    TextCurve::new((0..=96).map(|i| {
        let (x, y) = curve_point(&TOP, start + (end - start) * i as f32 / 96.0);
        (x, y + offset)
    }))
}

pub(super) struct Classic {
    ribbon: ID2D1PathGeometry,
    brow: ID2D1PathGeometry,
    lower_edge: ID2D1PathGeometry,
    straight_text: TextCurve,
    delta_text: TextCurve,
    fuel_text: [TextCurve; 2],
}

impl Classic {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        Ok(Self {
            ribbon: bezier_path(
                factory,
                TOP[0],
                &[
                    [TOP[1], TOP[2], TOP[3]],
                    [(776.0, 198.0), (776.0, 209.0), (776.0, 220.0)],
                    [(591.0, 119.0), (249.0, 119.0), (64.0, 220.0)],
                ],
                true,
            )?,
            brow: bezier_path(
                factory,
                (178.0, 98.0),
                &[
                    [(305.0, 47.0), (355.0, 16.0), (420.0, 16.0)],
                    [(485.0, 16.0), (535.0, 47.0), (662.0, 98.0)],
                ],
                false,
            )?,
            lower_edge: bezier_path(factory, LOWER[0], &[[LOWER[1], LOWER[2], LOWER[3]]], false)?,
            straight_text: left_text_curve(0.03, 0.27, TEXT_ROWS[0]),
            delta_text: left_text_curve(0.03, 0.27, TEXT_ROWS[1]),
            fuel_text: TEXT_ROWS.map(|offset| left_text_curve(0.26, 0.335, offset)),
        })
    }

    pub fn draw_frame(&self, p: &Painter<'_>, accent: [u8; 3], opacity: f32) {
        p.geometry(&self.ribbon, INK, opacity);
        p.stroke_geometry(&self.brow, WHITE, 0.9, 0.78);
        for (width, alpha) in [(20.0, 0.015), (12.0, 0.035), (5.0, 0.1), (1.6, 0.95)] {
            p.stroke_geometry(&self.lower_edge, accent, width, alpha);
        }
    }

    pub fn draw_wings(
        &self,
        p: &Painter<'_>,
        tel: Option<CarTelemetry>,
        status: Option<CarStatus>,
        aero: Option<Aero>,
        delta: Option<i64>,
        settings: &Settings,
    ) {
        let accent = settings.colours.accent;
        let sm_active = aero.is_some_and(|a| a.regulations_2026 && a.straight);
        draw_straight_mode(
            p,
            aero,
            &self.straight_text,
            if sm_active {
                settings.colours.throttle
            } else if aero.is_some() {
                WHITE
            } else {
                MUTED
            },
        );
        draw_lap_delta(p, delta, &self.delta_text, settings);
        draw_fuel(p, status, &self.fuel_text, 1.0);
        p.rotated((650.0, 160.0), 22.0, |p| {
            let wing = rect(550.0, 124.0, 224.0, 70.0);
            p.rounded(wing, 10.0, INK, settings.halo.opacity);
            p.rounded_outline(wing, 10.0, EDGE, 0.9, 0.85);
            p.text("GEARS", rect(615.0, 122.0, 94.0, 16.0), HudLabel, WHITE);
            for i in 0..9 {
                let gear = if i == 0 && tel.is_some_and(|t| t.gear == -1) {
                    -1
                } else {
                    i
                };
                let selected = tel.is_some_and(|t| t.gear == gear);
                let x = 552.0 + i as f32 * 24.0;
                if selected {
                    p.rounded(rect(x, 139.0, 23.0, 27.0), 3.0, accent, 0.22);
                    p.fill(rect(x + 5.0, 164.0, 13.0, 2.0), accent, 1.0);
                }
                p.text(
                    gear_label(gear),
                    rect(x, 139.0, 23.0, 26.0),
                    HudNumber,
                    if selected { WHITE } else { MUTED },
                );
            }
            p.fill(rect(555.0, 171.0, 210.0, 3.0), EDGE, 0.8);
            if let Some(fraction) = rpm_fraction(tel, status) {
                p.fill(rect(555.0, 171.0, 210.0 * fraction, 3.0), accent, 1.0);
            }
            p.text("0", rect(548.0, 179.0, 19.0, 14.0), HudLabel, WHITE);
            if let Some(t) = tel {
                p.textf(
                    format_args!("{:.1} RPM x1000", t.rpm as f32 / 1000.0),
                    rect(576.0, 179.0, 161.0, 14.0),
                    HudLabel,
                    WHITE,
                );
            } else {
                p.text(
                    "-- RPM x1000",
                    rect(576.0, 179.0, 161.0, 14.0),
                    HudLabel,
                    MUTED,
                );
            }
            if let Some(s) = status.filter(|s| s.max_rpm > 0) {
                p.textf(
                    format_args!("{:.0}", s.max_rpm as f32 / 1000.0),
                    rect(746.0, 179.0, 31.0, 14.0),
                    HudLabel,
                    WHITE,
                );
            } else {
                p.text("--", rect(746.0, 179.0, 31.0, 14.0), HudLabel, MUTED);
            }
        });
    }
}

pub(super) fn bezier_path(
    factory: &ID2D1Factory1,
    start: (f32, f32),
    curves: &[[(f32, f32); 3]],
    filled: bool,
) -> Result<ID2D1PathGeometry> {
    // SAFETY: these immutable geometries are closed once and owned by the UI thread.
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(
            point(start.0, start.1),
            if filled {
                D2D1_FIGURE_BEGIN_FILLED
            } else {
                D2D1_FIGURE_BEGIN_HOLLOW
            },
        );
        for curve in curves {
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: point(curve[0].0, curve[0].1),
                point2: point(curve[1].0, curve[1].1),
                point3: point(curve[2].0, curve[2].1),
            });
        }
        sink.EndFigure(if filled {
            D2D1_FIGURE_END_CLOSED
        } else {
            D2D1_FIGURE_END_OPEN
        });
        sink.Close()?;
        Ok(path.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_rotation_matches_the_lower_border_tangent() {
        for t in [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0] {
            let x = curve_point(&LOWER, t).0;
            let dx = 516.0 + 720.0 * t - 720.0 * t * t;
            let dy = -408.0 + 816.0 * t;
            let expected = dy.atan2(dx).to_degrees();
            assert!((lower_angle(x) - expected).abs() < 0.02);
        }
        let fuel_angle = lower_angle(260.0);
        assert!((-20.0..-16.0).contains(&fuel_angle));
    }
}
