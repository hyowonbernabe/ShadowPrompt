use anyhow::{Result, Context};
use reqwest::{Client, header};
use regex::Regex;
use crate::config::SearchConfig;

pub async fn perform_search(query: &str, config: &SearchConfig) -> Result<String> {
    // Try Serper first if configured
    if config.engine == "serper" {
        match perform_serper_search(query, config.max_results, &config.serper_api_key).await {
            Ok(results) if !results.is_empty() => return Ok(results),
            Ok(_) => {
                log::warn!("Serper returned empty results, falling back to DuckDuckGo...");
            }
            Err(e) => {
                log::warn!("Serper failed: {}, falling back to DuckDuckGo...", e);
            }
        }
    }
    
    // Fallback to DuckDuckGo
    perform_duckduckgo_search(query, config.max_results).await
}

async fn perform_serper_search(query: &str, max_results: usize, api_key: &Option<String>) -> Result<String> {
    let api_key = api_key.as_ref().context("Serper API key not configured")?;
    
    let client = Client::new();
    let res = client.post("https://google.serper.dev/search")
        .header("X-API-KEY", api_key)
        .json(&serde_json::json!({
            "q": query,
            "num": max_results
        }))
        .send()
        .await?
        .text()
        .await?;
    
    let json: serde_json::Value = serde_json::from_str(&res)
        .context("Failed to parse Serper response")?;
    
    let organic = json["organic"].as_array()
        .context("No results from Serper")?;
    
    let mut results = String::new();
    for item in organic.iter().take(max_results) {
        let title = item["title"].as_str().unwrap_or("");
        let snippet = item["snippet"].as_str().unwrap_or("");
        results.push_str(&format!("- {}\n  {}\n", title, snippet));
    }
    
    if results.is_empty() {
        return Ok("No search results found.".to_string());
    }
    
    Ok(results)
}

async fn perform_duckduckgo_search(query: &str, max_results: usize) -> Result<String> {
    let client = Client::new();
    let url = "https://html.duckduckgo.com/html/";
    
    // DDG requires User-Agent
    let res = client.post(url)
        .form(&[("q", query)])
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await?
        .text()
        .await?;

    // Regex to extract snippets (Simplistic approach)
    // Looking for: <a class="result__snippet" ...>(.*?)</a>
    // Note: HTML structure changes, but this is the standard "html" version which is older/simpler.
    let re_snippet = Regex::new(r#"<a[^>]*class="[^"]*result__snippet[^"]*"[^>]*>(.*?)</a>"#)?;
    let re_tags = Regex::new(r"<[^>]*>")?; // To strip internal tags

    let mut results = String::new();


    // We iterate manually to match title/snippet pairs (rough heuristic)
    // Actually, splitting by result__body might be safer, but let's try simple capture first.
    // The HTML version lists results sequentially.
    
    // Better Regex: Capture the whole block? No, streams are hard.
    // Let's just grab snippets. The snippet usually contains the answer context.
    
    for (count, cap) in re_snippet.captures_iter(&res).enumerate() {
        if count >= max_results { break; }
        
        let raw_snippet = &cap[1];
        let clean_snippet = re_tags.replace_all(raw_snippet, "").to_string();
        
        // Manual decode for basic entities to save deps
        let final_text = clean_snippet
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'");

        results.push_str(&format!("- {}\n", final_text));
    }

    if results.is_empty() {
        return Ok("No search results found.".to_string());
    }

    Ok(results)
}
