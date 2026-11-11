use windows::{
    Win32::{System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*},
    core::{PCWSTR, Result},
};

// Win32 represents an integer resource ID as a pointer-sized ordinal.
pub const ICON_RESOURCE: PCWSTR = PCWSTR(std::ptr::without_provenance(1));

pub struct Icon(pub HICON);

impl Icon {
    pub fn load(size: i32) -> Result<Self> {
        let instance = unsafe { GetModuleHandleW(None)? };
        let image = unsafe {
            LoadImageW(
                Some(instance.into()),
                ICON_RESOURCE,
                IMAGE_ICON,
                size,
                size,
                LR_DEFAULTCOLOR,
            )?
        };
        Ok(Self(HICON(image.0)))
    }
}

impl Drop for Icon {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyIcon(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;
    use windows::Win32::Graphics::Gdi::{BITMAP, DeleteObject, GetObjectW};

    #[test]
    fn embedded_logo_loads_at_windows_icon_sizes() {
        for size in [16, 20, 24, 32, 40, 48, 64, 96, 128, 256] {
            let icon = Icon::load(size).unwrap();
            let mut info = ICONINFO::default();
            unsafe { GetIconInfo(icon.0, &mut info).unwrap() };
            let mut bitmap = BITMAP::default();
            let copied = unsafe {
                GetObjectW(
                    info.hbmColor.into(),
                    size_of::<BITMAP>() as i32,
                    Some((&mut bitmap as *mut BITMAP).cast()),
                )
            };
            unsafe {
                let _ = DeleteObject(info.hbmColor.into());
                let _ = DeleteObject(info.hbmMask.into());
            }
            assert_ne!(copied, 0);
            assert_eq!((bitmap.bmWidth, bitmap.bmHeight), (size, size));
            assert_eq!(bitmap.bmBitsPixel, 32);
        }
    }
}
