mod config;

use crate::config::Config;
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
            // In a real stealth app, we might log to file and exit silently.
            // For dev/debug, we print to stderr.
            return Err(e);
        }
    };

    println!("[*] Loaded Configuration. Mode: {}", config.general.mode);
    println!("[*] Active Provider: {}", config.models.provider);

    // 2. Start Visual Feedback Thread (Placeholder for now)
    let visuals_config = config.visuals.clone();
    thread::spawn(move || {
        println!("[Visuals] Green Dot Initialized (Mock)");
        // TODO: Implement actual GDI drawing here
    });

    // 3. Start Input Listener (Placeholder)
    println!("[*] Listening for Hotkeys: Wake={}, Model={}, Panic={}", 
        config.general.wake_key, 
        config.general.model_key, 
        config.general.panic_key
    );

    // 4. Main Loop / Heartbeat
    loop {
        // Heartbeat check (USB existence) would go here
        thread::sleep(Duration::from_secs(5));
    }
}
