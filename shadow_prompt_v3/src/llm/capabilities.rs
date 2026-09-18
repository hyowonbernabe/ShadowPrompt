// Hand-maintained per-model capability table — deliberately not a dynamic catalog lookup.
// OPENCODE.md §8 "Skip" list: with a fixed, small model set (design doc §8's 5-model chain,
// not user-configurable to arbitrary models), there's no product need to dynamically discover
// model capabilities from a live catalog. A small hand-maintained struct is the right size.
//
// The one thing this table exists to prevent: silently sending an image to a model that can't
// read one. Confirmed via live OpenRouter research (docs/OPENROUTER.md):
//   - google/gemini-3.5-flash-lite: text+image+video+file+audio input. Vision: yes.
//   - anthropic/claude-sonnet-5: text+image+file input. Vision: yes.
//   - nex-agi/nex-n2.5-mini:free: text+image input. Vision: yes.
//   - inclusionai/ling-3.0-flash-fin:free: text-only. Vision: NO.
//   - inclusionai/ling-3.0-flash-sante:free: text-only. Vision: NO.

pub fn supports_vision(model_id: &str) -> bool {
    !matches!(
        model_id,
        "inclusionai/ling-3.0-flash-fin:free" | "inclusionai/ling-3.0-flash-sante:free"
    )
}

/// Filters a model fallback chain down to vision-capable entries only. Call this before
/// building any request whose initial content includes an image (Screenshot Query, or a Forms
/// page with image questions) — never send the unfiltered chain when there's an image in the
/// request, or a fallback landing on a text-only model either errors or silently drops the
/// picture.
pub fn filter_for_vision(models: &[String]) -> Vec<String> {
    models.iter().filter(|m| supports_vision(m)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ling_models_are_not_vision_capable() {
        assert!(!supports_vision("inclusionai/ling-3.0-flash-fin:free"));
        assert!(!supports_vision("inclusionai/ling-3.0-flash-sante:free"));
    }

    #[test]
    fn the_other_three_are_vision_capable() {
        assert!(supports_vision("google/gemini-3.5-flash-lite"));
        assert!(supports_vision("anthropic/claude-sonnet-5"));
        assert!(supports_vision("nex-agi/nex-n2.5-mini:free"));
    }

    #[test]
    fn filtering_preserves_order_and_drops_text_only() {
        let chain = vec![
            "google/gemini-3.5-flash-lite".to_string(),
            "anthropic/claude-sonnet-5".to_string(),
            "inclusionai/ling-3.0-flash-sante:free".to_string(),
            "inclusionai/ling-3.0-flash-fin:free".to_string(),
            "nex-agi/nex-n2.5-mini:free".to_string(),
        ];
        let filtered = filter_for_vision(&chain);
        assert_eq!(
            filtered,
            vec![
                "google/gemini-3.5-flash-lite".to_string(),
                "anthropic/claude-sonnet-5".to_string(),
                "nex-agi/nex-n2.5-mini:free".to_string(),
            ]
        );
    }
}
