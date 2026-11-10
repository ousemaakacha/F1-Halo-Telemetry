use std::{
    cell::RefCell,
    collections::HashMap,
    mem::size_of,
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND},
        System::Threading::{
            OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
            QueryFullProcessImageNameW,
        },
        UI::{
            Input::{XboxController::*, *},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
    core::{PWSTR, Result},
};

pub const POLL_INTERVAL: Duration = Duration::from_millis(16);
const RAW_TTL: Duration = Duration::from_millis(500);

thread_local! {
    static DEVICES: RefCell<HashMap<usize, bool>> = RefCell::new(HashMap::new());
}

fn axes_from_report(report: &[u8]) -> Option<[f32; 2]> {
    let y = match report.first()? {
        0x01 if matches!(report.len(), 10 | 64 | 78) => 4,
        0x31 if report.len() == 78 => 5,
        _ => return None,
    };
    Some([report[y - 1], report[y]].map(|value| {
        let centered = value as f32 - 128.0;
        centered / if value >= 128 { 127.0 } else { 128.0 }
    }))
}

fn latest_axes(data: &[u8], stride: usize, count: usize) -> Option<[f32; 2]> {
    if stride == 0 || count == 0 || stride.checked_mul(count)? > data.len() {
        return None;
    }
    data[..stride * count]
        .chunks_exact(stride)
        .filter_map(axes_from_report)
        .next_back()
}

pub fn held(was_held: bool, eligible: bool, axes: Option<[f32; 2]>) -> bool {
    eligible
        && axes.is_some_and(|[horizontal, down]| {
            let value = horizontal.abs().max(down);
            [horizontal, down]
                .iter()
                .all(|axis| axis.is_finite() && (-1.0..=1.0).contains(axis))
                && if was_held {
                    value > 0.35
                } else {
                    value >= 0.55
                }
        })
}

fn is_dualsense(handle: HANDLE) -> bool {
    let key = handle.0 as usize;
    DEVICES.with(|devices| {
        if let Some(accepted) = devices.borrow().get(&key) {
            return *accepted;
        }
        let mut info = RID_DEVICE_INFO {
            cbSize: size_of::<RID_DEVICE_INFO>() as u32,
            ..Default::default()
        };
        let mut size = info.cbSize;
        // SAFETY: the API receives a correctly sized, initialized output structure.
        let result = unsafe {
            GetRawInputDeviceInfoW(
                Some(handle),
                RIDI_DEVICEINFO,
                Some((&mut info as *mut RID_DEVICE_INFO).cast()),
                &mut size,
            )
        };
        if result == u32::MAX || info.dwType != RIM_TYPEHID {
            return false;
        }
        let hid = unsafe { info.Anonymous.hid };
        let accepted = hid.dwVendorId == 0x054c
            && matches!(hid.dwProductId, 0x0ce6 | 0x0df2)
            && hid.usUsagePage == 1
            && matches!(hid.usUsage, 4 | 5);
        devices.borrow_mut().insert(key, accepted);
        accepted
    })
}

pub fn forget_device(device: usize) {
    DEVICES.with(|devices| {
        devices.borrow_mut().remove(&device);
    });
}

pub fn read_raw(handle: HRAWINPUT) -> Option<(usize, [f32; 2])> {
    let mut storage = [0u64; 128];
    let mut size = size_of::<[u64; 128]>() as u32;
    // Read during WM_INPUT dispatch, before DefWindowProc releases the handle.
    let copied = unsafe {
        GetRawInputData(
            handle,
            RID_INPUT,
            Some(storage.as_mut_ptr().cast()),
            &mut size,
            size_of::<RAWINPUTHEADER>() as u32,
        )
    };
    let offset = size_of::<RAWINPUTHEADER>() + 8;
    if copied == u32::MAX || (copied as usize) < offset || copied as usize > size_of::<[u64; 128]>()
    {
        return None;
    }
    // SAFETY: storage is aligned and initialized; copied header/HID lengths are checked.
    let raw = unsafe { &*storage.as_ptr().cast::<RAWINPUT>() };
    if raw.header.dwType != RIM_TYPEHID.0 || !is_dualsense(raw.header.hDevice) {
        return None;
    }
    let hid = unsafe { raw.data.hid };
    let bytes = unsafe {
        std::slice::from_raw_parts(
            storage.as_ptr().cast::<u8>().add(offset),
            copied as usize - offset,
        )
    };
    latest_axes(bytes, hid.dwSizeHid as usize, hid.dwCount as usize)
        .map(|axes| (raw.header.hDevice.0 as usize, axes))
}

pub struct RearInput {
    raw: Option<(usize, [f32; 2], Instant)>,
    xinput_slot: Option<u32>,
    next_scan: Instant,
    foreground: Option<(HWND, u32, bool)>,
    pub raw_registered: bool,
    pub raw_samples: u64,
    pub last_axes: Option<[f32; 2]>,
    pub source: &'static str,
}

impl Default for RearInput {
    fn default() -> Self {
        Self {
            raw: None,
            xinput_slot: None,
            next_scan: Instant::now(),
            foreground: None,
            raw_registered: false,
            raw_samples: 0,
            last_axes: None,
            source: "none",
        }
    }
}

impl RearInput {
    pub fn register(&mut self, hwnd: HWND) -> Result<()> {
        let devices = [4, 5].map(|usage| RAWINPUTDEVICE {
            usUsagePage: 1,
            usUsage: usage,
            dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
            hwndTarget: hwnd,
        });
        // INPUTSINK shares input with the game; no exclusive capture or suppression.
        unsafe {
            RegisterRawInputDevices(&devices, size_of::<RAWINPUTDEVICE>() as u32)?;
        }
        self.raw_registered = true;
        Ok(())
    }

    pub fn receive(&mut self, device: usize, axes: [f32; 2], now: Instant) {
        self.raw = Some((device, axes, now));
        self.last_axes = Some(axes);
        self.source = "DualSense HID";
        self.raw_samples = self.raw_samples.saturating_add(1);
    }

    pub fn remove(&mut self, device: usize) {
        if self.raw.is_some_and(|(id, _, _)| id == device) {
            self.raw = None;
            self.last_axes = None;
            self.source = "none";
            self.xinput_slot = None;
            self.next_scan = Instant::now() + Duration::from_secs(1);
        }
    }

    fn recent_raw(&self, now: Instant) -> Option<[f32; 2]> {
        self.raw
            .filter(|(_, _, received)| now.saturating_duration_since(*received) <= RAW_TTL)
            .map(|(_, axes, _)| axes)
    }

    pub fn poll(&mut self, now: Instant) -> Option<[f32; 2]> {
        if let Some(axes) = self.recent_raw(now) {
            self.source = "DualSense HID";
            self.last_axes = Some(axes);
            return Some(axes);
        }
        if let Some(slot) = self.xinput_slot {
            if let Some(axes) = xinput_axes(slot) {
                self.source = "XInput / Steam Input";
                self.last_axes = Some(axes);
                return Some(axes);
            }
            self.xinput_slot = None;
        }
        self.source = "none";
        self.last_axes = None;
        if now >= self.next_scan {
            self.next_scan = now + Duration::from_secs(1);
            for slot in 0..4 {
                if let Some(axes) = xinput_axes(slot) {
                    self.xinput_slot = Some(slot);
                    self.source = "XInput / Steam Input";
                    self.last_axes = Some(axes);
                    return Some(axes);
                }
            }
        }
        None
    }

    pub fn game_focused(&mut self) -> bool {
        let hwnd = unsafe { GetForegroundWindow() };
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if let Some((old_hwnd, old_pid, accepted)) = self.foreground {
            if hwnd == old_hwnd && pid == old_pid {
                return accepted;
            }
        }
        let accepted = game_process(pid);
        self.foreground = Some((hwnd, pid, accepted));
        accepted
    }
}

fn axes_from_xinput(gamepad: &XINPUT_GAMEPAD) -> [f32; 2] {
    [
        gamepad.sThumbRX as f32 / 32768.0,
        -(gamepad.sThumbRY as f32) / 32768.0,
    ]
}

fn xinput_axes(slot: u32) -> Option<[f32; 2]> {
    let mut state = XINPUT_STATE::default();
    if unsafe { XInputGetState(slot, &mut state) } != 0 {
        return None;
    }
    Some(axes_from_xinput(&state.Gamepad))
}

fn game_process(pid: u32) -> bool {
    // Only process metadata is queried; game memory and controls are untouched.
    unsafe {
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return false;
        };
        let mut path = [0u16; 1024];
        let mut len = path.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(path.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(process);
        if result.is_err() {
            return false;
        }
        String::from_utf16_lossy(&path[..len as usize])
            .rsplit(['\\', '/'])
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case("F1_25.exe"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_right_stick_usb_and_bluetooth_without_using_clicks_or_left_stick() {
        for (id, length, y) in [(1, 10, 4), (1, 64, 4), (1, 78, 4), (0x31, 78, 5)] {
            let mut report = vec![128; length];
            report[0] = id;
            report[y] = 255;
            assert_eq!(axes_from_report(&report), Some([0.0, 1.0]));
            report[y] = 0;
            assert_eq!(axes_from_report(&report), Some([0.0, -1.0]));
            report[y] = 128;
            for (x, expected) in [(0, -1.0), (255, 1.0)] {
                report[y - 1] = x;
                assert_eq!(axes_from_report(&report), Some([expected, 0.0]));
                assert!(held(false, true, axes_from_report(&report)));
            }
            report[y - 1] = 128;
            report[y - 3] = 255;
            report[y - 2] = 255;
            report[6] = 255;
            report[7] = 255;
            report[8] = 255;
            report[9] = 255;
            if id == 0x31 {
                report[10] = 255;
            }
            assert_eq!(axes_from_report(&report), Some([0.0, 0.0]));
        }
        assert_eq!(axes_from_report(&[]), None);
        assert_eq!(axes_from_report(&[0x31; 77]), None);
        assert_eq!(axes_from_report(&[1; 63]), None);
        assert_eq!(axes_from_report(&[2; 64]), None);
    }

    #[test]
    fn xinput_uses_right_stick_axes_and_preserves_down_direction() {
        for (x, y) in [(i16::MIN, 0), (i16::MAX, 0), (0, i16::MIN)] {
            let gamepad = XINPUT_GAMEPAD {
                sThumbRX: x,
                sThumbRY: y,
                ..Default::default()
            };
            assert!(held(false, true, Some(axes_from_xinput(&gamepad))));
        }
        let gamepad = XINPUT_GAMEPAD {
            sThumbLX: i16::MAX,
            sThumbLY: i16::MIN,
            wButtons: XINPUT_GAMEPAD_RIGHT_THUMB,
            ..Default::default()
        };
        assert_eq!(axes_from_xinput(&gamepad), [0.0, 0.0]);
        assert!(!held(false, true, Some(axes_from_xinput(&gamepad))));
        assert!(!held(
            false,
            true,
            Some(axes_from_xinput(&XINPUT_GAMEPAD {
                sThumbRY: i16::MAX,
                ..Default::default()
            }))
        ));
    }

    #[test]
    fn batched_reports_use_the_release_and_reject_invalid_sizes() {
        let mut reports = [128u8; 20];
        reports[0] = 1;
        reports[3] = 255;
        reports[4] = 255;
        reports[10] = 1;
        assert_eq!(latest_axes(&reports, 10, 2), Some([0.0, 0.0]));
        assert_eq!(latest_axes(&reports, 0, 2), None);
        assert_eq!(latest_axes(&reports, 10, 0), None);
        assert_eq!(latest_axes(&reports, 10, 3), None);
        assert_eq!(latest_axes(&reports, usize::MAX, 2), None);
    }

    #[test]
    fn hold_has_hysteresis_and_clears_on_release_disabled_context_or_lost_input() {
        for direction in [[0.0_f32, 1.0], [-1.0, 0.0], [1.0, 0.0]] {
            let axes = |amount| Some(direction.map(|value| value * amount));
            assert!(!held(false, true, axes(0.1)));
            assert!(held(false, true, axes(0.55)));
            assert!(held(true, true, axes(0.45)));
            assert!(!held(false, true, axes(0.45)));
            assert!(!held(true, true, axes(0.35)));
            assert!(!held(true, true, axes(0.0)));
            assert!(!held(true, false, axes(1.0)));
        }
        assert!(!held(true, true, Some([0.0, -1.0])));
        assert!(!held(true, true, None));
        for invalid in [f32::NAN, f32::INFINITY, -1.1, 1.1] {
            assert!(!held(true, true, Some([invalid, 1.0])));
            assert!(!held(true, true, Some([1.0, invalid])));
        }
    }

    #[test]
    fn side_look_stays_hidden_until_center_and_preserves_hud_preferences() {
        use crate::settings::Settings;

        for visible in [false, true] {
            let mut settings = Settings::default();
            settings.halo.visible = visible;
            let before = serde_json::to_value(&settings).unwrap();
            let mut hidden = false;
            for (axes, expected) in [
                ([-1.0, 0.0], true),
                ([-0.7, 0.7], true),
                ([0.0, 1.0], true),
                ([0.7, 0.7], true),
                ([1.0, 0.0], true),
                ([0.45, 0.0], true),
                ([0.0, 0.0], false),
                ([-0.45, 0.0], false),
            ] {
                hidden = held(hidden, true, Some(axes));
                assert_eq!(hidden, expected);
                let layouts = settings.effective_layouts(true, hidden);
                assert_eq!(layouts[0].visible, visible && !expected);
                assert_eq!(layouts[1].visible, settings.tyres.visible);
            }
            assert_eq!(serde_json::to_value(&settings).unwrap(), before);
        }
    }

    #[test]
    fn stale_or_disconnected_native_input_cannot_leave_halo_hidden() {
        let now = Instant::now();
        let mut input = RearInput::default();
        input.receive(7, [-1.0, 0.0], now);
        assert_eq!(input.recent_raw(now + RAW_TTL), Some([-1.0, 0.0]));
        assert_eq!(
            input.recent_raw(now + RAW_TTL + Duration::from_millis(1)),
            None
        );
        input.remove(8);
        assert_eq!(input.recent_raw(now), Some([-1.0, 0.0]));
        input.remove(7);
        assert_eq!(input.recent_raw(now), None);
    }
}
