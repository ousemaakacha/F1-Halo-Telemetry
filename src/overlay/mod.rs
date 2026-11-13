use crate::{hud, settings::Layout};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    mem::size_of,
    sync::Arc,
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::{LibraryLoader::GetModuleHandleW, Threading::*},
        UI::{HiDpi::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::{Result, w},
};

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Dirty,
    LayoutChanged(usize),
    Wheel(usize, i16, bool),
    CycleTyres,
    ContextMenu,
    Command(u16),
    Hotkey(i32),
    ToggleEdit,
    ClosePanel,
    DisplayChanged,
    PanelDpi(u32, RECT),
    PanelScroll(bool, u16, u16),
    PanelWheel(i16),
    ControllerInput(usize, crate::input::rear_view::ControllerState),
    ControllerRemoved(usize),
    Quit,
}

thread_local! { static ACTIONS: RefCell<VecDeque<Action>> = const { RefCell::new(VecDeque::new()) }; }
pub fn push(action: Action) {
    ACTIONS.with(|v| v.borrow_mut().push_back(action));
}
pub fn pop() -> Option<Action> {
    ACTIONS.with(|v| v.borrow_mut().pop_front())
}

pub fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

pub struct Wake {
    handle: HANDLE,
}
// SAFETY: Windows kernel events support concurrent SetEvent. Arc owns the handle
// until the UDP thread is joined, and no thread accesses mutable Rust data here.
unsafe impl Send for Wake {}
unsafe impl Sync for Wake {}
impl Wake {
    pub fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(Self {
            handle: unsafe { CreateEventW(None, false, false, None)? },
        }))
    }
    pub fn set(&self) {
        unsafe {
            let _ = SetEvent(self.handle);
        }
    }
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
}
impl Drop for Wake {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}

pub struct FrameTimer {
    handle: HANDLE,
}
impl FrameTimer {
    pub fn new() -> Result<Self> {
        let handle = unsafe {
            CreateWaitableTimerExW(
                None,
                None,
                CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
                TIMER_ALL_ACCESS.0,
            )
            .or_else(|_| CreateWaitableTimerExW(None, None, 0, TIMER_ALL_ACCESS.0))?
        };
        Ok(Self { handle })
    }
    pub fn handle(&self) -> HANDLE {
        self.handle
    }
    pub fn arm(&self, duration: Duration) -> Result<()> {
        let units = (duration.as_nanos() / 100).clamp(1, i64::MAX as u128) as i64;
        unsafe { SetWaitableTimer(self.handle, &-units, 0, None, None, false) }
    }
}
impl Drop for FrameTimer {
    fn drop(&mut self) {
        unsafe {
            let _ = CancelWaitableTimer(self.handle);
            let _ = CloseHandle(self.handle);
        }
    }
}

#[derive(Clone, Copy)]
struct Drag {
    cursor: POINT,
    rect: RECT,
    corner: u8,
}
struct WindowData {
    index: usize,
    edit: Cell<bool>,
    dpi: Cell<u32>,
    drag: Cell<Option<Drag>>,
    base: (f32, f32),
}

pub struct Overlay {
    pub hwnd: HWND,
    data: Box<WindowData>,
}

impl Overlay {
    pub fn register() -> Result<()> {
        let instance = unsafe { GetModuleHandleW(None)? };
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: w!("F1HaloOverlay"),
            hCursor: unsafe { LoadCursorW(None, IDC_ARROW)? },
            hIcon: unsafe { LoadIconW(Some(instance.into()), crate::branding::ICON_RESOURCE)? },
            style: CS_DBLCLKS,
            ..Default::default()
        };
        if unsafe { RegisterClassW(&class) } == 0 {
            return Err(windows::core::Error::from_thread());
        }
        Ok(())
    }

    pub fn new(index: usize, layout: Layout) -> Result<Self> {
        let base = if index == 0 {
            (hud::halo::WIDTH, hud::halo::HEIGHT)
        } else {
            (hud::tyres::WIDTH, hud::tyres::HEIGHT)
        };
        let dpi = monitor_dpi(POINT {
            x: layout.x,
            y: layout.y,
        });
        let data = Box::new(WindowData {
            index,
            edit: Cell::new(false),
            dpi: Cell::new(dpi),
            drag: Cell::new(None),
            base,
        });
        let title = if index == 0 {
            w!("F1 Halo Telemetry — Halo")
        } else {
            w!("F1 Halo Telemetry — Tyres")
        };
        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
                w!("F1HaloOverlay"),
                title,
                WS_POPUP,
                layout.x,
                layout.y,
                (base.0 * layout.scale * dpi as f32 / 96.0) as i32,
                (base.1 * layout.scale * dpi as f32 / 96.0) as i32,
                None,
                None,
                Some(GetModuleHandleW(None)?.into()),
                Some((&*data as *const WindowData).cast()),
            )?
        };
        let overlay = Self { hwnd, data };
        overlay.set_edit(false)?;
        overlay.apply_layout(layout)?;
        Ok(overlay)
    }

    pub fn set_edit(&self, edit: bool) -> Result<()> {
        self.data.edit.set(edit);
        // Layered + TRANSPARENT provides cross-process mouse pass-through. DComp
        // owns the GPU surface; SetLayeredWindowAttributes never uploads pixels.
        unsafe {
            let mut style = WINDOW_EX_STYLE(GetWindowLongPtrW(self.hwnd, GWL_EXSTYLE) as u32);
            if edit {
                style &= !(WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW);
                style |= WS_EX_APPWINDOW;
            } else {
                style &= !WS_EX_APPWINDOW;
                style |= WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
            }
            SetWindowLongPtrW(self.hwnd, GWL_EXSTYLE, style.0 as isize);
            if !edit {
                SetLayeredWindowAttributes(self.hwnd, COLORREF(0), 255, LWA_ALPHA)?;
            }
            SetWindowPos(
                self.hwnd,
                None,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER | SWP_FRAMECHANGED,
            )?;
        }
        Ok(())
    }

    pub fn apply_layout(&self, mut layout: Layout) -> Result<()> {
        let dpi = monitor_dpi(POINT {
            x: layout.x,
            y: layout.y,
        });
        self.data.dpi.set(dpi);
        let work = work_area(POINT {
            x: layout.x,
            y: layout.y,
        });
        let ratio = dpi as f32 / 96.0;
        let max_scale = ((work.right - work.left) as f32 / (self.data.base.0 * ratio))
            .min((work.bottom - work.top) as f32 / (self.data.base.1 * ratio));
        layout.scale = layout.scale.min(max_scale);
        let width = (self.data.base.0 * layout.scale * ratio).round().max(1.0) as i32;
        let height = (self.data.base.1 * layout.scale * ratio).round().max(1.0) as i32;
        let x = layout
            .x
            .clamp(work.left, (work.right - width).max(work.left));
        let y = layout
            .y
            .clamp(work.top, (work.bottom - height).max(work.top));
        unsafe {
            SetWindowPos(
                self.hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_NOZORDER,
            )?;
            let _ = ShowWindow(
                self.hwnd,
                if layout.visible {
                    SW_SHOWNOACTIVATE
                } else {
                    SW_HIDE
                },
            );
            if layout.visible && !IsWindowVisible(self.hwnd).as_bool() {
                let _ = ShowWindow(self.hwnd, SW_SHOWNOACTIVATE);
            }
        }
        Ok(())
    }

    pub fn layout(&self, old: Layout) -> Layout {
        let r = window_rect(self.hwnd);
        Layout {
            x: r.left,
            y: r.top,
            scale: ((r.right - r.left) as f32
                / self.data.base.0
                / (self.data.dpi.get() as f32 / 96.0))
                .clamp(0.35, 3.0),
            ..old
        }
    }

    pub fn size(&self) -> (u32, u32) {
        let r = window_rect(self.hwnd);
        (
            (r.right - r.left).max(1) as u32,
            (r.bottom - r.top).max(1) as u32,
        )
    }
}

impl Drop for Overlay {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

pub fn window_rect(hwnd: HWND) -> RECT {
    let mut rect = RECT::default();
    unsafe {
        let _ = GetWindowRect(hwnd, &mut rect);
    }
    rect
}

pub fn work_area(point: POINT) -> RECT {
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool() {
        info.rcWork
    } else {
        RECT {
            left: 0,
            top: 0,
            right: 1920,
            bottom: 1080,
        }
    }
}

pub fn monitor_dpi(point: POINT) -> u32 {
    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let (mut x, mut y) = (96, 96);
    unsafe {
        let _ = GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x, &mut y);
    }
    x.max(96)
}

pub fn monitor_refresh(hwnd: HWND) -> u32 {
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFOEXW {
        monitorInfo: MONITORINFO {
            cbSize: size_of::<MONITORINFOEXW>() as u32,
            ..Default::default()
        },
        ..Default::default()
    };
    let mut mode = DEVMODEW {
        dmSize: size_of::<DEVMODEW>() as u16,
        ..Default::default()
    };
    unsafe {
        if GetMonitorInfoW(monitor, &mut info.monitorInfo).as_bool()
            && EnumDisplaySettingsW(
                windows::core::PCWSTR(info.szDevice.as_ptr()),
                ENUM_CURRENT_SETTINGS,
                &mut mode,
            )
            .as_bool()
        {
            return mode.dmDisplayFrequency.clamp(30, 360);
        }
    }
    60
}

fn corner_at(r: RECT, p: POINT, dpi: u32) -> u8 {
    let edge = (14 * dpi / 96).max(10) as i32;
    let left = p.x - r.left < edge;
    let right = r.right - p.x < edge;
    let top = p.y - r.top < edge;
    let bottom = r.bottom - p.y < edge;
    match (left, right, top, bottom) {
        (true, _, true, _) => 1,
        (_, true, true, _) => 2,
        (true, _, _, true) => 3,
        (_, true, _, true) => 4,
        _ => 0,
    }
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: CreateWindow receives a stable Box pointer. It outlives HWND and
    // only interior-mutable Cells are changed in reentrant window callbacks.
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(lparam.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowData;
        if pointer.is_null() {
            return DefWindowProcW(hwnd, message, wparam, lparam);
        }
        let data = &*pointer;
        match message {
            WM_MOUSEACTIVATE => {
                return LRESULT(if data.edit.get() {
                    MA_ACTIVATE
                } else {
                    MA_NOACTIVATE
                } as isize);
            }
            WM_KEYDOWN if data.edit.get() && wparam.0 == VK_ESCAPE.0 as usize => {
                push(Action::ToggleEdit);
                return LRESULT(0);
            }
            WM_ERASEBKGND => return LRESULT(1),
            WM_PAINT => {
                // Complete the paint cycle even though DirectComposition draws the HUD.
                let mut paint = PAINTSTRUCT::default();
                let _ = BeginPaint(hwnd, &mut paint);
                let _ = EndPaint(hwnd, &paint);
                push(Action::Dirty);
                return LRESULT(0);
            }
            WM_NCHITTEST if !data.edit.get() => return LRESULT(HTTRANSPARENT as isize),
            WM_LBUTTONDOWN if data.edit.get() => {
                // Use the position stored with this input message. GetCursorPos
                // can already point to the end of a fast, queued drag gesture.
                let cursor = message_point();
                let r = window_rect(hwnd);
                let corner = corner_at(r, cursor, data.dpi.get());
                data.drag.set(Some(Drag {
                    cursor,
                    rect: r,
                    corner,
                }));
                SetCapture(hwnd);
                return LRESULT(0);
            }
            WM_MOUSEMOVE if data.edit.get() => {
                let cursor = message_point();
                if let Some(drag) = data.drag.get() {
                    let dx = cursor.x - drag.cursor.x;
                    let dy = cursor.y - drag.cursor.y;
                    let r = drag.rect;
                    if drag.corner == 0 {
                        let _ = SetWindowPos(
                            hwnd,
                            None,
                            r.left + dx,
                            r.top + dy,
                            0,
                            0,
                            SWP_NOACTIVATE | SWP_NOSIZE | SWP_NOZORDER,
                        );
                    } else {
                        let left = matches!(drag.corner, 1 | 3);
                        let top = matches!(drag.corner, 1 | 2);
                        let dw = if left { -dx } else { dx };
                        let dh = if top { -dy } else { dy };
                        let width_scale = (r.right - r.left + dw) as f32 / data.base.0;
                        let height_scale = (r.bottom - r.top + dh) as f32 / data.base.1;
                        let old_scale = (r.right - r.left) as f32 / data.base.0;
                        let ratio = data.dpi.get() as f32 / 96.0;
                        let chosen = if (width_scale - old_scale).abs()
                            >= (height_scale - old_scale).abs()
                        {
                            width_scale
                        } else {
                            height_scale
                        };
                        let scale = chosen.clamp(0.35 * ratio, 3.0 * ratio);
                        let width = (data.base.0 * scale).round() as i32;
                        let height = (data.base.1 * scale).round() as i32;
                        let x = if left { r.right - width } else { r.left };
                        let y = if top { r.bottom - height } else { r.top };
                        let _ = SetWindowPos(
                            hwnd,
                            None,
                            x,
                            y,
                            width,
                            height,
                            SWP_NOACTIVATE | SWP_NOZORDER,
                        );
                    }
                    push(Action::Dirty);
                } else {
                    let corner = corner_at(window_rect(hwnd), cursor, data.dpi.get());
                    let shape = match corner {
                        1 | 4 => IDC_SIZENWSE,
                        2 | 3 => IDC_SIZENESW,
                        _ => IDC_SIZEALL,
                    };
                    if let Ok(icon) = LoadCursorW(None, shape) {
                        SetCursor(Some(icon));
                    }
                }
                return LRESULT(0);
            }
            WM_LBUTTONUP | WM_CAPTURECHANGED => {
                if data.drag.replace(None).is_some() {
                    if message == WM_LBUTTONUP {
                        let _ = ReleaseCapture();
                    }
                    push(Action::LayoutChanged(data.index));
                }
                return LRESULT(0);
            }
            WM_MOUSEWHEEL if data.edit.get() => {
                let delta = ((wparam.0 >> 16) & 0xffff) as i16;
                push(Action::Wheel(
                    data.index,
                    delta,
                    GetKeyState(VK_SHIFT.0 as i32) < 0,
                ));
                return LRESULT(0);
            }
            WM_LBUTTONDBLCLK if data.edit.get() && data.index == 1 => {
                push(Action::CycleTyres);
                return LRESULT(0);
            }
            WM_RBUTTONUP if data.edit.get() => {
                push(Action::ContextMenu);
                return LRESULT(0);
            }
            WM_DPICHANGED => {
                data.dpi.set((wparam.0 & 0xffff) as u32);
                let r = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOACTIVATE | SWP_NOZORDER,
                );
                push(Action::LayoutChanged(data.index));
                return LRESULT(0);
            }
            WM_DISPLAYCHANGE => push(Action::DisplayChanged),
            WM_SIZE => push(Action::Dirty),
            WM_CLOSE => {
                push(Action::Quit);
                return LRESULT(0);
            }
            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            _ => (),
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

fn message_point() -> POINT {
    let value = unsafe { GetMessagePos() };
    POINT {
        x: (value as u16 as i16) as i32,
        y: ((value >> 16) as u16 as i16) as i32,
    }
}
