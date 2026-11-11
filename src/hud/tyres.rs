use crate::{
    graphics::{Font::*, *},
    settings::{Settings, TyreMode},
    telemetry::{
        state::{DAMAGE_TTL, FAST_TTL},
        *,
    },
};
use std::time::Instant;
use windows::Win32::Graphics::Direct2D::ID2D1Bitmap1;

pub const WIDTH: f32 = 310.0;
pub const HEIGHT: f32 = 370.0;
const COLUMN_WIDTH: f32 = 82.0;

fn overall_wear(wear: Option<[f32; 4]>) -> Option<f32> {
    wear.map(|values| values.iter().sum::<f32>() / 4.0)
}

fn temperature_colour(value: f32, settings: &Settings) -> [u8; 3] {
    if value < settings.temperature_min_c {
        settings.colours.cold
    } else if value > settings.temperature_max_c {
        settings.colours.hot
    } else {
        settings.colours.optimal
    }
}

pub fn draw(
    p: &Painter<'_>,
    car: &ID2D1Bitmap1,
    snapshot: &Snapshot,
    settings: &Settings,
    edit: bool,
    now: Instant,
) {
    let active = snapshot.driving(now);
    let tel = snapshot
        .telemetry
        .filter(|t| active && t.fresh(now, FAST_TTL))
        .map(|t| t.value);
    let wear = snapshot
        .damage
        .filter(|t| active && t.fresh(now, DAMAGE_TTL))
        .map(|t| t.value.wear);
    let accent = settings.colours.accent;
    p.rounded(
        rect(4.0, 4.0, WIDTH - 8.0, HEIGHT - 28.0),
        12.0,
        BACKGROUND,
        settings.tyres.opacity,
    );
    p.text("TYRES", rect(20.0, 10.0, 150.0, 32.0), TyreTitle, WHITE);
    let summary = rect(208.0, 10.0, COLUMN_WIDTH, 32.0);
    if let Some(value) = overall_wear(wear) {
        p.textf(format_args!("{value:.0}%"), summary, HudNumber, accent);
    } else {
        p.text("--%", summary, HudNumber, MUTED);
    }
    p.line((20.0, 47.0), (290.0, 47.0), DIM, 1.0, 1.0);
    // The body and axles are intentionally separate from the four data blocks.
    let size = unsafe { car.GetSize() };
    p.bitmap(
        car,
        rect(107.0, 57.0, 96.0, 96.0 * size.height / size.width),
    );
    for (wheel, x, y) in [
        (Wheel::FrontLeft, 20.0, 66.0),
        (Wheel::FrontRight, 208.0, 66.0),
        (Wheel::RearLeft, 20.0, 213.0),
        (Wheel::RearRight, 208.0, 213.0),
    ] {
        let index = wheel as usize;
        p.text(
            wheel.label(),
            rect(x, y, COLUMN_WIDTH, 22.0),
            TyreLabel,
            MUTED,
        );
        let wear_colour = wear
            .map(|w| {
                if w[index] >= settings.wear_critical_percent {
                    settings.colours.hot
                } else if w[index] >= settings.wear_warning_percent {
                    [241, 190, 83]
                } else {
                    settings.colours.optimal
                }
            })
            .unwrap_or(DIM);
        if let Some(w) = wear {
            p.textf(
                format_args!("{:.0}%", w[index]),
                rect(x, y + 23.0, COLUMN_WIDTH, 38.0),
                TyreWear,
                WHITE,
            );
            p.fill(rect(x, y + 63.0, COLUMN_WIDTH, 3.0), DIM, 1.0);
            p.fill(
                rect(x, y + 63.0, COLUMN_WIDTH * w[index] / 100.0, 3.0),
                wear_colour,
                1.0,
            );
        } else {
            p.text(
                "--%",
                rect(x, y + 23.0, COLUMN_WIDTH, 38.0),
                TyreWear,
                MUTED,
            );
        }
        let values = match settings.tyre_mode {
            TyreMode::Surface => [("S", tel.map(|t| t.surface_c[index])), ("", None)],
            TyreMode::Inner => [("I", tel.map(|t| t.inner_c[index])), ("", None)],
            TyreMode::Both => [
                ("S", tel.map(|t| t.surface_c[index])),
                ("I", tel.map(|t| t.inner_c[index])),
            ],
        };
        for (row, (label, value)) in values.into_iter().enumerate() {
            if label.is_empty() {
                continue;
            }
            let line = rect(x, y + 72.0 + row as f32 * 20.0, COLUMN_WIDTH, 20.0);
            if let Some(v) = value {
                p.textf(
                    format_args!("{label} {v}°C"),
                    line,
                    TyreLabel,
                    temperature_colour(v as f32, settings),
                );
            } else {
                p.textf(format_args!("{label} --°C"), line, TyreLabel, MUTED);
            }
        }
    }
    if edit {
        super::edit_frame(p, WIDTH, HEIGHT, accent);
    }
}

#[cfg(test)]
mod tests {
    use super::overall_wear;

    #[test]
    fn overall_wear_averages_all_four_tyres_and_preserves_missing_data() {
        assert_eq!(overall_wear(Some([18.0, 22.0, 13.0, 15.0])), Some(17.0));
        assert_eq!(overall_wear(Some([0.0, 25.0, 50.0, 100.0])), Some(43.75));
        assert_eq!(overall_wear(Some([0.0; 4])), Some(0.0));
        assert_eq!(overall_wear(Some([100.0; 4])), Some(100.0));
        assert_eq!(overall_wear(None), None);
    }
}
