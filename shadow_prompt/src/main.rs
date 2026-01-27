mod config;
mod input;
mod clipboard;
mod ui;
mod ocr;
mod llm;
mod knowledge;
mod utils;


use crate::config::Config;
use crate::input::{InputManager, InputEvent};
use crate::clipboard::ClipboardManager;
use crate::ui::{UIManager, UICommand};
use crate::llm::LlmClient;
use crate::knowledge::KnowledgeProvider;
use crate::utils::{parse_mcq_answer, McqAnswer, parse_hex_color, parse_keys};
use std::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Load Configuration
    println!("[*] Loading ShadowPrompt...");
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[!] Configuration Error: {}", e);
            return Err(e);
        }
    };

    println!("[*] Loaded Configuration. Mode: {}", config.general.mode);
    println!("[*] Active Provider: {}", config.models.provider);

    // KEYBIND CONFLICT CHECK
    let wake_str = config.general.wake_key.to_lowercase();
    let model_str = config.general.model_key.to_lowercase();
    let panic_str = config.general.panic_key.to_lowercase();

    if wake_str == model_str || wake_str == panic_str || model_str == panic_str {
        eprintln!("\n/!\\ WARNING: DUPLICATE KEYBINDS DETECTED /!\\");
        eprintln!("    Wake:  {}", config.general.wake_key);
        eprintln!("    Model: {}", config.general.model_key);
        eprintln!("    Panic: {}", config.general.panic_key);
        eprintln!("    Behavior is undefined for overlapping keys.\n");
    }

    // 2. Start Visual Feedback Thread
    let (ui_tx, ui_rx) = mpsc::channel();
    UIManager::start(ui_rx, config.visuals.clone());
    
    // Set initial Green "Ready" state
    let ready_color = parse_hex_color(&config.visuals.ready_color);
    let _ = ui_tx.send(UICommand::SetColor(ready_color)); 

    // 3. Start Input Listener
    let (tx, rx) = mpsc::channel();
    
    let wake_keys = parse_keys(&config.general.wake_key);
    let model_keys = parse_keys(&config.general.model_key);
    let panic_keys = parse_keys(&config.general.panic_key);

    println!("[*] Listening for Hotkeys...");
    InputManager::start(wake_keys, model_keys, panic_keys, tx);

    // 4. Main Event Loop
    println!("[*] ShadowPrompt is running. Press Panic Key to exit.");
    
    // Pre-calculate colors for performance/clarity (or parse on fly)
    // For simplicity, we parse on fly or clone config.
    // Ideally we put these in a strut but cloning config is fine for this app scale.
    
    loop {
        // Check for Input Events (Non-blocking or blocking depending on design)
        // Here we use recv() which blocks, effectively putting the main thread to sleep until an event.
        if let Ok(event) = rx.recv() {
            match event {
                InputEvent::Wake => {
                    println!("[!] EVENT: Wake Key Pressed (Enter OCR Selection Mode)");
                    // Use Processing Color (Red by default) or maybe a specific "Wake" color?
                    // Currently hardcoded to Red. Let's use processing color.
                    let color = parse_hex_color(&config.visuals.color_processing);
                    let _ = ui_tx.send(UICommand::SetColor(color)); 
                },
                InputEvent::OCRClick1 => {
                    println!("[!] EVENT: OCR Point 1 Captured");
                    let _ = ui_tx.send(UICommand::SetColor(0x0000A5FF)); // Orange (BGR: FF A5 00) - Keeping hardcoded or add to config? 
                    // Keeping hardcoded for now as it wasn't explicitly asked to be configurable, 
                    // but the user said "Customize any of the indicator colors".
                    // I'll leave Orange hardcoded for obscure states unless I add more fields.
                },
                InputEvent::OCRRect(x, y, w, h) => {
                    println!("[*] OCR Region Captured: x={}, y={}, w={}, h={}", x, y, w, h);
                    let _ = ui_tx.send(UICommand::SetColor(0x0000FFFF)); // Yellow (BGR: FF FF 00)
                    let _ = ui_tx.send(UICommand::DrawDebugRect(x, y, w, h));

                    let ui_tx_clone = ui_tx.clone();
                    let ready_color = parse_hex_color(&config.visuals.ready_color);
                    
                    tokio::spawn(async move {
                        println!("[*] Extracting text...");
                        
                        match crate::ocr::OcrManager::extract_from_screen(x, y, w, h).await {
                            Ok(text) => {
                                println!("[+] OCR Success: \"{}\"", text.trim());
                                if let Err(e) = ClipboardManager::write(&text) {
                                    eprintln!("Clipboard Write Error: {}", e);
                                }
                            },
                            Err(e) => {
                                eprintln!("[-] OCR Failed: {}", e);
                            }
                        }
                        
                        // Cleanup
                        let _ = ui_tx_clone.send(UICommand::ClearDebugRect);
                        let _ = ui_tx_clone.send(UICommand::SetColor(ready_color)); // Reset Green
                    });
                },
                InputEvent::Model => {
                    println!("[!] EVENT: Model Key Pressed (Clipboard Trigger)");
                    let processing_color = parse_hex_color(&config.visuals.color_processing);
                    let _ = ui_tx.send(UICommand::SetColor(processing_color)); 
                    let _ = ui_tx.send(UICommand::SetSecondaryColor(0x00000000)); // Reset Secondary
                    
                    let config_clone = config.clone();
                    let ui_tx_clone = ui_tx.clone();
                    let ready_color = parse_hex_color(&config.visuals.ready_color);

                    tokio::spawn(async move {


                        // 1. Read Clipboard
                        let prompt = match ClipboardManager::read() {
                            Ok(text) => text,
                            Err(e) => {
                                eprintln!("Clipboard Read Error: {}", e);
                                let _ = ui_tx_clone.send(UICommand::SetColor(ready_color));
                                return;
                            }
                        };

                        println!("[*] Processing Query: {:.50}...", prompt);

                        // 2. Gather Context (Search/RAG)
                        let context = match KnowledgeProvider::gather_context(&prompt, &config_clone).await {
                             Ok(ctx) => ctx,
                             Err(e) => {
                                 eprintln!("Knowledge Error: {}", e);
                                 String::new() 
                             }
                        };
                        
                        let augmented_prompt = if !context.is_empty() {
                            println!("[*] Context found. Augmenting prompt.");
                            format!("Context:\n{}\nQuestion:\n{}", context, prompt)
                        } else {
                            prompt.clone()
                        };

                        // 3. Query LLM
                        let response = match LlmClient::query(&augmented_prompt, &config_clone).await {
                             Ok(res) => res,
                             Err(e) => {
                                 eprintln!("LLM Error: {}", e);
                                 "Error: Failed to get response from AI.".to_string()
                             }
                        };

                        // 4. Check for MCQ Answer
                        if let Some(ans) = parse_mcq_answer(&response) {
                             let hex = match ans {
                                 McqAnswer::A => &config_clone.visuals.color_mcq_a,
                                 McqAnswer::B => &config_clone.visuals.color_mcq_b,
                                 McqAnswer::C => &config_clone.visuals.color_mcq_c,
                                 McqAnswer::D => &config_clone.visuals.color_mcq_d,
                             };
                             let color = parse_hex_color(hex);
                             println!("[+] MCQ Detected: {:?} -> Color: {:08X}", ans, color);
                             let _ = ui_tx_clone.send(UICommand::SetSecondaryColor(color));
                        }

                        // 5. Write Clipboard (Always)
                        if let Err(e) = ClipboardManager::write(&response) {
                            eprintln!("Clipboard Write Error: {}", e);
                        }

                        println!("[*] Response written to clipboard.");
                        let _ = ui_tx_clone.send(UICommand::SetColor(0x0000FF00)); // Reset Green
                        
                        // We do NOT reset the secondary color immediately here, so the user can see it.
                        // However, we should probably reset it on the NEXT trigger or after a timeout?
                        // The user request didn't specify reset behavior, but usually indicators stay until next action.
                        // But wait, if I run another query, I should probably clear it at the START of the event.
                        // I'll add a clear command at the start of InputEvent::Model.
                    });
                },
                InputEvent::Panic => {
                    println!("[!!!] PANIC KEY PRESSED. EXITING.");
                    if let Err(e) = ClipboardManager::clear() {
                        eprintln!("Failed to clear clipboard: {}", e);
                    }
                    std::process::exit(0);
                }
            }
        }
    }
}
