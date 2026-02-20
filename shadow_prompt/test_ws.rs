use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Polling Chrome Debug Port...");
    let res = reqwest::get("http://127.0.0.1:9222/json/version").await?;
    let status = res.status();
    println!("Status: {}", status);
    
    let text = res.text().await?;
    println!("Raw Response Body: {}", text);
    
    let json: Value = serde_json::from_str(&text)?;
    let ws_url = json.get("webSocketDebuggerUrl");
    println!("WebSocket URL: {:?}", ws_url);
    
    Ok(())
}
