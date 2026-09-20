// Answer injection for v3 legacy Forms — design doc §7.4. Ported from shadow_prompt_v2's
// `injector.rs` (fuzzy label matching, not exact-string), patched with a live-DOM re-check right
// before acting on each field/row: every branch below checks the field's/row's current live
// state and skips itself if already filled, regardless of what the answer map says for it. This
// is the hard write-lock design doc §7.4 commits to — "the model has no code path to alter an
// existing value no matter what it outputs" — mirroring v3 new's `fill_page` live-recheck
// backstop, just implemented in plain JS instead of the AX-tree bridge, since legacy doesn't use
// that bridge at all.

use std::collections::HashMap;

pub fn build_injector_js(answers: &HashMap<String, String>) -> String {
    let json = serde_json::to_string(answers).unwrap_or_else(|_| "{}".into());
    format!(
        r#"
(async () => {{
  const answers = {json};
  // Must match extractor.rs's selector exactly — see its comment for why the `:not()` guard is
  // there (excludes each checkbox/radio option's own nested listitem).
  const items = document.querySelectorAll('[role="listitem"]:not([role="listitem"] [role="listitem"])');
  const sleep = ms => new Promise(r => setTimeout(r, ms));

  const setNative = (el, val) => {{
    const proto = el.tagName === 'TEXTAREA' ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    const setter = Object.getOwnPropertyDescriptor(proto, 'value').set;
    setter.call(el, val);
    el.dispatchEvent(new Event('input', {{ bubbles: true }}));
    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
  }};

  const looseMatch = (a, b) => {{
    const A = (a || '').trim().toLowerCase();
    const B = (b || '').trim().toLowerCase();
    if (!A || !B) return false;
    return A === B || A.startsWith(B) || B.startsWith(A) || A.includes(B) || B.includes(A);
  }};

  // Hard write-lock helper: true if any radio/checkbox inside this group already has a live
  // selection, independent of whatever the answer map claims.
  const groupAlreadyChecked = (g) => Array.from(g.querySelectorAll('[role="radio"], [role="checkbox"]'))
    .some(r => r.getAttribute('aria-checked') === 'true');

  // Must match extractor.rs's `gridRowLabel` exactly, or a row's answer here (keyed by that same
  // label in the model's JSON) never finds its group. Confirmed live: CheckboxGrid row groups
  // carry no aria-label at all (unlike Grid's, which happens to still have one), so relying on
  // aria-label alone silently matched nothing for every CheckboxGrid row.
  const rowLabel = (g) => {{
    const al = (g.getAttribute('aria-label') || '').trim();
    if (al) return al;
    const leaf = Array.from(g.querySelectorAll('div')).find(d => d.children.length === 0 && (d.innerText || '').trim());
    return leaf ? leaf.innerText.trim() : '';
  }};

  for (let idx = 0; idx < items.length; idx++) {{
    const item = items[idx];
    const id = 'q' + idx;
    const val = answers[id];
    if (val === undefined || val === null) continue;

    // Date
    const dateInput = item.querySelector('input[type="date"]');
    if (dateInput) {{
      if (dateInput.value) continue;
      setNative(dateInput, val); continue;
    }}

    // Time
    const timeInput = item.querySelector('input[type="time"]');
    if (timeInput) {{
      if (timeInput.value) continue;
      setNative(timeInput, val); continue;
    }}

    // Grid (multiple radiogroups) — row-level write-lock (design doc §7.4)
    const radioGroups = item.querySelectorAll('[role="radiogroup"]');
    if (radioGroups.length > 1) {{
      let map = val;
      if (typeof val === 'string') {{ try {{ map = JSON.parse(val); }} catch (e) {{ map = {{}}; }} }}
      radioGroups.forEach(g => {{
        if (groupAlreadyChecked(g)) return; // this row is already answered live — skip it
        const row = rowLabel(g);
        const want = map[row];
        if (!want) return;
        const radios = g.querySelectorAll('[role="radio"]');
        for (const r of radios) {{
          if (looseMatch(r.getAttribute('aria-label') || r.innerText, want)) {{ r.click(); break; }}
        }}
      }});
      continue;
    }}

    // Single radio / Scale — whole-field write-lock
    const radios = item.querySelectorAll('[role="radio"]');
    if (radios.length > 0) {{
      if (Array.from(radios).some(r => r.getAttribute('aria-checked') === 'true')) continue;
      const want = String(val).replace(/^[A-Za-z]\)\s*/, '').trim();
      for (const r of radios) {{
        if (looseMatch(r.getAttribute('aria-label') || r.innerText, want)) {{ r.click(); break; }}
      }}
      continue;
    }}

    // Checkbox grid — row-level write-lock
    const checkGroups = Array.from(item.querySelectorAll('[role="group"]'))
      .filter(g => g.querySelector('[role="checkbox"]'));
    if (checkGroups.length > 1) {{
      let map = val;
      if (typeof val === 'string') {{ try {{ map = JSON.parse(val); }} catch (e) {{ map = {{}}; }} }}
      checkGroups.forEach(g => {{
        if (groupAlreadyChecked(g)) return;
        const row = rowLabel(g);
        const want = (map[row] || '').split(',').map(s => s.trim()).filter(Boolean);
        const cols = g.querySelectorAll('[role="checkbox"]');
        for (const c of cols) {{
          const label = c.getAttribute('aria-label') || c.innerText;
          if (want.some(w => looseMatch(label, w)) && c.getAttribute('aria-checked') !== 'true') {{ c.click(); }}
        }}
      }});
      continue;
    }}

    // Single checkbox set — whole-field write-lock (never per-option, same as design doc §7.4)
    const checks = item.querySelectorAll('[role="checkbox"]');
    if (checks.length > 0) {{
      if (Array.from(checks).some(c => c.getAttribute('aria-checked') === 'true')) continue;
      const want = String(val).split(',').map(s => s.replace(/^[A-Za-z]\)\s*/, '').trim()).filter(Boolean);
      for (const c of checks) {{
        const label = c.getAttribute('aria-label') || c.innerText;
        if (want.some(w => looseMatch(label, w)) && c.getAttribute('aria-checked') !== 'true') {{ c.click(); }}
      }}
      continue;
    }}

    // Dropdown — deliberately NOT handled here. Confirmed live (real Google Form, real Chrome):
    // this widget's option-select handler ignores a plain synthetic `.click()` — it opens fine,
    // the right option is found and "clicked" with no error, but the selection never actually
    // registers (`aria-selected` stays on the placeholder). Only a real, OS-trusted mouse event
    // gets it to take. `select_dropdown_option` below does that via chromiumoxide's
    // `Element::click()`, which dispatches a genuine CDP mouse event at the element's real screen
    // coordinates (the same mechanism already used successfully for the Next-page button) instead
    // of a scripted DOM method call. Skip here so that pass has a clean, unopened dropdown to
    // work with.
    if (item.querySelector('[role="listbox"]')) {{
      continue;
    }}

    // Long text
    const textarea = item.querySelector('textarea');
    if (textarea) {{
      if (textarea.value) continue;
      setNative(textarea, val); continue;
    }}

    // Short text variants
    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"], input[type="tel"], input[type="url"]');
    if (input) {{
      if (input.value) continue;
      setNative(input, val); continue;
    }}
  }}
  return true;
}})()
"#
    )
}

/// Selects a Dropdown-kind question's option via real, OS-trusted clicks instead of the JS
/// injector (see the comment in `build_injector_js`'s Dropdown branch for why plain `.click()`
/// doesn't work on this widget). `idx` is the question's position among `[role="listitem"]`
/// elements, i.e. the numeric suffix of its `qN` id — the same indexing `build_injector_js` and
/// `extractor.rs` both rely on, valid as long as the DOM hasn't changed since extraction.
pub async fn select_dropdown_option(
    page: &chromiumoxide::Page,
    idx: usize,
    val: &str,
) -> anyhow::Result<()> {
    // Must match extractor.rs's/build_injector_js's selector exactly — see extractor.rs's
    // comment for why the `:not()` guard is there.
    let listitems = page
        .find_elements(r#"[role="listitem"]:not([role="listitem"] [role="listitem"])"#)
        .await
        .map_err(|e| anyhow::anyhow!("dropdown q{idx}: listing questions: {e}"))?;
    let item = listitems
        .get(idx)
        .ok_or_else(|| anyhow::anyhow!("dropdown q{idx}: page has only {} question(s)", listitems.len()))?;

    let listbox = item
        .find_element(r#"[role="listbox"]"#)
        .await
        .map_err(|e| anyhow::anyhow!("dropdown q{idx}: no listbox found: {e}"))?;

    // Hard write-lock, same rule as every other field in this file: never touch a field that
    // already carries a live selection, no matter what the answer map says. Must check
    // `data-value`, not just non-empty text — confirmed live that the unpicked placeholder
    // option is itself `aria-selected="true"` by default with non-empty placeholder text
    // ("Choose"/"Pumili"/etc.), so a text-emptiness check alone reads a totally untouched
    // dropdown as already answered. Same rule extractor.rs uses to build `current_value`.
    if let Ok(selected) = listbox.find_element(r#"[aria-selected="true"]"#).await {
        let has_real_value = selected
            .attribute("data-value")
            .await
            .ok()
            .flatten()
            .is_some_and(|v| !v.trim().is_empty());
        if has_real_value {
            return Ok(());
        }
    }

    listbox.click().await.map_err(|e| anyhow::anyhow!("dropdown q{idx}: opening: {e}"))?;
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    // Options render into a portal outside the listitem (confirmed live), so search the whole
    // document, same as the old JS injector always did for this field type.
    let options = page
        .find_elements(r#"[role="option"]"#)
        .await
        .map_err(|e| anyhow::anyhow!("dropdown q{idx}: reading opened options: {e}"))?;

    for opt in &options {
        let text = opt.inner_text().await.ok().flatten().unwrap_or_default();
        if loose_match(&text, val) {
            opt.click().await.map_err(|e| anyhow::anyhow!("dropdown q{idx}: selecting {val:?}: {e}"))?;
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;
            return Ok(());
        }
    }

    anyhow::bail!("dropdown q{idx}: no option matched {val:?}")
}

/// Same fuzzy-match rule as the JS injector's `looseMatch`, ported to Rust since this path never
/// touches the page's JS context.
fn loose_match(a: &str, b: &str) -> bool {
    let a = a.trim().to_lowercase();
    let b = b.trim().to_lowercase();
    if a.is_empty() || b.is_empty() {
        return false;
    }
    a == b || a.starts_with(&b) || b.starts_with(&a) || a.contains(&b) || b.contains(&a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_injector_js_embeds_the_answers_map_as_json() {
        let mut answers = HashMap::new();
        answers.insert("q0".to_string(), "Blue".to_string());
        let js = build_injector_js(&answers);
        assert!(js.contains(r#""q0":"Blue""#));
    }

    #[test]
    fn build_injector_js_falls_back_to_empty_object_never_panics_on_odd_input() {
        // Answers with characters that need JSON-escaping shouldn't break the generated script.
        let mut answers = HashMap::new();
        answers.insert("q0".to_string(), "She said \"hi\"".to_string());
        let js = build_injector_js(&answers);
        assert!(js.contains(r#"She said \"hi\""#));
    }

    #[test]
    fn build_injector_js_contains_the_write_lock_checks() {
        let js = build_injector_js(&HashMap::new());
        assert!(js.contains("groupAlreadyChecked"), "row/whole-field live re-check must be present");
        assert!(js.contains("if (dateInput.value) continue;"));
        assert!(js.contains("if (textarea.value) continue;"));
        assert!(js.contains("if (input.value) continue;"));
    }

    #[test]
    fn build_injector_js_skips_dropdowns_without_clicking_in_js() {
        let js = build_injector_js(&HashMap::new());
        assert!(!js.contains("listbox.click()"), "dropdown selection must happen via real clicks, not JS");
    }

    #[test]
    fn loose_match_is_case_insensitive_and_substring_tolerant() {
        assert!(loose_match("Spring", "spring"));
        assert!(loose_match("  Fall ", "fall"));
        assert!(loose_match("Autumn (Fall)", "Fall"));
        assert!(!loose_match("Spring", "Winter"));
        assert!(!loose_match("", "Spring"));
    }
}
