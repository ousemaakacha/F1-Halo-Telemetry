use super::*;

const RADIUS: f32 = 710.0;
const BAND_DEPTH: f32 = 76.0;
const TEXT_ROWS: [f32; 2] = [-53.0, -20.0];

fn edge(x: f32, offset: f32) -> f32 {
    153.0 + RADIUS - ((RADIUS - offset).powi(2) - (x - CENTER_X).powi(2)).sqrt()
}

fn angle(x: f32, offset: f32) -> f32 {
    ((x - CENTER_X) / (RADIUS - offset)).asin().to_degrees()
}

fn left_edge(depth: f32) -> f32 {
    42.0 + 48.0 * depth - 24.0 * depth * depth
}

fn left_text_curve(start: f32, end: f32, offset: f32) -> TextCurve {
    TextCurve::new((0..=96).map(|i| {
        let x = start + (end - start) * i as f32 / 96.0;
        (x, edge(x, offset))
    }))
}

pub(super) struct Modern {
    body: ID2D1PathGeometry,
    brow: ID2D1PathGeometry,
    top: ID2D1PathGeometry,
    lower_edge: ID2D1PathGeometry,
    straight_text: TextCurve,
    delta_text: TextCurve,
    fuel_text: [TextCurve; 2],
    rpm_segments: [((f32, f32), (f32, f32)); 48],
}

impl Modern {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        Ok(Self {
            body: strip(factory, 0.0, 1.0)?,
            brow: classic::bezier_path(
                factory,
                (162.0, 102.0),
                &[
                    [(305.0, 33.0), (354.0, 8.0), (420.0, 8.0)],
                    [(486.0, 8.0), (535.0, 33.0), (678.0, 102.0)],
                ],
                false,
            )?,
            top: arc(factory, 42.0, 798.0, -BAND_DEPTH)?,
            lower_edge: arc(factory, 66.0, 774.0, 0.0)?,
            straight_text: left_text_curve(42.0, 196.0, TEXT_ROWS[0]),
            delta_text: left_text_curve(42.0, 196.0, TEXT_ROWS[1]),
            fuel_text: TEXT_ROWS.map(|offset| left_text_curve(202.0, 262.0, offset)),
            rpm_segments: std::array::from_fn(|i| {
                let x = 566.0 + i as f32 * 194.0 / 48.0;
                let next = 566.0 + (i + 1) as f32 * 194.0 / 48.0;
                ((x, edge(x, -26.0)), (next, edge(next, -26.0)))
            }),
        })
    }

    pub fn draw_frame(&self, p: &Painter<'_>, accent: [u8; 3], opacity: f32) {
        p.material_geometry(&self.body, opacity);
        p.stroke_geometry(&self.top, [111, 123, 133], 0.5, 0.16);
        p.stroke_geometry(&self.brow, [5, 8, 12], 2.0, 0.25);
        p.stroke_geometry(&self.brow, [204, 211, 218], 0.75, 0.68);
        for (width, alpha) in [(18.0, 0.025), (9.0, 0.06), (4.0, 0.18), (2.1, 1.0)] {
            p.stroke_geometry(&self.lower_edge, accent, width, alpha);
        }
    }

    pub fn draw_wings(
        &self,
        p: &Painter<'_>,
        tel: Option<CarTelemetry>,
        status: Option<CarStatus>,
        aero: Option<Aero>,
        delta: Option<u32>,
        settings: &Settings,
    ) {
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
        draw_gap_behind(p, delta, &self.delta_text);
        draw_fuel(p, status, &self.fuel_text, 1.2);
        for i in 0..9 {
            let gear = if i == 0 && tel.is_some_and(|t| t.gear == -1) {
                -1
            } else {
                i
            };
            let selected = tel.is_some_and(|t| t.gear == gear);
            let x = 580.0 + i as f32 * 22.0;
            let y = edge(x, -47.0);
            p.rotated((x, y), angle(x, -47.0), |p| {
                p.text(
                    gear_label(gear),
                    rect(x - 12.0, y - 14.0, 24.0, 28.0),
                    HudNumber,
                    if selected { WHITE } else { MUTED },
                );
                if selected {
                    p.fill(
                        rect(x - 8.0, y + 13.0, 16.0, 2.0),
                        settings.colours.accent,
                        1.0,
                    );
                }
            });
        }
        let rpm = rpm_fraction(tel, status);
        for (i, &(a, b)) in self.rpm_segments.iter().enumerate() {
            p.line(a, b, EDGE, 3.3, 0.75);
            if let Some(ratio) = rpm {
                let level = (ratio * self.rpm_segments.len() as f32 - i as f32).clamp(0.0, 1.0);
                if level > 0.0 {
                    p.line(
                        a,
                        (a.0 + (b.0 - a.0) * level, a.1 + (b.1 - a.1) * level),
                        settings.colours.accent,
                        3.3,
                        1.0,
                    );
                }
            }
        }
        for (x, width) in [(566.0, 19.0), (663.0, 151.0), (760.0, 31.0)] {
            let y = edge(x, -9.0);
            p.rotated((x, y), angle(x, -9.0), |p| {
                let r = rect(x - width / 2.0, y - 7.0, width, 14.0);
                if x == 566.0 {
                    p.text("0", r, HudLabel, WHITE);
                } else if x == 663.0 {
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

fn strip(factory: &ID2D1Factory1, from: f32, to: f32) -> Result<ID2D1PathGeometry> {
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        let start_x = left_edge(from);
        let top = -BAND_DEPTH * (1.0 - from);
        sink.BeginFigure(point(start_x, edge(start_x, top)), D2D1_FIGURE_BEGIN_FILLED);
        for i in 1..=96 {
            let x = start_x + (840.0 - 2.0 * start_x) * i as f32 / 96.0;
            sink.AddLine(point(x, edge(x, top)));
        }
        for i in 1..=8 {
            let depth = from + (to - from) * i as f32 / 8.0;
            let x = 840.0 - left_edge(depth);
            sink.AddLine(point(x, edge(x, -BAND_DEPTH * (1.0 - depth))));
        }
        let end_x = left_edge(to);
        let bottom = -BAND_DEPTH * (1.0 - to);
        for i in 1..=96 {
            let x = 840.0 - end_x - (840.0 - 2.0 * end_x) * i as f32 / 96.0;
            sink.AddLine(point(x, edge(x, bottom)));
        }
        for i in 1..=8 {
            let depth = to - (to - from) * i as f32 / 8.0;
            let x = left_edge(depth);
            sink.AddLine(point(x, edge(x, -BAND_DEPTH * (1.0 - depth))));
        }
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        Ok(path.into())
    }
}

fn arc(factory: &ID2D1Factory1, left: f32, right: f32, offset: f32) -> Result<ID2D1PathGeometry> {
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(point(left, edge(left, offset)), D2D1_FIGURE_BEGIN_HOLLOW);
        for i in 1..=128 {
            let x = left + (right - left) * i as f32 / 128.0;
            sink.AddLine(point(x, edge(x, offset)));
        }
        sink.EndFigure(D2D1_FIGURE_END_OPEN);
        sink.Close()?;
        Ok(path.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuel_and_sm_follow_the_same_two_rows() {
        let factory: ID2D1Factory1 =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let modern = Modern::new(&factory).unwrap();
        for (row, offset) in TEXT_ROWS.into_iter().enumerate() {
            let sm_curve = if row == 0 {
                &modern.straight_text
            } else {
                &modern.delta_text
            };
            for curve in [sm_curve, &modern.fuel_text[row]] {
                let (x, y) = curve.centre();
                let radius = (x - CENTER_X).hypot(y - 153.0 - RADIUS);
                assert!((radius - (RADIUS - offset)).abs() < 0.02);
            }
        }
    }

    #[test]
    fn continuous_band_is_symmetric_and_keeps_live_values_inside_the_canvas() {
        for depth in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let left = left_edge(depth);
            let offset = -BAND_DEPTH * (1.0 - depth);
            for i in 0..=100 {
                let x = left + (840.0 - 2.0 * left) * i as f32 / 100.0;
                let y = edge(x, offset);
                assert!(y.is_finite() && (0.0..HEIGHT - 35.0).contains(&y));
                assert!((y - edge(840.0 - x, offset)).abs() < 0.001);
                assert!(angle(x, offset).is_finite());
            }
        }
        for i in 0..9 {
            let x = 580.0 + i as f32 * 22.0;
            assert!(edge(x, -47.0) + 17.0 < edge(x, 0.0));
            assert!(x + CONTENT_PADDING + 14.0 < WIDTH);
        }
    }
}
