// Hand-maintained per-model capability table — deliberately not a dynamic catalog lookup.
// OPENCODE.md §8 "Skip" list: with a fixed, small model set (design doc §8's 5-model chain,
// not user-configurable to arbitrary models), there's no product need to dynamically discover
// model capabilities from a live catalog. A small hand-maintained struct is the right size.
//
// The one thing this table exists to prevent: silently sending an image to a model that can't
// read one. Checked against OpenRouter's live model list (input_modalities) on 2026-11-05:
//   - anthropic/claude-sonnet-5.5: text+image+file input. Vision: yes.
//   - google/gemini-3.8-flash: text+image+video+file+audio input. Vision: yes.
//   - nvidia/nemotron-3.5-super-vl:free: Vision: yes.
//   - dots-studio/dots-3-note-preview:free: Vision: yes.
//   - nvidia/nemotron-3-ultra-550b-a55b:free: text-only. Vision: NO.

pub fn supports_vision(model_id: &str) -> bool {
    !matches!(model_id, "nvidia/nemotron-3-ultra-550b-a55b:free")
}

/// Filters a model fallback chain down to vision-capable entries only. Call this before
/// building any request whose content (new message or history) includes an image — never send
/// the unfiltered chain when there's an image in the request, or a fallback landing on a
/// text-only model either errors or silently drops the picture.
pub fn filter_for_vision(models: &[String]) -> Vec<String> {
    models.iter().filter(|m| supports_vision(m)).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_preserves_order_and_drops_text_only() {
        let chain: Vec<String> = crate::config::schema::default_model_chain();
        assert_eq!(
            filter_for_vision(&chain),
            vec![
                "anthropic/claude-sonnet-5.5".to_string(),
                "google/gemini-3.8-flash".to_string(),
                "nvidia/nemotron-3.5-super-vl:free".to_string(),
                "dots-studio/dots-3-note-preview:free".to_string(),
            ]
        );
    }
}
