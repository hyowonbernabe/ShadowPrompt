use anyhow::{Result, Context};
use reqwest::Client;
use serde_json::{json, Value};
use crate::config::Config;
use std::time::Duration;
use tokio::time::sleep;

pub struct LlmClient;

impl LlmClient {
    pub async fn query(prompt: &str, config: &Config) -> Result<String> {
        let connect_timeout = Duration::from_secs(config.http.connect_timeout_secs);
        let read_timeout = Duration::from_secs(config.http.read_timeout_secs);
        
        let client = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .timeout(read_timeout)
            .build()?;
        
        match config.models.provider.as_str() {
            "groq" => Self::query_with_retry_groq(&client, prompt, config).await,
            "openrouter" => Self::query_with_retry_openrouter(&client, prompt, config).await,
            "ollama" => Self::query_with_retry_ollama(&client, prompt, config).await,
            "auto" => Self::query_with_fallback(&client, prompt, config).await,
            "github_copilot" => anyhow::bail!("GitHub Copilot provider not fully implemented yet"),
            _ => anyhow::bail!("Unknown provider: {}", config.models.provider),
        }
    }

    /// Retry wrapper for Groq
    async fn query_with_retry_groq(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        let max_retries = 3;
        let base_delay = Duration::from_secs(1);
        
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match Self::query_groq(client, prompt, config).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    let error_str = last_error.as_ref().unwrap().to_string().to_lowercase();
                    
                    if Self::is_retryable_error(&error_str) && attempt < max_retries - 1 {
                        let delay = base_delay * 2u32.pow(attempt as u32);
                        log::warn!("Groq attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries failed")))
    }

    /// Retry wrapper for OpenRouter
    async fn query_with_retry_openrouter(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        let max_retries = 3;
        let base_delay = Duration::from_secs(1);
        
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match Self::query_openrouter(client, prompt, config).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    let error_str = last_error.as_ref().unwrap().to_string().to_lowercase();
                    
                    if Self::is_retryable_error(&error_str) && attempt < max_retries - 1 {
                        let delay = base_delay * 2u32.pow(attempt as u32);
                        log::warn!("OpenRouter attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries failed")))
    }

    /// Retry wrapper for Ollama
    async fn query_with_retry_ollama(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        let max_retries = 3;
        let base_delay = Duration::from_secs(1);
        
        let mut last_error = None;
        
        for attempt in 0..max_retries {
            match Self::query_ollama(client, prompt, config).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    let error_str = last_error.as_ref().unwrap().to_string().to_lowercase();
                    
                    if Self::is_retryable_error(&error_str) && attempt < max_retries - 1 {
                        let delay = base_delay * 2u32.pow(attempt as u32);
                        log::warn!("Ollama attempt {} failed, retrying in {:?}...", attempt + 1, delay);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retries failed")))
    }

    /// Check if an error is retryable (transient failures)
    fn is_retryable_error(error_str: &str) -> bool {
        error_str.contains("429") 
            || error_str.contains("rate limit")
            || error_str.contains("too many requests")
            || error_str.contains("quota exceeded")
            || error_str.contains("5")
            || error_str.contains("connection")
            || error_str.contains("timeout")
            || error_str.contains("network")
            || error_str.contains("temporarily unavailable")
            || error_str.contains("service unavailable")
    }

    /// Auto-LLM Selection with fallback chain: Groq -> OpenRouter -> Ollama
    /// Each provider is tried with retry logic before falling back
    async fn query_with_fallback(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        // Priority 1: Groq (fastest, free tier)
        if let Some(groq) = &config.models.groq {
            if !groq.api_key.is_empty() && groq.api_key != "your_groq_api_key_here" {
                match Self::query_with_retry_groq(client, prompt, config).await {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        let error_str = e.to_string().to_lowercase();
                        if Self::is_retryable_error(&error_str) {
                            log::warn!("Groq failed (retryable): {}. Falling back...", e);
                        } else {
                            log::error!("Groq failed: {}. Trying next provider...", e);
                        }
                    }
                }
            }
        }
        
        // Priority 2: OpenRouter (wider model selection)
        if let Some(or) = &config.models.openrouter {
            if !or.api_key.is_empty() && or.api_key != "your_openrouter_api_key_here" {
                match Self::query_with_retry_openrouter(client, prompt, config).await {
                    Ok(res) => return Ok(res),
                    Err(e) => {
                        let error_str = e.to_string().to_lowercase();
                        if Self::is_retryable_error(&error_str) {
                            log::warn!("OpenRouter failed (retryable): {}. Falling back...", e);
                        } else {
                            log::error!("OpenRouter failed: {}. Trying next provider...", e);
                        }
                    }
                }
            }
        }
        
        // Priority 3: Ollama (local, no rate limits)
        if config.models.ollama.is_some() {
            match Self::query_with_retry_ollama(client, prompt, config).await {
                Ok(res) => return Ok(res),
                Err(e) => {
                    log::error!("Ollama failed: {}", e);
                }
            }
        }
        
        anyhow::bail!("All providers failed. Please check your API keys and network connection.")
    }

    async fn query_groq(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        let groq_config = config.models.groq.as_ref()
            .context("Groq config missing")?;

        let system_prompt = Self::load_system_prompt();

        let body = json!({
            "model": groq_config.model_id,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        });

        let res = client.post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", groq_config.api_key.trim()))
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let err_text = res.text().await?;
             anyhow::bail!("Groq API Error: {}", err_text);
        }

        let json: Value = res.json().await?;
        Self::extract_content(&json)
    }

    fn load_system_prompt() -> String {
        let path = crate::config::get_exe_dir()
            .join("config")
            .join("system_prompt.txt");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|_| "You are a concise assistant.".to_string())
    }

    async fn query_openrouter(client: &Client, prompt: &str, config: &Config) -> Result<String> {
        let openrouter_config = config.models.openrouter.as_ref()
            .context("OpenRouter config missing")?;

        let system_prompt = Self::load_system_prompt();

        let body = json!({
            "model": openrouter_config.model_id,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": prompt}
            ]
        });

        let res = client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", openrouter_config.api_key.trim()))
            // .header("HTTP-Referer", "http://localhost:3000") // Optional
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let err_text = res.text().await?;
             anyhow::bail!("OpenRouter API Error: {}", err_text);
        }

        let json: Value = res.json().await?;
        Self::extract_content(&json)
    }

    async fn query_ollama(client: &Client, prompt: &str, config: &Config) -> Result<String> {
         let ollama_config = config.models.ollama.as_ref().context("Ollama config missing")?;
         let system_prompt = Self::load_system_prompt();
         
         let body = json!({
             "model": ollama_config.model_id,
             "prompt": prompt,
             "system": system_prompt,
             "stream": false
         });

         let url = format!("{}/api/generate", ollama_config.base_url);
         let res = client.post(&url)
             .json(&body)
             .send()
             .await?;

         if !res.status().is_success() {
             anyhow::bail!("Ollama Error: {}", res.status());
         }

         let json: Value = res.json().await?;
        let response = json["response"].as_str().context("No response field")?.to_string();
        Ok(response)
    }

    /// Test provider connectivity with a simple prompt
    pub async fn test_provider(provider: &str, config: &Config) -> Result<String> {
        let connect_timeout = Duration::from_secs(5);
        let read_timeout = Duration::from_secs(15);
        
        let client = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .timeout(read_timeout)
            .build()?;
        
        let test_prompt = "Reply with only the word 'OK' if you can read this.";
        
        match provider {
            "groq" => Self::query_groq(&client, test_prompt, config).await,
            "openrouter" => Self::query_openrouter(&client, test_prompt, config).await,
            "ollama" => Self::query_ollama(&client, test_prompt, config).await,
            _ => anyhow::bail!("Unknown provider: {}", provider),
        }
    }

    /// Query LLM with an image (for vision-capable models)
    pub async fn query_with_image(prompt: &str, image_base64: &str, config: &Config) -> Result<String> {
        let connect_timeout = Duration::from_secs(config.http.connect_timeout_secs);
        let read_timeout = Duration::from_secs(config.http.read_timeout_secs);
        
        let client = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .timeout(read_timeout)
            .build()?;

        match config.models.provider.as_str() {
            "groq" => Self::query_groq_with_image(&client, prompt, image_base64, config).await,
            "openrouter" => Self::query_openrouter_with_image(&client, prompt, image_base64, config).await,
            "ollama" => Self::query_ollama_with_image(&client, prompt, image_base64, config).await,
            "auto" => {
                if let Some(groq) = &config.models.groq {
                    if !groq.api_key.is_empty() && groq.api_key != "your_groq_api_key_here" {
                        if let Ok(res) = Self::query_groq_with_image(&client, prompt, image_base64, config).await {
                            return Ok(res);
                        }
                    }
                }
                if let Some(or) = &config.models.openrouter {
                    if !or.api_key.is_empty() && or.api_key != "your_openrouter_api_key_here" {
                        if let Ok(res) = Self::query_openrouter_with_image(&client, prompt, image_base64, config).await {
                            return Ok(res);
                        }
                    }
                }
                anyhow::bail!("No vision-capable provider available")
            }
            _ => anyhow::bail!("Provider does not support vision: {}", config.models.provider),
        }
    }

    async fn query_groq_with_image(client: &Client, prompt: &str, image_base64: &str, config: &Config) -> Result<String> {
        let groq_config = config.models.groq.as_ref()
            .context("Groq config missing")?;

        let system_prompt = Self::load_system_prompt();

        let body = json!({
            "model": groq_config.model_id,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}}
                ]}
            ]
        });

        let res = client.post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", groq_config.api_key.trim()))
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let err_text = res.text().await?;
             anyhow::bail!("Groq Vision API Error: {}", err_text);
        }

        let json: Value = res.json().await?;
        Self::extract_content(&json)
    }

    async fn query_openrouter_with_image(client: &Client, prompt: &str, image_base64: &str, config: &Config) -> Result<String> {
        let openrouter_config = config.models.openrouter.as_ref()
            .context("OpenRouter config missing")?;

        let system_prompt = Self::load_system_prompt();

        let body = json!({
            "model": openrouter_config.model_id,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": [
                    {"type": "text", "text": prompt},
                    {"type": "image_url", "image_url": {"url": format!("data:image/png;base64,{}", image_base64)}}
                ]}
            ]
        });

        let res = client.post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", openrouter_config.api_key.trim()))
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
             let err_text = res.text().await?;
             anyhow::bail!("OpenRouter Vision API Error: {}", err_text);
        }

        let json: Value = res.json().await?;
        Self::extract_content(&json)
    }

    async fn query_ollama_with_image(client: &Client, prompt: &str, image_base64: &str, config: &Config) -> Result<String> {
        let ollama_config = config.models.ollama.as_ref()
            .context("Ollama config missing")?;

        let system_prompt = Self::load_system_prompt();
        let body = json!({
            "model": ollama_config.model_id,
            "prompt": prompt,
            "system": system_prompt,
            "images": [image_base64]
        });

        let url = format!("{}/api/generate", ollama_config.base_url);
        let res = client.post(&url)
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            anyhow::bail!("Ollama Vision Error: {}", res.status());
        }

        let json: Value = res.json().await?;
        let response = json["response"].as_str().context("No response field")?.to_string();
        Ok(response)
    }

    pub fn extract_content(json: &Value) -> Result<String> {
        let content = &json["choices"][0]["message"]["content"];

        // Case 1: plain string (most models)
        if let Some(s) = content.as_str() {
            return Ok(s.to_string());
        }

        // Case 2: content array (multimodal Gemini responses)
        if let Some(arr) = content.as_array() {
            let text = arr
                .iter()
                .filter_map(|part| {
                    if part["type"].as_str() == Some("text") {
                        part["text"].as_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            if !text.is_empty() {
                return Ok(text);
            }
        }

        anyhow::bail!("Could not extract text content from LLM response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_content_string() {
        let json = json!({
            "choices": [{"message": {"content": "Paris"}}]
        });
        assert_eq!(LlmClient::extract_content(&json).unwrap(), "Paris");
    }

    #[test]
    fn test_extract_content_array() {
        let json = json!({
            "choices": [{"message": {"content": [
                {"type": "text", "text": "The answer is"},
                {"type": "text", "text": "Nitrogen"}
            ]}}]
        });
        assert_eq!(LlmClient::extract_content(&json).unwrap(), "The answer is\nNitrogen");
    }

    #[test]
    fn test_extract_content_array_skips_non_text() {
        let json = json!({
            "choices": [{"message": {"content": [
                {"type": "image_url", "image_url": {"url": "data:..."}},
                {"type": "text", "text": "B) Nitrogen"}
            ]}}]
        });
        assert_eq!(LlmClient::extract_content(&json).unwrap(), "B) Nitrogen");
    }

    #[test]
    fn test_extract_content_missing_fails() {
        let json = json!({"choices": [{"message": {"content": null}}]});
        assert!(LlmClient::extract_content(&json).is_err());
    }
}
