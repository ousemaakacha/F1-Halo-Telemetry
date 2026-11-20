use super::*;

// Match Modern's frame coordinates while leaving its renderer unchanged.
const RADIUS: f32 = 710.0;
const ROWS: [f32; 3] = [-61.5, -36.5, -11.5];
const LEFT_COLUMNS: [(f32, f32); 2] = [(-28.0, -20.25), (-19.75, -12.0)];
const RIGHT_COLUMNS: [(f32, f32); 3] = [(12.0, 17.0), (17.5, 22.5), (23.0, 28.0)];
const PEDALS: [(f32, f32); 2] = [(301.0, -9.0), (539.0, 9.0)];
const PEDAL_WIDTH: f32 = 36.0;
const PEDAL_BOTTOM: f32 = 140.0;
const PEDAL_STEP: f32 = 5.5;
const PEDAL_HEIGHT: f32 = 4.2;
const PEDAL_LABEL_Y: f32 = 148.0;
pub(super) const SHIFT_RED: [u8; 3] = [255, 55, 64];
const LAUNCH_YELLOW: [u8; 3] = [255, 209, 68];

pub(super) fn shift_flash(tel: Option<CarTelemetry>, elapsed: std::time::Duration) -> bool {
    tel.is_some_and(|t| (1..=8).contains(&t.gear) && t.rpm >= 12_500)
        && elapsed.as_millis() / 100 % 2 == 0
}

fn launch_angle(status: Option<CarStatus>) -> Option<f32> {
    status
        .filter(|s| s.max_rpm >= 10_000)
        .map(|s| 135.0 + 270.0 * 10_000.0 / s.max_rpm as f32)
}

pub(super) fn draw_launch_marker(p: &Painter<'_>, status: Option<CarStatus>) {
    if let Some(angle) = launch_angle(status) {
        let (sin, cos) = angle.to_radians().sin_cos();
        let at = |radius: f32| (CENTER_X + radius * cos, 94.0 + radius * sin);
        p.line(at(58.0), at(71.0), LAUNCH_YELLOW, 2.6, 1.0);
        let (x, y) = at(81.0);
        p.text(
            "10k",
            rect(x - 13.0, y - 6.5, 26.0, 13.0),
            HudLabel,
            LAUNCH_YELLOW,
        );
    }
}

fn band_point(angle: f32, offset: f32) -> (f32, f32) {
    let radius = RADIUS - offset;
    (
        CENTER_X + radius * angle.sin(),
        153.0 + RADIUS - radius * angle.cos(),
    )
}

fn text_curve(start: f32, end: f32, offset: f32) -> TextCurve {
    TextCurve::new((0..=96).map(|i| {
        let angle = (start + (end - start) * i as f32 / 96.0).to_radians();
        band_point(angle, offset)
    }))
}

fn panel(factory: &ID2D1Factory1, start: f32, end: f32) -> Result<ID2D1PathGeometry> {
    let left = start.to_radians() * RADIUS;
    let right = end.to_radians() * RADIUS;
    let corner = 3.0;
    let corners = [
        (left + corner, -70.0 + corner, 180.0_f32),
        (right - corner, -70.0 + corner, 270.0),
        (right - corner, -4.0 - corner, 0.0),
        (left + corner, -4.0 - corner, 90.0),
    ];
    let mut contour = Vec::new();
    for (index, &(x, y, start)) in corners.iter().enumerate() {
        for i in 0..=8 {
            let angle = (start + 90.0 * i as f32 / 8.0).to_radians();
            contour.push((x + corner * angle.cos(), y + corner * angle.sin()));
        }
        let &(next_x, next_y, next_start) = &corners[(index + 1) % corners.len()];
        let start = *contour.last().unwrap();
        let end = (
            next_x + corner * next_start.to_radians().cos(),
            next_y + corner * next_start.to_radians().sin(),
        );
        for i in 1..=32 {
            let t = i as f32 / 32.0;
            contour.push((
                start.0 + (end.0 - start.0) * t,
                start.1 + (end.1 - start.1) * t,
            ));
        }
    }
    // SAFETY: each immutable contour is closed once and cached on the UI thread.
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        for (i, (x, offset)) in contour.into_iter().enumerate() {
            let (x, y) = band_point(x / RADIUS, offset);
            if i == 0 {
                sink.BeginFigure(point(x, y), D2D1_FIGURE_BEGIN_FILLED);
            } else {
                sink.AddLine(point(x, y));
            }
        }
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close()?;
        Ok(path.into())
    }
}

pub(super) struct Left {
    panels: [ID2D1PathGeometry; 2],
    labels: [TextCurve; 2],
    values: [TextCurve; 2],
    units: [TextCurve; 2],
}

impl Left {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        Ok(Self {
            panels: [
                panel(factory, LEFT_COLUMNS[0].0, LEFT_COLUMNS[0].1)?,
                panel(factory, LEFT_COLUMNS[1].0, LEFT_COLUMNS[1].1)?,
            ],
            labels: LEFT_COLUMNS.map(|(start, end)| text_curve(start + 0.55, end - 0.55, ROWS[0])),
            values: LEFT_COLUMNS.map(|(start, end)| text_curve(start + 0.55, end - 0.55, ROWS[1])),
            units: LEFT_COLUMNS.map(|(start, end)| text_curve(start + 0.55, end - 0.55, ROWS[2])),
        })
    }

    pub fn draw(&self, p: &Painter<'_>, gaps: [Option<GapReading>; 2], settings: &Settings) {
        for (i, label) in ["AHEAD", "BEHIND"].into_iter().enumerate() {
            p.stroke_geometry(&self.panels[i], settings.colours.accent, 1.0, 0.85);
            p.curved_text(label, &self.labels[i], HudLabel, MUTED);
            if let Some(gap) = gaps[i] {
                p.curved_textf(
                    format_args!("{}.{:03}", gap.millis / 1000, gap.millis % 1000),
                    &self.values[i],
                    HudNumber,
                    WHITE,
                );
            } else {
                p.curved_text("--", &self.values[i], HudNumber, MUTED);
            }
            p.curved_text("s", &self.units[i], HudLabel, MUTED);
            let angle = (LEFT_COLUMNS[i].0 + LEFT_COLUMNS[i].1) / 2.0 + 2.0;
            let centre = band_point(angle.to_radians(), ROWS[2]);
            p.rotated(centre, angle, |p| {
                let (x, y) = centre;
                let direction = match gaps[i].map(|gap| gap.trend) {
                    Some(GapTrend::Shrinking) => 1.0,
                    Some(GapTrend::Growing) => -1.0,
                    _ => {
                        p.line((x - 3.0, y), (x + 3.0, y), MUTED, 1.4, 1.0);
                        return;
                    }
                };
                let tip = (x, y + direction * 4.0);
                p.line((x, y - direction * 4.0), tip, WHITE, 1.4, 1.0);
                for dx in [-3.0, 3.0] {
                    p.line((x + dx, y), tip, WHITE, 1.4, 1.0);
                }
            });
        }
    }
}

fn right_emphasis(
    status: Option<CarStatus>,
    aero: Option<Aero>,
    colours: &crate::settings::Colours,
) -> [Option<[u8; 3]>; 3] {
    [
        status.and_then(|s| {
            if s.ers_joules == 0.0 {
                Some(colours.brake)
            } else if s.deploy_mode == 3 {
                Some(colours.accent)
            } else {
                None
            }
        }),
        aero.filter(|a| a.regulations_2026).and_then(|a| {
            if a.straight {
                Some(colours.throttle)
            } else if a.available {
                Some(colours.accent)
            } else {
                None
            }
        }),
        fuel_margin(status)
            .filter(|&laps| fuel_sign(laps) == "-")
            .map(|_| colours.brake),
    ]
}

pub(super) struct Right {
    panels: Vec<ID2D1PathGeometry>,
    labels: [TextCurve; 3],
    values: [TextCurve; 3],
    details: [TextCurve; 3],
}

impl Right {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        let curves =
            |offset| RIGHT_COLUMNS.map(|(start, end)| text_curve(start + 0.55, end - 0.55, offset));
        Ok(Self {
            labels: curves(ROWS[0]),
            values: curves(ROWS[1]),
            details: curves(ROWS[2]),
            panels: RIGHT_COLUMNS
                .into_iter()
                .map(|(start, end)| panel(factory, start, end))
                .collect::<Result<_>>()?,
        })
    }

    pub fn draw(
        &self,
        p: &Painter<'_>,
        status: Option<CarStatus>,
        aero: Option<Aero>,
        capacity: Option<f32>,
        settings: &Settings,
    ) {
        let emphasis = right_emphasis(status, aero, &settings.colours);
        let percent = status.and_then(|s| ers_percent(s.ers_joules, capacity));
        let energy_label = if percent.is_none() && status.is_some() {
            "ERS (MJ)"
        } else {
            "ERS"
        };
        for (i, label) in [energy_label, "STRAIGHT", "FUEL"].into_iter().enumerate() {
            p.curved_text(label, &self.labels[i], HudLabel, MUTED);
            p.stroke_geometry(
                &self.panels[i],
                emphasis[i].unwrap_or(settings.colours.accent),
                if emphasis[i].is_some() { 1.5 } else { 1.0 },
                0.85,
            );
        }

        let energy_colour = if status.is_some_and(|s| s.ers_joules == 0.0) {
            settings.colours.brake
        } else {
            ers_colour(aero)
        };
        if let Some(percent) = percent {
            p.curved_textf(
                format_args!("{percent:.0}%"),
                &self.values[0],
                HudNumber,
                energy_colour,
            );
        } else if let Some(s) = status {
            let mj = s.ers_joules / 1_000_000.0;
            if mj < 1000.0 {
                p.curved_textf(
                    format_args!("{mj:.2}"),
                    &self.values[0],
                    HudNumber,
                    energy_colour,
                );
            } else {
                p.curved_textf(
                    format_args!("{mj:.1e}"),
                    &self.values[0],
                    HudNumber,
                    energy_colour,
                );
            }
        } else {
            p.curved_text("--%", &self.values[0], HudNumber, MUTED);
        }
        p.curved_text(
            deploy_label(status),
            &self.details[0],
            HudLabel,
            if status.is_none() {
                MUTED
            } else {
                emphasis[0].unwrap_or(WHITE)
            },
        );

        let straight_label = straight_mode_label(aero);
        p.curved_text(
            straight_label.strip_prefix("SM ").unwrap_or(straight_label),
            &self.values[1],
            HudStraight,
            if aero.is_none() {
                MUTED
            } else {
                emphasis[1].unwrap_or(WHITE)
            },
        );
        if let Some(distance) = straight_mode_distance(aero) {
            p.curved_textf(
                format_args!("{distance} m"),
                &self.details[1],
                HudLabel,
                WHITE,
            );
        }

        if let Some(laps) = fuel_margin(status) {
            let rgb = emphasis[2].unwrap_or(WHITE);
            if laps.abs() < 1000.0 {
                p.curved_textf(
                    format_args!("{}{:.1}", fuel_sign(laps), laps.abs()),
                    &self.values[2],
                    HudNumber,
                    rgb,
                );
            } else {
                p.curved_textf(
                    format_args!("{}{:.1e}", fuel_sign(laps), laps.abs()),
                    &self.values[2],
                    HudNumber,
                    rgb,
                );
            }
        } else {
            p.curved_text("--", &self.values[2], HudNumber, MUTED);
        }
        p.curved_text("LAPS", &self.details[2], HudLabel, MUTED);
    }
}

pub(super) fn draw_pedals(p: &Painter<'_>, tel: Option<CarTelemetry>, settings: &Settings) {
    for (index, (x, angle)) in PEDALS.into_iter().enumerate() {
        let (label, value, rgb) = if index == 0 {
            ("BRK", tel.map(|t| t.brake), settings.colours.brake)
        } else {
            ("THR", tel.map(|t| t.throttle), settings.colours.throttle)
        };
        p.rotated((x, 112.0), angle, |p| {
            for i in 0..10 {
                let y = PEDAL_BOTTOM - i as f32 * PEDAL_STEP;
                p.fill(
                    rect(x - PEDAL_WIDTH / 2.0, y, PEDAL_WIDTH, PEDAL_HEIGHT),
                    [31, 38, 46],
                    0.9,
                );
                if let Some(value) = value {
                    let level = (pedal_percent(value) / 10.0 - i as f32).clamp(0.0, 1.0);
                    if level > 0.0 {
                        p.fill(
                            rect(
                                x - PEDAL_WIDTH / 2.0,
                                y + PEDAL_HEIGHT * (1.0 - level),
                                PEDAL_WIDTH,
                                PEDAL_HEIGHT * level,
                            ),
                            rgb,
                            1.0,
                        );
                    }
                }
            }
            p.text(
                label,
                rect(x - 20.0, PEDAL_LABEL_Y, 40.0, 13.0),
                HudLabel,
                WHITE,
            );
            if value.is_none() {
                p.text("--", rect(x - 16.0, 102.0, 32.0, 21.0), HudFuel, MUTED);
            }
        });
    }
}

pub(super) fn draw_alert(p: &Painter<'_>, alert: RaceAlert, opacity: f32) {
    let rgb = match alert {
        RaceAlert::RedFlag
        | RaceAlert::TimePenalty(_)
        | RaceAlert::DriveThrough
        | RaceAlert::StopGo => [255, 64, 56],
        RaceAlert::Flag(1) => [71, 229, 142],
        RaceAlert::Flag(2) => [54, 160, 255],
        RaceAlert::ChequeredFlag => WHITE,
        _ => [255, 210, 64],
    };
    let bounds = rect(266.0, 208.0, 308.0, 34.0);
    p.rounded(bounds, 5.0, INK, opacity);
    p.rounded_outline(bounds, 5.0, rgb, 1.4, 1.0);
    p.fill(rect(276.0, 219.0, 12.0, 12.0), rgb, 1.0);
    let text = rect(296.0, 210.0, 266.0, 30.0);
    let label = match alert {
        RaceAlert::Flag(1) => "GREEN FLAG",
        RaceAlert::Flag(2) => "BLUE FLAG",
        RaceAlert::Flag(3) => "YELLOW FLAG",
        RaceAlert::SafetyCar(1) => "SAFETY CAR",
        RaceAlert::SafetyCar(2) => "VIRTUAL SAFETY CAR",
        RaceAlert::DriveThrough => "DRIVE THROUGH PENALTY",
        RaceAlert::StopGo => "STOP GO PENALTY",
        RaceAlert::RedFlag => "RED FLAG",
        RaceAlert::ChequeredFlag => "CHEQUERED FLAG",
        RaceAlert::TimePenalty(seconds) => {
            p.textf(
                format_args!("PENALTY +{seconds} s"),
                text,
                HudStraight,
                WHITE,
            );
            return;
        }
        RaceAlert::Warning(count) => {
            p.textf(
                format_args!("OFFICIAL WARNING +{count}"),
                text,
                HudStraight,
                WHITE,
            );
            return;
        }
        _ => return,
    };
    p.text(label, text, HudStraight, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn shift_warning_starts_at_12500_rpm_and_flashes_at_five_hertz() {
        for gear in 1..=8 {
            for rpm in [12_500, 13_099, 16_000] {
                let tel = CarTelemetry {
                    gear,
                    rpm,
                    ..CarTelemetry::default()
                };
                for millis in [0, 99, 200, 299, 400] {
                    assert!(shift_flash(Some(tel), Duration::from_millis(millis)));
                }
                for millis in [100, 199, 300, 399] {
                    assert!(!shift_flash(Some(tel), Duration::from_millis(millis)));
                }
            }
        }
        let normal = CarTelemetry {
            gear: 1,
            rpm: 12_499,
            rev_percent: 100,
            ..CarTelemetry::default()
        };
        assert!(!shift_flash(Some(normal), Duration::ZERO));
        assert!(!shift_flash(None, Duration::ZERO));
        for gear in [-1, 0] {
            assert!(!shift_flash(
                Some(CarTelemetry {
                    gear,
                    rpm: 15_000,
                    ..normal
                }),
                Duration::ZERO
            ));
        }
    }

    #[test]
    fn launch_tick_tracks_ten_thousand_rpm_on_the_live_scale() {
        for (max_rpm, expected) in [
            (10_000, 405.0),
            (12_000, 360.0),
            (15_000, 315.0),
            (20_000, 270.0),
        ] {
            assert_eq!(
                launch_angle(Some(CarStatus {
                    max_rpm,
                    ..CarStatus::default()
                })),
                Some(expected)
            );
        }
        assert_eq!(launch_angle(None), None);
        for max_rpm in [0, 9_999] {
            assert_eq!(
                launch_angle(Some(CarStatus {
                    max_rpm,
                    ..CarStatus::default()
                })),
                None
            );
        }
    }

    #[test]
    fn modern_panels_have_mirrored_extents_and_contain_all_text_rows() {
        let factory: ID2D1Factory1 =
            unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap() };
        let left = Left::new(&factory).unwrap();
        let right = Right::new(&factory).unwrap();
        assert_eq!(LEFT_COLUMNS[0].0, -RIGHT_COLUMNS[2].1);
        assert_eq!(LEFT_COLUMNS[1].1, -RIGHT_COLUMNS[0].0);
        for (wing, (start, end)) in left
            .panels
            .iter()
            .zip(LEFT_COLUMNS)
            .chain(right.panels.iter().zip(RIGHT_COLUMNS))
        {
            for (offset, half_height) in ROWS.into_iter().zip([7.5, 16.5, 7.5]) {
                for step in 0..=16 {
                    let angle =
                        (start + 0.55 + (end - start - 1.1) * step as f32 / 16.0).to_radians();
                    for offset in [offset - half_height, offset + half_height] {
                        let (x, y) = band_point(angle, offset);
                        assert!(unsafe {
                            wing.FillContainsPoint(point(x, y), None, 0.05)
                                .unwrap()
                                .as_bool()
                        });
                    }
                }
            }
        }
        for offset in [-70.0, -4.0] {
            let depth = (offset + 76.0) / 76.0;
            let edge = 42.0 + 48.0 * depth - 24.0 * depth * depth;
            for angle in 12..=28 {
                let (x, y) = band_point((angle as f32).to_radians(), offset);
                let (mx, my) = band_point(-(angle as f32).to_radians(), offset);
                assert!((x + mx - 840.0).abs() < 0.001 && (y - my).abs() < 0.001);
                assert!(mx >= edge && x <= 840.0 - edge);
                assert!(y > 0.0 && y < HEIGHT - 35.0);
            }
        }
    }

    #[test]
    fn modern_pedals_and_labels_fit_inside_the_band_with_equal_margins() {
        assert_eq!(PEDALS[0].0 + PEDALS[1].0, 2.0 * CENTER_X);
        assert_eq!(PEDALS[0].1, -PEDALS[1].1);
        for (x, angle) in PEDALS {
            let (sin, cos) = angle.to_radians().sin_cos();
            for (left, top, width, height) in [
                (
                    x - PEDAL_WIDTH / 2.0,
                    PEDAL_BOTTOM - 9.0 * PEDAL_STEP,
                    PEDAL_WIDTH,
                    9.0 * PEDAL_STEP + PEDAL_HEIGHT,
                ),
                (x - 20.0, PEDAL_LABEL_Y, 40.0, 13.0),
            ] {
                for px in [left, left + width] {
                    for py in [top, top + height] {
                        let rx = x + (px - x) * cos - (py - 112.0) * sin;
                        let ry = 112.0 + (px - x) * sin + (py - 112.0) * cos;
                        let edge = |offset: f32| {
                            153.0 + RADIUS
                                - ((RADIUS - offset).powi(2) - (rx - CENTER_X).powi(2)).sqrt()
                        };
                        assert!(ry > edge(-76.0) && ry < edge(0.0));
                    }
                }
            }
        }
    }

    #[test]
    fn right_emphasis_uses_only_reported_states_and_the_displayed_fuel_sign() {
        let colours = crate::settings::Colours::default();
        assert_eq!(right_emphasis(None, None, &colours), [None; 3]);
        let status = CarStatus {
            ers_joules: 1000.0,
            fuel_remaining_laps: -0.04,
            ..CarStatus::default()
        };
        assert_eq!(right_emphasis(Some(status), None, &colours), [None; 3]);
        let status = CarStatus {
            deploy_mode: 3,
            fuel_remaining_laps: -0.05,
            ..status
        };
        assert_eq!(
            right_emphasis(Some(status), None, &colours),
            [Some(colours.accent), None, Some(colours.brake)]
        );
        assert_eq!(
            right_emphasis(
                Some(CarStatus {
                    ers_joules: 0.0,
                    ..status
                }),
                None,
                &colours
            )[0],
            Some(colours.brake)
        );
        assert_eq!(
            right_emphasis(
                Some(CarStatus {
                    fuel_remaining_laps: f32::NAN,
                    ..status
                }),
                None,
                &colours
            )[2],
            None
        );
        let aero = Aero {
            available: true,
            regulations_2026: true,
            ..Aero::default()
        };
        assert_eq!(
            right_emphasis(None, Some(aero), &colours)[1],
            Some(colours.accent)
        );
        assert_eq!(
            right_emphasis(
                None,
                Some(Aero {
                    straight: true,
                    ..aero
                }),
                &colours
            )[1],
            Some(colours.throttle)
        );
        assert_eq!(
            right_emphasis(
                None,
                Some(Aero {
                    regulations_2026: false,
                    ..aero
                }),
                &colours
            )[1],
            None
        );
    }
}
