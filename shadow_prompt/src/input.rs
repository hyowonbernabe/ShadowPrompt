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
}

#[allow(dead_code)]
pub struct InputManager {
    wake_key_combo: Vec<Key>,
    model_key_combo: Vec<Key>,
    panic_key_combo: Vec<Key>,
    sender: Sender<InputEvent>,
}

impl InputManager {
    pub fn start(
        wake_keys: Vec<Key>,
        model_keys: Vec<Key>,
        panic_keys: Vec<Key>,
        hide_keys: Vec<Key>,
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
                        }
                    }
                    EventType::KeyRelease(key) => {
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

fn check_combo(pressed: &HashSet<Key>, target: &[Key]) -> bool {
    if target.is_empty() {
        return false;
    }
    target.iter().all(|k| pressed.contains(k))
}
