// Input layer: global keyboard + mouse hook (rdev) → InputEvent on channel.

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
            if SHARED_INSTA.lock().map(|mut s| s.tick()).map(|t| matches!(t, Transition::Disarmed)).unwrap_or(false) {
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
}

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
        }
    }

    fn on_event(&self, ev: Event) {
        match ev.event_type {
            EventType::KeyPress(k) => self.on_press(k),
            EventType::KeyRelease(k) => self.on_release(k),
            EventType::MouseMove { x, y } => {
                *self.cursor.lock().unwrap() = (x as i32, y as i32);
            }
            EventType::ButtonPress(Button::Left) => self.on_left_click(),
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
                log::debug!("ocr region: P1 at {:?}", cur);
            }
            Some((x1, y1)) => {
                let (x2, y2) = cur;
                let x = x1.min(x2);
                let y = y1.min(y2);
                let w = (x1 - x2).abs();
                let h = (y1 - y2).abs();
                *p1 = None;
                *self.selecting.lock().unwrap() = false;
                log::debug!("ocr region: P2 at {:?}; rect {}x{} @ ({},{})", cur, w, h, x, y);
                if w > 0 && h > 0 {
                    let _ = self.tx.send(InputEvent::OcrRegion { x, y, w, h });
                } else {
                    let _ = self.tx.send(InputEvent::OcrCancel);
                }
            }
        }
    }

    fn on_press(&self, k: Key) {
        if self.update_modifier(k, true) {
            return;
        }
        let mods = *self.mods.lock().unwrap();

        if self.matches(&self.insta, k, mods) && self.debounce(&self.insta) {
            let trans = SHARED_INSTA.lock().unwrap().on_press();
            match trans {
                Transition::Armed => { let _ = self.tx.send(InputEvent::InstaDeleteArmed); }
                Transition::Confirmed => { let _ = self.tx.send(InputEvent::InstaDeleteConfirmed); }
                _ => {}
            }
            return;
        }

        for b in &self.bindings {
            if self.matches(&b.combo, k, mods) && self.debounce(&b.combo) {
                let event = (b.make_event)();
                // Special-case: OcrQuery enters region-selection mode.
                match &event {
                    InputEvent::OcrQuery => {
                        *self.p1.lock().unwrap() = None;
                        *self.selecting.lock().unwrap() = true;
                    }
                    InputEvent::Abort => {
                        let was_selecting = *self.selecting.lock().unwrap();
                        *self.selecting.lock().unwrap() = false;
                        *self.p1.lock().unwrap() = None;
                        if was_selecting {
                            let _ = self.tx.send(InputEvent::OcrCancel);
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
            Key::ControlLeft | Key::ControlRight => { m.ctrl = down; true }
            Key::ShiftLeft | Key::ShiftRight => { m.shift = down; true }
            Key::Alt | Key::AltGr => { m.alt = down; true }
            Key::MetaLeft | Key::MetaRight => { m.win = down; true }
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
