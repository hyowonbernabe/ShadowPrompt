mod config;
mod input;
mod clipboard;
mod ui;
mod ocr;
mod llm;
mod knowledge;
mod auth;

use crate::config::Config;
use crate::input::{InputManager, InputEvent, parse_keys};
use crate::clipboard::ClipboardManager;
use crate::ui::{UIManager, UICommand};
use crate::llm::LlmClient;
use crate::knowledge::KnowledgeProvider;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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

    // 2. Start Visual Feedback Thread
    let (ui_tx, ui_rx) = mpsc::channel();
    UIManager::start(ui_rx);
    
    // Set initial Green "Ready" state
    let _ = ui_tx.send(UICommand::SetColor(0x0000FF00)); // Green

    // 3. Start Input Listener
    let (tx, rx) = mpsc::channel();
    
    let wake_keys = parse_keys(&config.general.wake_key);
    let model_keys = parse_keys(&config.general.model_key);
    let panic_keys = parse_keys(&config.general.panic_key);

    println!("[*] Listening for Hotkeys...");
    InputManager::start(wake_keys, model_keys, panic_keys, tx);

    // 4. Main Event Loop
    println!("[*] ShadowPrompt is running. Press Panic Key to exit.");
    
    loop {
        // Check for Input Events (Non-blocking or blocking depending on design)
        // Here we use recv() which blocks, effectively putting the main thread to sleep until an event.
        if let Ok(event) = rx.recv() {
            match event {
                InputEvent::Wake => {
                    println!("[!] EVENT: Wake Key Pressed (Enter OCR Selection Mode)");
                    let _ = ui_tx.send(UICommand::SetColor(0x000000FF)); // Red
                },
                InputEvent::OCRClick1 => {
                    println!("[!] EVENT: OCR Point 1 Captured");
                    let _ = ui_tx.send(UICommand::SetColor(0x0000A5FF)); // Orange (BGR: FF A5 00)
                },
                InputEvent::OCRRect(x, y, w, h) => {
                    println!("[*] OCR Region Captured: x={}, y={}, w={}, h={}", x, y, w, h);
                    let _ = ui_tx.send(UICommand::SetColor(0x0000FFFF)); // Yellow (BGR: FF FF 00)
                    let _ = ui_tx.send(UICommand::DrawDebugRect(x, y, w, h));

                    let ui_tx_clone = ui_tx.clone();
                    
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
                        let _ = ui_tx_clone.send(UICommand::SetColor(0x0000FF00)); // Reset Green
                    });
                },
                InputEvent::Model => {
                    println!("[!] EVENT: Model Key Pressed (Clipboard Trigger)");
                    let _ = ui_tx.send(UICommand::SetColor(0x000000FF)); // Red
                    
                    let config_clone = config.clone();
                    let ui_tx_clone = ui_tx.clone();

                    tokio::spawn(async move {
                        // 0. Authentication Check
                        if let Some(auth_config) = &config_clone.auth {
                            if let Some(google_config) = &auth_config.google {
                                if google_config.enabled {
                                    // Check if we have a token
                                    let token = crate::auth::AuthManager::load_token();
                                    
                                    if token.is_none() {
                                        println!("[*] Authentication Required.");
                                        let _ = ui_tx_clone.send(UICommand::SetColor(0x0000A5FF)); // Orange (Waiting for Auth)
                                        
                                        // Trigger Auth Flow
                                        let res = crate::auth::google::perform_auth(
                                            google_config.client_id.clone(),
                                            google_config.client_secret.clone(),
                                            8006
                                        ).await;

                                        match res {
                                            Ok(data) => {
                                                println!("[+] Authentication Successful.");
                                                if let Err(e) = crate::auth::AuthManager::save_token(&data) {
                                                    eprintln!("[-] Failed to save token: {}", e);
                                                }
                                                // Proceed...
                                            },
                                            Err(e) => {
                                                eprintln!("[-] Authentication Failed: {}", e);
                                                let _ = ui_tx_clone.send(UICommand::SetColor(0x000000FF)); // Red Error
                                                return; 
                                            }
                                        }
                                    } else {
                                        // TODO: Check expiry and refresh if needed
                                    }
                                }
                            }
                        }

                        // 1. Read Clipboard
                        let prompt = match ClipboardManager::read() {
                            Ok(text) => text,
                            Err(e) => {
                                eprintln!("Clipboard Read Error: {}", e);
                                let _ = ui_tx_clone.send(UICommand::SetColor(0x0000FF00));
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

                        // 3. Write Clipboard
                        if let Err(e) = ClipboardManager::write(&response) {
                            eprintln!("Clipboard Write Error: {}", e);
                        }

                        println!("[*] Response written to clipboard.");
                        let _ = ui_tx_clone.send(UICommand::SetColor(0x0000FF00)); // Reset Green
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
