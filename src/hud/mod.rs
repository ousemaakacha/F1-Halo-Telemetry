pub mod halo;
pub mod tyres;

use crate::graphics::{Font, MUTED, Painter, rect};

pub fn edit_frame(p: &Painter<'_>, width: f32, height: f32, accent: [u8; 3]) {
    p.outline(rect(1.5, 1.5, width - 3.0, height - 3.0), accent, 1.0);
    for (x, y) in [
        (0.0, 0.0),
        (width - 9.0, 0.0),
        (0.0, height - 9.0),
        (width - 9.0, height - 9.0),
    ] {
        p.fill(rect(x, y, 9.0, 9.0), accent, 1.0);
    }
    p.text(
        "EDIT · drag · corners: scale · wheel: scale · Shift: opacity",
        rect(12.0, height - 21.0, width - 24.0, 18.0),
        Font::Caption,
        MUTED,
    );
}
