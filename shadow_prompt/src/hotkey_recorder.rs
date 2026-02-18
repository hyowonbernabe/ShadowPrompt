//! Hotkey Recorder Module
//! Provides UI component for recording keyboard shortcuts during setup.
//! Uses egui's native input handling for capturing hotkeys when focused.

use eframe::egui;
use std::collections::HashSet;

#[derive(Clone)]
#[allow(dead_code)]
pub struct RecordedHotkey {
    pub keys: Vec<egui::Key>,
    pub display_string: String,
}

pub struct HotkeyRecorder {
    is_recording: bool,
    current_keys: HashSet<egui::Key>,
    current_modifiers: egui::Modifiers,
    recorded_result: Option<RecordedHotkey>,
}

impl Default for HotkeyRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl HotkeyRecorder {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            current_keys: HashSet::new(),
            current_modifiers: egui::Modifiers::NONE,
            recorded_result: None,
        }
    }

    pub fn start_recording(&mut self) {
        self.is_recording = true;
        self.recorded_result = None;
        self.current_keys.clear();
        self.current_modifiers = egui::Modifiers::NONE;
    }

    pub fn cancel(&mut self) {
        self.is_recording = false;
        self.current_keys.clear();
        self.current_modifiers = egui::Modifiers::NONE;
    }

    /// Process input from egui and return the recorded hotkey if complete
    pub fn process_input(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.is_recording {
            return None;
        }

        ctx.input(|i| {
            // Update modifiers
            self.current_modifiers = i.modifiers;

            // Track pressed keys
            for event in &i.events {
                if let egui::Event::Key { key, pressed, .. } = event {
                    if *pressed {
                        self.current_keys.insert(*key);
                    }
                }
            }

            // Check if any key was released this frame
            let any_key_released = i
                .events
                .iter()
                .any(|e| matches!(e, egui::Event::Key { pressed: false, .. }));

            // When a key is released, check if we have a valid combination
            if any_key_released && self.has_valid_combination() {
                let display = self.get_current_display();
                self.is_recording = false;
                self.current_keys.clear();
                return Some(display);
            }

            None
        })
    }

    /// Check if we have a valid combination (modifier + key, or 2+ keys)
    fn has_valid_combination(&self) -> bool {
        let has_modifier = self.current_modifiers.ctrl
            || self.current_modifiers.shift
            || self.current_modifiers.alt;
        let has_key = !self.current_keys.is_empty();

        // Valid: modifier + any key, or 2+ regular keys
        (has_modifier && has_key) || self.current_keys.len() >= 2
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn get_current_display(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        // Add modifiers first
        if self.current_modifiers.ctrl {
            parts.push("Ctrl".to_string());
        }
        if self.current_modifiers.shift {
            parts.push("Shift".to_string());
        }
        if self.current_modifiers.alt {
            parts.push("Alt".to_string());
        }

        // Add regular keys
        for key in &self.current_keys {
            parts.push(key_to_string(*key));
        }

        if parts.is_empty() {
            "Press a key combination...".to_string()
        } else {
            parts.join("+")
        }
    }
}

/// UI widget for hotkey recording
pub fn hotkey_field(
    ui: &mut egui::Ui,
    label: &str,
    current_value: &mut String,
    recorder: &mut HotkeyRecorder,
    _field_id: &str,
) -> bool {
    let mut changed = false;

    // Process input when recording
    if recorder.is_recording() {
        if let Some(result) = recorder.process_input(ui.ctx()) {
            *current_value = result;
            changed = true;
        }
    }

    ui.horizontal(|ui| {
        ui.label(label);

        if recorder.is_recording() {
            // Recording mode
            let current_display = recorder.get_current_display();

            // Make the text field focusable to capture keyboard input
            let response = ui.add(
                egui::TextEdit::singleline(&mut current_display.clone())
                    .desired_width(200.0)
                    .interactive(false),
            );

            // Request focus while recording
            response.request_focus();

            if ui.button("Cancel").clicked() {
                recorder.cancel();
            }
        } else {
            // Normal mode
            ui.add(
                egui::TextEdit::singleline(current_value)
                    .desired_width(200.0)
                    .interactive(false),
            );

            if ui.button("Record").clicked() {
                recorder.start_recording();
            }
        }
    });

    changed
}

/// Validate hotkeys don't conflict
pub fn validate_hotkeys(
    wake: &str,
    model: &str,
    panic: &str,
    hide: Option<&str>,
) -> Result<(), String> {
    if wake == model {
        return Err("Wake and Model hotkeys cannot be the same".to_string());
    }
    if wake == panic {
        return Err("Wake and Panic hotkeys cannot be the same".to_string());
    }
    if model == panic {
        return Err("Model and Panic hotkeys cannot be the same".to_string());
    }
    if let Some(hide_key) = hide {
        if !hide_key.is_empty() {
            if wake == hide_key {
                return Err("Wake and Hide hotkeys cannot be the same".to_string());
            }
            if model == hide_key {
                return Err("Model and Hide hotkeys cannot be the same".to_string());
            }
            if panic == hide_key {
                return Err("Panic and Hide hotkeys cannot be the same".to_string());
            }
        }
    }
    Ok(())
}

fn key_to_string(key: egui::Key) -> String {
    match key {
        egui::Key::A => "A",
        egui::Key::B => "B",
        egui::Key::C => "C",
        egui::Key::D => "D",
        egui::Key::E => "E",
        egui::Key::F => "F",
        egui::Key::G => "G",
        egui::Key::H => "H",
        egui::Key::I => "I",
        egui::Key::J => "J",
        egui::Key::K => "K",
        egui::Key::L => "L",
        egui::Key::M => "M",
        egui::Key::N => "N",
        egui::Key::O => "O",
        egui::Key::P => "P",
        egui::Key::Q => "Q",
        egui::Key::R => "R",
        egui::Key::S => "S",
        egui::Key::T => "T",
        egui::Key::U => "U",
        egui::Key::V => "V",
        egui::Key::W => "W",
        egui::Key::X => "X",
        egui::Key::Y => "Y",
        egui::Key::Z => "Z",

        egui::Key::Num0 => "0",
        egui::Key::Num1 => "1",
        egui::Key::Num2 => "2",
        egui::Key::Num3 => "3",
        egui::Key::Num4 => "4",
        egui::Key::Num5 => "5",
        egui::Key::Num6 => "6",
        egui::Key::Num7 => "7",
        egui::Key::Num8 => "8",
        egui::Key::Num9 => "9",

        egui::Key::F1 => "F1",
        egui::Key::F2 => "F2",
        egui::Key::F3 => "F3",
        egui::Key::F4 => "F4",
        egui::Key::F5 => "F5",
        egui::Key::F6 => "F6",
        egui::Key::F7 => "F7",
        egui::Key::F8 => "F8",
        egui::Key::F9 => "F9",
        egui::Key::F10 => "F10",
        egui::Key::F11 => "F11",
        egui::Key::F12 => "F12",

        egui::Key::Space => "Space",
        egui::Key::Tab => "Tab",
        egui::Key::Escape => "Esc",
        egui::Key::Enter => "Enter",
        egui::Key::Backspace => "Backspace",
        egui::Key::Delete => "Delete",
        egui::Key::Insert => "Insert",
        egui::Key::Home => "Home",
        egui::Key::End => "End",
        egui::Key::PageUp => "PageUp",
        egui::Key::PageDown => "PageDown",
        egui::Key::ArrowUp => "Up",
        egui::Key::ArrowDown => "Down",
        egui::Key::ArrowLeft => "Left",
        egui::Key::ArrowRight => "Right",

        _ => return format!("{:?}", key),
    }
    .to_string()
}
