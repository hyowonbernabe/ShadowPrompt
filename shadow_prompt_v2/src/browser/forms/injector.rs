// Inject answers from the LLM back into form DOM. Never touches already-answered
// questions. Never clicks Submit.

use std::collections::HashMap;

pub fn build_injector_js(answers: &HashMap<String, String>) -> String {
    let json = serde_json::to_string(answers).unwrap_or_else(|_| "{}".into());
    format!(r#"
(async () => {{
  const answers = {json};
  const items = document.querySelectorAll('[role="listitem"]');
  const sleep = ms => new Promise(r => setTimeout(r, ms));

  const setNative = (el, val) => {{
    const proto = el.tagName === 'TEXTAREA'
      ? HTMLTextAreaElement.prototype
      : HTMLInputElement.prototype;
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

  for (let idx = 0; idx < items.length; idx++) {{
    const item = items[idx];
    const id = 'q' + idx;
    const val = answers[id];
    if (val === undefined || val === null) continue;

    // Date
    const dateInput = item.querySelector('input[type="date"]');
    if (dateInput) {{ setNative(dateInput, val); continue; }}

    // Time
    const timeInput = item.querySelector('input[type="time"]');
    if (timeInput) {{ setNative(timeInput, val); continue; }}

    // Grid (multiple radiogroups)
    const radioGroups = item.querySelectorAll('[role="radiogroup"]');
    if (radioGroups.length > 1) {{
      let map = val;
      if (typeof val === 'string') {{ try {{ map = JSON.parse(val); }} catch (e) {{ map = {{}}; }} }}
      radioGroups.forEach(g => {{
        const row = (g.getAttribute('aria-label') || '').trim();
        const want = map[row];
        if (!want) return;
        const radios = g.querySelectorAll('[role="radio"]');
        for (const r of radios) {{
          if (looseMatch(r.getAttribute('aria-label') || r.innerText, want)) {{ r.click(); break; }}
        }}
      }});
      continue;
    }}

    // Single radio / Scale
    const radios = item.querySelectorAll('[role="radio"]');
    if (radios.length > 0) {{
      // Strip "X) " prefix if present
      const want = String(val).replace(/^[A-Za-z]\)\s*/, '').trim();
      for (const r of radios) {{
        if (looseMatch(r.getAttribute('aria-label') || r.innerText, want)) {{ r.click(); break; }}
      }}
      continue;
    }}

    // Checkbox grid
    const checkGroups = Array.from(item.querySelectorAll('[role="group"]'))
      .filter(g => g.querySelector('[role="checkbox"]'));
    if (checkGroups.length > 1) {{
      let map = val;
      if (typeof val === 'string') {{ try {{ map = JSON.parse(val); }} catch (e) {{ map = {{}}; }} }}
      checkGroups.forEach(g => {{
        const row = (g.getAttribute('aria-label') || '').trim();
        const want = (map[row] || '').split(',').map(s => s.trim()).filter(Boolean);
        const cols = g.querySelectorAll('[role="checkbox"]');
        for (const c of cols) {{
          const label = c.getAttribute('aria-label') || c.innerText;
          if (want.some(w => looseMatch(label, w)) && c.getAttribute('aria-checked') !== 'true') {{ c.click(); }}
        }}
      }});
      continue;
    }}

    // Single checkbox set
    const checks = item.querySelectorAll('[role="checkbox"]');
    if (checks.length > 0) {{
      const want = String(val).split(',').map(s => s.replace(/^[A-Za-z]\)\s*/, '').trim()).filter(Boolean);
      for (const c of checks) {{
        const label = c.getAttribute('aria-label') || c.innerText;
        if (want.some(w => looseMatch(label, w)) && c.getAttribute('aria-checked') !== 'true') {{ c.click(); }}
      }}
      continue;
    }}

    // Dropdown
    const listbox = item.querySelector('[role="listbox"]');
    if (listbox) {{
      listbox.click();
      await sleep(250);
      const opts = document.querySelectorAll('[role="option"]');
      for (const o of opts) {{
        if (looseMatch(o.innerText, val)) {{ o.click(); break; }}
      }}
      await sleep(100);
      continue;
    }}

    // Long text
    const textarea = item.querySelector('textarea');
    if (textarea) {{ setNative(textarea, val); continue; }}

    // Short text variants
    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"], input[type="tel"], input[type="url"]');
    if (input) {{ setNative(input, val); continue; }}
  }}
  return true;
}})()
"#)
}
