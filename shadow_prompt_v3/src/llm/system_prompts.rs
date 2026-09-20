// System prompts embedded at compile time. Design doc §9: one shared base + a small per-mode
// delivery addendum, not two fully-duplicated files like v2 had.

pub const BASE: &str = include_str!("system_prompts/base.txt");
pub const DELIVERY_GENERAL: &str = include_str!("system_prompts/delivery_general.txt");
pub const DELIVERY_FORMS: &str = include_str!("system_prompts/delivery_forms.txt");
/// v3 legacy Forms (design doc §7.4) — same base, different delivery contract: one structured
/// JSON answer map as plain text, not a `fill_page` tool call.
pub const DELIVERY_FORMS_LEGACY: &str = include_str!("system_prompts/delivery_forms_legacy.txt");

pub fn general_prompt() -> String {
    format!("{BASE}\n\n{DELIVERY_GENERAL}")
}

/// v3 new (§7.3, parked/secondary) — delivers via the `fill_page` tool.
pub fn forms_prompt() -> String {
    format!("{BASE}\n\n{DELIVERY_FORMS}")
}

/// v3 legacy (§7.4, default/primary) — delivers as one structured JSON text answer.
pub fn forms_legacy_prompt() -> String {
    format!("{BASE}\n\n{DELIVERY_FORMS_LEGACY}")
}
