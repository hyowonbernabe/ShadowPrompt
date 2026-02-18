use anyhow::{Result, Context};
use reqwest::{Client, header};
use regex::Regex;
use crate::config::SearchConfig;

pub async fn perform_search(query: &str, config: &SearchConfig) -> Result<String> {
    log::info!("[Search] Query: '{}', Engine: {}, Max Results: {}", query, config.engine, config.max_results);
    
    // Try Serper first if configured
    if config.engine == "serper" {
        log::info!("[Search] Attempting Serper.dev...");
        match perform_serper_search(query, config.max_results, &config.serper_api_key).await {
            Ok(results) if !results.is_empty() => {
                log::info!("[Search] Serper returned {} results", results.lines().count() / 2);
                return Ok(results);
            }
            Ok(_) => {
                log::warn!("[Search] Serper returned empty results, falling back to DuckDuckGo...");
            }
            Err(e) => {
                log::error!("[Search] Serper failed with error: {}. Falling back to DuckDuckGo...", e);
            }
        }
    } else {
        log::info!("[Search] Using DuckDuckGo (engine config: {})", config.engine);
    }
    
    // Fallback to DuckDuckGo
    log::info!("[Search] Attempting DuckDuckGo...");
    match perform_duckduckgo_search(query, config.max_results).await {
        Ok(results) => {
            log::info!("[Search] DuckDuckGo returned {} results", results.lines().count() / 2);
            Ok(results)
        }
        Err(e) => {
            log::error!("[Search] DuckDuckGo also failed: {}. Search completely unavailable.", e);
            Err(e)
        }
    }
}

async fn perform_serper_search(query: &str, max_results: usize, api_key: &Option<String>) -> Result<String> {
    let api_key = api_key.as_ref().context("Serper API key not configured. Please add serper_api_key in config.toml")?;
    
    log::debug!("[Search] Serper API key present, making request...");
    
    let client = Client::new();
    let res = client.post("https://google.serper.dev/search")
        .header("X-API-KEY", api_key)
        .json(&serde_json::json!({
            "q": query,
            "num": max_results
        }))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Serper network error: {}. Check your internet connection.", e))?;
    
    let status = res.status();
    let body = res.text().await?;
    
    if !status.is_success() {
        let err_msg = if body.contains("API_KEY") || body.contains("unauthorized") {
            "Invalid Serper API key. Check your serper_api_key in config.toml"
        } else if body.contains("rate") || body.contains("quota") {
            "Serper API rate limit or quota exceeded"
        } else {
            "Serper API error"
        };
        return Err(anyhow::anyhow!("{} (status: {}): {}", err_msg, status, &body[..body.len().min(200)]));
    }
    
    let json: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| anyhow::anyhow!("Serper returned invalid JSON: {}. Response: {}", e, &body[..body.len().min(500)]))?;
    
    let organic = json["organic"].as_array()
        .context("Serper response missing 'organic' results array")?;
    
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
    log::debug!("[Search] DuckDuckGo: making request for query: '{}'", query);
    
    let client = Client::new();
    let url = "https://html.duckduckgo.com/html/";
    
    // DDG requires User-Agent
    let res = client.post(url)
        .form(&[("q", query)])
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("DuckDuckGo network error: {}. Check your internet connection.", e))?;
    
    let status = res.status();
    if !status.is_success() {
        return Err(anyhow::anyhow!("DuckDuckGo returned error status: {}. This may indicate rate limiting.", status));
    }
    
    let body = res.text().await
        .map_err(|e| anyhow::anyhow!("DuckDuckGo failed to read response: {}", e))?;

    // Regex to extract snippets
    let re_snippet = Regex::new(r#"<a[^>]*class="[^"]*result__snippet[^"]*"[^>]*>(.*?)</a>"#)
        .map_err(|e| anyhow::anyhow!("DuckDuckGo: Failed to compile regex: {}", e))?;
    let re_tags = Regex::new(r"<[^>]*>")
        .map_err(|e| anyhow::anyhow!("DuckDuckGo: Failed to compile regex: {}", e))?;

    let mut results = String::new();

    for (count, cap) in re_snippet.captures_iter(&body).enumerate() {
        if count >= max_results { break; }
        
        let raw_snippet = &cap[1];
        let clean_snippet = re_tags.replace_all(raw_snippet, "").to_string();
        
        // Manual decode for basic entities
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
        log::warn!("[Search] DuckDuckGo: No results found for query: '{}'. HTML structure may have changed.", query);
        return Ok("No search results found.".to_string());
    }

    log::debug!("[Search] DuckDuckGo: successfully extracted {} results", results.lines().count());
    Ok(results)
}
