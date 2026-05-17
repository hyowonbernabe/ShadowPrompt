// Input layer: global keyboard + mouse hook (rdev) → InputEvent on channel.

pub mod bindings;
pub mod events;
pub mod parser;
pub mod state_machine;

pub use events::InputEvent;

use crate::config::schema::HotkeysConfig;
use bindings::{build, insta_delete_combo, Binding};
use parser::KeyCombo;
use rdev::{listen, Event, EventType, Key};
use state_machine::{InstaDeleteState, Transition};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Spawn the input listener OS thread + a tick worker for state-machine timeouts.
/// Returns a receiver of InputEvent.
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
        }
    }

    fn on_event(&self, ev: Event) {
        match ev.event_type {
            EventType::KeyPress(k) => self.on_press(k),
            EventType::KeyRelease(k) => self.on_release(k),
            _ => {}
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
                let _ = self.tx.send((b.make_event)());
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

    /// 200ms debounce so OS auto-repeat doesn't spam events.
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
