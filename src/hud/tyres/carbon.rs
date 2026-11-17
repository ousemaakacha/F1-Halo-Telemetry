use super::*;
use crate::hud::carbon::{self as style, GREEN, RED, SILVER};
use windows::Win32::Graphics::Direct2D::ID2D1PathGeometry;

const TOP: [[(f32, f32); 3]; 3] = [
    [(8.0, 42.0), (10.0, 29.0), (34.0, 22.0)],
    [(103.0, 3.0), (207.0, 3.0), (276.0, 22.0)],
    [(300.0, 29.0), (302.0, 42.0), (302.0, 64.0)],
];
const BOTTOM: [[(f32, f32); 3]; 3] = [
    [(8.0, 340.0), (13.0, 344.0), (36.0, 348.0)],
    [(114.0, 359.0), (196.0, 359.0), (274.0, 348.0)],
    [(297.0, 344.0), (302.0, 340.0), (302.0, 323.0)],
];

pub(super) struct Carbon {
    car: ID2D1Bitmap1,
    body: ID2D1PathGeometry,
    lower: ID2D1PathGeometry,
    weave: ID2D1PathGeometry,
}

impl Carbon {
    pub fn new(graphics: &Graphics) -> Result<Self> {
        let factory = &graphics.geometry_factory;
        let mut curves = TOP.to_vec();
        curves.extend([
            [(302.0, 150.0), (302.0, 280.0), (302.0, 323.0)],
            [(302.0, 340.0), (297.0, 344.0), (274.0, 348.0)],
            [(196.0, 359.0), (114.0, 359.0), (36.0, 348.0)],
            [(13.0, 344.0), (8.0, 340.0), (8.0, 323.0)],
            [(8.0, 280.0), (8.0, 150.0), (8.0, 64.0)],
        ]);
        let body = style::path(factory, (8.0, 64.0), &curves, true)?;
        let weave = style::weave(
            factory,
            &body,
            &[(8.0, 16.0, 29.0, 62.0), (273.0, 16.0, 29.0, 62.0)],
        )?;
        Ok(Self {
            car: graphics.bitmap(include_bytes!("../../../assets/tyre-car-carbon.png"))?,
            body,
            weave,
            lower: style::path(factory, (8.0, 323.0), &BOTTOM, false)?,
        })
    }

    pub fn draw(&self, p: &Painter<'_>, snapshot: &Snapshot, settings: &Settings, now: Instant) {
        let active = snapshot.driving(now);
        let tel = snapshot
            .telemetry
            .filter(|t| active && t.fresh(now, FAST_TTL))
            .map(|t| t.value);
        let wear = snapshot
            .damage
            .filter(|t| active && t.fresh(now, DAMAGE_TTL))
            .map(|t| t.value.wear);
        p.carbon_geometry(&self.body, settings.tyres.opacity);
        p.geometry(&self.weave, [111, 114, 119], 0.32 * settings.tyres.opacity);
        p.stroke_geometry(&self.body, SILVER, 0.9, 0.95);
        p.stroke_geometry(&self.lower, RED, 2.2, 1.0);
        p.text("TYRES", rect(29.0, 27.0, 124.0, 35.0), CarbonTitle, WHITE);
        if let Some(wear) = overall_wear(wear) {
            p.textf(
                format_args!("{wear:.0}%"),
                rect(196.0, 24.0, 84.0, 29.0),
                CarbonGear,
                WHITE,
            );
        } else {
            p.text("--%", rect(196.0, 24.0, 84.0, 29.0), CarbonGear, MUTED);
        }
        p.text(
            "AVG WEAR",
            rect(195.0, 52.0, 86.0, 15.0),
            CarbonLabel,
            SILVER,
        );
        p.line((20.0, 72.0), (290.0, 72.0), SILVER, 0.7, 0.7);

        let size = unsafe { self.car.GetSize() };
        p.bitmap(
            &self.car,
            rect(78.0, 82.0, 154.0, 154.0 * size.height / size.width),
        );
        for (wheel, x, y) in [
            (Wheel::FrontLeft, 20.0, 82.0),
            (Wheel::FrontRight, 201.0, 82.0),
            (Wheel::RearLeft, 20.0, 204.0),
            (Wheel::RearRight, 201.0, 204.0),
        ] {
            let index = wheel as usize;
            let value = wear.map(|w| w[index]);
            p.text(
                wheel.label(),
                rect(x, y, 86.0, 22.0),
                CarbonTemperature,
                SILVER,
            );
            if let Some(wear) = value {
                p.textf(
                    format_args!("{wear:.0}%"),
                    rect(x - 5.0, y + 19.0, 96.0, 42.0),
                    CarbonWear,
                    WHITE,
                );
            } else {
                p.text(
                    "--%",
                    rect(x - 5.0, y + 19.0, 96.0, 42.0),
                    CarbonWear,
                    MUTED,
                );
            }
            let wear_colour = match value {
                Some(v) if v >= settings.wear_critical_percent => settings.colours.hot,
                Some(v) if v >= settings.wear_warning_percent => [255, 196, 49],
                _ => GREEN,
            };
            style::segments(
                p,
                (x + 1.0, y + 65.0),
                (83.0, 8.0),
                8,
                value.map(|v| v / 100.0),
                wear_colour,
            );
            let values = match settings.tyre_mode {
                TyreMode::Surface => [("S", tel.map(|t| t.surface_c[index])), ("", None)],
                TyreMode::Inner => [("I", tel.map(|t| t.inner_c[index])), ("", None)],
                TyreMode::Both => [
                    ("S", tel.map(|t| t.surface_c[index])),
                    ("I", tel.map(|t| t.inner_c[index])),
                ],
            };
            for (row, (label, temperature)) in values.into_iter().enumerate() {
                if label.is_empty() {
                    continue;
                }
                let y = y + 80.0 + row as f32 * 19.0;
                p.text(label, rect(x, y, 14.0, 20.0), CarbonTemperature, SILVER);
                if let Some(value) = temperature {
                    let rgb = if (settings.temperature_min_c..=settings.temperature_max_c)
                        .contains(&(value as f32))
                    {
                        GREEN
                    } else {
                        temperature_colour(value as f32, settings)
                    };
                    p.textf(
                        format_args!("{value}°C"),
                        rect(x + 14.0, y, 73.0, 20.0),
                        CarbonTemperature,
                        rgb,
                    );
                } else {
                    p.text(
                        "--°C",
                        rect(x + 14.0, y, 73.0, 20.0),
                        CarbonTemperature,
                        MUTED,
                    );
                }
            }
        }
        for x in [20.0, 201.0] {
            p.line((x, 199.0), (x + 86.0, 199.0), SILVER, 0.5, 0.45);
        }
        let legend = match settings.tyre_mode {
            TyreMode::Surface => "S  SURFACE",
            TyreMode::Inner => "I  INNER",
            TyreMode::Both => "S  SURFACE    /    I  INNER",
        };
        p.text(legend, rect(31.0, 330.0, 248.0, 18.0), CarbonLabel, SILVER);
    }
}
