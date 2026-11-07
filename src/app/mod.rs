mod panel;

use crate::{
    diagnostics::Diagnostics,
    graphics::{Graphics, Surface},
    hud::{self, halo::Halo},
    input::{
        HotkeyBinding,
        rear_view::{self, RearInput},
        register_window_hotkeys, replace_window_hotkeys,
    },
    overlay::{self, Action, FrameTimer, Overlay, Wake},
    settings::{self, FpsLimit, Settings},
    telemetry::Snapshot,
    udp::Receiver,
};
use std::{
    mem::size_of,
    path::PathBuf,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::*,
        System::{Com::*, ProcessStatus::*, Threading::*},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::{PCWSTR, w},
};

#[derive(Default)]
pub struct Options {
    pub demo: bool,
    pub edit: bool,
    pub no_settings: bool,
    pub config: Option<PathBuf>,
    pub quit_after: Option<Duration>,
    pub diagnostics: Option<PathBuf>,
    pub fps: Option<FpsLimit>,
    pub recreate_after: Option<Duration>,
}

impl Options {
    pub fn parse() -> Result<Self, String> {
        let mut options = Self::default();
        let mut args = std::env::args().skip(1);
        while let Some(argument) = args.next() {
            match argument.as_str() {
                "--demo" => options.demo = true,
                "--edit" => options.edit = true,
                "--no-settings" => options.no_settings = true,
                "--config" => options.config = Some(args.next().ok_or("--config richiede un percorso")?.into()),
                "--diagnostics" => options.diagnostics = Some(args.next().ok_or("--diagnostics richiede un percorso")?.into()),
                "--quit-after" | "--recreate-device-after" => {
                    let seconds: f64 = args.next().ok_or("Manca durata in secondi")?.parse().map_err(|_| "Durata non valida")?;
                    if !seconds.is_finite() || !(0.1..=86400.0).contains(&seconds) { return Err("Durata: 0.1–86400 secondi".into()); }
                    let duration = Some(Duration::from_secs_f64(seconds));
                    if argument == "--quit-after" { options.quit_after = duration; } else { options.recreate_after = duration; }
                }
                "--fps" => options.fps = Some(match args.next().as_deref() {
                    Some("60") => FpsLimit::Fps60, Some("120") => FpsLimit::Fps120,
                    Some("adaptive") => FpsLimit::Adaptive, _ => return Err("--fps: usare 60, 120 o adaptive".into()),
                }),
                "--help" | "-h" => return Err("F1 Halo Telemetry\n\n--demo\n--edit\n--no-settings\n--config <settings.json>\n--fps <60|120|adaptive>\n--quit-after <secondi>\n--diagnostics <output.json>\n--recreate-device-after <secondi> (test ricreazione risorse)".into()),
                _ => return Err(format!("Opzione sconosciuta: {argument}")),
            }
        }
        Ok(options)
    }
}

struct ComApartment;
impl ComApartment {
    fn new() -> Result<Self, String> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(|e| e.to_string())?;
        }
        Ok(Self)
    }
}
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub fn run(options: Options) -> Result<(), String> {
    // SAFETY: DPI mode is set before creating any HWND. ERROR_ACCESS_DENIED is
    // harmless if a manifest or the host has already selected per-monitor mode.
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
    let _com = ComApartment::new()?;
    let config = options
        .config
        .clone()
        .unwrap_or_else(settings::default_path);
    let existed = config.exists();
    let mut settings = Settings::load(&config)?;
    if let Some(fps) = options.fps {
        settings.fps = fps;
    }
    if !existed {
        default_layout(&mut settings);
    }
    Overlay::register().map_err(|e| e.to_string())?;
    let panel = panel::Panel::new().map_err(|e| e.to_string())?;
    let overlays = [
        Overlay::new(0, settings.halo).map_err(|e| e.to_string())?,
        Overlay::new(1, settings.tyres).map_err(|e| e.to_string())?,
    ];
    settings.halo = overlays[0].layout(settings.halo);
    settings.tyres = overlays[1].layout(settings.tyres);
    let wake = Wake::new().map_err(|e| e.to_string())?;
    let worker_wake = wake.clone();
    let receiver = Receiver::start(&settings, options.demo, move || worker_wake.set())?;
    let timer = FrameTimer::new().map_err(|e| e.to_string())?;
    let mut app = App {
        panel,
        overlays,
        graphics: None,
        surfaces: [None, None],
        halo: None,
        settings,
        config,
        receiver,
        wake,
        timer,
        diagnostics: Diagnostics::default(),
        snapshot: Snapshot::default(),
        demo: options.demo,
        edit: false,
        overlays_visible: true,
        rear_hidden: false,
        rear_input: RearInput::default(),
        dirty: true,
        next_frame: Instant::now(),
        graphics_retry: Instant::now(),
        graphics_error: None,
        hotkeys: [None; 2],
        quit: false,
        start: Instant::now(),
        options,
        usage: ProcessUsage::new(),
        last_status: Instant::now(),
        last_counters: [0; 17],
        rates: [0.0; 17],
        peak_memory: 0,
        first_memory: 0,
        previous_live: false,
        forced_recreation: false,
        fresh_mask: u16::MAX,
        monitor_hz: 60,
        race_foreground: None,
    };
    app.create_graphics()?;
    app.monitor_hz = overlay::monitor_refresh(app.overlays[0].hwnd);
    app.panel.populate(&app.settings, app.demo);
    let (bindings, mut hotkey_errors) = register_window_hotkeys(
        app.panel.hwnd,
        [&app.settings.hotkey, &app.settings.visibility_hotkey],
    );
    app.hotkeys = bindings;
    if let Err(error) = app.rear_input.register(app.panel.hwnd) {
        hotkey_errors.push(format!("Input DualSense non disponibile: {error}"));
    }
    if !app.options.no_settings {
        app.panel.show();
    }
    if app.options.edit {
        app.set_edit(true)?;
    }
    if !hotkey_errors.is_empty() {
        app.panel.result(&hotkey_errors.join(" · "));
    }
    if !existed {
        app.settings.save(&app.config)?;
    }
    app.event_loop()?;
    if let Some(path) = app.options.diagnostics.clone() {
        app.export(&path)?;
    }
    for binding in app.hotkeys.into_iter().flatten() {
        unsafe {
            let _ = UnregisterHotKey(Some(app.panel.hwnd), binding.id);
        }
    }
    Ok(())
}

fn default_layout(settings: &mut Settings) {
    let work = overlay::work_area(POINT { x: 0, y: 0 });
    let dpi = overlay::monitor_dpi(POINT {
        x: work.left,
        y: work.top,
    }) as f32
        / 96.0;
    settings.halo.x =
        work.left + ((work.right - work.left) as f32 - hud::halo::WIDTH * dpi) as i32 / 2;
    settings.halo.y = work.bottom - (hud::halo::HEIGHT * dpi) as i32 - (36.0 * dpi) as i32;
    settings.tyres.x = work.left + (28.0 * dpi) as i32;
    settings.tyres.y = work.bottom - (hud::tyres::HEIGHT * dpi) as i32 - (36.0 * dpi) as i32;
}

struct App {
    panel: panel::Panel,
    overlays: [Overlay; 2],
    // Surfaces are explicitly released before replacing their shared device.
    graphics: Option<Graphics>,
    surfaces: [Option<Surface>; 2],
    halo: Option<Halo>,
    settings: Settings,
    config: PathBuf,
    receiver: Receiver,
    wake: std::sync::Arc<Wake>,
    timer: FrameTimer,
    diagnostics: Diagnostics,
    snapshot: Snapshot,
    demo: bool,
    edit: bool,
    overlays_visible: bool,
    rear_hidden: bool,
    rear_input: RearInput,
    dirty: bool,
    next_frame: Instant,
    graphics_retry: Instant,
    graphics_error: Option<String>,
    hotkeys: [Option<HotkeyBinding>; 2],
    quit: bool,
    start: Instant,
    options: Options,
    usage: ProcessUsage,
    last_status: Instant,
    last_counters: [u64; 17],
    rates: [f64; 17],
    peak_memory: u64,
    first_memory: u64,
    previous_live: bool,
    forced_recreation: bool,
    fresh_mask: u16,
    monitor_hz: u32,
    race_foreground: Option<HWND>,
}

impl App {
    fn create_graphics(&mut self) -> Result<(), String> {
        self.surfaces = [None, None];
        self.halo = None;
        self.graphics = None;
        let graphics = Graphics::new().map_err(|e| e.to_string())?;
        let halo = Halo::new(&graphics.geometry_factory).map_err(|e| e.to_string())?;
        for (index, overlay) in self.overlays.iter().enumerate() {
            let (width, height) = overlay.size();
            self.surfaces[index] = Some(
                graphics
                    .surface(overlay.hwnd, width, height)
                    .map_err(|e| e.to_string())?,
            );
        }
        self.graphics = Some(graphics);
        self.halo = Some(halo);
        self.graphics_error = None;
        self.dirty = true;
        Ok(())
    }

    fn event_loop(&mut self) -> Result<(), String> {
        let handles = [self.wake.handle(), self.timer.handle()];
        loop {
            let mut message = MSG::default();
            // A bounded message batch keeps a busy input queue from starving the HUD.
            for _ in 0..256 {
                if !unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
                    break;
                }
                if message.message == WM_QUIT {
                    self.quit = true;
                    break;
                }
                if !self.panel.visible()
                    || !unsafe { IsDialogMessageW(self.panel.hwnd, &message) }.as_bool()
                {
                    unsafe {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
            while let Some(action) = overlay::pop() {
                if let Err(error) = self.action(action) {
                    self.panel.result(&error);
                }
            }
            if self.quit {
                break;
            }
            if let Ok(snapshot) = self.receiver.shared.try_read() {
                self.snapshot = *snapshot;
            }
            let now = Instant::now();
            if self
                .options
                .quit_after
                .is_some_and(|duration| now.duration_since(self.start) >= duration)
            {
                break;
            }
            if !self.forced_recreation
                && self
                    .options
                    .recreate_after
                    .is_some_and(|duration| now.duration_since(self.start) >= duration)
            {
                self.forced_recreation = true;
                self.create_graphics()?;
                self.diagnostics.device_recreations += 1;
            }
            let mask = self.snapshot.freshness_mask(now);
            if mask != self.fresh_mask {
                self.fresh_mask = mask;
                self.dirty = true;
            }
            let poll_rear = self.update_rear_view(now);
            if self.graphics.is_none() && now >= self.graphics_retry {
                match self.create_graphics() {
                    Ok(()) => self.diagnostics.device_recreations += 1,
                    Err(error) => {
                        self.graphics_error = Some(error);
                        self.graphics_retry = now + Duration::from_secs(1);
                    }
                }
            }
            let visible = self
                .settings
                .effective_layouts(self.overlays_visible, self.rear_hidden)
                .iter()
                .any(|layout| layout.visible);
            if self.dirty && visible && self.graphics.is_some() && now >= self.next_frame {
                match self.render(now) {
                    Ok(()) => self.dirty = false,
                    Err(error) => {
                        self.diagnostics.presentation_failures += 1;
                        self.graphics_error = Some(error);
                        self.surfaces = [None, None];
                        self.halo = None;
                        self.graphics = None;
                        self.graphics_retry = Instant::now() + Duration::from_millis(500);
                    }
                }
                let hz = self.settings.fps.hz(self.monitor_hz);
                let period = Duration::from_secs_f64(1.0 / hz as f64);
                self.next_frame = if now.saturating_duration_since(self.next_frame) > period {
                    now + period
                } else {
                    self.next_frame + period
                };
            } else if !visible {
                self.dirty = false;
            }
            if now.duration_since(self.last_status) >= Duration::from_secs(1) {
                self.update_status(now);
            }
            let mut deadline = now + Duration::from_millis(250);
            if poll_rear {
                deadline = deadline.min(now + rear_view::POLL_INTERVAL);
            }
            if self.dirty && visible && self.graphics.is_some() {
                deadline = deadline.min(self.next_frame);
            }
            if self.graphics.is_none() {
                deadline = deadline.min(self.graphics_retry);
            }
            if let Some(duration) = self.options.quit_after {
                deadline = deadline.min(self.start + duration);
            }
            for (received, ttl) in [
                (
                    self.snapshot.telemetry.map(|v| v.received),
                    crate::telemetry::state::FAST_TTL,
                ),
                (
                    self.snapshot.status.map(|v| v.received),
                    crate::telemetry::state::FAST_TTL,
                ),
                (
                    self.snapshot.aero.map(|v| v.received),
                    crate::telemetry::state::FAST_TTL,
                ),
                (
                    self.snapshot.damage.map(|v| v.received),
                    crate::telemetry::state::DAMAGE_TTL,
                ),
            ] {
                if let Some(expiry) = received
                    .map(|r| r + ttl + Duration::from_millis(1))
                    .filter(|e| *e > now)
                {
                    deadline = deadline.min(expiry);
                }
            }
            self.timer
                .arm(deadline.saturating_duration_since(Instant::now()))
                .map_err(|e| e.to_string())?;
            let result = unsafe {
                MsgWaitForMultipleObjectsEx(
                    Some(&handles),
                    INFINITE,
                    QS_ALLINPUT,
                    MWMO_INPUTAVAILABLE,
                )
            };
            if result == WAIT_OBJECT_0 {
                self.dirty = true;
            }
            if result == WAIT_FAILED {
                return Err(windows::core::Error::from_thread().to_string());
            }
        }
        self.usage.update();
        Ok(())
    }

    fn render(&mut self, now: Instant) -> Result<(), String> {
        if !self.overlays_visible {
            return Ok(());
        }
        let start = Instant::now();
        if let Some(sample) = self.graphics.as_mut().and_then(Graphics::begin_gpu_sample) {
            self.diagnostics.gpu_sample(sample);
        }
        let layouts = self
            .settings
            .effective_layouts(self.overlays_visible, self.rear_hidden);
        for (index, layout) in layouts.into_iter().enumerate() {
            if !layout.visible {
                continue;
            }
            let (width, height) = self.overlays[index].size();
            let surface = self.surfaces[index].as_mut().unwrap();
            surface.resize(width, height).map_err(|e| e.to_string())?;
            let (base_width, base_height) = if index == 0 {
                (hud::halo::WIDTH, hud::halo::HEIGHT)
            } else {
                (hud::tyres::WIDTH, hud::tyres::HEIGHT)
            };
            let painter = surface
                .begin(base_width, base_height, layout.opacity)
                .map_err(|e| e.to_string())?;
            if index == 0 {
                self.halo.as_ref().unwrap().draw(
                    &painter,
                    &self.snapshot,
                    &self.settings,
                    self.edit,
                    now,
                );
            } else {
                hud::tyres::draw(&painter, &self.snapshot, &self.settings, self.edit, now);
            }
            surface.present().map_err(|e| e.to_string())?;
        }
        if let Some(graphics) = &mut self.graphics {
            graphics.end_gpu_sample();
        }
        let live = self.snapshot.driving(now);
        self.diagnostics.frame(
            start,
            start.elapsed(),
            self.snapshot.telemetry.filter(|_| live).map(|t| t.received),
            live && self.previous_live,
        );
        self.previous_live = live;
        Ok(())
    }

    fn apply_layouts(&self) -> Result<(), String> {
        for (overlay, layout) in self.overlays.iter().zip(
            self.settings
                .effective_layouts(self.overlays_visible, self.rear_hidden),
        ) {
            overlay
                .apply_layout(layout)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn update_rear_view(&mut self, now: Instant) -> bool {
        let eligible = self.settings.hide_halo_rear_view
            && !self.edit
            && !self.demo
            && self.snapshot.driving(now)
            && self.rear_input.game_focused();
        let axis = if eligible {
            self.rear_input.poll(now)
        } else {
            None
        };
        let hidden = rear_view::held(self.rear_hidden, eligible, axis);
        if hidden != self.rear_hidden {
            let previous = self.rear_hidden;
            self.rear_hidden = hidden;
            let layout = self
                .settings
                .effective_layouts(self.overlays_visible, hidden)[0];
            if let Err(error) = self.overlays[0].apply_layout(layout) {
                self.rear_hidden = previous;
                self.panel.result(&format!("Visibilità Halo: {error}"));
            } else {
                self.dirty = true;
                self.next_frame = now;
                self.previous_live = false;
            }
        }
        eligible
    }

    fn set_overlays_visible(&mut self, visible: bool) -> Result<(), String> {
        self.settings.halo = self.overlays[0].layout(self.settings.halo);
        self.settings.tyres = self.overlays[1].layout(self.settings.tyres);
        let previous = self.overlays_visible;
        self.overlays_visible = visible;
        if let Err(error) = self.apply_layouts() {
            self.overlays_visible = previous;
            let _ = self.apply_layouts();
            return Err(error);
        }
        self.dirty = visible;
        self.next_frame = Instant::now();
        self.previous_live = false;
        self.panel.result(if visible {
            "HUD mostrati secondo le preferenze individuali."
        } else {
            "HUD nascosti. La telemetria continua; premi la hotkey visibilità per mostrarli."
        });
        Ok(())
    }

    fn set_edit(&mut self, edit: bool) -> Result<(), String> {
        if edit {
            let foreground = unsafe { GetForegroundWindow() };
            if foreground != self.panel.hwnd && !self.overlays.iter().any(|o| o.hwnd == foreground)
            {
                self.race_foreground = Some(foreground);
            }
        }
        for overlay in &self.overlays {
            overlay.set_edit(edit).map_err(|e| e.to_string())?;
        }
        self.edit = edit;
        self.dirty = true;
        if !edit {
            let foreground = unsafe { GetForegroundWindow() };
            let overlay_had_focus = self.overlays.iter().any(|o| o.hwnd == foreground);
            if let Some(hwnd) = self
                .race_foreground
                .take()
                .filter(|h| overlay_had_focus && unsafe { IsWindow(Some(*h)).as_bool() })
            {
                unsafe {
                    let _ = SetForegroundWindow(hwnd);
                }
            }
        }
        self.panel.result(if edit { "EDIT: trascina HUD, usa gli angoli. Rotella: scala; Shift+rotella: opacità. Hotkey per Race Mode." }
            else { "RACE: HUD click-through. Il gioco mantiene il focus. Hotkey per modificare." });
        Ok(())
    }

    fn sync_layout(&mut self) -> Result<(), String> {
        self.settings.halo = self.overlays[0].layout(self.settings.halo);
        self.settings.tyres = self.overlays[1].layout(self.settings.tyres);
        self.monitor_hz = overlay::monitor_refresh(self.overlays[0].hwnd);
        self.apply_layouts()?;
        self.settings.halo = self.overlays[0].layout(self.settings.halo);
        self.settings.tyres = self.overlays[1].layout(self.settings.tyres);
        self.panel.populate_layout(&self.settings);
        self.settings.save(&self.config)?;
        self.dirty = true;
        Ok(())
    }

    fn apply(&mut self) -> Result<(), String> {
        let mut s = self.panel.read(&self.settings)?;
        let demo = self.panel.checked(panel::DEMO);
        let source_changed = (s.bind_ip.as_str(), s.port, demo)
            != (
                self.settings.bind_ip.as_str(),
                self.settings.port,
                self.demo,
            );
        let text = [s.hotkey.clone(), s.visibility_hotkey.clone()];
        let old_layouts = [
            self.overlays[0].layout(self.settings.halo),
            self.overlays[1].layout(self.settings.tyres),
        ];
        let bindings =
            replace_window_hotkeys(self.panel.hwnd, self.hotkeys, [&text[0], &text[1]], || {
                for (overlay, layout) in self.overlays.iter().zip(s.effective_layouts(
                    self.overlays_visible,
                    self.rear_hidden && s.hide_halo_rear_view,
                )) {
                    overlay
                        .apply_layout(layout)
                        .map_err(|error| error.to_string())?;
                }
                s.halo = self.overlays[0].layout(s.halo);
                s.tyres = self.overlays[1].layout(s.tyres);
                s.save(&self.config)?;
                if source_changed {
                    if let Err(error) = self.receiver.source(&s, demo) {
                        return match self.settings.save(&self.config) {
                            Ok(()) => Err(error),
                            Err(restore) => {
                                Err(format!("{error}; ripristino impostazioni: {restore}"))
                            }
                        };
                    }
                }
                Ok(())
            });
        match bindings {
            Ok(bindings) => self.hotkeys = bindings,
            Err(error) => {
                for (index, (overlay, layout)) in self.overlays.iter().zip(old_layouts).enumerate()
                {
                    let _ = overlay.apply_layout(layout.with_global_visibility(
                        self.overlays_visible && !(index == 0 && self.rear_hidden),
                    ));
                }
                return Err(error);
            }
        }
        self.settings = s;
        self.demo = demo;
        if source_changed {
            self.snapshot = Snapshot::default();
        }
        self.monitor_hz = overlay::monitor_refresh(self.overlays[0].hwnd);
        self.panel.populate_layout(&self.settings);
        self.panel.result("Impostazioni applicate e salvate.");
        self.dirty = true;
        Ok(())
    }

    fn halo_profile(&mut self, save: bool) -> Result<(), String> {
        let old_layout = self.overlays[0].layout(self.settings.halo);
        let mut next = self.settings.clone();
        next.halo = old_layout;
        let name = self.panel.text(panel::HALO_PROFILE);
        let name = if save {
            self.panel.halo_profile(old_layout)?.apply(&mut next.halo);
            next.save_halo_profile(&name)?
        } else {
            next.load_halo_profile(&name)?
        };
        next.validate()?;
        let result = (|| {
            self.overlays[0]
                .apply_layout(next.effective_layouts(self.overlays_visible, self.rear_hidden)[0])
                .map_err(|error| error.to_string())?;
            next.halo = self.overlays[0].layout(next.halo);
            if save {
                // Store the actual geometry after DPI/work-area clamping.
                next.save_halo_profile(&name)?;
            }
            next.save(&self.config)
        })();
        if let Err(error) = result {
            let _ = self.overlays[0].apply_layout(
                old_layout.with_global_visibility(self.overlays_visible && !self.rear_hidden),
            );
            return Err(error);
        }
        self.settings = next;
        self.monitor_hz = overlay::monitor_refresh(self.overlays[0].hwnd);
        self.panel.populate_halo_geometry(&self.settings);
        self.panel.populate_profiles(&self.settings);
        self.panel.result(&format!(
            "Profilo «{name}» {}. Posizione e scala Halo applicate.",
            if save { "salvato" } else { "caricato" }
        ));
        self.dirty = true;
        self.next_frame = Instant::now();
        self.previous_live = false;
        Ok(())
    }

    fn action(&mut self, action: Action) -> Result<(), String> {
        match action {
            Action::Dirty => self.dirty = true,
            Action::ControllerAxis(device, down) => {
                self.rear_input.receive(device, down, Instant::now())
            }
            Action::ControllerRemoved(device) => self.rear_input.remove(device),
            Action::Quit => self.quit = true,
            Action::Hotkey(id) => {
                if self.hotkeys[0].is_some_and(|binding| binding.id == id) {
                    self.set_edit(!self.edit)?;
                } else if self.hotkeys[1].is_some_and(|binding| binding.id == id) {
                    self.set_overlays_visible(!self.overlays_visible)?;
                }
            }
            Action::ToggleEdit => self.set_edit(!self.edit)?,
            Action::ClosePanel => self.panel.hide(),
            Action::PanelDpi(dpi, rect) => self.panel.change_dpi(dpi, rect),
            Action::PanelScroll(vertical, code, position) => {
                self.panel.scroll(vertical, code, position)
            }
            Action::PanelWheel(delta) => {
                for _ in 0..3 {
                    self.panel.scroll(true, if delta > 0 { 0 } else { 1 }, 0);
                }
            }
            Action::ContextMenu => {
                self.panel
                    .context_menu(self.edit, self.demo, self.overlays_visible)
            }
            Action::LayoutChanged(index) => {
                let _ = index;
                self.sync_layout()?;
            }
            Action::DisplayChanged => {
                self.panel.change_dpi(
                    unsafe { GetDpiForWindow(self.panel.hwnd) },
                    overlay::window_rect(self.panel.hwnd),
                );
                self.apply_layouts()?;
                self.sync_layout()?;
            }
            Action::Wheel(index, delta, opacity) => {
                let layout = if index == 0 {
                    &mut self.settings.halo
                } else {
                    &mut self.settings.tyres
                };
                if opacity {
                    layout.opacity =
                        (layout.opacity + delta as f32 / 120.0 * 0.05).clamp(0.15, 1.0);
                } else {
                    layout.scale = (layout.scale + delta as f32 / 120.0 * 0.05).clamp(0.35, 3.0);
                }
                self.apply_layouts()?;
                self.sync_layout()?;
            }
            Action::CycleTyres => self.cycle_tyres()?,
            Action::Command(command) => {
                match command {
                    panel::APPLY => self.apply()?,
                    panel::SAVE_PROFILE => self.halo_profile(true)?,
                    panel::LOAD_PROFILE => self.halo_profile(false)?,
                    panel::EDIT => self.set_edit(!self.edit)?,
                    panel::TOGGLE_OVERLAYS => self.set_overlays_visible(!self.overlays_visible)?,
                    panel::SHOW => {
                        if self.panel.visible() {
                            self.panel.hide();
                        } else {
                            self.panel.populate(&self.settings, self.demo);
                            self.panel.show();
                        }
                    }
                    panel::LAYOUT => {
                        let (halo_visible, tyres_visible) =
                            (self.settings.halo.visible, self.settings.tyres.visible);
                        self.settings.halo = Default::default();
                        self.settings.tyres = Default::default();
                        self.settings.halo.visible = halo_visible;
                        self.settings.tyres.visible = tyres_visible;
                        default_layout(&mut self.settings);
                        self.apply_layouts()?;
                        self.sync_layout()?;
                    }
                    panel::RESET => {
                        let mut defaults = Settings::default();
                        default_layout(&mut defaults);
                        defaults.halo_profiles = self.settings.halo_profiles.clone();
                        defaults.active_halo_profile = self.settings.active_halo_profile.clone();
                        self.panel.populate(&defaults, false);
                        self.panel.result("Valori iniziali caricati nel pannello. Applica e salva per confermarli.");
                    }
                    panel::EXPORT => {
                        let path = self
                            .config
                            .parent()
                            .unwrap_or(std::path::Path::new("."))
                            .join("diagnostics.json");
                        self.export(&path)?;
                        self.panel
                            .result(&format!("Diagnostica: {}", path.display()));
                    }
                    panel::EXIT => self.quit = true,
                    panel::CYCLE => self.cycle_tyres()?,
                    panel::TOGGLE_HALO | panel::TOGGLE_TYRES => {
                        let index = usize::from(command == panel::TOGGLE_TYRES);
                        let layout = if index == 0 {
                            &mut self.settings.halo
                        } else {
                            &mut self.settings.tyres
                        };
                        layout.visible = !layout.visible;
                        self.apply_layouts()?;
                        self.sync_layout()?;
                    }
                    panel::TOGGLE_DEMO => {
                        self.demo = !self.demo;
                        self.receiver.source(&self.settings, self.demo)?;
                        self.snapshot = Snapshot::default();
                        self.panel.check(panel::DEMO, self.demo);
                        self.dirty = true;
                    }
                    _ => (),
                }
            }
        }
        Ok(())
    }

    fn cycle_tyres(&mut self) -> Result<(), String> {
        self.settings.tyre_mode = self.settings.tyre_mode.next();
        self.settings.save(&self.config)?;
        self.panel.populate(&self.settings, self.demo);
        self.dirty = true;
        Ok(())
    }

    fn update_status(&mut self, now: Instant) {
        self.usage.update();
        self.peak_memory = self.peak_memory.max(self.usage.memory);
        if self.first_memory == 0 {
            self.first_memory = self.usage.memory;
        }
        let elapsed = now.duration_since(self.last_status).as_secs_f64();
        for i in 0..17 {
            self.rates[i] = self.snapshot.counters.per_packet[i]
                .saturating_sub(self.last_counters[i]) as f64
                / elapsed;
        }
        self.last_counters = self.snapshot.counters.per_packet;
        self.last_status = now;
        if !self.panel.visible() {
            return;
        }
        self.panel.battery_status(&self.snapshot.ers_capacity.map(|c| {
            let source = match c.source {
                crate::telemetry::CapacitySource::GameFiles => "file vettura del gioco",
                crate::telemetry::CapacitySource::Demo => "Demo simulata",
            };
            format!("Capacità ERS automatica: {:.2} MJ · {} · {source}", c.joules / 1_000_000.0, c.year)
        }).unwrap_or_else(|| "Capacità ERS automatica non disponibile: in attesa dati / file vettura compatibili".into()));
        let summary = self.diagnostics.summary(self.usage.memory, self.usage.cpu);
        let format = |v: Option<f64>| v.map(|v| format!("{v:.2}")).unwrap_or_else(|| "n/d".into());
        let c = self.snapshot.counters;
        let adapter = self
            .graphics
            .as_ref()
            .map(|g| g.adapter_name.as_str())
            .unwrap_or("device non disponibile");
        self.panel.diagnostics(&format!(
            "{}  |  {}:{}  |  {}\r\nUDP Hz: Telemetry {:.1} · Status {:.1} · Damage {:.1} · Aero {:.1}   |   Errori {} / formato {} / fuori ordine {}\r\nDraw CPU p95 {} ms · intervallo p95 {} ms · RX→Present p95 {} ms · GPU p95 {} ms\r\nCPU {:.2}% · RAM {:.1} MB · frame {} · ricreazioni {}   |   {}",
            self.snapshot.connection(now).label(), self.settings.bind_ip, self.settings.port, adapter,
            self.rates[6], self.rates[7], self.rates[10], self.rates[16], c.malformed, c.unsupported, c.reordered,
            format(summary.draw_cpu_ms_p95), format(summary.frame_interval_ms_p95),
            format(summary.receive_to_present_submit_ms_p95), format(summary.gpu_ms_p95),
            self.usage.cpu, self.usage.memory as f64 / 1_048_576.0, summary.render_samples, summary.device_recreations,
            self.graphics_error.as_deref().unwrap_or("Latenza di invio, non ottica; nessuna misura FPS del gioco.")));
    }

    fn export(&mut self, path: &std::path::Path) -> Result<(), String> {
        self.usage.update();
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let c = self.snapshot.counters;
        let report = serde_json::json!({
            "application": crate::APPLICATION_NAME, "version": env!("CARGO_PKG_VERSION"),
            "halo_model": self.settings.halo_model,
            "active_halo_profile": self.settings.active_halo_profile,
            "source": if self.demo { "demo" } else { "udp" },
            "connection_state": self.snapshot.connection(Instant::now()).label(),
            "socket_error": self.snapshot.socket_error,
            "source_epoch": self.snapshot.source_epoch,
            "ers_capacity": self.snapshot.ers_capacity.map(|c| serde_json::json!({
                "joules": c.joules, "year": c.year,
                "source": match c.source { crate::telemetry::CapacitySource::GameFiles => "game_files", crate::telemetry::CapacitySource::Demo => "demo" }
            })),
            "ers_energy_joules": self.snapshot.status.map(|s| s.value.ers_joules),
            "ers_percent": self.snapshot.status.and_then(|s| crate::telemetry::ers_percent(
                s.value.ers_joules, self.snapshot.ers_capacity.map(|c| c.joules))),
            "overlays_visible": self.overlays_visible,
            "rear_view_hidden": self.rear_hidden,
            "rear_view_enabled": self.settings.hide_halo_rear_view,
            "controller": { "raw_registered": self.rear_input.raw_registered,
                "raw_samples": self.rear_input.raw_samples, "source": self.rear_input.source,
                "right_stick_down": self.rear_input.last_axis },
            "elapsed_seconds": self.start.elapsed().as_secs_f64(),
            "adapter": self.graphics.as_ref().map(|g| &g.adapter_name),
            "fps_limit_hz": self.settings.fps.hz(self.monitor_hz),
            "metrics": self.diagnostics.summary(self.usage.memory, self.usage.average_cpu()),
            "process_cpu_percent_last_sample": self.usage.cpu,
            "memory_first_sample_bytes": self.first_memory, "memory_peak_sample_bytes": self.peak_memory,
            "network": { "received": c.received, "accepted": c.accepted, "malformed": c.malformed,
                "unsupported": c.unsupported, "reordered": c.reordered, "duplicates": c.duplicates,
                "sessions": c.sessions, "flashbacks": c.flashbacks, "restarts": c.restarts,
                "per_packet": c.per_packet },
            "window_checks": self.overlays.iter().map(|o| {
                let style = unsafe { GetWindowLongPtrW(o.hwnd, GWL_EXSTYLE) } as u32;
                let r = overlay::window_rect(o.hwnd);
                serde_json::json!({ "no_activate": style & WS_EX_NOACTIVATE.0 != 0,
                    "transparent_style": style & WS_EX_TRANSPARENT.0 != 0,
                    "topmost": style & WS_EX_TOPMOST.0 != 0, "visible": unsafe { IsWindowVisible(o.hwnd) }.as_bool(), "width": r.right-r.left, "height": r.bottom-r.top })
            }).collect::<Vec<_>>(),
        });
        std::fs::write(
            path,
            serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())
    }
}

struct ProcessUsage {
    last: Instant,
    ticks: u64,
    cpu: f64,
    memory: u64,
    start: Instant,
    initial_ticks: u64,
}
impl ProcessUsage {
    fn new() -> Self {
        let now = Instant::now();
        let ticks = process_ticks();
        Self {
            last: now,
            ticks,
            cpu: 0.0,
            memory: 0,
            start: now,
            initial_ticks: ticks,
        }
    }
    fn average_cpu(&self) -> f64 {
        process_ticks().saturating_sub(self.initial_ticks) as f64
            / 10_000_000.0
            / self.start.elapsed().as_secs_f64()
            / std::thread::available_parallelism()
                .map(|v| v.get())
                .unwrap_or(1) as f64
            * 100.0
    }
    fn update(&mut self) {
        let now = Instant::now();
        let ticks = process_ticks();
        let elapsed = now.duration_since(self.last).as_secs_f64();
        if elapsed >= 0.5 {
            self.cpu = ticks.saturating_sub(self.ticks) as f64
                / 10_000_000.0
                / elapsed
                / std::thread::available_parallelism()
                    .map(|v| v.get())
                    .unwrap_or(1) as f64
                * 100.0;
            self.ticks = ticks;
            self.last = now;
        }
        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        if unsafe {
            GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters,
                size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
        }
        .is_ok()
        {
            self.memory = counters.WorkingSetSize as u64;
        }
    }
}

fn process_ticks() -> u64 {
    let (mut created, mut exit, mut kernel, mut user) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    if unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut created,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    }
    .is_err()
    {
        return 0;
    }
    let ticks = |v: FILETIME| (v.dwHighDateTime as u64) << 32 | v.dwLowDateTime as u64;
    ticks(kernel) + ticks(user)
}

pub fn show_error(message: &str) {
    let text = overlay::wide(message);
    unsafe {
        MessageBoxW(
            None,
            PCWSTR(text.as_ptr()),
            w!("F1 Halo Telemetry"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}
