use headless_chrome::Browser;
use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching browser websocket from json/version...");
    let res = reqwest::get("http://127.0.0.1:9222/json/version").await?;
    let text = res.text().await?;
    let json: Value = serde_json::from_str(&text)?;
    let ws_url = json.get("webSocketDebuggerUrl").and_then(|v| v.as_str()).unwrap().to_string();
    
    println!("Connecting to Browser WS: {}", ws_url);
    let browser = Browser::connect(ws_url)?;
    
    println!("Waiting for tabs to populate...");
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    let tabs = browser.get_tabs().lock().unwrap();
    println!("Found {} tabs.", tabs.len());
    for (i, t) in tabs.iter().enumerate() {
        println!("Tab [{}]: {}", i, t.get_url());
    }
    
    Ok(())
}
