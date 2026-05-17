// Single-instance behavior: kill any prior shadowprompt.exe instances (other
// than this process) before acquiring our own named mutex. Re-running the
// daemon always wins; the old instance dies, the new one takes over.

use std::panic;
use std::thread::sleep;
use std::time::Duration;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcessId, OpenProcess, TerminateProcess, PROCESS_TERMINATE,
};

const MUTEX_NAME: &str = "Global\\ShadowPrompt_v2_SingleInstance";

pub struct InstanceLock {
    _handle: HANDLE,
}

pub fn acquire_single_instance() -> anyhow::Result<InstanceLock> {
    // Kill any other shadowprompt.exe instances first so we always win.
    let killed = kill_other_instances();
    if killed > 0 {
        log::info!("startup: terminated {killed} prior shadowprompt instance(s)");
        // Brief pause so the OS releases the prior mutex.
        sleep(Duration::from_millis(400));
    }

    let name: Vec<u16> = MUTEX_NAME.encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe { CreateMutexW(None, true, PCWSTR(name.as_ptr())) }?;
    let err = unsafe { GetLastError() };
    if err == ERROR_ALREADY_EXISTS {
        // Mutex still held by something we couldn't kill (e.g. SYSTEM-level
        // process). Bail explicitly rather than silently double-running.
        anyhow::bail!(
            "named mutex {MUTEX_NAME} still held after kill sweep; another \
             instance is running and we lack permission to terminate it"
        );
    }
    Ok(InstanceLock { _handle: handle })
}

fn kill_other_instances() -> u32 {
    let self_pid = unsafe { GetCurrentProcessId() };
    let mut killed = 0u32;
    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return 0,
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut entry).is_ok() {
            loop {
                let name_len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);
                if name.eq_ignore_ascii_case("shadowprompt.exe")
                    && entry.th32ProcessID != self_pid
                {
                    if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, entry.th32ProcessID) {
                        if TerminateProcess(h, 0).is_ok() {
                            killed += 1;
                        }
                        let _ = CloseHandle(h);
                    }
                }
                if Process32NextW(snap, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    killed
}

pub fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        log::error!("panic: {info}");
        if let Some(loc) = info.location() {
            log::error!("  at {}:{}", loc.file(), loc.line());
        }
    }));
}
