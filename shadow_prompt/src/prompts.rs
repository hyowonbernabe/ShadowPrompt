//! Single source of truth for all LLM prompt construction.
//!
//! Two system prompts exist as external files:
//!   config/system_prompt.txt       — general answering (clipboard, OCR, vision)
//!   config/forms_system_prompt.txt — Google Forms automation (JSON actions)
//!
//! This module owns every *user-message* prompt and the shared context builder.

/// Navigation mode for Google Forms automation.
pub enum FormsNav {
    /// Click Next after answering each page. Never click Submit.
    Auto,
    /// Single-page mode — do not touch any navigation buttons.
    SinglePage,
}

/// Builds the user message for vision (image) queries.
/// Injects web/RAG context when available (sourced from OCR of the same region).
/// Output format rules are enforced by the system prompt.
pub fn build_vision_query(web: &str, local: &str) -> String {
    let mut prompt = context_section(web, local);
    prompt.push_str(
        "Before answering: identify and record EVERY number, label, and measurement \
         visible in this image — including small or edge labels. \
         Then work through the problem completely. \
         Output only the final answer per the output rules.",
    );
    prompt
}

/// Builds the augmented user message for text-based queries (clipboard, OCR).
/// Injects web search results and local knowledge when present, then appends
/// the question under a `[QUESTION]` header.
pub fn build_text_query(web: &str, local: &str, question: &str) -> String {
    let mut prompt = context_section(web, local);
    prompt.push_str("[QUESTION]\n");
    prompt.push_str(question);
    prompt
}

/// Builds the user message for Google Forms automation.
/// Context (web + local) is injected at the top; the full form JSON and
/// action-format rules follow.
pub fn build_forms_prompt(web: &str, local: &str, form_json: &str, nav: FormsNav) -> String {
    let ctx = context_section(web, local);

    let nav_rule = match nav {
        FormsNav::Auto => {
            "CRITICAL RULE 1: If there is a `navigation` button of type `next`, include a click \
             action for it as the VERY LAST item in your array after all question answers.\n\
             CRITICAL RULE 2: NEVER click a button of type `submit`."
        }
        FormsNav::SinglePage => {
            "CRITICAL RULE: SINGLE-PAGE MODE. Do NOT interact with ANY navigation buttons. \
             Do not click `next` or `submit`."
        }
    };

    format!(
        "CONTEXT (use this to answer questions more accurately):\n{ctx}\n\
        ---\n\n\
        You are an automated quiz solver filling out a Google Form. \
        The JSON contains `questions` and `navigation` buttons. Answer every unanswered question.\n\n\
        QUESTION TYPES — use exactly these action formats:\n\
        - \"radio\": Click ONE option. Action: {{\"id\":\"<option_id>\",\"action\":\"click\"}}\n\
        - \"checkbox\": Multi-select — click ALL correct options (one action per option). Action: {{\"id\":\"<option_id>\",\"action\":\"click\"}}\n\
        - \"text\": Type answer. Action: {{\"id\":\"<input_id>\",\"action\":\"type\",\"value\":\"<answer>\"}}\n\
        - \"dropdown\" with \"id\" field (native select): Action: {{\"id\":\"<select_id>\",\"action\":\"select_native\",\"value\":\"<exact option text>\"}}\n\
        - \"dropdown\" with \"trigger_id\" field (custom dropdown): Action: {{\"id\":\"<trigger_id>\",\"action\":\"dropdown_select\",\"value\":\"<exact option text>\"}}\n\
        - \"grid_radio\": Matrix — one click per row. Each row in `grid_rows` needs exactly one selected column. Action: {{\"id\":\"<row_col_id>\",\"action\":\"click\"}}\n\
        - \"grid_checkbox\": Matrix with checkboxes — click all applicable options per row. Action: {{\"id\":\"<row_col_id>\",\"action\":\"click\"}}\n\
        - \"date\": Fill each field in `fields` by label (Month, Day, Year with numeric values). Action: {{\"id\":\"<field_id>\",\"action\":\"type\",\"value\":\"<number>\"}}\n\
        - \"time\": Fill each field in `fields` (Hour, Minute). Action: {{\"id\":\"<field_id>\",\"action\":\"type\",\"value\":\"<number>\"}}. For AM/PM field with is_select=true: {{\"id\":\"<ampm_id>\",\"action\":\"select_native\",\"value\":\"<AM or PM>\"}}\n\
        - \"datetime\": Combined date+time — fill all fields in `fields` by label (Month, Day, Year, Hour, Minute). Same actions as \"date\" and \"time\" combined.\n\
        - \"select_option\" (advanced): Direct click on an already-visible `[role=\"option\"]` element. Action: {{\"id\":\"<option_id>\",\"action\":\"select_option\"}}\n\n\
        {nav_rule}\n\n\
        Return ONLY a valid JSON array of actions. No markdown, no explanation, no extra text.\n\
        Form JSON: {form_json}",
    )
}

/// Builds the shared context header injected at the top of every text-based prompt.
/// Returns an empty string when both web and local are empty.
fn context_section(web: &str, local: &str) -> String {
    let mut section = String::new();
    if !web.is_empty() {
        section.push_str("[WEB SEARCH RESULTS]\n");
        section.push_str(web);
        section.push_str("\n\n");
    }
    if !local.is_empty() {
        section.push_str("[LOCAL KNOWLEDGE]\n");
        section.push_str(local);
        section.push_str("\n\n");
    }
    section
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_text_query_with_context() {
        let result = build_text_query("web results", "local docs", "What is 2+2?");
        assert!(result.contains("[WEB SEARCH RESULTS]\nweb results"));
        assert!(result.contains("[LOCAL KNOWLEDGE]\nlocal docs"));
        assert!(result.ends_with("[QUESTION]\nWhat is 2+2?"));
    }

    #[test]
    fn test_build_text_query_no_context() {
        let result = build_text_query("", "", "What is 2+2?");
        assert_eq!(result, "[QUESTION]\nWhat is 2+2?");
    }

    #[test]
    fn test_build_text_query_web_only() {
        let result = build_text_query("web results", "", "Q?");
        assert!(result.contains("[WEB SEARCH RESULTS]"));
        assert!(!result.contains("[LOCAL KNOWLEDGE]"));
    }

    #[test]
    fn test_build_forms_prompt_contains_nav_auto() {
        let result = build_forms_prompt("", "", "{}", FormsNav::Auto);
        assert!(result.contains("NEVER click a button of type `submit`"));
        assert!(!result.contains("SINGLE-PAGE MODE"));
    }

    #[test]
    fn test_build_forms_prompt_contains_nav_single() {
        let result = build_forms_prompt("", "", "{}", FormsNav::SinglePage);
        assert!(result.contains("SINGLE-PAGE MODE"));
        assert!(!result.contains("NEVER click a button of type `submit`"));
    }

    #[test]
    fn test_build_vision_query_no_context() {
        let result = build_vision_query("", "");
        assert!(result.contains("identify and record EVERY number"));
        assert!(result.contains("Output only the final answer"));
    }

    #[test]
    fn test_build_vision_query_with_context() {
        let result = build_vision_query("web results", "");
        assert!(result.contains("[WEB SEARCH RESULTS]\nweb results"));
        assert!(result.contains("identify and record EVERY number"));
    }
}
