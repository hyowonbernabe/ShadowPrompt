use anyhow::{anyhow, Result};
use headless_chrome::Tab;
use std::sync::Arc;
use headless_chrome::protocol::cdp::Network::CookieParam;

pub fn get_google_cookies() -> Result<Vec<rookie::Cookie>> {
    // Only fetch cookies for google.com domains to minimize payload
    let domains = Some(vec!["google.com"]);
    
    // Fallback to searching all installed browsers, but prioritizing Chrome
    let cookies = rookie::chrome(domains.clone())
        .or_else(|_| rookie::edge(domains.clone()))
        .or_else(|_| rookie::firefox(domains.clone()))
        .or_else(|_| rookie::brave(domains.clone()))
        .map_err(|e| anyhow!("Rookie failed to extract cookies: {}", e))?;
        
    Ok(cookies)
}

pub fn inject_cookies_into_tab(tab: &Arc<Tab>, cookies: Vec<rookie::Cookie>) -> Result<()> {
    let mut mapped_cookies = Vec::new();
    
    for c in cookies {
        mapped_cookies.push(CookieParam {
            name: c.name,
            value: c.value,
            url: None,
            domain: Some(c.host),
            path: Some(c.path),
            secure: Some(c.secure),
            http_only: Some(c.http_only),
            same_site: None,
            expires: None, // We can skip mapping expires purely for session riding
            priority: None,
            same_party: None,
            source_scheme: None,
            source_port: None,
            partition_key: None,
        });
    }
    
    tab.set_cookies(mapped_cookies)
        .map_err(|e| anyhow!("Failed to set cookies: {:?}", e))?;

    Ok(())
}
