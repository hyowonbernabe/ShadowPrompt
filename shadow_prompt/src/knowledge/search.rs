use anyhow::Result;
use reqwest::{Client, header};
use regex::Regex;

pub async fn perform_search(query: &str, max_results: usize) -> Result<String> {
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
    let mut count = 0;

    // We iterate manually to match title/snippet pairs (rough heuristic)
    // Actually, splitting by result__body might be safer, but let's try simple capture first.
    // The HTML version lists results sequentially.
    
    // Better Regex: Capture the whole block? No, streams are hard.
    // Let's just grab snippets. The snippet usually contains the answer context.
    
    for cap in re_snippet.captures_iter(&res) {
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
        count += 1;
    }

    if results.is_empty() {
        return Ok("No search results found.".to_string());
    }

    Ok(results)
}
