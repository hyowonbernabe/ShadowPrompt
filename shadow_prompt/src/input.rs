use rdev::{listen, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::mpsc::Sender;
use std::thread;

pub enum InputEvent {
    Wake,
    Model,
    Panic,
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
        sender: Sender<InputEvent>
    ) {
        thread::spawn(move || {
            let mut pressed_keys = HashSet::new();
            
            // This closure needs to handle the state
            let callback = move |event: Event| {
                match event.event_type {
                    EventType::KeyPress(key) => {
                        pressed_keys.insert(key);
                        
                        // Check combos
                        if check_combo(&pressed_keys, &panic_keys) {
                            let _ = sender.send(InputEvent::Panic);
                        } else if check_combo(&pressed_keys, &wake_keys) {
                            let _ = sender.send(InputEvent::Wake);
                        } else if check_combo(&pressed_keys, &model_keys) {
                            let _ = sender.send(InputEvent::Model);
                        }
                    }
                    EventType::KeyRelease(key) => {
                        pressed_keys.remove(&key);
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

fn check_combo(pressed: &HashSet<Key>, target: &Vec<Key>) -> bool {
    if target.is_empty() { return false; }
    target.iter().all(|k| pressed.contains(k))
}

// Helper to convert string config to Keys (Simplified for prototype)
pub fn parse_keys(config_str: &str) -> Vec<Key> {
    let mut keys = Vec::new();
    for part in config_str.split('+') {
        match part.trim().to_lowercase().as_str() {
            "ctrl" => keys.push(Key::ControlLeft), // Assume Left for simplicity or add both
            "shift" => keys.push(Key::ShiftLeft),
            "alt" => keys.push(Key::Alt),
            "space" => keys.push(Key::Space),
            "v" => keys.push(Key::KeyV),
            "f12" => keys.push(Key::F12),
            _ => {
                // In a real app, map all keys. For now, handle common ones.
                eprintln!("Warning: Unknown key in config: {}", part);
            }
        }
    }
    keys
}
