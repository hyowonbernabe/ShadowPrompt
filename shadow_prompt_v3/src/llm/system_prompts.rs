// System prompts embedded at compile time. Design doc §9: one shared base + a small per-mode
// delivery addendum, not two fully-duplicated files like v2 had.

pub const BASE: &str = include_str!("system_prompts/base.txt");
pub const DELIVERY_GENERAL: &str = include_str!("system_prompts/delivery_general.txt");
pub const DELIVERY_FORMS: &str = include_str!("system_prompts/delivery_forms.txt");

pub fn general_prompt() -> String {
    format!("{BASE}\n\n{DELIVERY_GENERAL}")
}

pub fn forms_prompt() -> String {
    format!("{BASE}\n\n{DELIVERY_FORMS}")
}
