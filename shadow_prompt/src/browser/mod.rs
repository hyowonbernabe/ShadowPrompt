pub mod cookies;
pub mod injector;

use crate::config::Config;
use crate::ui::UICommand;
use crate::llm::ModelUseCase;
use anyhow::{anyhow, Result};
use base64::Engine;
use headless_chrome::{Browser, LaunchOptions};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::process::Command;
use tokio::time::{sleep, Duration};
use serde_json::Value;

pub fn launch_incognito_debugger() -> Result<()> {
    // Attempt standard locations for chrome.exe or msedge.exe
    let possible_paths = vec![
        r#"C:\Program Files\Google\Chrome\Application\chrome.exe"#,
        r#"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe"#,
        r#"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"#,
    ];

    let mut binary_path = String::new();
    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            binary_path = path.to_string();
            break;
        }
    }

    if binary_path.is_empty() {
        return Err(anyhow!("Could not find Chrome/Edge executable in standard paths."));
    }

    // Create a temporary debug profile directory so Chrome doesn't fuse with the user's existing Chrome process
    let debug_profile_dir = std::env::temp_dir().join("shadow_chrome_debug");
    let _ = std::fs::create_dir_all(&debug_profile_dir);

    Command::new(binary_path)
        .arg("--incognito")
        .arg("--remote-debugging-port=9222")
        .arg(format!("--user-data-dir={}", debug_profile_dir.display()))
        // Start detached so ShadowPrompt doesn't block
        .spawn()
        .map_err(|e| anyhow!("Failed to spawn browser process: {}", e))?;

    Ok(())
}

pub async fn execute_form_flow(
    url: Option<&str>,
    _password: Option<&str>,
    config: Arc<Config>,
    ui_tx: Sender<UICommand>,
    is_auto: bool,
    knowledge_provider: Arc<crate::knowledge::KnowledgeProvider>,
) -> Result<()> {
    let debug_mode = config.general.debug;
    let ui_tx_clone = ui_tx.clone();
    let send_ui = move |msg: String| {
        if debug_mode {
            let _ = ui_tx_clone.send(UICommand::SetOverlayText(msg));
        }
    };

    send_ui("🧠 Checking for active sessions...".to_string());

    // 1. Try to connect to an existing Remote Debugging Session (Port 9222)
    let debug_ws_url = match reqwest::get("http://127.0.0.1:9222/json/version").await {
        Ok(res) => {
            if let Ok(json) = res.json::<Value>().await {
                json.get("webSocketDebuggerUrl")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        },
        Err(_) => None,
    };

    let browser;
    let tab;
    let mut needs_navigation = true;

    if let Some(ws_url) = debug_ws_url {
        // Connected to active debug session!
        send_ui("🔌 Connecting to active Incognito tab...".to_string());
        
        browser = Browser::connect(ws_url)
            .map_err(|e| anyhow!("Failed to connect to Debugger WebSocket: {}", e))?;
            
        let mut found_tab = None;
        send_ui("⏳ Waiting for tabs to sync...".to_string());
        
        // Headless Chrome needs a moment to receive Target.targetCreated events over the websocket
        for _ in 0..10 {
            {
                let tabs = browser.get_tabs().lock().unwrap();
                for t in tabs.iter() {
                    let u = t.get_url();
                    if u.contains("forms.gle") || u.contains("docs.google.com/forms") {
                        found_tab = Some(t.clone());
                        break;
                    }
                }
            }
            if found_tab.is_some() {
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }
        
        tab = found_tab.ok_or_else(|| anyhow!("Could not find an open Google Forms tab in Incognito. Open the form first."))?;
        needs_navigation = false;
        // We do NOT need to extract or inject cookies because this browser process 
        // already holds the user's active session state in RAM, and we are on the live page!
    } else {
        // We MUST have a URL from the clipboard if we are launching cold.
        let _ = url.ok_or_else(|| anyhow!("No valid Google Forms URL found on Clipboard (required for standard launch)."))?;

        // Fallback to Rookie Auth Extraction Strategy
        send_ui("🍪 Extracting local session...".to_string());

        let cookies = cookies::get_google_cookies()
            .map_err(|e| anyhow!("Failed to extract Google session cookies: {}", e))?;

        if cookies.is_empty() {
            return Err(anyhow!("No active Google account found. Please sign in or use Incognito approach."));
        }

        send_ui("🌐 Launching headless browser...".to_string());

        let options = LaunchOptions::default_builder()
            .headless(true)
            .sandbox(false)
            .enable_gpu(false)
            .build()
            .map_err(|e| anyhow!("Failed to build browser options: {}", e))?;

        browser = Browser::new(options)
            .map_err(|e| anyhow!("Failed to launch browser: {}", e))?;

        tab = browser.new_tab()
            .map_err(|e| anyhow!("Failed to open tab: {}", e))?;

        // Wait slightly for tab init
        sleep(Duration::from_millis(500)).await;

        cookies::inject_cookies_into_tab(&tab, cookies)?;
    }

    if needs_navigation {
        // If we are navigating, we are in cold launch, so we know `url` is Some.
        let target_url = url.unwrap();
        send_ui("🧭 Navigating to form...".to_string());
        tab.navigate_to(target_url)
            .map_err(|e| anyhow!("Failed to navigate: {}", e))?;
        tab.wait_until_navigated()
            .map_err(|e| anyhow!("Navigation timeout: {}", e))?;
    }

    sleep(Duration::from_secs(2)).await;

    // Check for Permission Error
    let title = tab.get_title().unwrap_or_default();
    if title.to_lowercase().contains("you need permission") || title.to_lowercase().contains("sign in") {
        return Err(anyhow!("Access Denied. Ensure your .edu active profile is logged in to Chrome."));
    }

    let mut page_count = 1;

    loop {
        send_ui(format!("🧠 Reading Page {}...", page_count));

        // 5. Extract JSON
        let extraction_res = tab.evaluate(injector::EXTRACTOR_JS, false)
            .map_err(|e| anyhow!("Extraction Script Error: {}", e))?;

        let json_val = extraction_res.value.ok_or(anyhow!("Extractor returned null"))?;
        let form_json = json_val.as_str().unwrap_or("[]");
        
        println!("\n[DEBUG] EXTRACTED JSON:\n{}", form_json);

        send_ui(format!("🤖 Calculating Page {}...", page_count));

        // 6a. Gather knowledge context from form questions
        let questions_summary = extract_questions_for_context(form_json);

        let bundle = match knowledge_provider.gather_context(&questions_summary, &config).await {
            Ok(b) => b,
            Err(e) => {
                let msg = format!("Knowledge System Error: {}", e);
                eprintln!("[!] {}", msg);
                crate::knowledge::ContextBundle {
                    web: String::new(),
                    local: String::new(),
                    warnings: vec![msg],
                }
            }
        };

        // 6b. Build prompt with context
        let nav = if is_auto {
            crate::prompts::FormsNav::Auto
        } else {
            crate::prompts::FormsNav::SinglePage
        };
        let prompt = crate::prompts::build_forms_prompt(&bundle.web, &bundle.local, form_json, nav);

        // 6c. If any questions contain image_url fields and the browser model supports vision,
        //     fetch those images and send a multimodal request instead of text-only.
        let image_urls = extract_image_urls(form_json);
        let llm_res = if !image_urls.is_empty()
            && crate::capabilities::ModelCapabilities::supports_vision_browser(&config)
        {
            let http_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(config.http.read_timeout_secs))
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .build()
                .unwrap_or_default();
            let images = fetch_images_as_base64(&image_urls, &http_client).await;
            println!("[DEBUG] Forms: fetched {}/{} images", images.len(), image_urls.len());
            crate::llm::LlmClient::query_with_multiple_images(
                &prompt,
                &images,
                &config,
                ModelUseCase::Browser,
            )
            .await?
        } else {
            crate::llm::LlmClient::query(&prompt, &config, ModelUseCase::Browser).await?
        };

        // Clean markdown and conversational preamble if present
        let mut raw_actions = llm_res.replace("```json", "").replace("```", "").trim().to_string();
        if let Some(start) = raw_actions.find('[') {
            if let Some(end) = raw_actions.rfind(']') {
                if end >= start {
                    raw_actions = raw_actions[start..=end].to_string();
                }
            }
        }
        
        println!("\n[DEBUG] LLM OUTPUT:\n{}", raw_actions);

        send_ui(format!("⚡ Injecting Page {}...", page_count));

        // 7. Inject Actions
        let injection_script = injector::build_injector_call(&raw_actions);
        let inject_res = tab.evaluate(&injection_script, true)
            .map_err(|e| anyhow!("Injection Script Error: {}", e))?;
        if let Some(val) = inject_res.value {
            if let Some(s) = val.as_str() {
                if s.starts_with("ERROR:") {
                    println!("[WARN] Injector JS error: {}", s);
                }
            }
        }

        // 8. Determine if we should loop
        if !is_auto {
            send_ui("✅ Single-Page Execution Complete.".to_string());
            break;
        }

        // We do a naive check: if the LLM output contained the `nav_next_btn` ID, we assume it pressed Next.
        if raw_actions.contains("\"nav_next_btn\"") || (raw_actions.contains("\"action\":\"click\"") && form_json.contains("\"nav_next_btn\"")) {
            send_ui("⏳ Waiting for Next Page...".to_string());
            // Wait for autosave flush and DOM Page transition
            sleep(Duration::from_secs(3)).await;
            page_count += 1;
            
            if page_count > 10 {
                return Err(anyhow!("Pagination limit exceeded (10 pages max). Aborting."));
            }
            continue;
        } else {
            // Reached the end (Submit button page, or LLM failed to click next)
            send_ui("✅ Execution Complete. Review and Submit manually.".to_string());
            break;
        }
    }

    Ok(())
}

fn extract_questions_for_context(form_json: &str) -> String {
    let mut questions = Vec::new();

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(form_json) {
        if let Some(arr) = json.get("questions").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(question) = item.get("question").and_then(|v| v.as_str()) {
                    questions.push(question.to_string());
                }
            }
        }
    }

    questions.join(" ")
}

/// Collect every `image_url` field from form questions (set by EXTRACTOR_JS when a question has an img).
fn extract_image_urls(form_json: &str) -> Vec<String> {
    let mut urls = Vec::new();
    if let Ok(json) = serde_json::from_str::<Value>(form_json) {
        if let Some(arr) = json.get("questions").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(url) = item.get("image_url").and_then(|v| v.as_str()) {
                    if !url.is_empty() {
                        urls.push(url.to_string());
                    }
                }
            }
        }
    }
    urls
}

/// Fetch each URL and return successfully-encoded base64 strings.
/// Handles inline `data:image/...;base64,<data>` URLs without an HTTP round-trip.
async fn fetch_images_as_base64(urls: &[String], client: &reqwest::Client) -> Vec<String> {
    let mut results = Vec::new();
    for url in urls {
        // Inline data URL — extract the base64 payload directly
        if let Some(rest) = url.strip_prefix("data:") {
            if let Some(comma_pos) = rest.find(',') {
                let payload = &rest[comma_pos + 1..];
                results.push(payload.to_string());
                continue;
            }
        }

        // Regular HTTP(S) URL — fetch and encode
        match client.get(url).send().await {
            Ok(res) if res.status().is_success() => match res.bytes().await {
                Ok(bytes) => {
                    results.push(
                        base64::engine::general_purpose::STANDARD.encode(&bytes),
                    );
                }
                Err(e) => log::warn!("Image byte read failed ({}): {}", url, e),
            },
            Ok(res) => log::warn!("Image fetch HTTP {} ({})", res.status(), url),
            Err(e) => log::warn!("Image fetch error ({}): {}", url, e),
        }
    }
    results
}
