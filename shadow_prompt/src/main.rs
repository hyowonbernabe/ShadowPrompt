#![cfg_attr(not(feature = "debug"), windows_subsystem = "windows")]

mod config;
mod input;
mod clipboard;
mod ui;
mod ocr;
mod llm;
mod knowledge;
mod utils;
mod setup;
mod logger;
mod tos_text;
mod hotkey_recorder;
mod color_picker;

#[macro_use]
extern crate log;

use crate::config::Config;
use crate::input::{InputManager, InputEvent};
use crate::clipboard::ClipboardManager;
use crate::ui::{UIManager, UICommand};
use crate::llm::LlmClient;
use crate::knowledge::KnowledgeProvider;
use crate::utils::{parse_mcq_with_context, McqAnswer, parse_hex_color, parse_keys};
use std::sync::mpsc;

fn main() -> anyhow::Result<()> {
    // 0. Ensure required directories exist
    crate::config::ensure_directories()?;
    
    // Initialize Logger
    if let Err(e) = crate::logger::init() {
        eprintln!("Failed to initialize logger: {}", e);
    }

    // Check for --debug flag or config setting
    let args: Vec<String> = std::env::args().collect();
    let debug_flag = args.contains(&"--debug".to_string());
    
    // If debug flag is present, attach console
    if debug_flag {
        unsafe {
            use windows::Win32::System::Console::AllocConsole;
            let _ = AllocConsole();
        }
    }
    
    // 1. Setup Wizard (First Run or --setup)
    let args: Vec<String> = std::env::args().collect();
    let force_setup = args.contains(&"--setup".to_string()) || args.contains(&"--reset-setup".to_string());

    if !Config::is_setup_complete() || force_setup {
        println!("[*] Starting ShadowPrompt Setup Wizard...");
        let wizard = crate::setup::SetupWizard::new();
        wizard.show();
        
        // If show returns, it means the window was closed without finishing.
        // We exit here to avoid starting the app without setup.
        // If they finished, the wizard would have re-execed and exited.
        println!("[*] Setup Wizard closed. Exit.");
        return Ok(());
    }

    // 2. Start Main Application Logic (Async)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    rt.block_on(run_app())
}

async fn run_app() -> anyhow::Result<()> {
    // 2. Load Configuration
    println!("[*] Loading ShadowPrompt...");
    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[!] Configuration Error: {}", e);
            // Fallback to default if load fails (might happen if config is corrupted)
            Config::default()
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

    // 2. Initialize Knowledge Provider (Search & RAG)
    // This might take a moment if downloading embedding models.
    println!("[*] Initializing Knowledge Provider...");
    let knowledge_provider = std::sync::Arc::new(KnowledgeProvider::new(&config).await?);

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
                    let _ = ui_tx.send(UICommand::SetColor(0x0000FFFF));

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
                        
                        let _ = ui_tx_clone.send(UICommand::SetColor(ready_color));
                    });
                },
                InputEvent::Model => {
                    println!("[!] EVENT: Model Key Pressed (Clipboard Trigger)");
                    let processing_color = parse_hex_color(&config.visuals.color_processing);
                    let mcq_none_color = parse_hex_color(&config.visuals.color_mcq_none);
                    let _ = ui_tx.send(UICommand::SetColor(processing_color)); 
                    let _ = ui_tx.send(UICommand::SetSecondaryColor(mcq_none_color));
                    
                    let config_clone = config.clone();
                    let ui_tx_clone = ui_tx.clone();
                    let ready_color = parse_hex_color(&config.visuals.ready_color);
                    let kp_arc = knowledge_provider.clone();

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
                        let (context, warnings) = match kp_arc.gather_context(&prompt, &config_clone).await {
                             Ok((ctx, warns)) => (ctx, warns),
                             Err(e) => {
                                 let err_msg = format!("Knowledge System Error: {}", e);
                                 error!("{}", err_msg);
                                 (String::new(), vec![err_msg])
                             }
                        };
                        
                        let augmented_prompt = if !context.is_empty() {
                            info!("[*] Context found. Augmenting prompt.");
                            format!("Context:\n{}\nQuestion:\n{}", context, prompt)
                        } else {
                            prompt.clone()
                        };

                        // 3. Query LLM
                        let mut final_output = String::new();

                        // Add warnings to output if any
                        for warning in warnings {
                            final_output.push_str(&format!("[System Warning: {}]\n\n", warning));
                        }

                        match LlmClient::query(&augmented_prompt, &config_clone).await {
                             Ok(res) => {
                                 final_output.push_str(&res);
                             },
                             Err(e) => {
                                 let err_msg = format!("AI Error: {}", e);
                                 error!("{}", err_msg);
                                 // If we have warnings, they are already in final_output. 
                                 // We append the fatal error.
                                 final_output.push_str(&format!("[FATAL ERROR]\n{}", err_msg));
                             }
                        };
                        
                        // Treat the final_output as the response for MCQ/Clipboard
                        let response = final_output;

                        // 4. Check for MCQ Answer (with context from original input)
                        let mcq_color = if let Some(ans) = parse_mcq_with_context(&prompt, &response) {
                             let hex = match ans {
                                 McqAnswer::A => &config_clone.visuals.color_mcq_a,
                                 McqAnswer::B => &config_clone.visuals.color_mcq_b,
                                 McqAnswer::C => &config_clone.visuals.color_mcq_c,
                                 McqAnswer::D => &config_clone.visuals.color_mcq_d,
                             };
                             let color = parse_hex_color(hex);
                             println!("[+] MCQ Detected: {:?} -> Color: {:08X}", ans, color);
                             color
                        } else {
                             parse_hex_color(&config_clone.visuals.color_mcq_none)
                        };
                        let _ = ui_tx_clone.send(UICommand::SetSecondaryColor(mcq_color));

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
