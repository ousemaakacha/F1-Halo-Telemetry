use crate::{
    input::rear_view,
    overlay::{self, Action, wide},
    settings::*,
};
use std::{collections::BTreeMap, mem::size_of};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{EM_SETLIMITTEXT, SetScrollInfo},
            HiDpi::*,
            Input::HRAWINPUT,
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
    core::{PCWSTR, Result, w},
};

pub const IP: u16 = 101;
pub const PORT: u16 = 102;
pub const DEMO: u16 = 103;
pub const HALO: u16 = 110;
pub const TYRES: u16 = 120;
pub const MODE: u16 = 130;
pub const TEMP_MIN: u16 = 131;
pub const TEMP_MAX: u16 = 132;
pub const WEAR_WARN: u16 = 133;
pub const WEAR_CRIT: u16 = 134;
pub const FPS: u16 = 140;
pub const HOTKEY: u16 = 141;
pub const VISIBILITY_HOTKEY: u16 = 142;
const BATTERY_STATUS: u16 = 143;
pub const HALO_MODEL: u16 = 144;
pub const REAR_VIEW: u16 = 145;
pub const HALO_PROFILE: u16 = 146;
pub const COLOUR: u16 = 150;
pub const APPLY: u16 = 201;
pub const EDIT: u16 = 202;
pub const LAYOUT: u16 = 203;
pub const RESET: u16 = 204;
pub const EXPORT: u16 = 205;
pub const EXIT: u16 = 206;
pub const SHOW: u16 = 207;
pub const CYCLE: u16 = 208;
pub const TOGGLE_HALO: u16 = 209;
pub const TOGGLE_TYRES: u16 = 210;
pub const TOGGLE_DEMO: u16 = 211;
pub const TOGGLE_OVERLAYS: u16 = 212;
pub const SAVE_PROFILE: u16 = 213;
pub const LOAD_PROFILE: u16 = 214;
const DIAGNOSTICS: u16 = 300;
const RESULT: u16 = 301;
const TRAY_MESSAGE: u32 = WM_APP + 1;

fn layout_decimal(text: &str, current: f32, label: &str) -> std::result::Result<f32, String> {
    let text = text.trim().replace(',', ".");
    // Preserve drag/resize precision when the displayed decimal is unchanged.
    if text == format!("{current:.2}") {
        Ok(current)
    } else {
        text.parse()
            .map_err(|_| format!("{label}: numero non valido"))
    }
}

pub struct Panel {
    pub hwnd: HWND,
    controls: BTreeMap<u16, HWND>,
    font: HFONT,
    dpi: u32,
    tray: bool,
    children: Vec<(HWND, (i32, i32, i32, i32))>,
    scroll_x: i32,
    scroll_y: i32,
}

impl Panel {
    pub fn new() -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        let class = WNDCLASSW {
            lpfnWndProc: Some(panel_proc),
            hInstance: instance.into(),
            lpszClassName: w!("F1HaloSettings"),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            hIcon: unsafe { LoadIconW(None, IDI_APPLICATION)? },
            hbrBackground: HBRUSH((COLOR_BTNFACE.0 + 1) as *mut _),
            ..Default::default()
        };
        if unsafe { RegisterClassW(&class) } == 0 {
            return Err(windows::core::Error::from_thread());
        }
        let dpi = unsafe { GetDpiForSystem() };
        let factor = dpi as f32 / 96.0;
        let style =
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VSCROLL | WS_HSCROLL;
        let mut bounds = RECT {
            left: 0,
            top: 0,
            right: (818.0 * factor) as i32 + unsafe { GetSystemMetricsForDpi(SM_CXVSCROLL, dpi) },
            bottom: (804.0 * factor) as i32,
        };
        unsafe {
            AdjustWindowRectExForDpi(&mut bounds, style, false, WINDOW_EX_STYLE(0), dpi)?;
        }
        let work = overlay::work_area(POINT { x: 0, y: 0 });
        let width = (bounds.right - bounds.left).min(work.right - work.left - 20);
        let height = (bounds.bottom - bounds.top).min(work.bottom - work.top - 20);
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_CONTROLPARENT,
                w!("F1HaloSettings"),
                w!("F1 Halo Telemetry · Impostazioni"),
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                Some(instance.into()),
                None,
            )?
        };
        let font = unsafe {
            CreateFontW(
                -(14.0 * factor) as i32,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
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
        };
        let mut panel = Self {
            hwnd,
            controls: BTreeMap::new(),
            font,
            dpi,
            tray: false,
            children: Vec::new(),
            scroll_x: 0,
            scroll_y: 0,
        };
        panel.build()?;
        panel.reflow();
        panel.add_tray();
        Ok(panel)
    }

    fn control(
        &mut self,
        class: PCWSTR,
        title: &str,
        id: u16,
        bounds: (i32, i32, i32, i32),
        style: WINDOW_STYLE,
    ) -> Result<HWND> {
        let (x, y, width, height) = bounds;
        let factor = self.dpi as f32 / 96.0;
        let title = wide(title);
        let hwnd = unsafe {
            CreateWindowExW(
                if class == w!("EDIT") {
                    WS_EX_CLIENTEDGE
                } else {
                    WINDOW_EX_STYLE(0)
                },
                class,
                PCWSTR(title.as_ptr()),
                WS_CHILD | WS_VISIBLE | style,
                (x as f32 * factor) as i32,
                (y as f32 * factor) as i32,
                (width as f32 * factor) as i32,
                (height as f32 * factor) as i32,
                Some(self.hwnd),
                Some(HMENU(id as usize as *mut _)),
                Some(GetModuleHandleW(None)?.into()),
                None,
            )?
        };
        unsafe {
            SendMessageW(
                hwnd,
                WM_SETFONT,
                Some(WPARAM(self.font.0 as usize)),
                Some(LPARAM(1)),
            );
        }
        if id != 0 {
            self.controls.insert(id, hwnd);
        }
        self.children.push((hwnd, bounds));
        Ok(hwnd)
    }

    fn label(&mut self, title: &str, x: i32, y: i32, width: i32) -> Result<()> {
        self.control(w!("STATIC"), title, 0, (x, y, width, 21), WINDOW_STYLE(0))?;
        Ok(())
    }
    fn edit(&mut self, id: u16, x: i32, y: i32, width: i32) -> Result<()> {
        self.control(
            w!("EDIT"),
            "",
            id,
            (x, y, width, 25),
            WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
        )?;
        unsafe {
            SendMessageW(self.controls[&id], EM_SETLIMITTEXT, Some(WPARAM(128)), None);
        }
        Ok(())
    }
    fn button(
        &mut self,
        text: &str,
        id: u16,
        x: i32,
        y: i32,
        width: i32,
        check: bool,
    ) -> Result<()> {
        self.control(
            w!("BUTTON"),
            text,
            id,
            (x, y, width, 29),
            WS_TABSTOP
                | WINDOW_STYLE(if check {
                    BS_AUTOCHECKBOX as u32
                } else {
                    BS_PUSHBUTTON as u32
                }),
        )?;
        Ok(())
    }
    fn combo(&mut self, id: u16, x: i32, y: i32, width: i32, values: &[&str]) -> Result<()> {
        let hwnd = self.control(
            w!("COMBOBOX"),
            "",
            id,
            (x, y, width, 150),
            WS_TABSTOP | WS_VSCROLL | WINDOW_STYLE(CBS_DROPDOWNLIST as u32),
        )?;
        for value in values {
            let value = wide(value);
            unsafe {
                SendMessageW(
                    hwnd,
                    CB_ADDSTRING,
                    None,
                    Some(LPARAM(value.as_ptr() as isize)),
                );
            }
        }
        Ok(())
    }

    fn build(&mut self) -> Result<()> {
        self.label("F1 HALO TELEMETRY   /   2026 SEASON PACK", 20, 16, 770)?;
        self.label("UDP: IP locale", 20, 54, 110)?;
        self.edit(IP, 130, 50, 176)?;
        self.label("Porta", 326, 54, 45)?;
        self.edit(PORT, 375, 50, 80)?;
        self.button("Demo (dati simulati)", DEMO, 485, 49, 290, true)?;
        self.label(
            "LAYOUT   ·   posizioni in pixel, scala e opacità indipendenti",
            20,
            97,
            770,
        )?;
        for (label, x, width) in [
            ("HUD", 20, 150),
            ("X", 188, 80),
            ("Y", 288, 80),
            ("Scala", 388, 80),
            ("Opacità", 498, 90),
        ] {
            self.label(label, x, 127, width)?;
        }
        for (base, text, y) in [
            (HALO, "Halo visibile", 151),
            (TYRES, "Pneumatici visibili", 187),
        ] {
            self.button(text, base, 20, y - 2, 165, true)?;
            for (offset, x, width) in [(1, 188, 80), (2, 288, 80), (3, 388, 90), (4, 498, 90)] {
                self.edit(base + offset, x, y, width)?;
            }
        }
        self.button("Edit / Race", EDIT, 622, 150, 170, false)?;
        self.button("Ripristina layout", LAYOUT, 622, 186, 170, false)?;
        self.label("Modello Halo", 680, 217, 114)?;
        self.combo(
            HALO_MODEL,
            680,
            238,
            114,
            &HaloModel::ALL.map(HaloModel::label),
        )?;
        self.label("PNEUMATICI", 20, 236, 175)?;
        self.combo(MODE, 188, 232, 130, &["Surface", "Inner", "Both"])?;
        self.label("Temp. min / max °C", 337, 237, 150)?;
        self.edit(TEMP_MIN, 498, 232, 70)?;
        self.edit(TEMP_MAX, 582, 232, 70)?;
        self.label("Usura: attenzione / critica %", 20, 276, 270)?;
        self.edit(WEAR_WARN, 291, 271, 70)?;
        self.edit(WEAR_CRIT, 375, 271, 70)?;
        self.label("0% usura = pneumatico nuovo", 475, 276, 315)?;
        self.label("COLORI   ·   formato #RRGGBB", 20, 317, 360)?;
        for (i, (label, x, y)) in [
            ("Accento", 20, 350),
            ("Gas", 287, 350),
            ("Freno", 554, 350),
            ("Freddo", 20, 386),
            ("Ottimale", 287, 386),
            ("Caldo", 554, 386),
        ]
        .into_iter()
        .enumerate()
        {
            self.label(label, x, y + 4, 85)?;
            self.edit(COLOUR + i as u16, x + 90, y, 130)?;
        }
        self.label("FPS", 20, 435, 45)?;
        self.combo(FPS, 70, 430, 150, &["60 FPS", "120 FPS", "Refresh monitor"])?;
        self.label("Hotkey Edit", 250, 435, 100)?;
        self.edit(HOTKEY, 356, 430, 220)?;
        self.button("Nascondi Halo dietro", REAR_VIEW, 622, 430, 170, true)?;
        self.label("Hotkey visibilità HUD", 20, 475, 177)?;
        self.edit(VISIBILITY_HOTKEY, 203, 470, 220)?;
        self.label("Entrambi gli HUD", 438, 475, 165)?;
        self.button(
            "Mostra / nascondi HUD",
            TOGGLE_OVERLAYS,
            622,
            470,
            170,
            false,
        )?;
        self.control(
            w!("STATIC"),
            "Capacità ERS automatica: in attesa dei dati vettura",
            BATTERY_STATUS,
            (20, 499, 775, 16),
            WINDOW_STYLE(0),
        )?;
        self.control(
            w!("STATIC"),
            "Inizializzazione…",
            DIAGNOSTICS,
            (20, 515, 775, 112),
            WINDOW_STYLE(0),
        )?;
        self.control(
            w!("STATIC"),
            "",
            RESULT,
            (20, 704, 775, 30),
            WINDOW_STYLE(0),
        )?;
        self.label("Profilo Halo", 20, 638, 120)?;
        let profile = self.control(
            w!("COMBOBOX"),
            "",
            HALO_PROFILE,
            (145, 632, 285, 180),
            WS_TABSTOP | WS_VSCROLL | WINDOW_STYLE((CBS_DROPDOWN | CBS_AUTOHSCROLL) as u32),
        )?;
        unsafe {
            SendMessageW(profile, CB_LIMITTEXT, Some(WPARAM(64)), None);
        }
        self.button("Carica profilo", LOAD_PROFILE, 452, 632, 150, false)?;
        self.button("Salva profilo", SAVE_PROFILE, 614, 632, 180, false)?;
        self.label(
            "Nome libero, es. Ferrari. Salva aggiorna posizione e scala; Carica le ripristina.",
            20,
            672,
            775,
        )?;
        for (text, id, x, width) in [
            ("Applica e salva", APPLY, 20, 151),
            ("Esporta diagnostica", EXPORT, 183, 174),
            ("Impostazioni iniziali", RESET, 369, 178),
            ("Chiudi pannello", SHOW, 559, 145),
            ("Esci", EXIT, 716, 78),
        ] {
            self.button(text, id, x, 752, width, false)?;
        }
        Ok(())
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = ShowWindow(self.hwnd, SW_RESTORE);
            let _ = SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
            let _ = SetForegroundWindow(self.hwnd);
        }
    }
    pub fn hide(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_HIDE);
        }
    }
    pub fn visible(&self) -> bool {
        unsafe { IsWindowVisible(self.hwnd).as_bool() }
    }

    pub fn set(&self, id: u16, text: &str) {
        let text = wide(text);
        unsafe {
            let _ = SetWindowTextW(self.controls[&id], PCWSTR(text.as_ptr()));
        }
    }
    pub fn text(&self, id: u16) -> String {
        let mut buffer = [0u16; 256];
        let n = unsafe { GetWindowTextW(self.controls[&id], &mut buffer) };
        String::from_utf16_lossy(&buffer[..n.max(0) as usize])
    }
    pub fn check(&self, id: u16, checked: bool) {
        unsafe {
            SendMessageW(
                self.controls[&id],
                BM_SETCHECK,
                Some(WPARAM(checked as usize)),
                None,
            );
        }
    }
    pub fn checked(&self, id: u16) -> bool {
        unsafe { SendMessageW(self.controls[&id], BM_GETCHECK, None, None).0 == 1 }
    }
    fn select(&self, id: u16, index: usize) {
        unsafe {
            SendMessageW(self.controls[&id], CB_SETCURSEL, Some(WPARAM(index)), None);
        }
    }
    fn selected(&self, id: u16) -> isize {
        unsafe { SendMessageW(self.controls[&id], CB_GETCURSEL, None, None).0 }
    }
    pub fn populate(&self, s: &Settings, demo: bool) {
        self.set(IP, &s.bind_ip);
        self.set(PORT, &s.port.to_string());
        self.check(DEMO, demo);
        self.check(REAR_VIEW, s.hide_halo_rear_view);
        self.populate_layout(s);
        self.populate_profiles(s);
        self.select(
            HALO_MODEL,
            match s.halo_model {
                HaloModel::Wide => 0,
                HaloModel::Classic => 1,
            },
        );
        self.select(
            MODE,
            match s.tyre_mode {
                TyreMode::Surface => 0,
                TyreMode::Inner => 1,
                TyreMode::Both => 2,
            },
        );
        for (id, value) in [
            (TEMP_MIN, s.temperature_min_c),
            (TEMP_MAX, s.temperature_max_c),
            (WEAR_WARN, s.wear_warning_percent),
            (WEAR_CRIT, s.wear_critical_percent),
        ] {
            self.set(id, &value.to_string());
        }
        for (i, rgb) in [
            s.colours.accent,
            s.colours.throttle,
            s.colours.brake,
            s.colours.cold,
            s.colours.optimal,
            s.colours.hot,
        ]
        .into_iter()
        .enumerate()
        {
            self.set(
                COLOUR + i as u16,
                &format!("#{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]),
            );
        }
        self.select(
            FPS,
            match s.fps {
                FpsLimit::Fps60 => 0,
                FpsLimit::Fps120 => 1,
                FpsLimit::Adaptive => 2,
            },
        );
        self.set(HOTKEY, &s.hotkey);
        self.set(VISIBILITY_HOTKEY, &s.visibility_hotkey);
    }

    pub fn populate_layout(&self, s: &Settings) {
        for (base, layout) in [(HALO, s.halo), (TYRES, s.tyres)] {
            self.populate_one_layout(base, layout);
        }
    }

    fn populate_one_layout(&self, base: u16, layout: Layout) {
        self.check(base, layout.visible);
        self.set(base + 1, &layout.x.to_string());
        self.set(base + 2, &layout.y.to_string());
        self.set(base + 3, &format!("{:.2}", layout.scale));
        self.set(base + 4, &format!("{:.2}", layout.opacity));
    }

    pub fn populate_halo_geometry(&self, s: &Settings) {
        self.set(HALO + 1, &s.halo.x.to_string());
        self.set(HALO + 2, &s.halo.y.to_string());
        self.set(HALO + 3, &format!("{:.2}", s.halo.scale));
    }

    pub fn populate_profiles(&self, s: &Settings) {
        let combo = self.controls[&HALO_PROFILE];
        unsafe {
            SendMessageW(combo, CB_RESETCONTENT, None, None);
            for name in s.halo_profiles.keys() {
                let name = wide(name);
                SendMessageW(
                    combo,
                    CB_ADDSTRING,
                    None,
                    Some(LPARAM(name.as_ptr() as isize)),
                );
            }
        }
        self.set(HALO_PROFILE, s.active_halo_profile.as_deref().unwrap_or(""));
    }

    pub fn halo_profile(&self, current: Layout) -> std::result::Result<HaloProfile, String> {
        Ok(HaloProfile {
            x: self
                .text(HALO + 1)
                .trim()
                .parse()
                .map_err(|_| "X Halo: intero non valido")?,
            y: self
                .text(HALO + 2)
                .trim()
                .parse()
                .map_err(|_| "Y Halo: intero non valido")?,
            scale: layout_decimal(&self.text(HALO + 3), current.scale, "Scala Halo")?,
        })
    }

    pub fn read(&self, old: &Settings) -> std::result::Result<Settings, String> {
        let number = |id, name: &str| -> std::result::Result<f32, String> {
            self.text(id)
                .trim()
                .replace(',', ".")
                .parse()
                .map_err(|_| format!("{name}: numero non valido"))
        };
        let integer = |id, name: &str| -> std::result::Result<i32, String> {
            self.text(id)
                .trim()
                .parse()
                .map_err(|_| format!("{name}: intero non valido"))
        };
        let mut s = old.clone();
        s.hide_halo_rear_view = self.checked(REAR_VIEW);
        s.halo_model = match self.selected(HALO_MODEL) {
            0 => HaloModel::Wide,
            1 => HaloModel::Classic,
            _ => return Err("Seleziona un modello Halo.".into()),
        };
        s.bind_ip = self.text(IP).trim().into();
        s.port = self
            .text(PORT)
            .trim()
            .parse()
            .map_err(|_| "Porta UDP non valida")?;
        for (base, layout) in [(HALO, &mut s.halo), (TYRES, &mut s.tyres)] {
            layout.visible = self.checked(base);
            layout.x = integer(base + 1, "X")?;
            layout.y = integer(base + 2, "Y")?;
            layout.scale = layout_decimal(&self.text(base + 3), layout.scale, "Scala")?;
            layout.opacity = layout_decimal(&self.text(base + 4), layout.opacity, "Opacità")?;
        }
        s.tyre_mode = match self.selected(MODE) {
            0 => TyreMode::Surface,
            1 => TyreMode::Inner,
            _ => TyreMode::Both,
        };
        s.temperature_min_c = number(TEMP_MIN, "Temperatura minima")?;
        s.temperature_max_c = number(TEMP_MAX, "Temperatura massima")?;
        s.wear_warning_percent = number(WEAR_WARN, "Usura attenzione")?;
        s.wear_critical_percent = number(WEAR_CRIT, "Usura critica")?;
        for (i, rgb) in [
            &mut s.colours.accent,
            &mut s.colours.throttle,
            &mut s.colours.brake,
            &mut s.colours.cold,
            &mut s.colours.optimal,
            &mut s.colours.hot,
        ]
        .into_iter()
        .enumerate()
        {
            let text = self.text(COLOUR + i as u16);
            let value = text.trim().trim_start_matches('#');
            if value.len() != 6 || !value.is_ascii() {
                return Err("Colori: usare #RRGGBB".into());
            }
            for k in 0..3 {
                rgb[k] = u8::from_str_radix(&value[k * 2..k * 2 + 2], 16)
                    .map_err(|_| "Colore esadecimale non valido")?;
            }
        }
        s.fps = match self.selected(FPS) {
            1 => FpsLimit::Fps120,
            2 => FpsLimit::Adaptive,
            _ => FpsLimit::Fps60,
        };
        s.hotkey = self.text(HOTKEY).trim().into();
        s.visibility_hotkey = self.text(VISIBILITY_HOTKEY).trim().into();
        s.validate()?;
        Ok(s)
    }

    pub fn result(&self, text: &str) {
        self.set(RESULT, text);
    }
    pub fn diagnostics(&self, text: &str) {
        if self.visible() {
            self.set(DIAGNOSTICS, text);
        }
    }
    pub fn battery_status(&self, text: &str) {
        self.set(BATTERY_STATUS, text);
    }

    fn reflow(&mut self) {
        let factor = self.dpi as f32 / 96.0;
        for _ in 0..2 {
            let mut client = RECT::default();
            unsafe {
                let _ = GetClientRect(self.hwnd, &mut client);
            }
            for (bar, full, page, offset) in [
                (
                    SB_HORZ,
                    (818.0 * factor) as i32,
                    client.right,
                    &mut self.scroll_x,
                ),
                (
                    SB_VERT,
                    (804.0 * factor) as i32,
                    client.bottom,
                    &mut self.scroll_y,
                ),
            ] {
                *offset = (*offset).clamp(0, (full - page).max(0));
                let info = SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                    nMin: 0,
                    nMax: full - 1,
                    nPage: page.max(0) as u32,
                    nPos: *offset,
                    ..Default::default()
                };
                unsafe {
                    SetScrollInfo(self.hwnd, bar, &info, true);
                }
            }
        }
        for &(hwnd, (x, y, width, height)) in &self.children {
            unsafe {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    (x as f32 * factor) as i32 - self.scroll_x,
                    (y as f32 * factor) as i32 - self.scroll_y,
                    (width as f32 * factor) as i32,
                    (height as f32 * factor) as i32,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
            }
        }
    }

    pub fn change_dpi(&mut self, dpi: u32, suggested: RECT) {
        self.dpi = dpi.max(96);
        let factor = self.dpi as f32 / 96.0;
        let old = self.font;
        self.font = unsafe {
            CreateFontW(
                -(14.0 * factor) as i32,
                0,
                0,
                0,
                FW_NORMAL.0 as i32,
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
        };
        let work = overlay::work_area(POINT {
            x: suggested.left,
            y: suggested.top,
        });
        let width = (suggested.right - suggested.left).min(work.right - work.left);
        let height = (suggested.bottom - suggested.top).min(work.bottom - work.top);
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                suggested.left.clamp(work.left, work.right - width),
                suggested.top.clamp(work.top, work.bottom - height),
                width,
                height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            for &(hwnd, _) in &self.children {
                SendMessageW(
                    hwnd,
                    WM_SETFONT,
                    Some(WPARAM(self.font.0 as usize)),
                    Some(LPARAM(1)),
                );
            }
            let _ = DeleteObject(old.into());
        }
        self.reflow();
    }

    pub fn scroll(&mut self, vertical: bool, code: u16, position: u16) {
        let mut client = RECT::default();
        unsafe {
            let _ = GetClientRect(self.hwnd, &mut client);
        }
        let page = if vertical {
            client.bottom
        } else {
            client.right
        };
        let offset = if vertical {
            &mut self.scroll_y
        } else {
            &mut self.scroll_x
        };
        let line = (32 * self.dpi / 96) as i32;
        *offset = match code as i32 {
            0 => *offset - line,
            1 => *offset + line,
            2 => *offset - page,
            3 => *offset + page,
            4 | 5 => position as i32,
            6 => 0,
            7 => i32::MAX,
            _ => *offset,
        };
        self.reflow();
    }

    pub fn context_menu(&self, edit: bool, demo: bool, overlays_visible: bool) {
        unsafe {
            let Ok(menu) = CreatePopupMenu() else {
                return;
            };
            for (id, title, checked) in [
                (SHOW, "Impostazioni", false),
                (EDIT, "Edit Mode", edit),
                (TOGGLE_OVERLAYS, "Mostra entrambi gli HUD", overlays_visible),
                (TOGGLE_HALO, "Mostra / nascondi Halo", false),
                (TOGGLE_TYRES, "Mostra / nascondi pneumatici", false),
                (CYCLE, "Cambia modalità temperature", false),
                (TOGGLE_DEMO, "Demo", demo),
                (EXIT, "Esci", false),
            ] {
                let text = wide(title);
                let _ = AppendMenuW(
                    menu,
                    MF_STRING | if checked { MF_CHECKED } else { MF_UNCHECKED },
                    id as usize,
                    PCWSTR(text.as_ptr()),
                );
            }
            let mut point = POINT::default();
            let _ = GetCursorPos(&mut point);
            let _ = SetForegroundWindow(self.hwnd);
            let selected = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                point.x,
                point.y,
                None,
                self.hwnd,
                None,
            );
            let _ = DestroyMenu(menu);
            if selected.0 != 0 {
                overlay::push(Action::Command(selected.0 as u16));
            }
        }
    }

    fn add_tray(&mut self) {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: TRAY_MESSAGE,
            hIcon: unsafe { LoadIconW(None, IDI_APPLICATION).unwrap_or_default() },
            ..Default::default()
        };
        let text = wide("F1 Halo Telemetry · doppio clic: impostazioni");
        data.szTip[..text.len()].copy_from_slice(&text);
        self.tray = unsafe { Shell_NotifyIconW(NIM_ADD, &data).as_bool() };
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        unsafe {
            if self.tray {
                let data = NOTIFYICONDATAW {
                    cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                    hWnd: self.hwnd,
                    uID: 1,
                    ..Default::default()
                };
                let _ = Shell_NotifyIconW(NIM_DELETE, &data);
            }
            let _ = DestroyWindow(self.hwnd);
            let _ = DeleteObject(self.font.into());
        }
    }
}

unsafe extern "system" fn panel_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_INPUT => {
            if let Some((device, down)) = rear_view::read_raw(HRAWINPUT(lparam.0 as *mut _)) {
                overlay::push(Action::ControllerAxis(device, down));
            }
        }
        WM_INPUT_DEVICE_CHANGE => {
            let device = lparam.0 as usize;
            rear_view::forget_device(device);
            if wparam.0 == GIDC_REMOVAL as usize {
                overlay::push(Action::ControllerRemoved(device));
            }
        }
        WM_COMMAND if (wparam.0 >> 16) == BN_CLICKED as usize => {
            overlay::push(Action::Command((wparam.0 & 0xffff) as u16));
            return LRESULT(0);
        }
        WM_HOTKEY => {
            overlay::push(Action::Hotkey(wparam.0 as i32));
            return LRESULT(0);
        }
        WM_CLOSE => {
            overlay::push(Action::ClosePanel);
            return LRESULT(0);
        }
        WM_DISPLAYCHANGE => overlay::push(Action::DisplayChanged),
        WM_DPICHANGED => {
            let rect = unsafe { *(lparam.0 as *const RECT) };
            overlay::push(Action::PanelDpi((wparam.0 & 0xffff) as u32, rect));
            return LRESULT(0);
        }
        WM_VSCROLL | WM_HSCROLL => {
            overlay::push(Action::PanelScroll(
                message == WM_VSCROLL,
                (wparam.0 & 0xffff) as u16,
                (wparam.0 >> 16) as u16,
            ));
            return LRESULT(0);
        }
        WM_MOUSEWHEEL => {
            overlay::push(Action::PanelWheel((wparam.0 >> 16) as i16));
            return LRESULT(0);
        }
        TRAY_MESSAGE => {
            match lparam.0 as u32 {
                WM_LBUTTONDBLCLK => overlay::push(Action::Command(SHOW)),
                WM_RBUTTONUP => overlay::push(Action::ContextMenu),
                _ => (),
            }
            return LRESULT(0);
        }
        _ => (),
    }
    unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::layout_decimal;

    #[test]
    fn choosing_a_model_preserves_unedited_mouse_scale_and_opacity_precision() {
        assert_eq!(
            layout_decimal("1.04", 1.036_904_8, "Scala"),
            Ok(1.036_904_8)
        );
        assert_eq!(
            layout_decimal(" 1,04 ", 1.036_904_8, "Scala"),
            Ok(1.036_904_8)
        );
        assert_eq!(layout_decimal("0.94", 0.938_7, "Opacità"), Ok(0.938_7));
        assert_eq!(layout_decimal("1.00", 1.036_904_8, "Scala"), Ok(1.0));
        assert_eq!(layout_decimal("1.20", 1.036_904_8, "Scala"), Ok(1.2));
        assert!(layout_decimal("invalid", 1.0, "Scala").is_err());
    }
}
