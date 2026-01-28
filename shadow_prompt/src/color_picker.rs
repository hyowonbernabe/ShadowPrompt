//! Color Picker Module
//! Provides a simple color picker widget for the setup wizard.

use eframe::egui;

/// Preset colors for quick selection
const PRESET_COLORS: &[(&str, &str)] = &[
    ("#00FFFF", "Cyan"),
    ("#FF00FF", "Magenta"),
    ("#FFFF00", "Yellow"),
    ("#FF0000", "Red"),
    ("#00FF00", "Green"),
    ("#0000FF", "Blue"),
    ("#FFA500", "Orange"),
    ("#800080", "Purple"),
    ("#FFFFFF", "White"),
    ("#000000", "Black"),
    ("#808080", "Gray"),
    ("#FFC0CB", "Pink"),
];

/// Convert hex string to egui Color32
pub fn hex_to_color32(hex: &str) -> egui::Color32 {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return egui::Color32::WHITE;
    }
    
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    
    egui::Color32::from_rgb(r, g, b)
}

/// Convert egui Color32 to hex string
#[allow(dead_code)]
pub fn color32_to_hex(color: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

/// Color picker widget with preset palette and hex input
/// Returns true if the color was changed
pub fn color_picker(
    ui: &mut egui::Ui,
    label: &str,
    hex_value: &mut String,
) -> bool {
    let mut changed = false;
    let current_color = hex_to_color32(hex_value);
    
    ui.horizontal(|ui| {
        ui.label(label);
        
        // Color preview swatch (clickable to open popup)
        let swatch_response = ui.allocate_response(
            egui::vec2(24.0, 24.0),
            egui::Sense::click(),
        );
        
        // Draw the swatch
        let painter = ui.painter();
        painter.rect_filled(swatch_response.rect, 4.0, current_color);
        painter.rect_stroke(swatch_response.rect, 4.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
        
        // Popup ID
        let popup_id = ui.make_persistent_id(format!("color_picker_{}", label));
        
        if swatch_response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        
        // Popup with color palette
        egui::popup_below_widget(ui, popup_id, &swatch_response, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
            ui.set_min_width(180.0);
            
            ui.label("Select Color:");
            ui.add_space(4.0);
            
            // Grid of preset colors
            egui::Grid::new("color_grid")
                .spacing([4.0, 4.0])
                .show(ui, |ui| {
                    for (i, (hex, name)) in PRESET_COLORS.iter().enumerate() {
                        let color = hex_to_color32(hex);
                        let is_selected = hex_value.to_uppercase() == hex.to_uppercase();
                        
                        let button_response = ui.allocate_response(
                            egui::vec2(28.0, 28.0),
                            egui::Sense::click(),
                        );
                        
                        let painter = ui.painter();
                        let fill_color = color;
                        let stroke = if is_selected {
                            egui::Stroke::new(2.0, egui::Color32::WHITE)
                        } else {
                            egui::Stroke::new(1.0, egui::Color32::GRAY)
                        };
                        
                        painter.rect_filled(button_response.rect, 4.0, fill_color);
                        painter.rect_stroke(button_response.rect, 4.0, stroke);
                        
                        if button_response.clicked() {
                            *hex_value = hex.to_string();
                            changed = true;
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                        
                        button_response.on_hover_text(*name);
                        
                        // New row every 4 colors
                        if (i + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });
            
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);
            
            // Hex input
            ui.horizontal(|ui| {
                ui.label("Hex:");
                let text_response = ui.add(
                    egui::TextEdit::singleline(hex_value)
                        .desired_width(80.0)
                        .char_limit(7)
                );
                if text_response.changed() {
                    changed = true;
                }
            });
        });
        
        // Show hex value next to swatch
        ui.add(
            egui::TextEdit::singleline(hex_value)
                .desired_width(80.0)
                .char_limit(7)
        ).on_hover_text("Click swatch to pick color");
    });
    
    changed
}

/// Compact color picker for MCQ colors (inline version)
pub fn color_picker_compact(
    ui: &mut egui::Ui,
    label: &str,
    hex_value: &mut String,
) -> bool {
    let mut changed = false;
    let current_color = hex_to_color32(hex_value);
    
    ui.horizontal(|ui| {
        // Small swatch
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(18.0, 18.0),
            egui::Sense::click(),
        );
        
        let painter = ui.painter();
        painter.rect_filled(rect, 3.0, current_color);
        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
        
        let popup_id = ui.make_persistent_id(format!("color_compact_{}", label));
        
        if response.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }
        
        egui::popup_below_widget(ui, popup_id, &response, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
            ui.set_min_width(150.0);
            
            // Compact grid
            egui::Grid::new("color_grid_compact")
                .spacing([3.0, 3.0])
                .show(ui, |ui| {
                    for (i, (hex, name)) in PRESET_COLORS.iter().enumerate() {
                        let color = hex_to_color32(hex);
                        
                        let (rect, button_response) = ui.allocate_exact_size(
                            egui::vec2(22.0, 22.0),
                            egui::Sense::click(),
                        );
                        
                        let painter = ui.painter();
                        painter.rect_filled(rect, 3.0, color);
                        painter.rect_stroke(rect, 3.0, egui::Stroke::new(1.0, egui::Color32::GRAY));
                        
                        if button_response.clicked() {
                            *hex_value = hex.to_string();
                            changed = true;
                            ui.memory_mut(|mem| mem.close_popup());
                        }
                        
                        button_response.on_hover_text(*name);
                        
                        if (i + 1) % 4 == 0 {
                            ui.end_row();
                        }
                    }
                });
            
            ui.add_space(4.0);
            
            // Hex input
            let text_response = ui.add(
                egui::TextEdit::singleline(hex_value)
                    .desired_width(70.0)
                    .char_limit(7)
            );
            if text_response.changed() {
                changed = true;
            }
        });
        
        ui.label(label);
    });
    
    changed
}
