// Single-instance lock via named Windows mutex. Panic hook routes panics to log.

use std::panic;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

const MUTEX_NAME: &str = "Global\\ShadowPrompt_v2_SingleInstance";

pub struct InstanceLock {
    _handle: HANDLE,
}

pub fn acquire_single_instance() -> anyhow::Result<InstanceLock> {
    let name: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe { CreateMutexW(None, true, PCWSTR(name.as_ptr())) }?;
    let err = unsafe { GetLastError() };
    if err == ERROR_ALREADY_EXISTS {
        anyhow::bail!("another shadowprompt instance is already running");
    }
    Ok(InstanceLock { _handle: handle })
}

pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        log::error!("panic: {info}");
        if let Some(loc) = info.location() {
            log::error!("  at {}:{}", loc.file(), loc.line());
        }
    }));
}
