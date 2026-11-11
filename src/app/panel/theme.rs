use super::*;
use std::cell::Cell;
use windows::Win32::{
    Graphics::Dwm::*,
    System::WindowsProgramming::MulDiv,
    UI::{Controls::WM_MOUSELEAVE, Input::KeyboardAndMouse::*},
};

pub const WIDTH: i32 = 1180;
pub const HEIGHT: i32 = 780;
pub const NAV: u16 = 400;
const BACK: u32 = 0x1a2e52;
const SIDE: u32 = 0x22375b;
const CARD: u32 = 0x23395e;
const INPUT: u32 = 0x192c4e;
const LINE: u32 = 0x51698e;
const TEXT: u32 = 0xeaf2ff;
const MUTED: u32 = 0xa9bbd6;
const CYAN: u32 = 0x64ddf5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Layout,
    Tyres,
    Colours,
    Connection,
    Shortcuts,
    Diagnostics,
}

impl Page {
    pub const ALL: [Self; 6] = [
        Self::Layout,
        Self::Tyres,
        Self::Colours,
        Self::Connection,
        Self::Shortcuts,
        Self::Diagnostics,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Self::Layout => "Halo & layout",
            Self::Tyres => "Tyres",
            Self::Colours => "Colours",
            Self::Connection => "Connection",
            Self::Shortcuts => "Shortcuts",
            Self::Diagnostics => "Diagnostics",
        }
    }

    fn description(self) -> &'static str {
        match self {
            Self::Layout => "Background 0–100%: fade the panels while text stays sharp.",
            Self::Tyres => "Set the temperature window and wear alerts.",
            Self::Colours => "Give every telemetry channel its own colour.",
            Self::Connection => "Connect to the UDP telemetry sent by F1 25.",
            Self::Shortcuts => "Keep overlay controls within reach while you drive.",
            Self::Diagnostics => "Live receiver, rendering and battery information.",
        }
    }
}

fn rgb(hex: u32) -> COLORREF {
    COLORREF(((hex & 0xff) << 16) | (hex & 0xff00) | (hex >> 16))
}

pub struct Theme {
    dpi: u32,
    pub page: Cell<Page>,
    pub layout: Cell<ResponsiveLayout>,
    pub children: Vec<PositionedControl>,
    reflowing: Cell<bool>,
    hot: Cell<Option<HWND>>,
    pub inputs: Vec<PositionedControl>,
    fonts: [HFONT; 5],
    brushes: [HBRUSH; 3],
    // Panel owns the icon across DPI-driven theme rebuilds.
    logo: HICON,
}

impl Theme {
    pub fn new(dpi: u32, logo: HICON) -> Self {
        let fonts = [(14, 400), (12, 400), (20, 500), (29, 400), (22, 600)].map(
            |(height, weight)| unsafe {
                CreateFontW(
                    -MulDiv(height, dpi as i32, 96),
                    0,
                    0,
                    0,
                    weight,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_DEFAULT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    CLEARTYPE_QUALITY,
                    DEFAULT_PITCH.0 as u32,
                    w!("Segoe UI"),
                )
            },
        );
        Self {
            dpi,
            page: Cell::new(Page::Layout),
            layout: Cell::new(ResponsiveLayout::new(WIDTH, HEIGHT, Page::Layout)),
            children: Vec::new(),
            reflowing: Cell::new(false),
            hot: Cell::new(None),
            inputs: Vec::new(),
            fonts,
            brushes: [SIDE, CARD, INPUT].map(|colour| unsafe { CreateSolidBrush(rgb(colour)) }),
            logo,
        }
    }

    pub fn font(&self) -> HFONT {
        self.fonts[0]
    }

    pub fn resize(&self, hwnd: HWND) {
        if self.reflowing.replace(true) {
            return;
        }
        unsafe {
            let mut client = RECT::default();
            let _ = GetClientRect(hwnd, &mut client);
            if client.right <= 0 || client.bottom <= 0 {
                self.reflowing.set(false);
                return;
            }
            for _ in 0..3 {
                let _ = GetClientRect(hwnd, &mut client);
                let layout = ResponsiveLayout::new(
                    client.right * 96 / self.dpi as i32,
                    client.bottom * 96 / self.dpi as i32,
                    self.page.get(),
                );
                self.layout.set(layout);
                for (bar, full, page) in [
                    (
                        SB_VERT,
                        MulDiv(layout.content_height, self.dpi as i32, 96),
                        client.bottom,
                    ),
                    (
                        SB_HORZ,
                        MulDiv(layout.width, self.dpi as i32, 96),
                        client.right,
                    ),
                ] {
                    let info = SCROLLINFO {
                        cbSize: size_of::<SCROLLINFO>() as u32,
                        fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                        nMax: full - 1,
                        nPage: page as u32,
                        nPos: GetScrollPos(hwnd, bar).clamp(0, (full - page).max(0)),
                        ..Default::default()
                    };
                    SetScrollInfo(hwnd, bar, &info, true);
                }
                let mut after = RECT::default();
                let _ = GetClientRect(hwnd, &mut after);
                if after == client {
                    break;
                }
            }
            let layout = self.layout.get();
            let offset = GetScrollPos(hwnd, SB_VERT);
            let offset_x = GetScrollPos(hwnd, SB_HORZ);
            let mut moves = Vec::new();
            for &(child, bounds, page) in &self.children {
                let visible = page.is_none_or(|page| page == self.page.get());
                let was_visible = GetWindowLongPtrW(child, GWL_STYLE) as u32 & WS_VISIBLE.0 != 0;
                let mut flags = SWP_NOACTIVATE | SWP_NOZORDER;
                if !visible {
                    if was_visible {
                        moves.push((
                            child,
                            (0, 0, 0, 0),
                            flags | SWP_HIDEWINDOW | SWP_NOMOVE | SWP_NOSIZE,
                        ));
                    }
                    continue;
                }
                if !was_visible {
                    flags |= SWP_SHOWWINDOW;
                }
                let mut mapped = layout.map(bounds);
                if let Some((_, frame, _)) =
                    self.inputs.iter().find(|(control, _, _)| *control == child)
                {
                    let (x, y, w, h) = layout.map(*frame);
                    let inner_height = 21.min(h - 8);
                    mapped = (x + 10, y + (h - inner_height) / 2, w - 20, inner_height);
                }
                let (x, y, w, h) = mapped;
                moves.push((
                    child,
                    (
                        MulDiv(x, self.dpi as i32, 96) - offset_x,
                        MulDiv(y, self.dpi as i32, 96) - offset,
                        MulDiv(x + w, self.dpi as i32, 96) - MulDiv(x, self.dpi as i32, 96),
                        MulDiv(y + h, self.dpi as i32, 96) - MulDiv(y, self.dpi as i32, 96),
                    ),
                    flags,
                ));
            }
            let deferred = BeginDeferWindowPos(moves.len() as i32).and_then(|batch| {
                let batch =
                    moves
                        .iter()
                        .try_fold(batch, |batch, &(child, (x, y, w, h), flags)| {
                            DeferWindowPos(batch, child, None, x, y, w, h, flags)
                        })?;
                EndDeferWindowPos(batch)
            });
            if deferred.is_err() {
                for (child, (x, y, w, h), flags) in moves {
                    let _ = SetWindowPos(child, None, x, y, w, h, flags);
                }
            }
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        self.reflowing.set(false);
    }

    fn ensure_visible(&self, parent: HWND, child: HWND) {
        if self.reflowing.get() {
            return;
        }
        unsafe {
            let mut bounds = RECT::default();
            let mut client = RECT::default();
            if GetWindowRect(child, &mut bounds).is_err()
                || GetClientRect(parent, &mut client).is_err()
            {
                return;
            }
            let mut origin = POINT {
                x: bounds.left,
                y: bounds.top,
            };
            let _ = ScreenToClient(parent, &mut origin);
            let mut changed = false;
            for (bar, start, end, page) in [
                (
                    SB_VERT,
                    origin.y,
                    origin.y + bounds.bottom - bounds.top,
                    client.bottom,
                ),
                (
                    SB_HORZ,
                    origin.x,
                    origin.x + bounds.right - bounds.left,
                    client.right,
                ),
            ] {
                let delta = if start < 8 {
                    start - 8
                } else if end > page - 8 {
                    end - page + 8
                } else {
                    0
                };
                if delta != 0 {
                    SetScrollInfo(
                        parent,
                        bar,
                        &SCROLLINFO {
                            cbSize: size_of::<SCROLLINFO>() as u32,
                            fMask: SIF_POS,
                            nPos: GetScrollPos(parent, bar) + delta,
                            ..Default::default()
                        },
                        true,
                    );
                    changed = true;
                }
            }
            if changed {
                self.resize(parent);
            }
        }
    }

    fn draw_button(&self, hwnd: HWND, dc: HDC) {
        let theme = self;
        unsafe {
            let saved = SaveDC(dc);
            let p = Painter {
                dc,
                theme,
                x: 0,
                y: 0,
                responsive: false,
            };
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let width = MulDiv(rect.right, 96, theme.dpi as i32);
            let height = MulDiv(rect.bottom, 96, theme.dpi as i32);
            let id = GetDlgCtrlID(hwnd) as u16;
            let nav = id
                .checked_sub(NAV)
                .and_then(|i| Page::ALL.get(i as usize))
                .copied();
            let selected = nav.is_some_and(|page| page == theme.page.get());
            let primary = id == APPLY || selected;
            let checked = SendMessageW(hwnd, BM_GETCHECK, None, None).0 == 1;
            let toggle = matches!(id, DEMO | HALO | TYRES | REAR_VIEW);
            let state = SendMessageW(hwnd, BM_GETSTATE, None, None).0;
            let pressed = state & 4 != 0;
            let hover = theme.hot.get() == Some(hwnd);
            let fill = if primary {
                if pressed {
                    0x45bbd9
                } else if hover {
                    0x8ce8fa
                } else {
                    CYAN
                }
            } else if pressed {
                INPUT
            } else if hover {
                0x304d73
            } else {
                CARD
            };
            p.fill(
                (0, 0, width, height),
                if nav.is_some() || matches!(id, EDIT | TOGGLE_OVERLAYS | EXPORT | EXIT | SHOW) {
                    SIDE
                } else {
                    CARD
                },
            );
            if !toggle {
                p.round(
                    (0, 0, width, height),
                    fill,
                    if primary { CYAN } else { LINE },
                    8,
                );
            } else if hover {
                p.round((0, 0, width, height), fill, fill, 6);
            }
            let mut buffer = [0u16; 256];
            let n = GetWindowTextW(hwnd, &mut buffer);
            let text = String::from_utf16_lossy(&buffer[..n.max(0) as usize]);
            let colour = if primary { INPUT } else { TEXT };
            let (left, reserved, alignment) = if nav.is_some() {
                (48, 56, DT_LEFT)
            } else if toggle {
                (0, 54, DT_LEFT)
            } else {
                (5, 10, DT_CENTER)
            };
            p.text(
                &text,
                (left, 0, width - reserved, height),
                if nav.is_some() && width < 150 { 1 } else { 0 },
                colour,
                DT_SINGLELINE | DT_VCENTER | alignment,
            );
            if let Some(page) = nav {
                p.icon(page, colour);
            }
            if toggle {
                p.round(
                    (width - 44, (height - 22) / 2, 43, 22),
                    if checked { 0x386481 } else { 0x3c5072 },
                    LINE,
                    22,
                );
                p.round(
                    (
                        width - if checked { 21 } else { 41 },
                        (height - 16) / 2,
                        16,
                        16,
                    ),
                    if checked { CYAN } else { 0x8d9eb9 },
                    if checked { CYAN } else { 0x8d9eb9 },
                    16,
                );
            }
            if GetFocus() == hwnd {
                let mut focus = rect;
                let _ = InflateRect(&mut focus, -3, -3);
                SetTextColor(dc, rgb(CYAN));
                let _ = DrawFocusRect(dc, &focus);
            }
            let _ = RestoreDC(dc, saved);
        }
    }

    fn draw_combo_frame(&self, hwnd: HWND, dc: HDC) {
        let theme = self;
        unsafe {
            let saved = SaveDC(dc);
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let p = Painter {
                dc,
                theme,
                x: 0,
                y: 0,
                responsive: false,
            };
            let width = MulDiv(rect.right, 96, theme.dpi as i32);
            let height = MulDiv(rect.bottom, 96, theme.dpi as i32);
            p.fill((0, 0, width, 3), INPUT);
            p.fill((0, height - 3, width, 3), INPUT);
            p.fill((0, 0, 3, height), INPUT);
            p.fill((width - 3, 0, 3, height), INPUT);
            p.fill((width - 28, 1, 27, height - 2), INPUT);
            p.line(
                width - 19,
                height / 2 - 2,
                width - 14,
                height / 2 + 3,
                MUTED,
            );
            p.line(width - 14, height / 2 + 3, width - 9, height / 2 - 2, MUTED);
            p.outline(
                (0, 0, width, height),
                if GetFocus() == hwnd { CYAN } else { LINE },
                7,
            );
            let _ = RestoreDC(dc, saved);
        }
    }

    pub fn attach(&self, hwnd: HWND) {
        unsafe {
            // The Box stays at the same address until after DestroyWindow.
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, self as *const Self as isize);
            let enabled = 1i32;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &enabled as *const _ as _,
                size_of::<i32>() as u32,
            );
            let colour = rgb(BACK);
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_CAPTION_COLOR,
                &colour as *const _ as _,
                size_of::<COLORREF>() as u32,
            );
        }
    }

    fn paint(&self, hwnd: HWND) {
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let dc = BeginPaint(hwnd, &mut ps);
            let saved = SaveDC(dc);
            let p = Painter {
                dc,
                theme: self,
                x: -GetScrollPos(hwnd, SB_HORZ),
                y: -GetScrollPos(hwnd, SB_VERT),
                responsive: true,
            };
            let mut client = RECT::default();
            let _ = GetClientRect(hwnd, &mut client);
            SetDCBrushColor(dc, rgb(BACK));
            FillRect(dc, &client, HBRUSH(GetStockObject(DC_BRUSH).0));
            p.fill((0, 0, 216, HEIGHT), SIDE);
            p.fill((892, 86, 288, HEIGHT - 86), SIDE);
            p.line(0, 85, WIDTH, 85, LINE);
            p.line(216, 0, 216, HEIGHT, LINE);
            p.line(892, 86, 892, HEIGHT, 0x30476c);

            let logo = p.rect((10, 12, 60, 60));
            let _ = DrawIconEx(
                dc,
                logo.left,
                logo.top,
                self.logo,
                logo.right - logo.left,
                logo.bottom - logo.top,
                0,
                None,
                DI_NORMAL,
            );
            p.text("HALO", (72, 21, 124, 30), 4, TEXT, DT_SINGLELINE);
            p.text("TELEMETRY", (73, 50, 122, 17), 1, MUTED, DT_SINGLELINE);
            p.text(
                "Software settings",
                (244, 20, 580, 43),
                3,
                TEXT,
                DT_SINGLELINE,
            );
            p.text(
                "F1 25  /  2026 SEASON PACK",
                (916, 33, 242, 23),
                1,
                MUTED,
                DT_SINGLELINE,
            );
            p.text("WORKSPACE", (26, 112, 170, 20), 1, MUTED, DT_SINGLELINE);
            p.text(
                "Made for your cockpit.",
                (26, 615, 172, 23),
                0,
                TEXT,
                DT_SINGLELINE,
            );
            p.text(
                "Native overlays\nUDP format 2026",
                (26, 645, 170, 43),
                1,
                MUTED,
                DT_WORDBREAK,
            );
            p.line(22, 700, 194, 700, LINE);

            let page = self.page.get();
            p.text(page.title(), (244, 114, 620, 30), 2, TEXT, DT_SINGLELINE);
            p.text(
                page.description(),
                (244, 153, 620, 24),
                0,
                MUTED,
                DT_WORDBREAK,
            );
            match page {
                Page::Layout => {
                    for (x, id) in [(244, HALO), (564, TYRES)] {
                        let checked =
                            GetDlgItem(Some(hwnd), id as i32)
                                .ok()
                                .is_some_and(|control| {
                                    SendMessageW(control, BM_GETCHECK, None, None).0 == 1
                                });
                        p.round(
                            (x, 196, 300, 238),
                            CARD,
                            if checked { CYAN } else { LINE },
                            16,
                        );
                    }
                    p.round((244, 454, 620, 66), CARD, LINE, 10);
                    p.round((244, 540, 620, 100), CARD, LINE, 10);
                    p.text(
                        "Position & scale",
                        (660, 555, 184, 21),
                        1,
                        MUTED,
                        DT_RIGHT | DT_SINGLELINE,
                    );
                }
                Page::Tyres => {
                    p.card((244, 196, 620, 110), "Temperature display");
                    p.card((244, 326, 300, 236), "Temperature window");
                    p.card((564, 326, 300, 236), "Wear alerts");
                    p.text("°C", (490, 350, 34, 23), 0, CYAN, DT_RIGHT);
                    p.text("%", (810, 350, 34, 23), 0, CYAN, DT_RIGHT);
                    p.text(
                        "Below minimum: cold\nAbove maximum: hot",
                        (264, 580, 280, 44),
                        1,
                        MUTED,
                        DT_WORDBREAK,
                    );
                    p.text(
                        "0% wear = a new tyre",
                        (584, 580, 278, 24),
                        1,
                        MUTED,
                        DT_SINGLELINE,
                    );
                }
                Page::Colours => {
                    for (i, (title, note)) in [
                        ("Accent", "Halo arcs & details"),
                        ("Throttle", "Acceleration input"),
                        ("Brake", "Braking input"),
                        ("Cold", "Below the temperature window"),
                        ("Optimal", "Inside the temperature window"),
                        ("Hot", "Above the temperature window"),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let x = 244 + (i % 2) as i32 * 320;
                        let y = 196 + (i / 2) as i32 * 146;
                        p.card((x, y, 300, 126), title);
                        p.text(note, (x + 20, y + 46, 264, 20), 1, MUTED, DT_SINGLELINE);
                        let mut value = [0u16; 16];
                        if let Ok(control) = GetDlgItem(Some(hwnd), (COLOUR + i as u16) as i32) {
                            let n = GetWindowTextW(control, &mut value);
                            if let Ok(colour) = u32::from_str_radix(
                                String::from_utf16_lossy(&value[..n.max(0) as usize])
                                    .trim()
                                    .trim_start_matches('#'),
                                16,
                            ) {
                                p.round((x + 20, y + 80, 32, 30), colour, LINE, 7);
                            }
                        }
                    }
                }
                Page::Connection => {
                    p.card((244, 196, 620, 154), "UDP receiver");
                    p.card((244, 370, 620, 240), "Set these values in F1 25");
                    for (i, (label, value)) in [
                        ("UDP telemetry", "On"),
                        ("Broadcast mode", "Off"),
                        ("Send rate", "60 Hz"),
                        ("UDP format", "2026"),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let y = 425 + i as i32 * 40;
                        p.text(label, (264, y, 310, 24), 0, MUTED, DT_SINGLELINE);
                        p.text(value, (660, y, 184, 24), 0, CYAN, DT_RIGHT | DT_SINGLELINE);
                        if i < 3 {
                            p.line(264, y + 30, 844, y + 30, 0x354d73);
                        }
                    }
                    p.text(
                        "Use the receiver IP and port above as the game's UDP destination.",
                        (244, 628, 620, 24),
                        1,
                        MUTED,
                        DT_WORDBREAK,
                    );
                }
                Page::Shortcuts => {
                    p.card((244, 196, 620, 166), "Edit / Race Mode");
                    p.card((244, 382, 620, 166), "HUD visibility");
                    p.text("Edit: drag to move · resize at the corners\nWheel: scale · Shift + wheel: background · Esc: Race Mode", (264, 572, 580, 54), 0, MUTED, DT_WORDBREAK);
                }
                Page::Diagnostics => {
                    p.card((244, 196, 620, 138), "Automatic ERS capacity");
                    p.card((244, 354, 620, 286), "Live diagnostics");
                }
            }

            for &(control, bounds, page) in &self.inputs {
                if page.is_none_or(|page| page == self.page.get()) {
                    p.round(
                        bounds,
                        INPUT,
                        if GetFocus() == control { CYAN } else { LINE },
                        7,
                    );
                }
            }
            p.text(
                "General controls",
                (916, 115, 244, 31),
                2,
                TEXT,
                DT_SINGLELINE,
            );
            p.line(916, 157, 1156, 157, LINE);
            p.text("OVERLAYS", (916, 180, 240, 20), 1, TEXT, DT_SINGLELINE);
            p.text("Frame rate", (916, 264, 240, 22), 0, MUTED, DT_SINGLELINE);
            p.line(916, 345, 1156, 345, LINE);
            p.text("BEHAVIOUR", (916, 369, 240, 20), 1, TEXT, DT_SINGLELINE);
            p.text(
                "Use Apply and save to apply changes\nto the fields and switches.",
                (916, 497, 242, 44),
                1,
                MUTED,
                DT_WORDBREAK,
            );
            p.line(916, 556, 1156, 556, LINE);
            p.text("SESSION", (916, 578, 240, 20), 1, TEXT, DT_SINGLELINE);
            p.line(244, 700, 864, 700, LINE);
            let _ = RestoreDC(dc, saved);
            let _ = EndPaint(hwnd, &ps);
        }
    }
}

impl Drop for Theme {
    fn drop(&mut self) {
        unsafe {
            for font in self.fonts {
                let _ = DeleteObject(font.into());
            }
            for brush in self.brushes {
                let _ = DeleteObject(brush.into());
            }
        }
    }
}

struct Painter<'a> {
    dc: HDC,
    theme: &'a Theme,
    x: i32,
    y: i32,
    responsive: bool,
}

impl Painter<'_> {
    fn s(&self, value: i32) -> i32 {
        unsafe { MulDiv(value, self.theme.dpi as i32, 96) }
    }
    fn rect(&self, bounds: Bounds) -> RECT {
        let (x, y, w, h) = if self.responsive {
            self.theme.layout.get().map(bounds)
        } else {
            bounds
        };
        RECT {
            left: self.x + self.s(x),
            top: self.y + self.s(y),
            right: self.x + self.s(x + w),
            bottom: self.y + self.s(y + h),
        }
    }
    fn fill(&self, bounds: (i32, i32, i32, i32), colour: u32) {
        unsafe {
            SetDCBrushColor(self.dc, rgb(colour));
            FillRect(
                self.dc,
                &self.rect(bounds),
                HBRUSH(GetStockObject(DC_BRUSH).0),
            );
        }
    }
    fn round(&self, bounds: (i32, i32, i32, i32), fill: u32, stroke: u32, radius: i32) {
        let r = self.rect(bounds);
        unsafe {
            SelectObject(self.dc, GetStockObject(DC_BRUSH));
            SelectObject(self.dc, GetStockObject(DC_PEN));
            SetDCBrushColor(self.dc, rgb(fill));
            SetDCPenColor(self.dc, rgb(stroke));
            let _ = RoundRect(
                self.dc,
                r.left,
                r.top,
                r.right,
                r.bottom,
                self.s(radius),
                self.s(radius),
            );
        }
    }
    fn line(&self, x1: i32, y1: i32, x2: i32, y2: i32, colour: u32) {
        let rect = self.rect((x1, y1, x2 - x1, y2 - y1));
        unsafe {
            SelectObject(self.dc, GetStockObject(DC_PEN));
            SetDCPenColor(self.dc, rgb(colour));
            let _ = MoveToEx(self.dc, rect.left, rect.top, None);
            let _ = LineTo(self.dc, rect.right, rect.bottom);
        }
    }
    fn outline(&self, bounds: (i32, i32, i32, i32), colour: u32, radius: i32) {
        let r = self.rect(bounds);
        unsafe {
            SelectObject(self.dc, GetStockObject(HOLLOW_BRUSH));
            SelectObject(self.dc, GetStockObject(DC_PEN));
            SetDCPenColor(self.dc, rgb(colour));
            let _ = RoundRect(
                self.dc,
                r.left,
                r.top,
                r.right,
                r.bottom,
                self.s(radius),
                self.s(radius),
            );
        }
    }
    fn text(
        &self,
        text: &str,
        bounds: (i32, i32, i32, i32),
        font: usize,
        colour: u32,
        flags: DRAW_TEXT_FORMAT,
    ) {
        let mut text: Vec<u16> = text.encode_utf16().collect();
        unsafe {
            SelectObject(self.dc, self.theme.fonts[font].into());
            SetTextColor(self.dc, rgb(colour));
            SetBkMode(self.dc, TRANSPARENT);
            DrawTextW(
                self.dc,
                &mut text,
                &mut self.rect(bounds),
                flags | DT_NOPREFIX,
            );
        }
    }
    fn card(&self, bounds: (i32, i32, i32, i32), title: &str) {
        self.round(bounds, CARD, LINE, 12);
        self.text(
            title,
            (bounds.0 + 20, bounds.1 + 19, bounds.2 - 40, 28),
            0,
            TEXT,
            DT_SINGLELINE,
        );
    }
    fn icon(&self, page: Page, colour: u32) {
        match page {
            Page::Layout => {
                for (x, y) in [(16, 14), (27, 14), (16, 25), (27, 25)] {
                    self.round((x, y, 7, 7), colour, colour, 2);
                }
            }
            Page::Tyres => {
                self.round((16, 12, 8, 24), colour, colour, 5);
                self.round((29, 12, 8, 24), colour, colour, 5);
            }
            Page::Colours => {
                for (x, y) in [(17, 14), (27, 14), (22, 25)] {
                    self.round((x, y, 8, 8), colour, colour, 8);
                }
            }
            Page::Connection => {
                self.line(26, 15, 26, 30, colour);
                self.line(17, 30, 35, 30, colour);
                for (x, y) in [(23, 12), (14, 28), (32, 28)] {
                    self.round((x, y, 6, 6), colour, colour, 2);
                }
            }
            Page::Shortcuts => {
                self.round((14, 16, 24, 17), colour, colour, 3);
                self.line(18, 21, 34, 21, SIDE);
                self.line(20, 28, 32, 28, SIDE);
            }
            Page::Diagnostics => {
                for (x, y, h) in [(16, 25, 9), (25, 17, 17), (34, 11, 23)] {
                    self.round((x, y, 4, h), colour, colour, 2);
                }
            }
        }
    }
}

unsafe fn from_window<'a>(hwnd: HWND) -> Option<&'a Theme> {
    unsafe { (GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const Theme).as_ref() }
}

pub fn message(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    let theme = unsafe { from_window(hwnd)? };
    match message {
        WM_SIZE if wparam.0 as u32 != SIZE_MINIMIZED => {
            theme.resize(hwnd);
            Some(LRESULT(0))
        }
        WM_PAINT => {
            theme.paint(hwnd);
            Some(LRESULT(0))
        }
        WM_ERASEBKGND => Some(LRESULT(1)),
        WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX | WM_CTLCOLORSTATIC => unsafe {
            let dc = HDC(wparam.0 as *mut _);
            let id = GetDlgCtrlID(HWND(lparam.0 as *mut _)) as u16;
            let (colour, brush) = if message != WM_CTLCOLORSTATIC {
                (INPUT, theme.brushes[2])
            } else if matches!(id, CONNECTION_STATUS | RUNTIME_STATUS) {
                (SIDE, theme.brushes[0])
            } else if id == RESULT {
                SetDCBrushColor(dc, rgb(BACK));
                (BACK, HBRUSH(GetStockObject(DC_BRUSH).0))
            } else {
                (CARD, theme.brushes[1])
            };
            SetTextColor(
                dc,
                rgb(if id == RESULT || id == CONNECTION_STATUS {
                    CYAN
                } else {
                    TEXT
                }),
            );
            SetBkColor(dc, rgb(colour));
            Some(LRESULT(brush.0 as isize))
        },
        WM_DRAWITEM => unsafe {
            let item = &*(lparam.0 as *const windows::Win32::UI::Controls::DRAWITEMSTRUCT);
            if item.CtlType == windows::Win32::UI::Controls::ODT_BUTTON {
                theme.draw_button(item.hwndItem, item.hDC);
                return Some(LRESULT(1));
            }
            if item.CtlType != windows::Win32::UI::Controls::ODT_COMBOBOX {
                return None;
            }
            let dc = item.hDC;
            let saved = SaveDC(dc);
            let selected = item.itemState.0 & windows::Win32::UI::Controls::ODS_SELECTED.0 != 0;
            SetDCBrushColor(dc, rgb(if selected { 0x355475 } else { INPUT }));
            FillRect(dc, &item.rcItem, HBRUSH(GetStockObject(DC_BRUSH).0));
            if item.itemID != u32::MAX {
                // CB_GETLBTEXT needs the full length, including saved 64-character profile names.
                let length = SendMessageW(
                    item.hwndItem,
                    CB_GETLBTEXTLEN,
                    Some(WPARAM(item.itemID as usize)),
                    None,
                )
                .0;
                if (0..=1024).contains(&length) {
                    let mut text = vec![0u16; length as usize + 1];
                    SendMessageW(
                        item.hwndItem,
                        CB_GETLBTEXT,
                        Some(WPARAM(item.itemID as usize)),
                        Some(LPARAM(text.as_mut_ptr() as isize)),
                    );
                    let mut rect = item.rcItem;
                    rect.left += MulDiv(10, theme.dpi as i32, 96);
                    SelectObject(dc, theme.font().into());
                    SetBkMode(dc, TRANSPARENT);
                    SetTextColor(dc, rgb(TEXT));
                    DrawTextW(
                        dc,
                        &mut text[..length as usize],
                        &mut rect,
                        DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
                    );
                }
            }
            if item.itemState.0 & windows::Win32::UI::Controls::ODS_FOCUS.0 != 0 {
                let _ = DrawFocusRect(dc, &item.rcItem);
            }
            let _ = RestoreDC(dc, saved);
            Some(LRESULT(1))
        },
        WM_COMMAND if (wparam.0 >> 16) == EN_CHANGE as usize => {
            let index = (wparam.0 as u16).wrapping_sub(COLOUR);
            if index < 6 && theme.page.get() == Page::Colours {
                unsafe {
                    let x = 264 + (index % 2) as i32 * 320;
                    let y = 276 + (index / 2) as i32 * 146;
                    let (x, y, w, h) = theme.layout.get().map((x, y, 32, 30));
                    let rect = RECT {
                        left: MulDiv(x, theme.dpi as i32, 96) - GetScrollPos(hwnd, SB_HORZ),
                        top: MulDiv(y, theme.dpi as i32, 96) - GetScrollPos(hwnd, SB_VERT),
                        right: MulDiv(x + w, theme.dpi as i32, 96) - GetScrollPos(hwnd, SB_HORZ),
                        bottom: MulDiv(y + h, theme.dpi as i32, 96) - GetScrollPos(hwnd, SB_VERT),
                    };
                    let _ = InvalidateRect(Some(hwnd), Some(&rect), false);
                }
            }
            None
        }
        _ => None,
    }
}

pub unsafe extern "system" fn control_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    kind: usize,
) -> LRESULT {
    unsafe {
        let parent = GetParent(hwnd).unwrap_or_default();
        let theme = from_window(parent);
        let owner_draw =
            kind == 1 && GetWindowLongPtrW(hwnd, GWL_STYLE) as i32 & BS_TYPEMASK == BS_OWNERDRAW;
        if let Some(theme) = theme {
            if message == WM_PRINTCLIENT && kind == 1 {
                theme.draw_button(hwnd, HDC(wparam.0 as *mut _));
                return LRESULT(0);
            }
            if message == WM_PAINT && ((kind == 1 && !owner_draw) || kind == 3) {
                let mut ps = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut ps);
                if kind == 1 {
                    theme.draw_button(hwnd, dc);
                } else {
                    // Paint native content and its skin in the same buffered paint cycle.
                    let saved = SaveDC(dc);
                    DefSubclassProc(
                        hwnd,
                        WM_PRINTCLIENT,
                        WPARAM(dc.0 as usize),
                        LPARAM(PRF_CLIENT as isize),
                    );
                    let _ = RestoreDC(dc, saved);
                    theme.draw_combo_frame(hwnd, dc);
                }
                let _ = EndPaint(hwnd, &ps);
                return LRESULT(0);
            }
            if message == WM_PRINTCLIENT && kind == 3 {
                let dc = HDC(wparam.0 as *mut _);
                let saved = SaveDC(dc);
                let result = DefSubclassProc(hwnd, message, wparam, lparam);
                let _ = RestoreDC(dc, saved);
                theme.draw_combo_frame(hwnd, dc);
                return result;
            }
            if message == WM_ERASEBKGND && matches!(kind, 1 | 3) {
                return LRESULT(1);
            }
            if matches!(message, WM_MOUSEMOVE | WM_MOUSELEAVE) && matches!(kind, 1 | 3) {
                let mut rect = RECT::default();
                let _ = GetClientRect(hwnd, &mut rect);
                let point = POINT {
                    x: lparam.0 as i16 as i32,
                    y: (lparam.0 >> 16) as i16 as i32,
                };
                let inside = message == WM_MOUSEMOVE && PtInRect(&rect, point).as_bool();
                let old = theme.hot.get();
                let next = if inside {
                    Some(hwnd)
                } else {
                    old.filter(|control| *control != hwnd)
                };
                if next != old {
                    theme.hot.set(next);
                    for control in [old, next].into_iter().flatten() {
                        let _ = InvalidateRect(Some(control), None, false);
                    }
                    if inside {
                        let mut track = TRACKMOUSEEVENT {
                            cbSize: size_of::<TRACKMOUSEEVENT>() as u32,
                            dwFlags: TME_LEAVE,
                            hwndTrack: hwnd,
                            ..Default::default()
                        };
                        let _ = TrackMouseEvent(&mut track);
                    }
                }
                // Preserve drag-out cancellation while a native control owns mouse capture.
                if GetCapture() != hwnd {
                    return LRESULT(0);
                }
            }
        }
        let id = GetDlgCtrlID(hwnd) as u16;
        let toggle = kind == 1 && !owner_draw;
        let checked = toggle && DefSubclassProc(hwnd, BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == 1;
        let result = DefSubclassProc(hwnd, message, wparam, lparam);
        let focus_changed = matches!(message, WM_SETFOCUS | WM_KILLFOCUS);
        if message == WM_SETFOCUS {
            if let Some(theme) = theme {
                theme.ensure_visible(parent, hwnd);
            }
        }
        if kind == 2 && focus_changed {
            let _ = InvalidateRect(Some(parent), None, false);
        }
        if toggle
            && matches!(
                message,
                WM_LBUTTONDOWN
                    | WM_LBUTTONUP
                    | WM_MOUSEMOVE
                    | WM_KEYDOWN
                    | WM_KEYUP
                    | WM_SETFOCUS
                    | WM_KILLFOCUS
                    | WM_ENABLE
                    | WM_SETTEXT
                    | WM_SETFONT
                    | WM_UPDATEUISTATE
                    | BM_SETCHECK
                    | BM_SETSTATE
            )
        {
            let _ = InvalidateRect(Some(hwnd), None, false);
            // Native checkboxes keep keyboard/accessibility behaviour; finish their custom paint now.
            let _ = UpdateWindow(hwnd);
            if matches!(id, HALO | TYRES)
                && checked != (SendMessageW(hwnd, BM_GETCHECK, None, None).0 == 1)
            {
                let _ = InvalidateRect(Some(parent), None, false);
            }
        }
        if kind == 3
            && (focus_changed
                || matches!(
                    message,
                    WM_ENABLE | WM_SETTEXT | WM_SETFONT | CB_SETCURSEL | CB_SHOWDROPDOWN
                ))
        {
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
        if message == WM_NCDESTROY {
            if let Some(theme) = theme {
                if theme.hot.get() == Some(hwnd) {
                    theme.hot.set(None);
                }
            }
            let _ = RemoveWindowSubclass(hwnd, Some(control_proc), 1);
        }
        result
    }
}
