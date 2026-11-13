mod layout;
mod theme;

use crate::{
    branding::Icon,
    input::rear_view,
    overlay::{self, Action, wide},
    settings::*,
};
use layout::{Bounds, ResponsiveLayout};
use std::{collections::BTreeMap, mem::size_of};
use theme::{Page, Theme};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{
                EM_GETFIRSTVISIBLELINE, EM_LINESCROLL, EM_SETLIMITTEXT, SetScrollInfo,
                SetWindowTheme,
            },
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
const CONNECTION_STATUS: u16 = 302;
const RUNTIME_STATUS: u16 = 303;
const TRAY_MESSAGE: u32 = WM_APP + 1;
type PositionedControl = (HWND, Bounds, Option<Page>);

fn layout_decimal(text: &str, current: f32, label: &str) -> std::result::Result<f32, String> {
    let text = text.trim().replace(',', ".");
    // Preserve drag/resize precision when the displayed decimal is unchanged.
    if text == format!("{current:.2}") {
        Ok(current)
    } else {
        text.parse().map_err(|_| format!("{label}: invalid number"))
    }
}

fn opacity_percent(text: &str, current: f32) -> std::result::Result<f32, String> {
    let percent: f32 = text
        .trim()
        .replace(',', ".")
        .parse()
        .map_err(|_| "Background: enter a percentage from 0 to 100")?;
    if percent == (current * 100.0).round() {
        Ok(current)
    } else {
        Ok(percent / 100.0)
    }
}

pub struct Panel {
    pub hwnd: HWND,
    controls: BTreeMap<u16, HWND>,
    theme: Box<Theme>,
    dpi: u32,
    tray: bool,
    icons: [Icon; 3],
    building_page: Option<Page>,
}

impl Panel {
    pub fn new() -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        let mut cursor = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut cursor);
        }
        let dpi = overlay::monitor_dpi(cursor);
        let icons = [
            Icon::load(unsafe { GetSystemMetricsForDpi(SM_CXICON, dpi) })?,
            Icon::load(unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) })?,
            Icon::load(256)?,
        ];
        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(panel_proc),
            hInstance: instance.into(),
            lpszClassName: w!("F1HaloSettings"),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            hIcon: icons[0].0,
            hIconSm: icons[1].0,
            ..Default::default()
        };
        if unsafe { RegisterClassExW(&class) } == 0 {
            return Err(windows::core::Error::from_thread());
        }
        let factor = dpi as f32 / 96.0;
        let style = WS_OVERLAPPEDWINDOW | WS_VSCROLL | WS_HSCROLL | WS_CLIPCHILDREN;
        let mut bounds = RECT {
            left: 0,
            top: 0,
            right: (theme::WIDTH as f32 * factor) as i32
                + unsafe { GetSystemMetricsForDpi(SM_CXVSCROLL, dpi) },
            bottom: (theme::HEIGHT as f32 * factor) as i32,
        };
        unsafe {
            AdjustWindowRectExForDpi(&mut bounds, style, false, WINDOW_EX_STYLE(0), dpi)?;
        }
        let work = overlay::work_area(cursor);
        let margin = (24.0 * factor) as i32;
        let width = (bounds.right - bounds.left).min(work.right - work.left - margin);
        let frame_width = bounds.right - bounds.left - (theme::WIDTH as f32 * factor) as i32;
        let width = width.max(((360.0 * factor) as i32 + frame_width).min(work.right - work.left));
        let frame_height = bounds.bottom - bounds.top - (theme::HEIGHT as f32 * factor) as i32;
        let preferred = ResponsiveLayout::new(
            ((width - frame_width) as f32 / factor) as i32,
            theme::HEIGHT,
            Page::Layout,
        );
        let height = ((preferred.content_height as f32 * factor) as i32 + frame_height)
            .min(work.bottom - work.top - margin);
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_CONTROLPARENT,
                w!("F1HaloSettings"),
                w!("F1 Halo Telemetry · Settings"),
                style,
                work.left + (work.right - work.left - width) / 2,
                work.top + (work.bottom - work.top - height) / 2,
                width,
                height,
                None,
                None,
                Some(instance.into()),
                None,
            )?
        };
        let mut panel = Self {
            hwnd,
            controls: BTreeMap::new(),
            theme: Box::new(Theme::new(dpi, icons[2].0)),
            dpi,
            tray: false,
            icons,
            building_page: None,
        };
        panel.build()?;
        panel.theme.attach(hwnd);
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
                WINDOW_EX_STYLE(0),
                class,
                PCWSTR(title.as_ptr()),
                WS_CHILD
                    | WS_VISIBLE
                    | style
                    | if class == w!("COMBOBOX") {
                        WINDOW_STYLE((CBS_OWNERDRAWFIXED | CBS_HASSTRINGS) as u32)
                    } else {
                        WINDOW_STYLE(0)
                    },
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
                Some(WPARAM(self.theme.font().0 as usize)),
                Some(LPARAM(1)),
            );
            if class != w!("STATIC") {
                let _ = SetWindowTheme(hwnd, w!(""), w!(""));
                let kind = if class == w!("BUTTON") {
                    1
                } else if class == w!("EDIT") {
                    2
                } else {
                    3
                };
                if !SetWindowSubclass(hwnd, Some(theme::control_proc), 1, kind).as_bool() {
                    return Err(windows::core::Error::from_thread());
                }
                if class == w!("COMBOBOX") {
                    SendMessageW(
                        hwnd,
                        CB_SETITEMHEIGHT,
                        Some(WPARAM(usize::MAX)),
                        Some(LPARAM((30.0 * factor) as isize)),
                    );
                    SendMessageW(
                        hwnd,
                        CB_SETITEMHEIGHT,
                        Some(WPARAM(0)),
                        Some(LPARAM((30.0 * factor) as isize)),
                    );
                }
            }
        }
        if id != 0 {
            self.controls.insert(id, hwnd);
        }
        self.theme.children.push((hwnd, bounds, self.building_page));
        Ok(hwnd)
    }

    fn label(&mut self, title: &str, x: i32, y: i32, width: i32) -> Result<()> {
        self.control(w!("STATIC"), title, 0, (x, y, width, 21), WINDOW_STYLE(0))?;
        Ok(())
    }
    fn edit(&mut self, id: u16, x: i32, y: i32, width: i32) -> Result<()> {
        let hwnd = self.control(
            w!("EDIT"),
            "",
            id,
            (x + 10, y + 8, width - 20, 21),
            WS_TABSTOP | WINDOW_STYLE(ES_AUTOHSCROLL as u32),
        )?;
        self.theme
            .inputs
            .push((hwnd, (x, y, width, 36), self.building_page));
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
            (x, y, width, 38),
            WS_TABSTOP
                | WINDOW_STYLE(if check {
                    BS_AUTOCHECKBOX as u32
                } else {
                    BS_OWNERDRAW as u32
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
        for (i, page) in Page::ALL.into_iter().enumerate() {
            self.control(
                w!("BUTTON"),
                page.title(),
                theme::NAV + i as u16,
                (16, 148 + i as i32 * 54, 184, 46),
                WS_TABSTOP | WINDOW_STYLE(BS_OWNERDRAW as u32),
            )?;
        }
        self.button("Exit application", EXIT, 22, 724, 172, false)?;
        self.button("Edit / Race", EDIT, 916, 210, 114, false)?;
        self.button("Toggle HUDs", TOGGLE_OVERLAYS, 1042, 210, 114, false)?;
        self.combo(
            FPS,
            916,
            292,
            240,
            &["60 FPS", "120 FPS", "Monitor refresh"],
        )?;
        self.button("Demo (simulated data)", DEMO, 916, 399, 240, true)?;
        self.button("Hide Halo: rear / sides", REAR_VIEW, 916, 448, 240, true)?;
        self.control(
            w!("STATIC"),
            "Waiting for UDP",
            CONNECTION_STATUS,
            (916, 611, 240, 32),
            WINDOW_STYLE(0),
        )?;
        self.control(
            w!("STATIC"),
            "",
            RUNTIME_STATUS,
            (916, 651, 240, 40),
            WINDOW_STYLE(0),
        )?;
        self.control(
            w!("STATIC"),
            "",
            RESULT,
            (244, 654, 620, 40),
            WINDOW_STYLE(0),
        )?;
        for (text, id, x, width) in [
            ("Apply and save", APPLY, 244, 174),
            ("Default settings", RESET, 434, 158),
            ("Close panel", SHOW, 608, 146),
            ("Export diagnostics", EXPORT, 916, 240),
        ] {
            self.button(text, id, x, 724, width, false)?;
        }

        self.building_page = Some(Page::Layout);
        self.button("Reset layout", LAYOUT, 738, 111, 126, false)?;
        for (base, text, x) in [(HALO, "Show Halo", 264), (TYRES, "Show tyres", 584)] {
            self.button(text, base, x, 213, 260, true)?;
            for (label, offset, dx, y) in [
                ("Position X", 1, 0, 266),
                ("Position Y", 2, 138, 266),
                ("Scale", 3, 0, 350),
                ("Background %", 4, 138, 350),
            ] {
                self.label(label, x + dx, y, 122)?;
                self.edit(base + offset, x + dx, y + 24, 122)?;
            }
        }
        self.label("Halo model", 264, 477, 220)?;
        self.combo(
            HALO_MODEL,
            504,
            470,
            340,
            &HaloModel::ALL.map(HaloModel::label),
        )?;
        self.label("Halo profile", 264, 555, 270)?;
        let profile = self.control(
            w!("COMBOBOX"),
            "",
            HALO_PROFILE,
            (264, 585, 270, 180),
            WS_TABSTOP | WS_VSCROLL | WINDOW_STYLE((CBS_DROPDOWN | CBS_AUTOHSCROLL) as u32),
        )?;
        unsafe {
            SendMessageW(profile, CB_LIMITTEXT, Some(WPARAM(64)), None);
        }
        self.button("Load profile", LOAD_PROFILE, 548, 585, 132, false)?;
        self.button("Save profile", SAVE_PROFILE, 692, 585, 152, false)?;

        self.building_page = Some(Page::Tyres);
        self.label("Temperature source", 264, 258, 226)?;
        self.combo(MODE, 504, 250, 340, &["Surface", "Inner", "Both"])?;
        for (title, id, x, y) in [
            ("Minimum temperature", TEMP_MIN, 264, 392),
            ("Maximum temperature", TEMP_MAX, 264, 475),
            ("Warning at", WEAR_WARN, 584, 392),
            ("Critical at", WEAR_CRIT, 584, 475),
        ] {
            self.label(title, x, y, 260)?;
            self.edit(id, x, y + 25, 260)?;
        }

        self.building_page = Some(Page::Colours);
        for i in 0..6 {
            let x = 244 + (i % 2) * 320;
            let y = 196 + (i / 2) * 146;
            self.edit(COLOUR + i as u16, x + 68, y + 80, 212)?;
        }

        self.building_page = Some(Page::Connection);
        self.label("Local listening IP", 264, 254, 352)?;
        self.edit(IP, 264, 282, 352)?;
        self.label("UDP port", 640, 254, 204)?;
        self.edit(PORT, 640, 282, 204)?;

        self.building_page = Some(Page::Shortcuts);
        self.label(
            "Switch between driving and positioning your HUDs.",
            264,
            251,
            580,
        )?;
        self.edit(HOTKEY, 264, 291, 580)?;
        self.label(
            "Show or hide both HUDs. Controller: double-press View / Share / Create.",
            264,
            437,
            580,
        )?;
        self.edit(VISIBILITY_HOTKEY, 264, 477, 580)?;

        self.building_page = Some(Page::Diagnostics);
        self.control(
            w!("STATIC"),
            "Waiting for compatible car data.",
            BATTERY_STATUS,
            (264, 252, 580, 64),
            WINDOW_STYLE(0),
        )?;
        self.control(
            w!("EDIT"),
            "Initializing…",
            DIAGNOSTICS,
            (264, 410, 580, 210),
            WS_TABSTOP
                | WS_VSCROLL
                | WINDOW_STYLE((ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL) as u32),
        )?;
        self.building_page = None;
        Ok(())
    }

    pub fn navigate(&mut self, command: u16) -> bool {
        let Some(page) = command
            .checked_sub(theme::NAV)
            .and_then(|index| Page::ALL.get(index as usize))
            .copied()
        else {
            return false;
        };
        self.theme.page.set(page);
        unsafe {
            SetScrollInfo(
                self.hwnd,
                SB_VERT,
                &SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_POS,
                    nPos: 0,
                    ..Default::default()
                },
                false,
            );
        }
        self.reflow();
        true
    }

    pub fn show(&self) {
        unsafe {
            let _ = ShowWindow(self.hwnd, SW_SHOW);
            let _ = ShowWindow(
                self.hwnd,
                if IsIconic(self.hwnd).as_bool() {
                    SW_RESTORE
                } else {
                    SW_SHOW
                },
            );
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
            let _ = InvalidateRect(Some(self.controls[&id]), None, false);
            if matches!(id, HALO | TYRES) {
                let _ = InvalidateRect(Some(self.hwnd), None, false);
            }
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
                HaloModel::Modern => 2,
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
        self.set(base + 4, &format!("{:.0}", layout.opacity * 100.0));
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
                .map_err(|_| "Halo X: invalid integer")?,
            y: self
                .text(HALO + 2)
                .trim()
                .parse()
                .map_err(|_| "Halo Y: invalid integer")?,
            scale: layout_decimal(&self.text(HALO + 3), current.scale, "Halo scale")?,
        })
    }

    pub fn read(&self, old: &Settings) -> std::result::Result<Settings, String> {
        let number = |id, name: &str| -> std::result::Result<f32, String> {
            self.text(id)
                .trim()
                .replace(',', ".")
                .parse()
                .map_err(|_| format!("{name}: invalid number"))
        };
        let integer = |id, name: &str| -> std::result::Result<i32, String> {
            self.text(id)
                .trim()
                .parse()
                .map_err(|_| format!("{name}: invalid integer"))
        };
        let mut s = old.clone();
        s.hide_halo_rear_view = self.checked(REAR_VIEW);
        s.halo_model = match self.selected(HALO_MODEL) {
            0 => HaloModel::Wide,
            1 => HaloModel::Classic,
            2 => HaloModel::Modern,
            _ => return Err("Select a Halo model.".into()),
        };
        s.bind_ip = self.text(IP).trim().into();
        s.port = self
            .text(PORT)
            .trim()
            .parse()
            .map_err(|_| "Invalid UDP port")?;
        for (base, layout) in [(HALO, &mut s.halo), (TYRES, &mut s.tyres)] {
            layout.visible = self.checked(base);
            layout.x = integer(base + 1, "X")?;
            layout.y = integer(base + 2, "Y")?;
            layout.scale = layout_decimal(&self.text(base + 3), layout.scale, "Scale")?;
            layout.opacity = opacity_percent(&self.text(base + 4), layout.opacity)?;
        }
        s.tyre_mode = match self.selected(MODE) {
            0 => TyreMode::Surface,
            1 => TyreMode::Inner,
            _ => TyreMode::Both,
        };
        s.temperature_min_c = number(TEMP_MIN, "Minimum temperature")?;
        s.temperature_max_c = number(TEMP_MAX, "Maximum temperature")?;
        s.wear_warning_percent = number(WEAR_WARN, "Wear warning")?;
        s.wear_critical_percent = number(WEAR_CRIT, "Critical wear")?;
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
                return Err("Colours: use #RRGGBB".into());
            }
            for k in 0..3 {
                rgb[k] = u8::from_str_radix(&value[k * 2..k * 2 + 2], 16)
                    .map_err(|_| "Invalid hexadecimal colour")?;
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
            let control = self.controls[&DIAGNOSTICS];
            let first = unsafe { SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None).0 };
            self.set(DIAGNOSTICS, text);
            let after = unsafe { SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None).0 };
            if first != after {
                unsafe {
                    SendMessageW(control, EM_LINESCROLL, None, Some(LPARAM(first - after)));
                }
            }
        }
    }
    pub fn battery_status(&self, text: &str) {
        self.set(BATTERY_STATUS, text);
    }

    pub fn session_status(&self, connection: &str, cpu: f64, memory: u64) {
        self.set(CONNECTION_STATUS, connection);
        self.set(
            RUNTIME_STATUS,
            &format!(
                "CPU {cpu:.1}%  ·  RAM {:.1} MB",
                memory as f64 / 1_048_576.0
            ),
        );
    }

    fn reflow(&self) {
        self.theme.resize(self.hwnd);
    }

    pub fn change_dpi(&mut self, dpi: u32, suggested: RECT) {
        self.dpi = dpi.max(96);
        let factor = self.dpi as f32 / 96.0;
        let mut theme = Box::new(Theme::new(self.dpi, self.icons[2].0));
        theme.page.set(self.theme.page.get());
        theme.inputs.clone_from(&self.theme.inputs);
        theme.children.clone_from(&self.theme.children);
        theme.layout.set(self.theme.layout.get());
        theme.attach(self.hwnd);
        let old = std::mem::replace(&mut self.theme, theme);
        let work = overlay::work_area(POINT {
            x: suggested.left,
            y: suggested.top,
        });
        let maximized = unsafe { IsZoomed(self.hwnd).as_bool() };
        let width = if maximized {
            suggested.right - suggested.left
        } else {
            (suggested.right - suggested.left).min(work.right - work.left)
        };
        let height = if maximized {
            suggested.bottom - suggested.top
        } else {
            (suggested.bottom - suggested.top).min(work.bottom - work.top)
        };
        unsafe {
            let _ = SetWindowPos(
                self.hwnd,
                None,
                if maximized {
                    suggested.left
                } else {
                    suggested.left.clamp(work.left, work.right - width)
                },
                if maximized {
                    suggested.top
                } else {
                    suggested.top.clamp(work.top, work.bottom - height)
                },
                width,
                height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            );
            for &(hwnd, _, _) in &self.theme.children {
                SendMessageW(
                    hwnd,
                    WM_SETFONT,
                    Some(WPARAM(self.theme.font().0 as usize)),
                    Some(LPARAM(1)),
                );
                let mut class = [0u16; 32];
                let n = GetClassNameW(hwnd, &mut class);
                if String::from_utf16_lossy(&class[..n.max(0) as usize]) == "ComboBox" {
                    let height = (30.0 * factor) as isize;
                    SendMessageW(
                        hwnd,
                        CB_SETITEMHEIGHT,
                        Some(WPARAM(usize::MAX)),
                        Some(LPARAM(height)),
                    );
                    SendMessageW(
                        hwnd,
                        CB_SETITEMHEIGHT,
                        Some(WPARAM(0)),
                        Some(LPARAM(height)),
                    );
                }
            }
        }
        drop(old);
        self.reflow();
    }

    pub fn scroll(&self, vertical: bool, code: u16, position: u16) {
        unsafe {
            let mut client = RECT::default();
            let _ = GetClientRect(self.hwnd, &mut client);
            let bar = if vertical { SB_VERT } else { SB_HORZ };
            let page = if vertical {
                client.bottom
            } else {
                client.right
            };
            let current = GetScrollPos(self.hwnd, bar);
            let line = (32 * self.dpi / 96) as i32;
            let next = match code {
                0 => current - line,
                1 => current + line,
                2 => current - page,
                3 => current + page,
                4 | 5 => position as i32,
                6 => 0,
                7 => i32::MAX,
                _ => current,
            };
            SetScrollInfo(
                self.hwnd,
                bar,
                &SCROLLINFO {
                    cbSize: size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_POS,
                    nPos: next,
                    ..Default::default()
                },
                true,
            );
        }
        self.reflow();
    }

    pub fn context_menu(&self, edit: bool, demo: bool, overlays_visible: bool) {
        unsafe {
            let Ok(menu) = CreatePopupMenu() else {
                return;
            };
            for (id, title, checked) in [
                (SHOW, "Settings", false),
                (EDIT, "Edit Mode", edit),
                (TOGGLE_OVERLAYS, "Show both HUDs", overlays_visible),
                (TOGGLE_HALO, "Show / hide Halo", false),
                (TOGGLE_TYRES, "Show / hide tyres", false),
                (CYCLE, "Cycle temperature mode", false),
                (TOGGLE_DEMO, "Demo", demo),
                (EXIT, "Exit", false),
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
            hIcon: self.icons[1].0,
            ..Default::default()
        };
        let text = wide("F1 Halo Telemetry · double-click: settings");
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
        }
    }
}

unsafe extern "system" fn panel_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if let Some(result) = theme::message(hwnd, message, wparam, lparam) {
        return result;
    }
    match message {
        WM_INPUT => {
            rear_view::read_raw(HRAWINPUT(lparam.0 as *mut _), |device, state| {
                overlay::push(Action::ControllerInput(device, state));
            });
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
        WM_GETMINMAXINFO => {
            unsafe {
                let dpi = GetDpiForWindow(hwnd).max(96);
                let mut minimum = RECT {
                    right: (360 * dpi / 96) as i32,
                    bottom: (320 * dpi / 96) as i32,
                    ..Default::default()
                };
                let _ = AdjustWindowRectExForDpi(
                    &mut minimum,
                    WS_OVERLAPPEDWINDOW | WS_VSCROLL,
                    false,
                    WS_EX_CONTROLPARENT,
                    dpi,
                );
                let bounds = overlay::window_rect(hwnd);
                let work = overlay::work_area(POINT {
                    x: bounds.left,
                    y: bounds.top,
                });
                let info = &mut *(lparam.0 as *mut MINMAXINFO);
                info.ptMinTrackSize.x = (minimum.right - minimum.left
                    + GetSystemMetricsForDpi(SM_CXVSCROLL, dpi))
                .min(work.right - work.left);
                info.ptMinTrackSize.y = (minimum.bottom - minimum.top).min(work.bottom - work.top);
            }
            return LRESULT(0);
        }
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
    use super::*;

    fn assert_hover_paints_locally(panel: &Panel) {
        unsafe {
            for id in [HALO + 1, APPLY, HALO, HALO_MODEL] {
                let control = panel.controls[&id];
                let _ = RedrawWindow(
                    Some(panel.hwnd),
                    None,
                    None,
                    RDW_INVALIDATE | RDW_ALLCHILDREN | RDW_UPDATENOW,
                );
                let _ = ValidateRect(Some(panel.hwnd), None);
                let _ = ValidateRect(Some(control), None);
                SendMessageW(control, WM_MOUSEMOVE, None, Some(LPARAM(10 | (10 << 16))));
                let mut dirty = RECT::default();
                let invalidated = GetUpdateRect(panel.hwnd, Some(&mut dirty), false).as_bool();
                let mut bounds = RECT::default();
                GetWindowRect(control, &mut bounds).unwrap();
                let mut origin = POINT {
                    x: bounds.left,
                    y: bounds.top,
                };
                let _ = ScreenToClient(panel.hwnd, &mut origin);
                assert!(
                    !invalidated
                        || (dirty.left >= origin.x
                            && dirty.top >= origin.y
                            && dirty.right <= origin.x + bounds.right - bounds.left
                            && dirty.bottom <= origin.y + bounds.bottom - bounds.top),
                    "Hover over control {id} invalidated unrelated settings: {dirty:?}"
                );
                let _ = ValidateRect(Some(control), None);
                for x in 11..30 {
                    SendMessageW(control, WM_MOUSEMOVE, None, Some(LPARAM(x | (10 << 16))));
                }
                assert!(
                    !GetUpdateRect(control, None, false).as_bool(),
                    "Moving within control {id} repainted an unchanged hover state"
                );
                SendMessageW(
                    control,
                    windows::Win32::UI::Controls::WM_MOUSELEAVE,
                    None,
                    None,
                );
            }
        }
    }

    fn assert_control_print_uses_the_skin(panel: &Panel) {
        unsafe {
            let screen = GetDC(None);
            let dc = CreateCompatibleDC(Some(screen));
            let bitmap = CreateCompatibleBitmap(
                screen,
                (400 * panel.dpi / 96) as i32,
                (60 * panel.dpi / 96) as i32,
            );
            let previous = SelectObject(dc, bitmap.into());
            for id in [APPLY, HALO_MODEL] {
                let control = panel.controls[&id];
                let mut rect = RECT::default();
                GetClientRect(control, &mut rect).unwrap();
                SetDCBrushColor(dc, COLORREF(0x4e2c19));
                FillRect(dc, &rect, HBRUSH(GetStockObject(DC_BRUSH).0));
                SendMessageW(
                    control,
                    WM_PRINTCLIENT,
                    Some(WPARAM(dc.0 as usize)),
                    Some(LPARAM(PRF_CLIENT as isize)),
                );
                let colour = GetPixel(
                    dc,
                    if id == APPLY { 5 } else { rect.right - 4 },
                    rect.bottom / 2,
                )
                .0;
                let (red, green, blue) =
                    (colour & 0xff, (colour >> 8) & 0xff, (colour >> 16) & 0xff);
                // Allow display colour management while rejecting the native grey button chrome.
                let themed = if id == APPLY {
                    green > red + 60 && blue > red + 60
                } else {
                    red.max(green).max(blue) < 100
                };
                assert!(
                    themed,
                    "Control {id} exposed native paint below its skin: {colour:x}"
                );
                if id == HALO_MODEL {
                    let has_caption = (8..28).any(|y| {
                        (10..90).any(|x| {
                            let pixel = GetPixel(dc, x, y).0;
                            (pixel & 0xff) + ((pixel >> 8) & 0xff) + ((pixel >> 16) & 0xff) > 600
                        })
                    });
                    assert!(
                        has_caption,
                        "Custom combo painting lost the selected model caption"
                    );
                }
            }
            SelectObject(dc, previous);
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            ReleaseDC(None, screen);
        }
    }

    fn assert_responsive_controls(panel: &mut Panel) {
        unsafe {
            let style = GetWindowLongPtrW(panel.hwnd, GWL_STYLE) as u32;
            assert_ne!(style & WS_THICKFRAME.0, 0);
            assert_ne!(style & WS_MAXIMIZEBOX.0, 0);
            assert_eq!(style & WS_HSCROLL.0, 0);
        }
        for (width, height, dpi) in [
            (1180, 780, 96),
            (1920, 1040, 96),
            (1080, 1800, 96),
            (900, 1400, 96),
            (720, 1100, 144),
            (540, 960, 192),
            (1280, 600, 96),
            (360, 700, 96),
        ] {
            panel.change_dpi(dpi, overlay::window_rect(panel.hwnd));
            let mut outer = RECT {
                right: width * dpi as i32 / 96,
                bottom: height * dpi as i32 / 96,
                ..Default::default()
            };
            unsafe {
                let style = WINDOW_STYLE(GetWindowLongPtrW(panel.hwnd, GWL_STYLE) as u32);
                AdjustWindowRectExForDpi(&mut outer, style, false, WS_EX_CONTROLPARENT, dpi)
                    .unwrap();
                SetWindowPos(
                    panel.hwnd,
                    None,
                    0,
                    0,
                    outer.right - outer.left,
                    outer.bottom - outer.top,
                    SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
                )
                .unwrap();
            }
            for (index, page) in Page::ALL.into_iter().enumerate() {
                panel.navigate(theme::NAV + index as u16);
                let layout = panel.theme.layout.get();
                let mut client = RECT::default();
                unsafe {
                    GetClientRect(panel.hwnd, &mut client).unwrap();
                }
                let mut visible = Vec::new();
                for &(child, _, owner) in &panel.theme.children {
                    if owner.is_some_and(|owner| owner != page) {
                        continue;
                    }
                    let mut bounds = RECT::default();
                    unsafe {
                        GetWindowRect(child, &mut bounds).unwrap();
                    }
                    let mut origin = POINT {
                        x: bounds.left,
                        y: bounds.top,
                    };
                    unsafe {
                        let _ = ScreenToClient(panel.hwnd, &mut origin);
                    }
                    let rect = (
                        origin.x,
                        origin.y,
                        bounds.right - bounds.left,
                        bounds.bottom - bounds.top,
                    );
                    let id = unsafe { GetDlgCtrlID(child) };
                    let (x, y, w, h) = rect;
                    assert!(
                        x >= 0
                            && w > 0
                            && x + w <= client.right
                            && y >= 0
                            && h > 0
                            && y + h <= (layout.content_height * dpi as i32 + 95) / 96,
                        "{page:?} at {width}x{height}/{dpi} DPI: control {id} escaped the layout: {rect:?}"
                    );
                    visible.push((id, rect));
                }
                for (i, &(id, (x, y, w, h))) in visible.iter().enumerate() {
                    for &(other, (rx, ry, rw, rh)) in &visible[i + 1..] {
                        assert!(
                            x + w <= rx || rx + rw <= x || y + h <= ry || ry + rh <= y,
                            "{page:?} at {width}x{height}/{dpi} DPI: controls {id} {other} overlap: {:?} {:?}",
                            (x, y, w, h),
                            (rx, ry, rw, rh)
                        );
                    }
                }
            }
        }
        panel.navigate(theme::NAV + 3);
        panel.scroll(true, 7, 0);
        unsafe {
            let input = panel.controls[&IP];
            SendMessageW(input, WM_SETFOCUS, None, None);
            let mut bounds = RECT::default();
            GetWindowRect(input, &mut bounds).unwrap();
            let mut origin = POINT {
                x: bounds.left,
                y: bounds.top,
            };
            let _ = ScreenToClient(panel.hwnd, &mut origin);
            let mut client = RECT::default();
            GetClientRect(panel.hwnd, &mut client).unwrap();
            assert!(
                origin.y >= 0 && origin.y + bounds.bottom - bounds.top <= client.bottom,
                "Keyboard focus left the editable field outside the narrow viewport"
            );
        }
    }

    #[test]
    fn switching_settings_pages_preserves_unsaved_values() {
        let mut panel = Panel::new().unwrap();
        let settings = Settings::default();
        panel.populate(&settings, false);
        unsafe {
            let _ = ShowWindow(panel.hwnd, SW_SHOWNOACTIVATE);
        }
        assert_hover_paints_locally(&panel);
        assert_control_print_uses_the_skin(&panel);
        panel.navigate(theme::NAV + 5);
        let log = (0..100)
            .map(|line| format!("Diagnostic line {line}\r\n"))
            .collect::<String>();
        panel.set(DIAGNOSTICS, &log);
        unsafe {
            let control = panel.controls[&DIAGNOSTICS];
            SendMessageW(control, EM_LINESCROLL, None, Some(LPARAM(20)));
            let first = SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None).0;
            assert!(first > 0);
            panel.diagnostics(&format!("{log}Updated status"));
            assert_eq!(
                SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None).0,
                first
            );
        }
        panel.navigate(theme::NAV);
        panel.hide();
        panel.set(IP, "127.0.0.2");
        panel.set(TEMP_MIN, "87");
        panel.set(COLOUR, "#64DDF5");
        panel.set(HOTKEY, "Ctrl+Shift+F9");
        panel.set(HALO_PROFILE, "Ferrari test profile");
        panel.check(REAR_VIEW, false);
        assert_responsive_controls(&mut panel);
        for (index, page) in Page::ALL.into_iter().enumerate() {
            let id = theme::NAV + index as u16;
            unsafe {
                SendMessageW(panel.controls[&id], BM_CLICK, None, None);
            }
            let mut clicked = false;
            while let Some(action) = overlay::pop() {
                if let Action::Command(command) = action {
                    assert_eq!(command, id);
                    clicked = panel.navigate(command);
                }
            }
            assert!(clicked);
            assert!(panel.theme.page.get() == page);
            for &(hwnd, _, owner) in &panel.theme.children {
                let visible =
                    unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_VISIBLE.0 != 0 };
                assert_eq!(visible, owner.is_none_or(|owner| owner == page));
            }
        }
        assert!(!panel.navigate(APPLY));
        for dpi in [144, 192, 96] {
            panel.change_dpi(dpi, overlay::window_rect(panel.hwnd));
            assert_eq!(panel.dpi, dpi);
            assert!(panel.theme.page.get() == Page::Diagnostics);
        }
        assert_eq!(panel.text(HALO_PROFILE), "Ferrari test profile");
        let read = panel.read(&settings).unwrap();
        assert_eq!(read.bind_ip, "127.0.0.2");
        assert_eq!(read.temperature_min_c, 87.0);
        assert_eq!(read.colours.accent, [100, 221, 245]);
        assert_eq!(read.hotkey, "Ctrl+Shift+F9");
        assert!(!read.hide_halo_rear_view);
        assert_eq!(read.halo.scale, settings.halo.scale);
        assert_eq!(read.tyres.opacity, settings.tyres.opacity);
        assert_opacity_controls_validate_percent_and_keep_unedited_precision(&panel);
    }

    fn assert_opacity_controls_validate_percent_and_keep_unedited_precision(panel: &Panel) {
        let mut settings = Settings::default();
        settings.halo.opacity = 0.938_7;
        settings.tyres.opacity = 0.456_7;
        panel.populate(&settings, false);
        assert_eq!(panel.text(HALO + 4), "94");
        assert_eq!(panel.text(TYRES + 4), "46");
        let unchanged = panel.read(&settings).unwrap();
        assert_eq!(unchanged.halo.opacity, settings.halo.opacity);
        assert_eq!(unchanged.tyres.opacity, settings.tyres.opacity);

        panel.set(HALO + 4, "100");
        panel.set(TYRES + 4, "0");
        let changed = panel.read(&settings).unwrap();
        assert_eq!(changed.halo.opacity, 1.0);
        assert_eq!(changed.tyres.opacity, 0.0);
        let saved: Settings =
            serde_json::from_str(&serde_json::to_string(&changed).unwrap()).unwrap();
        assert_eq!(saved.halo.opacity, 1.0);
        assert_eq!(saved.tyres.opacity, 0.0);

        panel.set(TYRES + 4, "48,5");
        assert_eq!(panel.read(&settings).unwrap().tyres.opacity, 0.485);
        for id in [HALO + 4, TYRES + 4] {
            for invalid in ["-1", "101", "NaN", "inf", "invalid", ""] {
                panel.populate(&settings, false);
                panel.set(id, invalid);
                assert!(panel.read(&settings).is_err(), "Accepted {invalid}");
            }
        }
    }

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
