// Static capability table — model id → supported features.
// Rebuilt when new model families need support.

#[derive(Debug, Clone, Copy)]
pub struct Capabilities {
    pub vision: bool,
    pub reasoning: bool,
    pub prompt_caching: bool,
    pub context_window: usize,
}

pub fn for_model(id: &str) -> Capabilities {
    if id.starts_with("anthropic/claude-sonnet-4") || id.starts_with("anthropic/claude-opus-4") {
        return Capabilities {
            vision: true,
            reasoning: true,
            prompt_caching: true,
            context_window: 1_000_000,
        };
    }
    if id.starts_with("google/gemini-2.5-pro") || id.starts_with("google/gemini-3-pro") {
        return Capabilities {
            vision: true,
            reasoning: true,
            prompt_caching: false,
            context_window: 1_000_000,
        };
    }
    if id.starts_with("openai/gpt-5") || id.starts_with("openai/gpt-4o") {
        return Capabilities {
            vision: true,
            reasoning: true,
            prompt_caching: false,
            context_window: 200_000,
        };
    }
    // Unknown — conservative; startup validation rejects if vision missing.
    Capabilities {
        vision: false,
        reasoning: false,
        prompt_caching: false,
        context_window: 128_000,
    }
}
