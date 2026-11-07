use crate::{
    graphics::{Font::*, *},
    settings::{Settings, TyreMode},
    telemetry::{
        state::{DAMAGE_TTL, FAST_TTL},
        *,
    },
};
use std::time::Instant;

pub const WIDTH: f32 = 310.0;
pub const HEIGHT: f32 = 370.0;

fn temperature_colour(value: f32, settings: &Settings) -> [u8; 3] {
    if value < settings.temperature_min_c {
        settings.colours.cold
    } else if value > settings.temperature_max_c {
        settings.colours.hot
    } else {
        settings.colours.optimal
    }
}

pub fn draw(p: &Painter<'_>, snapshot: &Snapshot, settings: &Settings, edit: bool, now: Instant) {
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
        0.94,
    );
    p.text("TYRES / 26", rect(20.0, 13.0, 150.0, 25.0), Label, WHITE);
    p.text(
        settings.tyre_mode.label(),
        rect(235.0, 16.0, 66.0, 22.0),
        Caption,
        accent,
    );
    p.line((20.0, 47.0), (290.0, 47.0), DIM, 1.0, 1.0);
    // The body and axles are intentionally separate from the four data blocks.
    p.rounded(rect(145.0, 74.0, 20.0, 172.0), 8.0, [48, 64, 78], 0.8);
    p.rounded(rect(135.0, 145.0, 40.0, 97.0), 8.0, [41, 54, 68], 1.0);
    p.rounded(rect(142.0, 149.0, 26.0, 42.0), 10.0, BACKGROUND, 1.0);
    p.line((155.0, 89.0), (155.0, 140.0), accent, 2.0, 0.7);
    p.fill(rect(125.0, 87.0, 60.0, 5.0), MUTED, 0.7);
    p.fill(rect(125.0, 268.0, 60.0, 7.0), MUTED, 0.7);
    for y in [119.0, 240.0] {
        p.line((124.0, y), (186.0, y), MUTED, 2.0, 0.6);
        for x in [116.0, 178.0] {
            p.rounded(rect(x, y - 18.0, 16.0, 36.0), 4.0, DIM, 1.0);
        }
    }
    for (wheel, x, y) in [
        (Wheel::FrontLeft, 20.0, 66.0),
        (Wheel::FrontRight, 210.0, 66.0),
        (Wheel::RearLeft, 20.0, 213.0),
        (Wheel::RearRight, 210.0, 213.0),
    ] {
        let index = wheel as usize;
        p.text(wheel.label(), rect(x, y, 80.0, 22.0), Label, MUTED);
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
                rect(x, y + 23.0, 86.0, 38.0),
                Medium,
                WHITE,
            );
            p.fill(rect(x, y + 63.0, 77.0, 3.0), DIM, 1.0);
            p.fill(
                rect(x, y + 63.0, 77.0 * w[index] / 100.0, 3.0),
                wear_colour,
                1.0,
            );
        } else {
            p.text("--%", rect(x, y + 23.0, 86.0, 38.0), Medium, MUTED);
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
            let line = rect(x, y + 72.0 + row as f32 * 20.0, 92.0, 20.0);
            if let Some(v) = value {
                p.textf(
                    format_args!("{label} {v}°C"),
                    line,
                    Label,
                    temperature_colour(v as f32, settings),
                );
            } else {
                p.textf(format_args!("{label} --°C"), line, Label, MUTED);
            }
        }
    }
    p.text(
        "USURA · 0% = NUOVO",
        rect(20.0, 320.0, 200.0, 17.0),
        Caption,
        MUTED,
    );
    if edit {
        super::edit_frame(p, WIDTH, HEIGHT, accent);
    }
}
