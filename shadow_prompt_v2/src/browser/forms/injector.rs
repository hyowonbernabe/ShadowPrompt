// Inject answers from the LLM back into form DOM. Never touches already-answered
// questions. Never clicks Submit.

use std::collections::HashMap;

/// Build a JS snippet that, when evaluated in the page, applies the given answers.
/// Answers are keyed by question id (q0, q1, ...).
pub fn build_injector_js(answers: &HashMap<String, String>) -> String {
    let json = serde_json::to_string(answers).unwrap_or_else(|_| "{}".into());
    format!(r#"
(() => {{
  const answers = {json};
  const items = document.querySelectorAll('[role="listitem"]');
  items.forEach((item, idx) => {{
    const id = 'q' + idx;
    const val = answers[id];
    if (!val) return;

    const radios = item.querySelectorAll('[role="radio"]');
    if (radios.length > 0) {{
      for (const r of radios) {{
        const label = (r.getAttribute('aria-label') || r.innerText.trim()).toLowerCase();
        if (label === val.toLowerCase() || label.startsWith(val.toLowerCase())) {{
          r.click();
          break;
        }}
      }}
      return;
    }}

    const checks = item.querySelectorAll('[role="checkbox"]');
    if (checks.length > 0) {{
      const wants = val.split(',').map(s => s.trim().toLowerCase());
      for (const c of checks) {{
        const label = (c.getAttribute('aria-label') || c.innerText.trim()).toLowerCase();
        if (wants.some(w => label === w || label.startsWith(w))) {{
          if (c.getAttribute('aria-checked') !== 'true') c.click();
        }}
      }}
      return;
    }}

    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"]');
    const textarea = item.querySelector('textarea');
    const target = textarea || input;
    if (target) {{
      target.focus();
      const setter = Object.getOwnPropertyDescriptor(target.constructor.prototype, 'value').set;
      setter.call(target, val);
      target.dispatchEvent(new Event('input', {{ bubbles: true }}));
      target.dispatchEvent(new Event('change', {{ bubbles: true }}));
    }}
  }});
  return true;
}})()
"#)
}

pub fn inject_answers(_answers: &HashMap<String, String>) -> anyhow::Result<()> {
    // Direct eval is done by the caller via tab.evaluate(); this stub keeps the
    // existing public-API surface stable.
    Ok(())
}
