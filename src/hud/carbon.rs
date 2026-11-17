use crate::graphics::{Painter, rect};
use windows::{
    Win32::Graphics::Direct2D::{Common::*, *},
    core::Result,
};
use windows_numerics::Vector2;

pub const RED: [u8; 3] = [255, 20, 32];
pub const SILVER: [u8; 3] = [190, 194, 201];
pub const GREEN: [u8; 3] = [78, 255, 0];
pub const GREY: [u8; 3] = [61, 65, 69];

pub fn point((x, y): (f32, f32)) -> Vector2 {
    Vector2 { X: x, Y: y }
}

pub fn path(
    factory: &ID2D1Factory1,
    start: (f32, f32),
    curves: &[[(f32, f32); 3]],
    filled: bool,
) -> Result<ID2D1PathGeometry> {
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        sink.BeginFigure(
            point(start),
            if filled {
                D2D1_FIGURE_BEGIN_FILLED
            } else {
                D2D1_FIGURE_BEGIN_HOLLOW
            },
        );
        for curve in curves {
            sink.AddBezier(&D2D1_BEZIER_SEGMENT {
                point1: point(curve[0]),
                point2: point(curve[1]),
                point3: point(curve[2]),
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

pub fn arc(
    factory: &ID2D1Factory1,
    centre: (f32, f32),
    radius: f32,
    from: f32,
    to: f32,
) -> Result<ID2D1PathGeometry> {
    unsafe {
        let path = factory.CreatePathGeometry()?;
        let sink = path.Open()?;
        let steps = ((to - from).abs() / 3.0).ceil() as usize;
        for i in 0..=steps {
            let angle = (from + (to - from) * i as f32 / steps as f32).to_radians();
            let position = point((
                centre.0 + radius * angle.cos(),
                centre.1 + radius * angle.sin(),
            ));
            if i == 0 {
                sink.BeginFigure(position, D2D1_FIGURE_BEGIN_HOLLOW);
            } else {
                sink.AddLine(position);
            }
        }
        sink.EndFigure(D2D1_FIGURE_END_OPEN);
        sink.Close()?;
        Ok(path.into())
    }
}

pub fn weave(
    factory: &ID2D1Factory1,
    mask: &ID2D1PathGeometry,
    regions: &[(f32, f32, f32, f32)],
) -> Result<ID2D1PathGeometry> {
    // Intersect once at device creation; the render loop only draws the cached weave.
    unsafe {
        let tiles = factory.CreatePathGeometry()?;
        let sink = tiles.Open()?;
        for &(x, y, width, height) in regions {
            for row in 0..(height / 4.0).ceil() as i32 {
                for column in 0..(width / 5.0).ceil() as i32 {
                    let left = x + column as f32 * 5.0;
                    let top = y + row as f32 * 4.0;
                    let slant = if (row + column) % 2 == 0 { 2.4 } else { -2.4 };
                    sink.BeginFigure(point((left, top)), D2D1_FIGURE_BEGIN_FILLED);
                    sink.AddLines(&[
                        point((left + 3.6, top)),
                        point((left + 3.6 + slant, top + 2.5)),
                        point((left + slant, top + 2.5)),
                    ]);
                    sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                }
            }
        }
        sink.Close()?;
        let clipped = factory.CreatePathGeometry()?;
        let sink = clipped.Open()?;
        tiles.CombineWithGeometry(mask, D2D1_COMBINE_MODE_INTERSECT, None, 0.15, &sink)?;
        sink.Close()?;
        Ok(clipped.into())
    }
}

pub fn segments(
    p: &Painter<'_>,
    origin: (f32, f32),
    size: (f32, f32),
    count: usize,
    value: Option<f32>,
    rgb: [u8; 3],
) {
    let step = size.0 / count as f32;
    for i in 0..count {
        let x = origin.0 + i as f32 * step;
        p.rotated((x + step / 2.0, origin.1 + size.1 / 2.0), 16.0, |p| {
            let r = rect(x, origin.1, step - 1.3, size.1);
            p.fill(r, GREY, 1.0);
            let level = value.map_or(0.0, |v| {
                (v.clamp(0.0, 1.0) * count as f32 - i as f32).clamp(0.0, 1.0)
            });
            if level > 0.0 {
                p.fill(rect(x, origin.1, (step - 1.3) * level, size.1), rgb, 1.0);
            }
        });
    }
}
