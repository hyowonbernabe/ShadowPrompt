// Keyboard-hook watchdog. Windows silently removes a low-level hook whose callback is ever too
// slow (and some security software unhooks others); when that happens every hotkey dies with no
// error anywhere. This notices and restarts the app.
//
// How: if Windows saw user input (GetLastInputInfo) that our hook didn't, while a normal
// (non-elevated) window is in front, the hook may be dead — so we send ourselves one F24 key
// press (no keyboard has F24; the hook swallows it, so no app ever sees it) and wait for the hook
// to report it. No report → the hook is gone → restart via the same path as the restart hotkey.
//
// Skipped while an elevated window (Task Manager, an admin terminal, some exam browsers) is in
// front: a non-admin process's hook legitimately doesn't see that input, and an injected probe
// is blocked there too, so it would look dead when it isn't.
//
// ponytail: the restart rate is limited by MIN_UPTIME (at most one per minute) rather than a
// persisted attempt counter — a hook that can never be installed would restart once a minute.
// Persist a counter in data/ if that ever happens in practice.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// The probe key, as rdev reports it (VK_F24 = 0x87).
pub const PROBE_KEY: rdev::Key = rdev::Key::Unknown(0x87);

static LAST_HOOK_TICK: AtomicU32 = AtomicU32::new(0);
static PROBE_SEEN: AtomicBool = AtomicBool::new(false);

const CHECK_EVERY: Duration = Duration::from_secs(5);
/// Input the hook missed by more than this counts as suspicious.
const MISSED_INPUT_MS: i32 = 3000;
const MIN_UPTIME: Duration = Duration::from_secs(60);
const PROBE_WAIT: Duration = Duration::from_secs(1);
const PROBE_COOLDOWN: Duration = Duration::from_secs(30);

/// Called by the hook for every event it sees.
pub fn note_hook_event() {
    if simulating_dead_hook() {
        return;
    }
    LAST_HOOK_TICK.store(tick(), Ordering::SeqCst);
}

/// Called by the hook when the probe key arrives.
pub fn note_probe_seen() {
    if simulating_dead_hook() {
        return;
    }
    PROBE_SEEN.store(true, Ordering::SeqCst);
}

/// Debug builds only: with SHADOWPROMPT_SIMULATE_DEAD_HOOK set, the watchdog's view of the hook
/// goes silent after 65 s (input itself keeps working) — the only way to exercise the restart
/// path live, since a real hook removal can't be triggered on demand.
#[cfg(feature = "debug")]
fn simulating_dead_hook() -> bool {
    static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    std::env::var_os("SHADOWPROMPT_SIMULATE_DEAD_HOOK").is_some()
        && START.get_or_init(Instant::now).elapsed() > Duration::from_secs(65)
}

#[cfg(not(feature = "debug"))]
fn simulating_dead_hook() -> bool {
    false
}

pub fn spawn() -> std::io::Result<()> {
    note_hook_event();
    std::thread::Builder::new().name("hook-watchdog".into()).spawn(run)?;
    Ok(())
}

fn run() {
    let started = Instant::now();
    let mut last_probe: Option<Instant> = None;
    loop {
        std::thread::sleep(CHECK_EVERY);
        if started.elapsed() < MIN_UPTIME || last_probe.is_some_and(|t| t.elapsed() < PROBE_COOLDOWN) {
            continue;
        }
        let Some(system) = last_input_tick() else { continue };
        if !hook_missed_input(system, LAST_HOOK_TICK.load(Ordering::SeqCst)) || foreground_is_elevated_or_unknown() {
            continue;
        }
        last_probe = Some(Instant::now());
        PROBE_SEEN.store(false, Ordering::SeqCst);
        if send_probe() != 2 {
            continue; // couldn't inject (e.g. secure desktop) — no conclusion possible
        }
        std::thread::sleep(PROBE_WAIT);
        if PROBE_SEEN.load(Ordering::SeqCst) {
            log::debug!("hook watchdog: probe seen, hook alive");
            continue;
        }
        log::error!("hook watchdog: keyboard hook stopped receiving input; restarting");
        if let Err(e) = crate::lifecycle::self_restart::execute() {
            log::error!("hook watchdog: restart failed: {e}");
        }
    }
}

/// Whether Windows' last-input tick is meaningfully newer than the hook's (tick counters wrap
/// every ~49 days, hence the wrapping, signed comparison).
fn hook_missed_input(system_tick: u32, hook_tick: u32) -> bool {
    (system_tick.wrapping_sub(hook_tick) as i32) > MISSED_INPUT_MS
}

fn tick() -> u32 {
    // SAFETY: plain Win32 query.
    unsafe { windows::Win32::System::SystemInformation::GetTickCount() }
}

fn last_input_tick() -> Option<u32> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    // SAFETY: plain Win32 query into a correctly sized local struct.
    unsafe { GetLastInputInfo(&mut info) }.as_bool().then_some(info.dwTime)
}

/// True if the foreground window belongs to an elevated process, or if that can't be told
/// (no foreground window, access denied — which is itself what an elevated process looks like
/// from a non-admin one). Conservative on purpose: "unknown" never triggers a restart.
fn foreground_is_elevated_or_unknown() -> bool {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    // SAFETY: Win32 queries on handles we open here and always close before returning.
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return true;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return true;
        }
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return true };
        let mut token = HANDLE::default();
        let elevated = if OpenProcessToken(process, TOKEN_QUERY, &mut token).is_ok() {
            let mut elevation = TOKEN_ELEVATION::default();
            let mut len = 0u32;
            let ok = GetTokenInformation(
                token,
                TokenElevation,
                Some(&mut elevation as *mut _ as *mut core::ffi::c_void),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut len,
            )
            .is_ok();
            let _ = CloseHandle(token);
            !ok || elevation.TokenIsElevated != 0
        } else {
            true
        };
        let _ = CloseHandle(process);
        elevated
    }
}

/// One F24 press + release. Returns how many events were injected.
fn send_probe() -> u32 {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_F24,
    };
    let key = |flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_F24, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
    };
    let inputs = [key(KEYBD_EVENT_FLAGS(0)), key(KEYEVENTF_KEYUP)];
    // SAFETY: plain Win32 call with a correctly sized, fully initialised INPUT array.
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missed_input_comparison_handles_order_and_wraparound() {
        assert!(hook_missed_input(10_000, 5_000), "system 5s ahead of hook");
        assert!(!hook_missed_input(6_000, 5_000), "within the grace window");
        assert!(!hook_missed_input(5_000, 9_000), "hook newer than system");
        assert!(hook_missed_input(4_000, u32::MAX - 1_000), "tick counter wrapped");
    }
}
