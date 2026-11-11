//! All COM objects are owned and used on the UI thread. The only GPU readback
//! is an occasional nonblocking timestamp query, never a framebuffer copy.
use std::{
    fmt::{self, Write},
    mem::size_of,
};
use windows::{
    Win32::{
        Foundation::{HMODULE, HWND, S_OK},
        Graphics::{
            Direct2D::{Common::*, *},
            Direct3D::D3D_DRIVER_TYPE_HARDWARE,
            Direct3D11::*,
            DirectComposition::*,
            DirectWrite::*,
            Dxgi::{Common::*, *},
            Imaging::*,
        },
        System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance},
    },
    core::{Interface, Result, w},
};
use windows_numerics::{Matrix3x2, Vector2};

pub const WHITE: [u8; 3] = [240, 245, 249];
pub const MUTED: [u8; 3] = [153, 169, 185];
pub const DIM: [u8; 3] = [45, 58, 72];
pub const BACKGROUND: [u8; 3] = [10, 16, 24];

pub fn colour(rgb: [u8; 3], alpha: f32) -> D2D1_COLOR_F {
    D2D1_COLOR_F {
        r: rgb[0] as f32 / 255.0,
        g: rgb[1] as f32 / 255.0,
        b: rgb[2] as f32 / 255.0,
        a: alpha,
    }
}

pub fn rect(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}

pub struct TextCurve {
    samples: Vec<(f32, Vector2)>,
}

impl TextCurve {
    pub fn new(points: impl IntoIterator<Item = (f32, f32)>) -> Self {
        let mut samples = Vec::new();
        let mut distance = 0.0;
        let mut previous: Option<Vector2> = None;
        for (x, y) in points {
            if let Some(previous) = previous {
                distance += (x - previous.X).hypot(y - previous.Y);
            }
            let point = Vector2 { X: x, Y: y };
            samples.push((distance, point));
            previous = Some(point);
        }
        assert!(samples.len() >= 2 && distance > 0.0);
        Self { samples }
    }

    fn length(&self) -> f32 {
        self.samples.last().unwrap().0
    }

    pub fn centre(&self) -> (f32, f32) {
        let (point, _) = self.position(self.length() / 2.0);
        (point.X, point.Y)
    }

    fn position(&self, distance: f32) -> (Vector2, f32) {
        let distance = distance.clamp(0.0, self.length());
        let index = self
            .samples
            .partition_point(|sample| sample.0 < distance)
            .clamp(1, self.samples.len() - 1);
        let (start, a) = self.samples[index - 1];
        let (end, b) = self.samples[index];
        let fraction = (distance - start) / (end - start);
        (
            Vector2 {
                X: a.X + (b.X - a.X) * fraction,
                Y: a.Y + (b.Y - a.Y) * fraction,
            },
            (b.Y - a.Y).atan2(b.X - a.X).to_degrees(),
        )
    }
}

#[derive(Clone, Copy)]
pub enum Font {
    Caption,
    Label,
    Number,
    Medium,
    Speed,
    Gear,
    HudLabel,
    HudNumber,
    HudStraight,
    HudSpeed,
    HudSecondary,
    HudFuel,
    TyreTitle,
    TyreLabel,
    TyreWear,
}

const FONT_COUNT: usize = 15;

impl Font {
    fn index(self) -> usize {
        self as usize
    }
}

pub struct Graphics {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    factory: IDXGIFactory2,
    d2d: ID2D1Device,
    pub geometry_factory: ID2D1Factory1,
    composition: IDCompositionDevice,
    formats: [IDWriteTextFormat; FONT_COUNT],
    advances: [[f32; 95]; FONT_COUNT],
    gpu: Option<GpuTimer>,
    pub adapter_name: String,
}

impl Graphics {
    pub fn new() -> Result<Self> {
        // SAFETY: COM is initialized by App. Out-pointers are initialized Options;
        // all dependent objects are retained for their full lifetime.
        unsafe {
            let mut device = None;
            let mut context = None;
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
            let device = device.unwrap();
            let context = context.unwrap();
            let dxgi: IDXGIDevice = device.cast()?;
            let adapter = dxgi.GetAdapter()?;
            let desc = adapter.GetDesc()?;
            let adapter_name = String::from_utf16_lossy(
                &desc.Description[..desc.Description.iter().position(|&c| c == 0).unwrap_or(128)],
            );
            let factory: IDXGIFactory2 = adapter.GetParent()?;
            let geometry_factory: ID2D1Factory1 =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let d2d = geometry_factory.CreateDevice(&dxgi)?;
            let composition: IDCompositionDevice = DCompositionCreateDevice(&dxgi)?;
            let write: IDWriteFactory = DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?;
            let make = |size, weight, numeric| -> Result<IDWriteTextFormat> {
                let font = write.CreateTextFormat(
                    if numeric {
                        w!("Bahnschrift")
                    } else {
                        w!("Segoe UI")
                    },
                    None,
                    weight,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    size,
                    w!("en-GB"),
                )?;
                font.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
                font.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
                Ok(font)
            };
            let centered = |size, weight, numeric| -> Result<IDWriteTextFormat> {
                let font = make(size, weight, numeric)?;
                font.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)?;
                Ok(font)
            };
            let gear_format = centered(102.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, true)?;
            let hud = |size| centered(size, DWRITE_FONT_WEIGHT_BOLD, true);
            let formats = [
                make(11.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, false)?,
                make(14.0, DWRITE_FONT_WEIGHT_MEDIUM, false)?,
                make(23.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, true)?,
                make(31.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, true)?,
                make(66.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, true)?,
                gear_format,
                hud(10.0)?,
                hud(22.0)?,
                hud(19.0)?,
                hud(36.0)?,
                hud(27.0)?,
                hud(17.0)?,
                make(18.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, false)?,
                centered(14.0, DWRITE_FONT_WEIGHT_MEDIUM, false)?,
                centered(31.0, DWRITE_FONT_WEIGHT_SEMI_BOLD, true)?,
            ];
            // Cache advances once instead of measuring curved glyphs every frame.
            let ascii: [u16; 95] = std::array::from_fn(|i| i as u16 + 32);
            let mut advances = [[0.0; 95]; FONT_COUNT];
            for (format, widths) in formats.iter().zip(&mut advances) {
                let layout = write.CreateTextLayout(&ascii, format, 10000.0, 200.0)?;
                let mut clusters = [DWRITE_CLUSTER_METRICS::default(); 95];
                let mut count = 0;
                layout.GetClusterMetrics(Some(&mut clusters), &mut count)?;
                let mut start = 0;
                for cluster in &clusters[..count as usize] {
                    let end = start + cluster.length as usize;
                    widths[start..end].fill(cluster.width / cluster.length as f32);
                    start = end;
                }
            }
            let gpu = GpuTimer::new(&device).ok();
            Ok(Self {
                device,
                context,
                factory,
                d2d,
                geometry_factory,
                composition,
                formats,
                advances,
                gpu,
                adapter_name,
            })
        }
    }

    pub fn surface(&self, hwnd: HWND, width: u32, height: u32) -> Result<Surface> {
        // SAFETY: hwnd is a live UI-thread window. Composition and D2D resources
        // keep strong COM references; no temporary native pointers escape.
        unsafe {
            let description = DXGI_SWAP_CHAIN_DESC1 {
                Width: width.max(1),
                Height: height.max(1),
                Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 2,
                Scaling: DXGI_SCALING_STRETCH,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT.0 as u32,
                ..Default::default()
            };
            let chain =
                self.factory
                    .CreateSwapChainForComposition(&self.device, &description, None)?;
            let chain2: IDXGISwapChain2 = chain.cast()?;
            chain2.SetMaximumFrameLatency(1)?;
            let context = self
                .d2d
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
            context.SetUnitMode(D2D1_UNIT_MODE_PIXELS);
            context.SetTextAntialiasMode(D2D1_TEXT_ANTIALIAS_MODE_GRAYSCALE);
            let bitmap = target_bitmap(&context, &chain)?;
            context.SetTarget(&bitmap);
            let target = self.composition.CreateTargetForHwnd(hwnd, true)?;
            let visual = self.composition.CreateVisual()?;
            visual.SetContent(&chain)?;
            target.SetRoot(&visual)?;
            self.composition.Commit()?;
            let brush = context.CreateSolidColorBrush(&colour(WHITE, 1.0), None)?;
            let material = material_brush(&context)?;
            Ok(Surface {
                chain,
                context,
                bitmap: Some(bitmap),
                _target: target,
                _visual: visual,
                brush,
                material,
                formats: self.formats.clone(),
                advances: self.advances,
                width,
                height,
            })
        }
    }

    pub fn bitmap(&self, png: &'static [u8]) -> Result<ID2D1Bitmap1> {
        // Decode/upload once per device; the embedded bytes outlive the WIC stream.
        unsafe {
            let factory: IWICImagingFactory =
                CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)?;
            let stream = factory.CreateStream()?;
            stream.InitializeFromMemory(png)?;
            let decoder = factory.CreateDecoderFromStream(
                &stream,
                std::ptr::null(),
                WICDecodeMetadataCacheOnLoad,
            )?;
            let source =
                WICConvertBitmapSource(&GUID_WICPixelFormat32bppPBGRA, &decoder.GetFrame(0)?)?;
            self.d2d
                .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?
                .CreateBitmapFromWicBitmap(&source, None)
        }
    }

    pub fn begin_gpu_sample(&mut self) -> Option<f64> {
        self.gpu.as_mut().and_then(|g| g.begin(&self.context))
    }

    pub fn end_gpu_sample(&mut self) {
        if let Some(gpu) = &mut self.gpu {
            gpu.end(&self.context);
        }
    }
}

unsafe fn target_bitmap(
    context: &ID2D1DeviceContext,
    chain: &IDXGISwapChain1,
) -> Result<ID2D1Bitmap1> {
    // SAFETY: buffer zero exists on the live two-buffer swap chain.
    unsafe {
        let surface: IDXGISurface = chain.GetBuffer(0)?;
        let properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_B8G8R8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
            colorContext: Default::default(),
        };
        context.CreateBitmapFromDxgiSurface(&surface, Some(&properties))
    }
}

fn material_brush(context: &ID2D1DeviceContext) -> Result<ID2D1LinearGradientBrush> {
    unsafe {
        let target: ID2D1RenderTarget = context.cast()?;
        let stops = target.CreateGradientStopCollection(
            &[
                D2D1_GRADIENT_STOP {
                    position: 0.0,
                    color: colour([36, 42, 49], 0.98),
                },
                D2D1_GRADIENT_STOP {
                    position: 0.48,
                    color: colour([23, 29, 36], 0.98),
                },
                D2D1_GRADIENT_STOP {
                    position: 1.0,
                    color: colour([12, 17, 23], 0.98),
                },
            ],
            D2D1_GAMMA_2_2,
            D2D1_EXTEND_MODE_CLAMP,
        )?;
        context.CreateLinearGradientBrush(
            &D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
                startPoint: Vector2 { X: 0.0, Y: 65.0 },
                endPoint: Vector2 { X: 0.0, Y: 255.0 },
            },
            None,
            &stops,
        )
    }
}

pub struct Surface {
    chain: IDXGISwapChain1,
    context: ID2D1DeviceContext,
    bitmap: Option<ID2D1Bitmap1>,
    _target: IDCompositionTarget,
    _visual: IDCompositionVisual,
    brush: ID2D1SolidColorBrush,
    material: ID2D1LinearGradientBrush,
    formats: [IDWriteTextFormat; FONT_COUNT],
    advances: [[f32; 95]; FONT_COUNT],
    pub width: u32,
    pub height: u32,
}

impl Surface {
    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if (width, height) == (self.width, self.height) || width == 0 || height == 0 {
            return Ok(());
        }
        // SAFETY: unbind and release every backbuffer reference before ResizeBuffers.
        unsafe {
            self.context.SetTarget(None);
            self.bitmap = None;
            self.chain.ResizeBuffers(
                0,
                width,
                height,
                DXGI_FORMAT_UNKNOWN,
                DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT,
            )?;
            let bitmap = target_bitmap(&self.context, &self.chain)?;
            self.context.SetTarget(&bitmap);
            self.bitmap = Some(bitmap);
        }
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn begin(&mut self, base_width: f32, base_height: f32) -> Result<Painter<'_>> {
        // SAFETY: this context is never accessed from the telemetry thread.
        unsafe {
            self.context.BeginDraw();
            self.context.Clear(Some(&D2D1_COLOR_F::default()));
            let sx = self.width as f32 / base_width;
            let sy = self.height as f32 / base_height;
            self.context.SetTransform(&Matrix3x2 {
                M11: sx,
                M22: sy,
                ..Default::default()
            });
        }
        Ok(Painter {
            context: &self.context,
            brush: &self.brush,
            material: &self.material,
            formats: &self.formats,
            advances: &self.advances,
        })
    }

    pub fn present(&self) -> Result<()> {
        // SAFETY: BeginDraw/EndDraw are paired on the same UI-thread context.
        // DWM composes the alpha swap chain. Frame pacing is handled by the app's
        // high-resolution timer, avoiding a second blocking vsync per HUD.
        unsafe {
            self.context.EndDraw(None, None)?;
            self.chain.Present(0, DXGI_PRESENT(0)).ok()
        }
    }
}

pub struct Painter<'a> {
    context: &'a ID2D1DeviceContext,
    brush: &'a ID2D1SolidColorBrush,
    material: &'a ID2D1LinearGradientBrush,
    formats: &'a [IDWriteTextFormat; FONT_COUNT],
    advances: &'a [[f32; 95]; FONT_COUNT],
}

impl Painter<'_> {
    pub fn bitmap(&self, bitmap: &ID2D1Bitmap1, destination: D2D_RECT_F) {
        unsafe {
            self.context.DrawBitmap(
                bitmap,
                Some(&destination),
                1.0,
                D2D1_INTERPOLATION_MODE_HIGH_QUALITY_CUBIC,
                None,
                None,
            );
        }
    }

    pub fn material_geometry(&self, geometry: &ID2D1PathGeometry, opacity: f32) {
        unsafe {
            self.material.SetOpacity(opacity);
            self.context.FillGeometry(geometry, self.material, None);
        }
    }
    fn brush(&self, rgb: [u8; 3], opacity: f32) {
        // SAFETY: mutable native brush is confined to this thread and draw pass.
        unsafe {
            self.brush.SetColor(&colour(rgb, opacity));
        }
    }
    pub fn fill(&self, r: D2D_RECT_F, rgb: [u8; 3], opacity: f32) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.FillRectangle(&r, self.brush);
        }
    }
    pub fn rounded(&self, r: D2D_RECT_F, radius: f32, rgb: [u8; 3], opacity: f32) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.FillRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: r,
                    radiusX: radius,
                    radiusY: radius,
                },
                self.brush,
            );
        }
    }
    pub fn outline(&self, r: D2D_RECT_F, rgb: [u8; 3], width: f32) {
        self.brush(rgb, 1.0);
        unsafe {
            self.context.DrawRectangle(&r, self.brush, width, None);
        }
    }
    pub fn rounded_outline(
        &self,
        r: D2D_RECT_F,
        radius: f32,
        rgb: [u8; 3],
        width: f32,
        opacity: f32,
    ) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.DrawRoundedRectangle(
                &D2D1_ROUNDED_RECT {
                    rect: r,
                    radiusX: radius,
                    radiusY: radius,
                },
                self.brush,
                width,
                None,
            );
        }
    }
    pub fn translated(&self, offset: (f32, f32), draw: impl FnOnce(&Self)) {
        // Keep local padding independent of the surface's DPI/scale transform.
        unsafe {
            let mut previous = Matrix3x2::default();
            self.context.GetTransform(&mut previous);
            self.context
                .SetTransform(&(Matrix3x2::translation(offset.0, offset.1) * previous));
            draw(self);
            self.context.SetTransform(&previous);
        }
    }
    pub fn scaled(&self, centre: (f32, f32), scale: f32, draw: impl FnOnce(&Self)) {
        unsafe {
            let mut previous = Matrix3x2::default();
            self.context.GetTransform(&mut previous);
            self.context.SetTransform(
                &(Matrix3x2 {
                    M11: scale,
                    M22: scale,
                    M31: centre.0 * (1.0 - scale),
                    M32: centre.1 * (1.0 - scale),
                    ..Default::default()
                } * previous),
            );
            draw(self);
            self.context.SetTransform(&previous);
        }
    }
    pub fn rotated(&self, center: (f32, f32), degrees: f32, draw: impl FnOnce(&Self)) {
        // Local rotation precedes the surface's DPI/scale transform.
        unsafe {
            let mut previous = Matrix3x2::default();
            self.context.GetTransform(&mut previous);
            let rotation = Matrix3x2::rotation_around(
                degrees,
                Vector2 {
                    X: center.0,
                    Y: center.1,
                },
            );
            self.context.SetTransform(&(rotation * previous));
            draw(self);
            self.context.SetTransform(&previous);
        }
    }
    pub fn line(&self, a: (f32, f32), b: (f32, f32), rgb: [u8; 3], width: f32, opacity: f32) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.DrawLine(
                Vector2 { X: a.0, Y: a.1 },
                Vector2 { X: b.0, Y: b.1 },
                self.brush,
                width,
                None,
            );
        }
    }
    pub fn circle(&self, x: f32, y: f32, radius: f32, rgb: [u8; 3], opacity: f32) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.FillEllipse(
                &D2D1_ELLIPSE {
                    point: Vector2 { X: x, Y: y },
                    radiusX: radius,
                    radiusY: radius,
                },
                self.brush,
            );
        }
    }
    pub fn text(&self, value: &str, r: D2D_RECT_F, font: Font, rgb: [u8; 3]) {
        let mut wide = [0u16; 192];
        let mut len = 0;
        for code in value.encode_utf16().take(wide.len()) {
            wide[len] = code;
            len += 1;
        }
        self.brush(rgb, 1.0);
        unsafe {
            self.context.DrawText(
                &wide[..len],
                &self.formats[font.index()],
                &r,
                self.brush,
                D2D1_DRAW_TEXT_OPTIONS_CLIP,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
    }
    pub fn textf(&self, value: fmt::Arguments<'_>, r: D2D_RECT_F, font: Font, rgb: [u8; 3]) {
        let mut text = TextBuffer {
            bytes: [0; 192],
            len: 0,
        };
        let _ = text.write_fmt(value);
        self.text(text.as_str(), r, font, rgb);
    }
    pub fn curved_text(&self, value: &str, curve: &TextCurve, font: Font, rgb: [u8; 3]) {
        let advance = |character: char| {
            let index = (character as usize)
                .checked_sub(32)
                .filter(|&index| index < 95)
                .unwrap_or((b'?' - 32) as usize);
            self.advances[font.index()][index]
        };
        let width: f32 = value.chars().map(advance).sum();
        if width <= 0.0 {
            return;
        }
        let scale = (curve.length() / width).min(1.0);
        let height = unsafe { self.formats[font.index()].GetFontSize() } * 1.5;
        let mut distance = (curve.length() - width * scale) / 2.0;
        for character in value.chars() {
            let width = advance(character);
            let (point, angle) = curve.position(distance + width * scale / 2.0);
            let centre = (point.X, point.Y);
            self.rotated(centre, angle, |p| {
                p.scaled(centre, scale, |p| {
                    let mut buffer = [0; 4];
                    p.text(
                        character.encode_utf8(&mut buffer),
                        rect(
                            point.X - width / 2.0 - 2.0,
                            point.Y - height / 2.0,
                            width + 4.0,
                            height,
                        ),
                        font,
                        rgb,
                    );
                });
            });
            distance += width * scale;
        }
    }
    pub fn curved_textf(
        &self,
        value: fmt::Arguments<'_>,
        curve: &TextCurve,
        font: Font,
        rgb: [u8; 3],
    ) {
        let mut text = TextBuffer {
            bytes: [0; 192],
            len: 0,
        };
        let _ = text.write_fmt(value);
        self.curved_text(text.as_str(), curve, font, rgb);
    }
    pub fn geometry(&self, geometry: &ID2D1PathGeometry, rgb: [u8; 3], opacity: f32) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.FillGeometry(geometry, self.brush, None);
        }
    }
    pub fn stroke_geometry(
        &self,
        geometry: &ID2D1PathGeometry,
        rgb: [u8; 3],
        width: f32,
        opacity: f32,
    ) {
        self.brush(rgb, opacity);
        unsafe {
            self.context.DrawGeometry(geometry, self.brush, width, None);
        }
    }
}

struct TextBuffer {
    bytes: [u8; 192],
    len: usize,
}
impl TextBuffer {
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}
impl fmt::Write for TextBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.len + s.len() > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

struct GpuTimer {
    disjoint: ID3D11Query,
    start: ID3D11Query,
    finish: ID3D11Query,
    pending: bool,
    active: bool,
    frame: u64,
}

impl GpuTimer {
    unsafe fn new(device: &ID3D11Device) -> Result<Self> {
        let create = |kind| -> Result<ID3D11Query> {
            let mut query = None;
            unsafe {
                device.CreateQuery(
                    &D3D11_QUERY_DESC {
                        Query: kind,
                        MiscFlags: 0,
                    },
                    Some(&mut query),
                )?;
            }
            Ok(query.unwrap())
        };
        Ok(Self {
            disjoint: create(D3D11_QUERY_TIMESTAMP_DISJOINT)?,
            start: create(D3D11_QUERY_TIMESTAMP)?,
            finish: create(D3D11_QUERY_TIMESTAMP)?,
            pending: false,
            active: false,
            frame: 0,
        })
    }

    fn begin(&mut self, context: &ID3D11DeviceContext) -> Option<f64> {
        let mut result = None;
        if self.pending {
            let mut disjoint = D3D11_QUERY_DATA_TIMESTAMP_DISJOINT::default();
            let mut start = 0u64;
            let mut finish = 0u64;
            // GetData's high-level wrapper treats S_FALSE as success. Inspect the
            // HRESULT directly, so unfinished GPU queries never become fake samples.
            if query_ready(context, &self.disjoint, &mut disjoint)
                && query_ready(context, &self.start, &mut start)
                && query_ready(context, &self.finish, &mut finish)
            {
                self.pending = false;
                if !disjoint.Disjoint.as_bool() && disjoint.Frequency > 0 && finish >= start {
                    result = Some((finish - start) as f64 * 1000.0 / disjoint.Frequency as f64);
                }
            }
        }
        self.frame += 1;
        if !self.pending && self.frame % 30 == 0 {
            unsafe {
                context.Begin(&self.disjoint);
                context.End(&self.start);
            }
            self.active = true;
        }
        result
    }

    fn end(&mut self, context: &ID3D11DeviceContext) {
        if self.active {
            unsafe {
                context.End(&self.finish);
                context.End(&self.disjoint);
                context.Flush();
            }
            self.active = false;
            self.pending = true;
        }
    }
}

fn query_ready<T>(context: &ID3D11DeviceContext, query: &ID3D11Query, value: &mut T) -> bool {
    // SAFETY: each caller pairs a D3D11 query type with its documented output
    // structure. DONOTFLUSH prevents this diagnostic from stalling rendering.
    unsafe {
        (context.vtable().GetData)(
            context.as_raw(),
            query.as_raw(),
            (value as *mut T).cast(),
            size_of::<T>() as u32,
            D3D11_ASYNC_GETDATA_DONOTFLUSH.0 as u32,
        ) == S_OK
    }
}

#[cfg(test)]
mod tests {
    use super::TextCurve;

    #[test]
    fn curved_text_positions_follow_the_path_with_local_tangents() {
        let curve = TextCurve::new((0..=100).map(|i| {
            let x = -40.0 + 0.8 * i as f32;
            (x, x * x / 100.0)
        }));
        let (start, left_angle) = curve.position(-1.0);
        let (middle, middle_angle) = curve.position(curve.length() / 2.0);
        let (end, right_angle) = curve.position(curve.length() + 1.0);
        assert_eq!((start.X, start.Y), (-40.0, 16.0));
        assert_eq!((end.X, end.Y), (40.0, 16.0));
        assert!(middle.X.abs() < 0.001 && middle.Y.abs() < 0.001);
        assert!(left_angle < -35.0 && right_angle > 35.0 && middle_angle.abs() < 0.5);
    }
}
