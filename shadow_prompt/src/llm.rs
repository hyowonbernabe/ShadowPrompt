use anyhow::{Result, Context};
use reqwest::Client;
use serde_json::{json, Value};
use crate::config::Config;

pub struct LlmClient;

impl LlmClient {
    pub async fn query(prompt: &str, config: &Config) -> Result<String> {
        let client = Client::new();
        
        match config.models.provider.as_str() {
            "groq" => Self::query_groq(&client, prompt, config).await,
            "auto" => {
                // Priority: Groq -> OpenRouter
                if let Some(groq) = &config.models.groq {
                    if !groq.api_key.is_empty() && groq.api_key != "your_groq_api_key_here" {
                         match Self::query_groq(&client, prompt, config).await {
                             Ok(res) => return Ok(res),
                             Err(e) => error!("Auto-Groq failed: {}. Falling back...", e),
                         }
                    }
                }
                
                if let Some(or) = &config.models.openrouter {
                    if !or.api_key.is_empty() && or.api_key != "your_openrouter_api_key_here" {
                        return Self::query_openrouter(&client, prompt, config).await;
                    }
                }
                
                anyhow::bail!("Auto-Provider: No valid API keys found for Groq or OpenRouter. Please configure them in Setup or config.toml.")
            },

            "openrouter" => Self::query_openrouter(&client, prompt, config).await,
            "ollama" => Self::query_ollama(&client, prompt, config).await,
            "github_copilot" => anyhow::bail!("GitHub Copilot provider not fully implemented yet"),
            _ => anyhow::bail!("Unknown provider: {}", config.models.provider),
        }
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
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("Failed to parse Groq response")?
            .to_string();

        Ok(content)
    }

    fn load_system_prompt() -> String {
        std::fs::read_to_string("config/system_prompt.txt")
            .or_else(|_| std::fs::read_to_string("../config/system_prompt.txt"))
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
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .context("Failed to parse LLM response")?
            .to_string();

        Ok(content)
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


}
