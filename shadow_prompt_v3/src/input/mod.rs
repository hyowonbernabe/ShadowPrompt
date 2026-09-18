// Input layer: global keyboard + mouse hook (rdev) -> InputEvent on channel. Ported from v2 with
// the search-modifier hotkeys removed (design doc §3/§5: the model decides autonomously whether
// to search now, it's a tool call, not a separate forced hotkey) and OCR-region-select renamed to
// screenshot-region-select (OCR itself is gone, design doc §3 — this is now purely "pick a
// rectangle, send it as an image").

pub mod bindings;
pub mod events;
pub mod parser;
pub mod state_machine;

pub use events::InputEvent;

use crate::config::schema::HotkeysConfig;
use bindings::{build, insta_delete_combo, Binding};
use parser::KeyCombo;
use rdev::{listen, Button, Event, EventType, Key};
use state_machine::{InstaDeleteState, Transition};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub fn start(cfg: HotkeysConfig) -> anyhow::Result<UnboundedReceiver<InputEvent>> {
    let bindings = build(&cfg)?;
    let insta = insta_delete_combo(&cfg)?;

    let (tx, rx) = unbounded_channel();
    let tx_listener = tx.clone();
    let tx_tick = tx.clone();

    std::thread::Builder::new()
        .name("rdev-listener".into())
        .spawn(move || {
            let state = ListenerState::new(bindings, insta, tx_listener);
            let _ = listen(move |ev| state.on_event(ev));
        })?;

    std::thread::Builder::new()
        .name("insta-tick".into())
        .spawn(move || loop {
            std::thread::sleep(Duration::from_millis(250));
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

struct ListenerState {
    bindings: Vec<Binding>,
    insta: KeyCombo,
    tx: UnboundedSender<InputEvent>,
    mods: Mutex<Mods>,
    last_fire: Mutex<Option<(KeyCombo, Instant)>>,
    cursor: Mutex<(i32, i32)>,
    selecting: Mutex<bool>,
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
            p1: Mutex::new(None),
            last_drag_send: Mutex::new(None),
        }
    }

    fn on_event(&self, ev: Event) {
        match ev.event_type {
            EventType::KeyPress(k) => self.on_press(k),
            EventType::KeyRelease(k) => self.on_release(k),
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
                            return;
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
            EventType::ButtonPress(Button::Left) => self.on_left_click(),
            // Real gap found by actually thinking through the gesture, not just the two-click
            // model this was originally built around: a natural press-drag-release never fires a
            // second `ButtonPress`, so without this the selection stayed "armed" waiting for P2
            // forever — the next unrelated click anywhere (a desktop icon, a link) would silently
            // become P2 and fire a capture nobody asked for. See `on_left_release`.
            EventType::ButtonRelease(Button::Left) => self.on_left_release(),
            _ => {}
        }
    }

    fn on_left_click(&self) {
        if !*self.selecting.lock().unwrap() {
            return;
        }
        let cur = *self.cursor.lock().unwrap();
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
                log::debug!("screenshot region: P2 at {:?}; rect {}x{} @ ({},{})", cur, w, h, x, y);
                if w >= MIN_REGION_PX && h >= MIN_REGION_PX {
                    let _ = self.tx.send(InputEvent::ScreenshotRegion { x, y, w, h });
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
        let cur = *self.cursor.lock().unwrap();
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
        log::debug!("screenshot region (drag-release): rect {}x{} @ ({},{})", w, h, x, y);
        if w >= MIN_REGION_PX && h >= MIN_REGION_PX {
            let _ = self.tx.send(InputEvent::ScreenshotRegion { x, y, w, h });
        } else {
            let _ = self.tx.send(InputEvent::ScreenshotCancel);
        }
    }

    fn on_press(&self, k: Key) {
        if self.update_modifier(k, true) {
            return;
        }

        // Escape always cancels an in-progress screenshot selection, independent of whatever the
        // configured `abort` hotkey happens to be — the standard, no-modifiers-needed way out of
        // a selection you started by mistake.
        if k == Key::Escape && *self.selecting.lock().unwrap() {
            *self.selecting.lock().unwrap() = false;
            *self.p1.lock().unwrap() = None;
            let _ = self.tx.send(InputEvent::ScreenshotCancel);
            return;
        }

        let mods = *self.mods.lock().unwrap();

        if self.matches(&self.insta, k, mods) && self.debounce(&self.insta) {
            let trans = SHARED_INSTA.lock().unwrap().on_press();
            match trans {
                Transition::Armed => {
                    let _ = self.tx.send(InputEvent::InstaDeleteArmed);
                }
                Transition::Confirmed => {
                    let _ = self.tx.send(InputEvent::InstaDeleteConfirmed);
                }
                _ => {}
            }
            return;
        }

        for b in &self.bindings {
            if self.matches(&b.combo, k, mods) && self.debounce(&b.combo) {
                let event = (b.make_event)();
                match &event {
                    InputEvent::ScreenshotQuery => {
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
                return;
            }
        }
    }

    fn on_release(&self, k: Key) {
        self.update_modifier(k, false);
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
