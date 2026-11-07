//! Manual cross-process input test. Start this window underneath the race HUD,
//! click through visible HUD pixels and verify that its click count increases.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() -> windows::core::Result<()> {
    use std::time::Duration;
    use windows::{
        Win32::{
            Graphics::Gdi::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*,
        },
        core::w,
    };
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(probe_proc),
            hInstance: instance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(DKGRAY_BRUSH).0),
            lpszClassName: w!("F1HaloInputProbe"),
            ..Default::default()
        };
        RegisterClassW(&class);
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("F1HaloInputProbe"),
            w!("F1 Halo input probe · Clicks: 0"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            1440,
            1030,
            None,
            None,
            Some(instance.into()),
            None,
        )?;
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetForegroundWindow(hwnd);
        let seconds = std::env::args()
            .nth(1)
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(180);
        SetTimer(
            Some(hwnd),
            1,
            Duration::from_secs(seconds)
                .as_millis()
                .min(u32::MAX as u128) as u32,
            None,
        );
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    Ok(())
}

#[cfg(windows)]
unsafe extern "system" fn probe_proc(
    hwnd: windows::Win32::Foundation::HWND,
    message: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use std::sync::atomic::{AtomicU32, Ordering};
    use windows::{
        Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
        core::PCWSTR,
    };
    static CLICKS: AtomicU32 = AtomicU32::new(0);
    unsafe {
        match message {
            WM_LBUTTONDOWN => {
                let count = CLICKS.fetch_add(1, Ordering::Relaxed) + 1;
                let title: Vec<u16> = format!("F1 Halo input probe · Clicks: {count}")
                    .encode_utf16()
                    .chain(Some(0))
                    .collect();
                let _ = SetWindowTextW(hwnd, PCWSTR(title.as_ptr()));
                return LRESULT(0);
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let dc = BeginPaint(hwnd, &mut ps);
                SetTextColor(dc, COLORREF(0x00FFFFFF));
                SetBkMode(dc, TRANSPARENT);
                let mut rect = RECT {
                    left: 40,
                    top: 40,
                    right: 1250,
                    bottom: 180,
                };
                let mut text: Vec<u16> = "TEST INPUT TRA PROCESSI\nRace Mode: fare clic sui pixel visibili degli HUD.\nIl contatore nel titolo deve aumentare e questa finestra deve mantenere il focus.\nCtrl+Shift+H: Edit Mode; la telemetria continua.".encode_utf16().collect();
                DrawTextW(dc, &mut text, &mut rect, DT_LEFT);
                let _ = EndPaint(hwnd, &ps);
                return LRESULT(0);
            }
            WM_TIMER => {
                let _ = DestroyWindow(hwnd);
                return LRESULT(0);
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                return LRESULT(0);
            }
            _ => (),
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Windows only");
}
