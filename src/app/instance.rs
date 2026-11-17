use windows::{
    Win32::{
        Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE},
        System::Threading::CreateMutexW,
    },
    core::{PCWSTR, Result},
};

pub(super) struct SingleInstance(HANDLE);

impl SingleInstance {
    pub(super) fn acquire(name: PCWSTR) -> Result<Option<Self>> {
        let instance = Self(unsafe { CreateMutexW(None, false, name)? });
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            Ok(None)
        } else {
            Ok(Some(instance))
        }
    }
}

impl Drop for SingleInstance {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_duplicates_and_allows_restart() {
        let name = crate::overlay::wide(&format!(
            "Local\\F1HaloTelemetry.SingleInstance.Test.{}",
            std::process::id()
        ));
        let name = PCWSTR(name.as_ptr());
        let first = SingleInstance::acquire(name).unwrap().unwrap();
        assert!(SingleInstance::acquire(name).unwrap().is_none());
        assert!(SingleInstance::acquire(name).unwrap().is_none());
        drop(first);
        assert!(SingleInstance::acquire(name).unwrap().is_some());
    }
}
