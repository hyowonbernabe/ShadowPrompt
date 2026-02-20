use reqwest;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching list of open tabs from Chrome Debug Port...");
    let res = reqwest::get("http://127.0.0.1:9222/json/list").await?;
    let text = res.text().await?;
    
    let json: Value = serde_json::from_str(&text)?;
    if let Some(array) = json.as_array() {
        for (i, target) in array.iter().enumerate() {
            let title = target.get("title").and_then(|t| t.as_str()).unwrap_or("Unknown Title");
            let url = target.get("url").and_then(|u| u.as_str()).unwrap_or("Unknown URL");
            let target_type = target.get("type").and_then(|t| t.as_str()).unwrap_or("Unknown Type");
            
            println!("Tab [{}]:", i);
            println!("  Type: {}", target_type);
            println!("  Title: {}", title);
            println!("  URL: {}", url);
            println!("---");
        }
    } else {
        println!("Response was not a JSON array:\n{}", text);
    }
    
    Ok(())
}
