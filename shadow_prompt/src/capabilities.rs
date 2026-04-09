use crate::config::Config;

pub struct ModelCapabilities;

impl ModelCapabilities {
    #[allow(dead_code)]
    pub fn supports_search(config: &Config) -> bool {
        match config.models.provider.as_str() {
            "groq" => config
                .models
                .groq
                .as_ref()
                .map(|c| c.supports_search)
                .unwrap_or(false),
            "openrouter" => config
                .models
                .openrouter
                .as_ref()
                .map(|c| c.supports_search)
                .unwrap_or(false),
            "ollama" => config
                .models
                .ollama
                .as_ref()
                .map(|c| c.supports_search)
                .unwrap_or(false),
            _ => false,
        }
    }

    pub fn supports_vision(config: &Config) -> bool {
        match config.models.provider.as_str() {
            "groq" => config
                .models
                .groq
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false),
            "openrouter" => config
                .models
                .openrouter
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false),
            "ollama" => config
                .models
                .ollama
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false),
            _ => false,
        }
    }

    /// Like `supports_vision` but checks the browser provider when it is configured.
    /// Use this when evaluating vision capability for Google Forms automation.
    pub fn supports_vision_browser(config: &Config) -> bool {
        let provider = if !config.models.browser_provider.is_empty() {
            config.models.browser_provider.as_str()
        } else {
            config.models.provider.as_str()
        };
        match provider {
            "groq" => config
                .models
                .groq
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false),
            "openrouter" => config
                .models
                .openrouter
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false),
            "ollama" => config
                .models
                .ollama
                .as_ref()
                .map(|c| c.supports_vision)
                .unwrap_or(false),
            _ => false,
        }
    }

    #[allow(dead_code)]
    pub fn get_current_model_id(config: &Config) -> Option<String> {
        match config.models.provider.as_str() {
            "groq" => config.models.groq.as_ref().map(|c| c.model_id.clone()),
            "openrouter" => config
                .models
                .openrouter
                .as_ref()
                .map(|c| c.model_id.clone()),
            "ollama" => config.models.ollama.as_ref().map(|c| c.model_id.clone()),
            _ => None,
        }
    }
}
