use rdev::{listen, Button, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::thread;

pub enum InputEvent {
    Wake,
    Model,
    Panic,
    OCRClick1,
    OCRRect(i32, i32, i32, i32), // x, y, w, h
    HideToggle,
    BrowserPass,
    BrowserExec,
    BrowserExecSingle,
    BrowserAbort,
    BrowserIncognito,
}

#[allow(dead_code)]
pub struct InputManager {
    wake_key_combo: Vec<Key>,
    model_key_combo: Vec<Key>,
    panic_key_combo: Vec<Key>,
    sender: Sender<InputEvent>,
}

impl InputManager {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        wake_keys: Vec<Key>,
        model_keys: Vec<Key>,
        panic_keys: Vec<Key>,
        hide_keys: Vec<Key>,
        b_pass_keys: Vec<Key>,
        b_exec_keys: Vec<Key>,
        b_exec_single_keys: Vec<Key>,
        b_abort_keys: Vec<Key>,
        b_incognito_keys: Vec<Key>,
        sender: Sender<InputEvent>,
    ) {
        thread::spawn(move || {
            let mut pressed_keys = HashSet::new();
            let mut is_selecting = false;
            let mut p1: Option<(f64, f64)> = None;
            let mut current_pos = (0.0, 0.0);

            // This closure needs to handle the state
            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        let key = normalize_modifier(key);
                        pressed_keys.insert(key);

                        // Check combos
                        if check_combo(&pressed_keys, &panic_keys) {
                            let _ = sender.send(InputEvent::Panic);
                            is_selecting = false;
                            p1 = None; // Reset
                        } else if check_combo(&pressed_keys, &wake_keys) {
                            let _ = sender.send(InputEvent::Wake);
                            is_selecting = true; // Enter Selection Mode
                            p1 = None;
                            println!("[*] Input: Entering OCR Selection Mode");
                        } else if check_combo(&pressed_keys, &model_keys) {
                            let _ = sender.send(InputEvent::Model);
                            is_selecting = false;
                            p1 = None;
                        } else if check_combo(&pressed_keys, &hide_keys) {
                            let _ = sender.send(InputEvent::HideToggle);
                            is_selecting = false;
                            p1 = None;
                        } else if check_combo(&pressed_keys, &b_pass_keys) {
                            let _ = sender.send(InputEvent::BrowserPass);
                            is_selecting = false;
                            p1 = None;
                        } else if check_combo(&pressed_keys, &b_exec_keys) {
                            let _ = sender.send(InputEvent::BrowserExec);
                            is_selecting = false;
                            p1 = None;
                        } else if check_combo(&pressed_keys, &b_exec_single_keys) {
                            let _ = sender.send(InputEvent::BrowserExecSingle);
                            is_selecting = false;
                            p1 = None;
                        } else if check_combo(&pressed_keys, &b_abort_keys) {
                            let _ = sender.send(InputEvent::BrowserAbort);
                            is_selecting = false;
                            p1 = None;
                        } else if check_combo(&pressed_keys, &b_incognito_keys) {
                            let _ = sender.send(InputEvent::BrowserIncognito);
                            is_selecting = false;
                            p1 = None;
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let key = normalize_modifier(key);
                        pressed_keys.remove(&key);
                    }
                    EventType::MouseMove { x, y } => {
                        current_pos = (x, y);
                    }
                    EventType::ButtonPress(Button::Left) => {
                        if is_selecting {
                            if let Some(start) = p1 {
                                // Second Click -> P2
                                println!("[*] Input: Point 2 Captured at {:?}", current_pos);
                                let x = start.0.min(current_pos.0) as i32;
                                let y = start.1.min(current_pos.1) as i32;
                                let w = (start.0 - current_pos.0).abs() as i32;
                                let h = (start.1 - current_pos.1).abs() as i32;

                                if w > 0 && h > 0 {
                                    let _ = sender.send(InputEvent::OCRRect(x, y, w, h));
                                }

                                // Reset
                                is_selecting = false;
                                p1 = None;
                            } else {
                                // First Click -> P1
                                println!("[*] Input: Point 1 Captured at {:?}", current_pos);
                                p1 = Some(current_pos);
                                let _ = sender.send(InputEvent::OCRClick1);
                            }
                        }
                    }
                    _ => {}
                }
            };

            if let Err(error) = listen(callback) {
                eprintln!("Error: {:?}", error);
            }
        });
    }
}

/// Normalize right-side modifier keys to their left-side equivalents so that
/// check_combo() works regardless of which physical key the user pressed.
fn normalize_modifier(key: Key) -> Key {
    match key {
        Key::ControlRight => Key::ControlLeft,
        Key::ShiftRight   => Key::ShiftLeft,
        Key::AltGr        => Key::Alt,
        other             => other,
    }
}

fn check_combo(pressed: &HashSet<Key>, target: &[Key]) -> bool {
    if target.is_empty() {
        return false;
    }
    target.iter().all(|k| pressed.contains(k))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use rdev::Key;

    #[test]
    fn test_check_combo_left_ctrl() {
        let mut pressed = HashSet::new();
        pressed.insert(Key::ControlLeft);
        pressed.insert(Key::KeyV);
        let target = vec![Key::ControlLeft, Key::KeyV];
        assert!(check_combo(&pressed, &target));
    }

    #[test]
    fn test_check_combo_right_ctrl_not_matched_without_normalization() {
        // Right ctrl is in pressed set, target expects ControlLeft — fails without normalization
        let mut pressed = HashSet::new();
        pressed.insert(Key::ControlRight);
        pressed.insert(Key::KeyV);
        let target = vec![Key::ControlLeft, Key::KeyV];
        // Without normalization this would fail; after normalization it passes
        // This test documents the EXPECTED post-fix behavior
        assert!(!check_combo(&pressed, &target)); // still fails here — normalization is in the callback
    }
}
