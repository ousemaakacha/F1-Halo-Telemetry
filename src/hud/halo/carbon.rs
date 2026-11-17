use super::*;
use crate::hud::carbon::{self as style, GREEN, GREY, RED, SILVER};

const CENTRE: (f32, f32) = (500.0, 100.0);
const TOP_START: (f32, f32) = (32.0, 211.158);
const LOWER_START: (f32, f32) = (20.0, 286.707);
const TOP: [[(f32, f32); 3]; 3] = [
    [(122.304, 134.068), (274.183, 80.545), (418.0, 60.0)],
    [(470.0, 50.0), (530.0, 50.0), (582.0, 60.0)],
    [(725.817, 80.545), (877.696, 134.068), (968.0, 211.158)],
];
const LOWER: [[(f32, f32); 3]; 2] = [
    [(222.814, 177.271), (353.627, 137.0), (500.0, 137.0)],
    [(646.373, 137.0), (777.186, 177.271), (980.0, 286.707)],
];

fn edge_y(start: (f32, f32), curve: &[(f32, f32); 3], x: f32) -> f32 {
    let points = [start, curve[0], curve[1], curve[2]];
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..24 {
        let t = (low + high) / 2.0;
        if classic::curve_point(&points, t).0 < x {
            low = t;
        } else {
            high = t;
        }
    }
    classic::curve_point(&points, (low + high) / 2.0).1
}

fn band_point(x: f32, depth: f32) -> (f32, f32) {
    let mirrored_x = x.min(WIDTH - x);
    let top = edge_y(TOP_START, &TOP[0], mirrored_x);
    let bottom = edge_y(LOWER_START, &LOWER[0], mirrored_x);
    (x, top + (bottom - top) * depth)
}

fn band_angle(x: f32, depth: f32) -> f32 {
    let a = band_point(x - 0.5, depth);
    let b = band_point(x + 0.5, depth);
    (b.1 - a.1).atan2(b.0 - a.0).to_degrees()
}

fn text_curve(start: f32, end: f32, depth: f32) -> TextCurve {
    TextCurve::new((0..=96).map(|i| band_point(start + (end - start) * i as f32 / 96.0, depth)))
}

struct Pedal {
    label: TextCurve,
    value: TextCurve,
    segments: [((f32, f32), f32); 10],
}

impl Pedal {
    fn new(x: f32) -> Self {
        Self {
            label: text_curve(x - 48.0, x + 48.0, 0.30),
            value: text_curve(x - 48.0, x + 48.0, 0.55),
            segments: std::array::from_fn(|i| {
                let x = x - 43.2 + i as f32 * 9.6;
                (band_point(x, 0.84), band_angle(x, 0.84))
            }),
        }
    }
}

pub(super) struct Carbon {
    body: ID2D1PathGeometry,
    top: [ID2D1PathGeometry; 2],
    lower: ID2D1PathGeometry,
    weave: ID2D1PathGeometry,
    ring: ID2D1PathGeometry,
    red_ring: ID2D1PathGeometry,
    shifts: Vec<ID2D1PathGeometry>,
    straight_text: [TextCurve; 2],
    straight_light: (f32, f32),
    fuel_text: [TextCurve; 2],
    pedals: [Pedal; 2],
    rpm_text: TextCurve,
    rpm_lights: [(f32, f32); 7],
    behind_text: [TextCurve; 2],
    dividers: [((f32, f32), (f32, f32)); 4],
}

impl Carbon {
    pub fn new(factory: &ID2D1Factory1) -> Result<Self> {
        let mut body_curves = TOP.to_vec();
        body_curves.extend([
            // Close only the fill; the separate outlines leave both cuts open.
            [LOWER[1][2]; 3],
            [LOWER[1][1], LOWER[1][0], LOWER[0][2]],
            [LOWER[0][1], LOWER[0][0], LOWER_START],
        ]);
        let body = style::path(factory, TOP_START, &body_curves, true)?;
        let weave = style::weave(
            factory,
            &body,
            &[(-8.0, 174.0, 91.0, 128.0), (917.0, 174.0, 91.0, 128.0)],
        )?;
        Ok(Self {
            body,
            weave,
            straight_text: [text_curve(70.0, 159.0, 0.35), text_curve(65.0, 163.0, 0.78)],
            straight_light: band_point(57.0, 0.35),
            fuel_text: [
                text_curve(176.0, 286.0, 0.35),
                text_curve(172.0, 290.0, 0.72),
            ],
            pedals: [Pedal::new(354.0), Pedal::new(646.0)],
            rpm_text: text_curve(708.0, 818.0, 0.70),
            rpm_lights: std::array::from_fn(|i| band_point(721.0 + i as f32 * 16.0, 0.30)),
            behind_text: [
                text_curve(841.0, 930.0, 0.35),
                text_curve(837.0, 935.0, 0.78),
            ],
            dividers: [166.0, 297.0, 703.0, 834.0].map(|x| {
                let centre = band_point(x, 0.5);
                let angle = band_angle(x, 0.5).to_radians();
                let half = (band_point(x, 1.0).1 - band_point(x, 0.0).1) * angle.cos() * 0.38;
                let normal = (-angle.sin() * half, angle.cos() * half);
                (
                    (centre.0 - normal.0, centre.1 - normal.1),
                    (centre.0 + normal.0, centre.1 + normal.1),
                )
            }),
            top: [
                style::path(factory, TOP_START, &TOP[..1], false)?,
                style::path(factory, (582.0, 60.0), &TOP[2..], false)?,
            ],
            lower: style::path(factory, LOWER_START, &LOWER, false)?,
            ring: style::arc(factory, CENTRE, 90.0, 0.0, 360.0)?,
            red_ring: style::arc(factory, CENTRE, 88.0, 16.0, 164.0)?,
            shifts: (0..30)
                .map(|i| {
                    style::arc(
                        factory,
                        CENTRE,
                        81.0,
                        160.0 + i as f32 * 7.4,
                        165.8 + i as f32 * 7.4,
                    )
                })
                .collect::<Result<_>>()?,
        })
    }

    pub fn draw(&self, p: &Painter<'_>, snapshot: &Snapshot, settings: &Settings, now: Instant) {
        let active = snapshot.driving(now);
        let tel = snapshot
            .telemetry
            .filter(|t| active && t.fresh(now, FAST_TTL))
            .map(|t| t.value);
        let status = snapshot
            .status
            .filter(|s| active && s.fresh(now, FAST_TTL))
            .map(|s| s.value);
        let aero = current_aero(snapshot, now);
        let opacity = settings.halo.opacity;
        p.carbon_geometry(&self.body, opacity);
        p.geometry(&self.weave, [111, 114, 119], 0.22 * opacity);
        for edge in &self.top {
            p.stroke_geometry(edge, SILVER, 1.1, 0.95);
        }
        p.stroke_geometry(&self.lower, RED, 3.0, 1.0);

        for &(a, b) in &self.dividers {
            p.line(a, b, SILVER, 1.2, 0.72);
        }
        let straight_colour = if aero.is_some_and(|a| a.regulations_2026 && a.straight) {
            GREEN
        } else {
            SILVER
        };
        p.circle(
            self.straight_light.0,
            self.straight_light.1,
            4.5,
            straight_colour,
            1.0,
        );
        p.curved_text(
            straight_mode_label(aero),
            &self.straight_text[0],
            CarbonValue,
            WHITE,
        );
        if let Some(distance) = straight_mode_distance(aero) {
            p.curved_textf(
                format_args!("STRAIGHT  {distance} m"),
                &self.straight_text[1],
                CarbonLabel,
                SILVER,
            );
        } else {
            p.curved_text("STRAIGHT", &self.straight_text[1], CarbonLabel, SILVER);
        }
        p.curved_text("FUEL", &self.fuel_text[0], CarbonLabel, WHITE);
        if let Some(laps) = fuel_margin(status) {
            p.curved_textf(
                format_args!("{}{:.1} LAPS", fuel_sign(laps), laps.abs()),
                &self.fuel_text[1],
                CarbonValue,
                WHITE,
            );
        } else {
            p.curved_text("-- LAPS", &self.fuel_text[1], CarbonValue, MUTED);
        }
        for (pedal, label, value, rgb) in [
            (
                &self.pedals[0],
                "BRAKE",
                tel.map(|t| t.brake),
                settings.colours.brake,
            ),
            (
                &self.pedals[1],
                "THROTTLE",
                tel.map(|t| t.throttle),
                settings.colours.throttle,
            ),
        ] {
            p.curved_text(label, &pedal.label, CarbonLabel, WHITE);
            if let Some(value) = value {
                p.curved_textf(
                    format_args!("{:.0}%", pedal_percent(value)),
                    &pedal.value,
                    CarbonValue,
                    WHITE,
                );
            } else {
                p.curved_text("--%", &pedal.value, CarbonValue, MUTED);
            }
            for (i, &(centre, angle)) in pedal.segments.iter().enumerate() {
                p.rotated(centre, angle + 16.0, |p| {
                    let x = centre.0 - 4.15;
                    let y = centre.1 - 4.0;
                    p.fill(rect(x, y, 8.3, 8.0), GREY, 1.0);
                    let level = value.map_or(0.0, |v| {
                        (pedal_percent(v) / 10.0 - i as f32).clamp(0.0, 1.0)
                    });
                    if level > 0.0 {
                        p.fill(rect(x, y, 8.3 * level, 8.0), rgb, 1.0);
                    }
                });
            }
        }
        for (i, &(x, y)) in self.rpm_lights.iter().enumerate() {
            let rgb = match i {
                0..=3 => GREEN,
                4 => [255, 230, 0],
                5 => [255, 172, 0],
                _ => RED,
            };
            let mask = if i == 6 { 0x7000 } else { 0x3 << (i * 2) };
            let on = tel.is_some_and(|t| t.rev_bits & mask != 0);
            p.circle(x, y, 4.6, if on { rgb } else { GREY }, 1.0);
        }
        if let Some(t) = tel {
            p.curved_textf(
                format_args!("{} {:03} RPM", t.rpm / 1000, t.rpm % 1000),
                &self.rpm_text,
                CarbonValue,
                WHITE,
            );
        } else {
            p.curved_text("-- RPM", &self.rpm_text, CarbonValue, MUTED);
        }
        p.curved_text("BEHIND", &self.behind_text[0], CarbonLabel, WHITE);
        if let Some(millis) = snapshot.gap_behind_ms(now) {
            p.curved_textf(
                format_args!("+{}.{:03}", millis / 1000, millis % 1000),
                &self.behind_text[1],
                CarbonValue,
                WHITE,
            );
        } else {
            p.curved_text("--", &self.behind_text[1], CarbonValue, MUTED);
        }
        p.circle(CENTRE.0, CENTRE.1, 89.0, [9, 10, 12], opacity);
        p.stroke_geometry(&self.ring, [40, 42, 46], 4.0, 1.0);
        p.stroke_geometry(&self.ring, SILVER, 1.0, 1.0);
        p.stroke_geometry(&self.red_ring, RED, 2.8, 1.0);
        for (i, segment) in self.shifts.iter().enumerate() {
            let on = tel.is_some_and(|t| t.rev_bits & (1 << (i / 2)) != 0);
            p.stroke_geometry(
                segment,
                if on {
                    if i >= 22 { RED } else { WHITE }
                } else {
                    GREY
                },
                6.1,
                1.0,
            );
        }
        p.text("GEAR", rect(466.0, 28.0, 68.0, 15.0), CarbonLabel, WHITE);
        p.text(
            tel.map(|t| gear_label(t.gear)).unwrap_or("--"),
            rect(471.0, 40.0, 58.0, 33.0),
            CarbonGear,
            WHITE,
        );
        p.line((463.0, 59.0), (481.0, 59.0), SILVER, 2.0, 0.6);
        p.line((519.0, 59.0), (537.0, 59.0), SILVER, 2.0, 0.6);
        if let Some(t) = tel {
            p.textf(
                format_args!("{}", t.speed_kmh),
                rect(421.0, 66.0, 158.0, 60.0),
                CarbonSpeed,
                WHITE,
            );
        } else {
            p.text("--", rect(421.0, 66.0, 158.0, 60.0), CarbonSpeed, MUTED);
        }
        p.text("KM/H", rect(454.0, 120.0, 92.0, 15.0), CarbonLabel, WHITE);
        p.line((446.0, 135.0), (554.0, 135.0), SILVER, 0.7, 0.65);
        let energy_colour = if aero.is_some_and(|a| a.regulations_2026 && a.overtake_available) {
            [0, 227, 239]
        } else {
            WHITE
        };
        if let Some(percentage) =
            status.and_then(|s| ers_percent(s.ers_joules, snapshot.ers_capacity.map(|c| c.joules)))
        {
            p.textf(
                format_args!("ERS {percentage:.0}%"),
                rect(426.0, 137.0, 148.0, 24.0),
                CarbonValue,
                energy_colour,
            );
        } else if let Some(status) = status {
            p.textf(
                format_args!("ERS {:.2} MJ", status.ers_joules / 1_000_000.0),
                rect(426.0, 137.0, 148.0, 24.0),
                CarbonValue,
                energy_colour,
            );
        } else {
            p.text(
                "ERS --%",
                rect(426.0, 137.0, 148.0, 24.0),
                CarbonValue,
                MUTED,
            );
        }
        p.text(
            deploy_label(status),
            rect(451.0, 161.0, 98.0, 15.0),
            CarbonLabel,
            SILVER,
        );
        let connection = snapshot.connection(now);
        if connection != Connection::Live {
            p.text(
                connection.label(),
                rect(350.0, 261.0, 300.0, 17.0),
                HudLabel,
                SILVER,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn band_rows_follow_the_bezier_edges_and_mirror_their_tangents() {
        for (start, curve, depth) in [(TOP_START, TOP[0], 0.0), (LOWER_START, LOWER[0], 1.0)] {
            let points = [start, curve[0], curve[1], curve[2]];
            for t in [0.0, 0.05, 0.2, 0.4, 0.6, 0.8, 1.0] {
                let point = classic::curve_point(&points, t);
                assert!((band_point(point.0, depth).1 - point.1).abs() < 0.002);
            }
        }
        for x in [57.0, 100.0, 166.0, 231.0, 297.0, 354.0, 402.0] {
            for depth in [0.0, 0.30, 0.35, 0.55, 0.70, 0.72, 0.78, 0.84, 1.0] {
                let left = band_point(x, depth);
                let right = band_point(WIDTH - x, depth);
                assert!((left.1 - right.1).abs() < 0.002);
                assert!((band_angle(x, depth) + band_angle(WIDTH - x, depth)).abs() < 0.02);
            }
            assert!(band_point(x, 0.30).1 < band_point(x, 0.55).1);
            assert!(band_point(x, 0.55).1 < band_point(x, 0.84).1);
        }
    }

    #[test]
    fn pedal_curves_and_segments_keep_the_same_symmetric_flow() {
        let left = Pedal::new(354.0);
        let right = Pedal::new(646.0);
        for (a, b) in [(&left.label, &right.label), (&left.value, &right.value)] {
            let a = a.centre();
            let b = b.centre();
            assert!((a.0 + b.0 - WIDTH).abs() < 0.002);
            assert!((a.1 - b.1).abs() < 0.002);
        }
        for (&(a, angle_a), &(b, angle_b)) in left.segments.iter().zip(right.segments.iter().rev())
        {
            assert!((a.0 + b.0 - WIDTH).abs() < 0.002);
            assert!((a.1 - b.1).abs() < 0.002);
            assert!((angle_a + angle_b).abs() < 0.02);
            assert!(a.1 < band_point(a.0, 1.0).1 - 8.0);
        }
        for pair in left.segments.windows(2) {
            assert!((pair[0].1 - pair[1].1).abs() < 2.0);
        }
    }
}
