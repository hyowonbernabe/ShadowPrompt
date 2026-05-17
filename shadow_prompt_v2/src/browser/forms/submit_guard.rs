// Submit button detection + refusal. Auto-pagination clicks "Next"; when only
// "Submit" remains, the flow stops cleanly. Submission is the user's act.

pub fn next_button_selector() -> &'static str {
    // TODO: real selector for Google Forms Next button.
    r#"div[role="button"][aria-label*="Next"]"#
}

pub fn submit_button_selector() -> &'static str {
    r#"div[role="button"][aria-label*="Submit"]"#
}
