// Input layer: global keyboard + mouse hook (rdev) -> InputEvent on channel. Ported from v2 with
// the search-modifier hotkeys removed (design doc §3/§5: the model decides autonomously whether
// to search now, it's a tool call, not a separate forced hotkey) and OCR-region-select renamed to
// screenshot-region-select (OCR itself is gone, design doc §3 — this is now purely "pick a
// rectangle, send it as an image").
//
// Backup hotkeys: every action can also be fired with Caps Lock held, Tab tapped, then the
// action's own key (the letter of its normal combo, e.g. S for screenshot) — for when Ctrl/Shift/
// Alt combos are blocked or broken. A second key (Tab) is required on purpose: people tap Caps
// Lock to capitalise a letter and their fingers overlap, so Caps+letter alone would misfire.
// Tab and the letters pressed in that mode are swallowed (the focused app never sees them), and
// the Caps Lock toggle the chord caused is undone afterwards.

pub mod bindings;
pub mod events;
pub mod parser;
pub mod state_machine;

pub use events::{InputEvent, RegionPurpose};

use crate::config::schema::HotkeysConfig;
use bindings::{build, insta_delete_combo, Binding};
use parser::KeyCombo;
use rdev::{grab, Button, Event, EventType, Key};
use state_machine::{InstaDeleteState, Transition};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub fn start(cfg: HotkeysConfig) -> anyhow::Result<UnboundedReceiver<InputEvent>> {
    let bindings = build(&cfg)?;
    let insta = insta_delete_combo(&cfg)?;
    for w in bindings::duplicate_warnings(&bindings, &insta) {
        log::warn!("hotkey clash: {w}");
    }

    let (tx, rx) = unbounded_channel();
    let tx_listener = tx.clone();
    let tx_tick = tx.clone();

    let state = Arc::new(ListenerState::new(bindings, insta, tx_listener));
    let state_for_tick = state.clone();

    std::thread::Builder::new()
        .name("rdev-listener".into())
        .spawn(move || {
            // `listen` should block forever. If it ever returns (hook torn down, error), every
            // hotkey would silently die — re-install it, with backoff and a fixed attempt cap.
            for attempt in 1..=LISTEN_MAX_ATTEMPTS {
                let s = state.clone();
                let result = grab(move |ev| if s.on_event(&ev) { Some(ev) } else { None });
                log::error!("keyboard hook stopped ({result:?}); reinstalling, attempt {attempt}/{LISTEN_MAX_ATTEMPTS}");
                std::thread::sleep(Duration::from_millis(250 * (1 << attempt.min(6))));
            }
            log::error!("keyboard hook could not be kept alive; hotkeys are disabled until restart");
        })?;

    std::thread::Builder::new()
        .name("insta-tick".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_millis(250));
            state_for_tick.expire_stale_selection();
            if SHARED_INSTA
                .lock()
                .map(|mut s| s.tick())
                .map(|t| matches!(t, Transition::Disarmed))
                .unwrap_or(false)
            {
                let _ = tx_tick.send(InputEvent::InstaDeleteDisarmed);
            }
        })?;

    Ok(rx)
}

static SHARED_INSTA: Mutex<InstaDeleteState> = Mutex::new(InstaDeleteState::Idle);

const LISTEN_MAX_ATTEMPTS: u32 = 10;

/// An armed region selection nobody finishes is cancelled after this — otherwise an unrelated
/// click-drag much later would silently capture and send a region.
const SELECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// Longer than Windows' maximum keyboard repeat delay (1 s) — see `ListenerState::held`.
const STALE_HELD: Duration = Duration::from_millis(1100);

struct ListenerState {
    bindings: Vec<Binding>,
    insta: KeyCombo,
    tx: UnboundedSender<InputEvent>,
    mods: Mutex<Mods>,
    last_fire: Mutex<Option<(KeyCombo, Instant)>>,
    cursor: Mutex<(i32, i32)>,
    selecting: Mutex<bool>,
    /// When the current selection was armed — see `SELECTION_TIMEOUT`.
    armed_at: Mutex<Option<Instant>>,
    /// Keys currently held, with when each was last seen. A press for a key already held is OS
    /// auto-repeat, never a new press — unless that key has been silent longer than any repeat
    /// interval (`STALE_HELD`), which means its release was missed (lock screen, elevated
    /// window) and this is really a fresh press; otherwise that key would never fire again.
    /// press — without this, holding a combo re-fired it every debounce window, and holding the
    /// insta-delete combo for half a second counted as its confirming second tap.
    held: Mutex<HashMap<Key, Instant>>,
    /// Backup-hotkey state (see module comment): Caps Lock currently held, and whether Tab was
    /// tapped during this hold (the mode is armed until Caps Lock is released).
    caps_down: Mutex<bool>,
    backup_armed: Mutex<bool>,
    /// Keys whose press was swallowed — their release is swallowed too, so the focused app
    /// never sees a lone key-up.
    swallowed: Mutex<HashSet<Key>>,
    /// Caps Lock events we injected ourselves (to undo the chord's toggle) — passed through
    /// untouched instead of being treated as the user's.
    injected_caps: AtomicU8,
    /// A mouse button press was swallowed (selection in progress) — swallow its release too.
    left_swallowed: AtomicBool,
    right_swallowed: AtomicBool,
    /// Which hotkey armed the current selection — carried on the resulting `ScreenshotRegion`.
    purpose: Mutex<RegionPurpose>,
    p1: Mutex<Option<(i32, i32)>>,
    /// Real bug found in live testing: `rdev` fires `MouseMove` on every raw hardware tick (can
    /// be hundreds/sec), and each one was turned 1:1 into a `ScreenshotDragUpdate` -> a full UI
    /// redraw + `SetWindowPos` — laggy and pointless for a selection rectangle nobody needs
    /// updated faster than they can see. Throttled to `DRAG_UPDATE_INTERVAL` below.
    last_drag_send: Mutex<Option<Instant>>,
}

/// ~30fps — plenty smooth for a drag-select rectangle, a small fraction of raw mouse-move rate.
const DRAG_UPDATE_INTERVAL: Duration = Duration::from_millis(33);

/// Below this, in either dimension, a rectangle doesn't count as a real selection — used both to
/// tell a genuine drag-release apart from the initial click-down releasing near the same spot
/// (see `on_left_release`), and to reject a degenerate near-zero-area region a plain double-click
/// would otherwise produce (see `on_left_click`).
const MIN_REGION_PX: i32 = 6;

#[derive(Default, Clone, Copy)]
struct Mods {
    ctrl: bool,
    shift: bool,
    alt: bool,
    win: bool,
}

impl ListenerState {
    fn new(bindings: Vec<Binding>, insta: KeyCombo, tx: UnboundedSender<InputEvent>) -> Self {
        Self {
            bindings,
            insta,
            tx,
            mods: Mutex::new(Mods::default()),
            last_fire: Mutex::new(None),
            cursor: Mutex::new((0, 0)),
            selecting: Mutex::new(false),
            armed_at: Mutex::new(None),
            held: Mutex::new(HashMap::new()),
            caps_down: Mutex::new(false),
            backup_armed: Mutex::new(false),
            swallowed: Mutex::new(HashSet::new()),
            injected_caps: AtomicU8::new(0),
            left_swallowed: AtomicBool::new(false),
            right_swallowed: AtomicBool::new(false),
            purpose: Mutex::new(RegionPurpose::Query),
            p1: Mutex::new(None),
            last_drag_send: Mutex::new(None),
        }
    }

    /// Handles one hook event. Returns whether it should reach other apps (false = swallowed).
    fn on_event(&self, ev: &Event) -> bool {
        match ev.event_type {
            EventType::KeyPress(k) => return self.on_press(k),
            EventType::KeyRelease(k) => return self.on_release(k),
            EventType::MouseMove { x, y } => {
                let cur = (x as i32, y as i32);
                *self.cursor.lock().unwrap() = cur;
                // Real, confirmed gap fixed here: this was previously the only place cursor
                // position got tracked during drag-select, and nothing ever turned that into
                // visible feedback — the selection rectangle has never been shown, in any app,
                // ever. Once the first corner is placed, every subsequent move now sends the
                // current rectangle so the UI can actually draw it growing as you drag.
                if *self.selecting.lock().unwrap() {
                    if let Some((x1, y1)) = *self.p1.lock().unwrap() {
                        let mut last_send = self.last_drag_send.lock().unwrap();
                        let now = Instant::now();
                        if last_send.is_some_and(|t| now.duration_since(t) < DRAG_UPDATE_INTERVAL) {
                            return true;
                        }
                        *last_send = Some(now);
                        drop(last_send);

                        let (x2, y2) = cur;
                        let rect_x = x1.min(x2);
                        let rect_y = y1.min(y2);
                        let w = (x1 - x2).abs();
                        let h = (y1 - y2).abs();
                        let _ = self.tx.send(InputEvent::ScreenshotDragUpdate { x: rect_x, y: rect_y, w, h });
                    }
                }
            }
            // While a selection is being drawn, its clicks are swallowed: they belong to the
            // selection, and passing them through could tick an answer or follow a link on the
            // page underneath (a click-click selection over a multiple-choice option did).
            EventType::ButtonPress(Button::Left) => {
                let selecting = *self.selecting.lock().unwrap();
                self.on_left_click();
                if selecting {
                    self.left_swallowed.store(true, Ordering::SeqCst);
                    return false;
                }
            }
            // Real gap found by actually thinking through the gesture, not just the two-click
            // model this was originally built around: a natural press-drag-release never fires a
            // second `ButtonPress`, so without this the selection stayed "armed" waiting for P2
            // forever — the next unrelated click anywhere (a desktop icon, a link) would silently
            // become P2 and fire a capture nobody asked for. See `on_left_release`.
            EventType::ButtonRelease(Button::Left) => {
                self.on_left_release();
                if self.left_swallowed.swap(false, Ordering::SeqCst) {
                    return false;
                }
            }
            // Right-click is the natural "never mind" during a selection (and must not open the
            // page's context menu while doing so).
            EventType::ButtonPress(Button::Right) => {
                if *self.selecting.lock().unwrap() {
                    self.cancel_selection();
                    self.right_swallowed.store(true, Ordering::SeqCst);
                    return false;
                }
            }
            EventType::ButtonRelease(Button::Right) => {
                if self.right_swallowed.swap(false, Ordering::SeqCst) {
                    return false;
                }
            }
            _ => {}
        }
        true
    }

    /// Cancels an armed selection, if any, and tells the UI.
    fn cancel_selection(&self) {
        let was_selecting = std::mem::replace(&mut *self.selecting.lock().unwrap(), false);
        *self.p1.lock().unwrap() = None;
        *self.armed_at.lock().unwrap() = None;
        if was_selecting {
            let _ = self.tx.send(InputEvent::ScreenshotCancel);
        }
    }

    /// Called from the tick thread.
    fn expire_stale_selection(&self) {
        let stale = self.armed_at.lock().unwrap().is_some_and(|t| t.elapsed() > SELECTION_TIMEOUT);
        if stale {
            log::debug!("screenshot region: selection timed out");
            self.cancel_selection();
        }
    }

    fn on_left_click(&self) {
        if !*self.selecting.lock().unwrap() {
            return;
        }
        let cur = self.cursor_now();
        let mut p1 = self.p1.lock().unwrap();
        match *p1 {
            None => {
                *p1 = Some(cur);
                log::debug!("screenshot region: P1 at {:?}", cur);
            }
            Some((x1, y1)) => {
                let (x2, y2) = cur;
                let x = x1.min(x2);
                let y = y1.min(y2);
                let w = (x1 - x2).abs();
                let h = (y1 - y2).abs();
                *p1 = None;
                *self.selecting.lock().unwrap() = false;
                *self.armed_at.lock().unwrap() = None;
                log::debug!("screenshot region: P2 at {:?}; rect {}x{} @ ({},{})", cur, w, h, x, y);
                if w >= MIN_REGION_PX && h >= MIN_REGION_PX {
                    let _ = self.tx.send(InputEvent::ScreenshotRegion { x, y, w, h, purpose: *self.purpose.lock().unwrap() });
                } else {
                    let _ = self.tx.send(InputEvent::ScreenshotCancel);
                }
            }
        }
    }

    /// Completes a press-drag-release gesture. Only acts if the release happened meaningfully
    /// far from P1 (`MIN_REGION_PX` in either dimension) — a release right where P1 was set is
    /// just the initial click-down of the click-move-click model finishing, not a drag, so it's
    /// left alone and the state machine keeps waiting for a real second click (`on_left_click`'s
    /// `Some` branch). A release that *did* drag but still lands on a degenerate sliver (e.g. a
    /// near-perfectly horizontal drag) cancels outright instead of leaving the selection armed.
    fn on_left_release(&self) {
        if !*self.selecting.lock().unwrap() {
            return;
        }
        let cur = self.cursor_now();
        let mut p1 = self.p1.lock().unwrap();
        let Some((x1, y1)) = *p1 else {
            return;
        };
        let (x2, y2) = cur;
        let w = (x1 - x2).abs();
        let h = (y1 - y2).abs();
        if w < MIN_REGION_PX && h < MIN_REGION_PX {
            return;
        }
        let x = x1.min(x2);
        let y = y1.min(y2);
        *p1 = None;
        *self.selecting.lock().unwrap() = false;
        *self.armed_at.lock().unwrap() = None;
        log::debug!("screenshot region (drag-release): rect {}x{} @ ({},{})", w, h, x, y);
        if w >= MIN_REGION_PX && h >= MIN_REGION_PX {
            let _ = self.tx.send(InputEvent::ScreenshotRegion { x, y, w, h, purpose: *self.purpose.lock().unwrap() });
        } else {
            let _ = self.tx.send(InputEvent::ScreenshotCancel);
        }
    }

    fn on_press(&self, k: Key) -> bool {
        if k == Key::CapsLock && self.take_injected_caps() {
            return true;
        }
        let repeat = {
            let mut held = self.held.lock().unwrap();
            let now = Instant::now();
            let repeat = held.get(&k).is_some_and(|t| now.duration_since(*t) < STALE_HELD);
            held.insert(k, now);
            repeat
        };
        if repeat {
            // Auto-repeat of a key already down: same fate as its first press.
            return !self.swallowed.lock().unwrap().contains(&k);
        }
        if k == Key::CapsLock {
            *self.caps_down.lock().unwrap() = true;
            *self.backup_armed.lock().unwrap() = false;
            return true;
        }
        if *self.caps_down.lock().unwrap() {
            if k == Key::Tab {
                *self.backup_armed.lock().unwrap() = true;
                self.swallowed.lock().unwrap().insert(k);
                return false;
            }
            if *self.backup_armed.lock().unwrap() {
                self.swallowed.lock().unwrap().insert(k);
                self.fire_backup(k);
                return false;
            }
        }
        if self.update_modifier(k, true) {
            return true;
        }

        // Escape always cancels an in-progress screenshot selection, independent of whatever the
        // configured `abort` hotkey happens to be — the standard, no-modifiers-needed way out of
        // a selection you started by mistake.
        if k == Key::Escape && *self.selecting.lock().unwrap() {
            self.cancel_selection();
            return true;
        }

        // Ask Windows for the real modifier state instead of trusting tracked press/release
        // events: releases that happen on the secure desktop (Ctrl+Alt+Del, UAC, Win+L) or in
        // an elevated window are never seen by the hook, which left a modifier "stuck" so plain
        // keys matched hotkeys (or real hotkeys stopped matching). Re-syncing here heals it.
        let mods = live_mods().unwrap_or(*self.mods.lock().unwrap());
        *self.mods.lock().unwrap() = mods;

        if self.matches(&self.insta, k, mods) && self.debounce(&self.insta) {
            self.press_insta();
            return true;
        }

        for b in &self.bindings {
            if self.matches(&b.combo, k, mods) && self.debounce(&b.combo) {
                self.fire((b.make_event)());
                return true;
            }
        }
        true
    }

    /// Backup mode: `k` fires whichever action's normal combo uses that key, modifiers ignored.
    /// Guard for the destructive ones reached this way (panic_kill's End, insta_delete's X): the
    /// chord needs Caps Lock held AND Tab pressed first — two keys that never co-occur in normal
    /// typing — and insta_delete still needs its double tap.
    fn fire_backup(&self, k: Key) {
        if k == Key::Escape && *self.selecting.lock().unwrap() {
            self.cancel_selection();
            return;
        }
        if self.insta.key == k {
            if self.debounce(&self.insta) {
                self.press_insta();
            }
            return;
        }
        if let Some(b) = self.bindings.iter().find(|b| b.combo.key == k) {
            if self.debounce(&b.combo) {
                self.fire((b.make_event)());
            }
        }
    }

    /// Insta-delete's double-tap state machine (its guard against accidental triggering).
    fn press_insta(&self) {
        match SHARED_INSTA.lock().unwrap().on_press() {
            Transition::Armed => {
                let _ = self.tx.send(InputEvent::InstaDeleteArmed);
            }
            Transition::Confirmed => {
                let _ = self.tx.send(InputEvent::InstaDeleteConfirmed);
            }
            _ => {}
        }
    }

    /// Sends a bound action's event, doing the selection bookkeeping some actions need first.
    fn fire(&self, event: InputEvent) {
        match &event {
            InputEvent::ScreenshotQuery | InputEvent::AddContext => {
                // Real gap found by thinking through a double-press: re-firing this
                // hotkey mid-selection used to just reset `p1`/`selecting` here without
                // telling the UI, so any rectangle already on screen from the abandoned
                // attempt stayed stuck there forever (nothing was ever going to hide it —
                // the next `MouseMove` had a cleared `p1`, so it never sent another
                // update either). Cancel the old attempt first so the UI actually clears
                // before the new one starts.
                let was_selecting = *self.selecting.lock().unwrap();
                *self.p1.lock().unwrap() = None;
                *self.selecting.lock().unwrap() = true;
                *self.armed_at.lock().unwrap() = Some(Instant::now());
                *self.purpose.lock().unwrap() = if matches!(event, InputEvent::AddContext) {
                    RegionPurpose::Context
                } else {
                    RegionPurpose::Query
                };
                if was_selecting {
                    let _ = self.tx.send(InputEvent::ScreenshotCancel);
                }
            }
            InputEvent::Abort => {
                let was_selecting = *self.selecting.lock().unwrap();
                *self.selecting.lock().unwrap() = false;
                *self.p1.lock().unwrap() = None;
                if was_selecting {
                    let _ = self.tx.send(InputEvent::ScreenshotCancel);
                    return;
                }
            }
            _ => {}
        }
        let _ = self.tx.send(event);
    }

    /// Where the cursor is right now. Asked from Windows rather than taken only from tracked
    /// move events: a click can arrive with no move event since the hotkey (cursor already in
    /// place, or moved by software), which recorded a stale (0, 0) corner.
    fn cursor_now(&self) -> (i32, i32) {
        let pos = live_cursor().unwrap_or(*self.cursor.lock().unwrap());
        *self.cursor.lock().unwrap() = pos;
        pos
    }

    fn take_injected_caps(&self) -> bool {
        self.injected_caps.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |n| n.checked_sub(1)).is_ok()
    }

    /// The chord's Caps Lock press toggled caps; tap it once more so the user's caps state is
    /// what it was before. Injected from a short-lived thread, outside the hook callback.
    fn undo_caps_toggle(&self) {
        self.injected_caps.fetch_add(2, Ordering::SeqCst);
        #[cfg(not(test))]
        std::thread::spawn(|| {
            std::thread::sleep(Duration::from_millis(20));
            send_caps_tap();
        });
    }

    fn on_release(&self, k: Key) -> bool {
        if k == Key::CapsLock && self.take_injected_caps() {
            return true;
        }
        self.held.lock().unwrap().remove(&k);
        self.update_modifier(k, false);
        let was_swallowed = self.swallowed.lock().unwrap().remove(&k);
        if k == Key::CapsLock {
            *self.caps_down.lock().unwrap() = false;
            if std::mem::replace(&mut *self.backup_armed.lock().unwrap(), false) {
                self.undo_caps_toggle();
            }
            return true;
        }
        !was_swallowed
    }

    fn update_modifier(&self, k: Key, down: bool) -> bool {
        let mut m = self.mods.lock().unwrap();
        match k {
            Key::ControlLeft | Key::ControlRight => {
                m.ctrl = down;
                true
            }
            Key::ShiftLeft | Key::ShiftRight => {
                m.shift = down;
                true
            }
            Key::Alt | Key::AltGr => {
                m.alt = down;
                true
            }
            Key::MetaLeft | Key::MetaRight => {
                m.win = down;
                true
            }
            _ => false,
        }
    }

    fn matches(&self, c: &KeyCombo, k: Key, m: Mods) -> bool {
        c.key == k && c.ctrl == m.ctrl && c.shift == m.shift && c.alt == m.alt && c.win == m.win
    }

    fn debounce(&self, c: &KeyCombo) -> bool {
        let mut last = self.last_fire.lock().unwrap();
        let now = Instant::now();
        if let Some((prev, t)) = last.as_ref() {
            if prev == c && now.duration_since(*t) < Duration::from_millis(200) {
                return false;
            }
        }
        *last = Some((c.clone(), now));
        true
    }
}

/// The real, current modifier state from Windows, or None if it can't be read.
#[cfg(windows)]
fn live_mods() -> Option<Mods> {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VIRTUAL_KEY, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT};
    // SAFETY: GetAsyncKeyState is a plain Win32 query with no pointers or preconditions.
    let down = |vk: VIRTUAL_KEY| unsafe { GetAsyncKeyState(vk.0 as i32) } as u16 & 0x8000 != 0;
    Some(Mods { ctrl: down(VK_CONTROL), shift: down(VK_SHIFT), alt: down(VK_MENU), win: down(VK_LWIN) || down(VK_RWIN) })
}

#[cfg(not(windows))]
fn live_mods() -> Option<Mods> {
    None
}

/// The real cursor position (physical pixels — the process is per-monitor DPI aware).
#[cfg(windows)]
fn live_cursor() -> Option<(i32, i32)> {
    let mut p = windows::Win32::Foundation::POINT::default();
    // SAFETY: plain Win32 query writing into a local POINT.
    unsafe { windows::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut p) }.ok()?;
    Some((p.x, p.y))
}

#[cfg(not(windows))]
fn live_cursor() -> Option<(i32, i32)> {
    None
}

/// One synthetic Caps Lock press + release.
#[cfg(all(windows, not(test)))]
fn send_caps_tap() {
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_CAPITAL,
    };
    let key = |flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 { ki: KEYBDINPUT { wVk: VK_CAPITAL, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } },
    };
    let inputs = [key(KEYBD_EVENT_FLAGS(0)), key(KEYEVENTF_KEYUP)];
    // SAFETY: plain Win32 call with a correctly sized, fully initialised INPUT array.
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != 2 {
        log::warn!("backup hotkey: couldn't restore Caps Lock state");
    }
}

#[cfg(all(not(windows), not(test)))]
fn send_caps_tap() {}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with(combo: &str) -> (Arc<ListenerState>, UnboundedReceiver<InputEvent>) {
        let (tx, rx) = unbounded_channel();
        let bindings = vec![Binding { combo: parser::parse(combo).unwrap(), make_event: || InputEvent::HideToggle }];
        let insta = parser::parse("f12").unwrap();
        (Arc::new(ListenerState::new(bindings, insta, tx)), rx)
    }

    fn drain(rx: &mut UnboundedReceiver<InputEvent>) -> usize {
        let mut n = 0;
        while rx.try_recv().is_ok() {
            n += 1;
        }
        n
    }

    #[test]
    fn holding_a_key_fires_once_not_on_every_auto_repeat() {
        let (s, mut rx) = state_with("f9");
        for _ in 0..5 {
            s.on_press(Key::F9); // first is real, rest are auto-repeat
        }
        assert_eq!(drain(&mut rx), 1);
        s.on_release(Key::F9);
        std::thread::sleep(Duration::from_millis(210)); // past the per-combo debounce
        s.on_press(Key::F9);
        assert_eq!(drain(&mut rx), 1, "a fresh press after release fires again");
    }

    #[test]
    fn a_key_whose_release_was_missed_fires_again_after_it_goes_stale() {
        let (s, mut rx) = state_with("f9");
        s.on_press(Key::F9);
        assert_eq!(drain(&mut rx), 1);
        // release never seen (e.g. happened on the lock screen) ...
        s.held.lock().unwrap().insert(Key::F9, Instant::now() - STALE_HELD - Duration::from_millis(1));
        std::thread::sleep(Duration::from_millis(210)); // past debounce
        s.on_press(Key::F9);
        assert_eq!(drain(&mut rx), 1, "a press after a long silence is a fresh press, not a repeat");
    }

    #[test]
    fn backup_chord_fires_and_swallows_and_restores_caps() {
        let (s, mut rx) = state_with("ctrl+shift+alt+f9");
        assert!(s.on_press(Key::CapsLock), "caps press itself reaches apps");
        assert!(!s.on_press(Key::Tab), "tab is swallowed");
        assert!(!s.on_press(Key::F9), "the action key is swallowed");
        assert!(matches!(rx.try_recv(), Ok(InputEvent::HideToggle)), "fires the action whose key is F9");
        assert!(!s.on_release(Key::F9) && !s.on_release(Key::Tab), "their releases are swallowed too");
        assert!(s.on_release(Key::CapsLock));
        // our own injected caps tap (2 events) passes through and isn't treated as the user's
        assert!(s.on_press(Key::CapsLock) && s.on_release(Key::CapsLock));
        assert!(!*s.caps_down.lock().unwrap());
        assert_eq!(s.injected_caps.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn caps_then_letter_without_tab_is_normal_typing() {
        let (s, mut rx) = state_with("ctrl+shift+alt+f9");
        assert!(s.on_press(Key::CapsLock));
        assert!(s.on_press(Key::F9), "no Tab: the key reaches the app");
        assert!(rx.try_recv().is_err(), "and nothing fires");
        assert!(s.on_release(Key::F9) && s.on_release(Key::CapsLock));
        assert_eq!(s.injected_caps.load(Ordering::SeqCst), 0, "caps toggle left alone");
    }

    fn ev(t: EventType) -> Event {
        Event { event_type: t, time: std::time::SystemTime::now(), name: None }
    }

    #[test]
    fn clicks_are_swallowed_only_while_selecting() {
        let (s, mut rx) = state_with("f9");
        assert!(s.on_event(&ev(EventType::ButtonPress(Button::Left))), "normal clicks pass");
        assert!(s.on_event(&ev(EventType::ButtonRelease(Button::Left))));
        *s.selecting.lock().unwrap() = true;
        assert!(!s.on_event(&ev(EventType::ButtonPress(Button::Left))), "selection click swallowed");
        assert!(!s.on_event(&ev(EventType::ButtonRelease(Button::Left))), "and its release");
        assert!(!s.on_event(&ev(EventType::ButtonPress(Button::Right))), "right-click cancel swallowed");
        assert!(!s.on_event(&ev(EventType::ButtonRelease(Button::Right))));
        assert!(!*s.selecting.lock().unwrap());
        assert!(matches!(rx.try_recv(), Ok(InputEvent::ScreenshotCancel)));
        assert!(s.on_event(&ev(EventType::ButtonPress(Button::Right))), "right-click passes when idle");
    }

    #[test]
    fn stale_selection_is_cancelled_and_right_click_cancels() {
        let (s, mut rx) = state_with("f9");
        *s.selecting.lock().unwrap() = true;
        *s.armed_at.lock().unwrap() = Some(Instant::now() - SELECTION_TIMEOUT - Duration::from_secs(1));
        s.expire_stale_selection();
        assert!(!*s.selecting.lock().unwrap());
        assert!(matches!(rx.try_recv(), Ok(InputEvent::ScreenshotCancel)));

        *s.selecting.lock().unwrap() = true;
        *s.armed_at.lock().unwrap() = Some(Instant::now());
        s.expire_stale_selection();
        assert!(*s.selecting.lock().unwrap(), "a fresh selection is left alone");
        s.cancel_selection();
        assert!(matches!(rx.try_recv(), Ok(InputEvent::ScreenshotCancel)));
    }
}
