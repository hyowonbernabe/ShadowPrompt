# Google Forms Comprehensive Question Type Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend Google Forms automation to correctly extract and inject answers for all question types: single-choice radio, multi-select checkbox, dropdown (native and custom), date, time, and matrix/grid (radio grid and checkbox grid — also called "connect" questions).

**Architecture:** Two files change — `injector.rs` gets a fully rewritten `EXTRACTOR_JS` constant (covers all DOM patterns for all question types) and an async-capable `build_injector_call` (adds `select_native`, `dropdown_select`, `select_option` action types); `mod.rs` gets a comprehensive LLM prompt that explains every type and action format, plus `await_promise: true` on the injection `tab.evaluate` call so async dropdown interactions work.

**Tech Stack:** Rust 2021 · headless_chrome `tab.evaluate` · JavaScript (Chrome DevTools injection)

**Crate root:** All `cargo` commands run from `shadow_prompt/` directory.

---

## File Map

| File | Change |
|------|--------|
| `shadow_prompt/src/browser/injector.rs` | Full rewrite of `EXTRACTOR_JS`, extend `build_injector_call` to async + new actions, add unit tests |
| `shadow_prompt/src/browser/mod.rs` | Update LLM prompt, change `tab.evaluate` `await_promise: false` → `true` |

---

### Task 1: Add unit tests for `injector.rs` (TDD baseline)

**Files:**
- Modify: `shadow_prompt/src/browser/injector.rs`

These tests will FAIL against the current implementation — that's the point. They define the contract the next tasks must satisfy.

- [ ] **Step 1: Add test module at the bottom of `injector.rs`**

Append after the closing `}` of `build_injector_call`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_injector_call_contains_click_handler() {
        let out = build_injector_call(r#"[{"id":"x","action":"click"}]"#);
        assert!(out.contains("click") && out.contains("check"));
    }

    #[test]
    fn test_build_injector_call_contains_type_handler() {
        let out = build_injector_call(r#"[{"id":"x","action":"type","value":"hi"}]"#);
        assert!(out.contains("type") && out.contains("dispatchEvent"));
    }

    #[test]
    fn test_build_injector_call_contains_select_native_handler() {
        let out = build_injector_call(r#"[{"id":"x","action":"select_native","value":"A"}]"#);
        assert!(out.contains("select_native"));
    }

    #[test]
    fn test_build_injector_call_contains_dropdown_select_handler() {
        let out = build_injector_call(r#"[{"id":"x","action":"dropdown_select","value":"A"}]"#);
        assert!(out.contains("dropdown_select"));
    }

    #[test]
    fn test_extractor_js_handles_grid_radio() {
        assert!(EXTRACTOR_JS.contains("grid_radio"));
    }

    #[test]
    fn test_extractor_js_handles_grid_checkbox() {
        assert!(EXTRACTOR_JS.contains("grid_checkbox"));
    }

    #[test]
    fn test_extractor_js_handles_dropdown() {
        assert!(EXTRACTOR_JS.contains("dropdown"));
    }

    #[test]
    fn test_extractor_js_handles_date() {
        assert!(EXTRACTOR_JS.contains("'date'") || EXTRACTOR_JS.contains("\"date\""));
    }

    #[test]
    fn test_extractor_js_handles_time() {
        assert!(EXTRACTOR_JS.contains("'time'") || EXTRACTOR_JS.contains("\"time\""));
    }
}
```

- [ ] **Step 2: Run failing tests to confirm baseline**

```bash
cd shadow_prompt && cargo test -p shadow_prompt browser::injector::tests 2>&1 | tail -30
```

Expected: `test_build_injector_call_contains_select_native_handler` and `test_build_injector_call_contains_dropdown_select_handler` FAIL. `test_extractor_js_handles_grid_radio`, `test_extractor_js_handles_grid_checkbox` FAIL. Others may pass.

- [ ] **Step 3: Commit the failing tests**

```bash
git add shadow_prompt/src/browser/injector.rs
git commit -m "test: add failing tests for comprehensive Google Forms question type support"
```

---

### Task 2: Rewrite `EXTRACTOR_JS` to detect all question types

**Files:**
- Modify: `shadow_prompt/src/browser/injector.rs`

Replace the entire `pub const EXTRACTOR_JS: &str = r#"...`"#;` block with the following. The logic is:
- Multiple `[role="radiogroup"]` in one item → `grid_radio` (matrix)
- Single `[role="radiogroup"]` → `radio`
- `[role="checkbox"]` elements with multiple named groups → `grid_checkbox`
- `[role="checkbox"]` flat list → `checkbox`
- `[role="listbox"]` or `select` → `dropdown`
- inputs with date/time aria-labels → `date` / `time` / `datetime`
- Everything else → `text`

- [ ] **Step 1: Replace the `EXTRACTOR_JS` constant**

Replace the entire constant from its `pub const` declaration through its closing `"#;` with:

```rust
pub const EXTRACTOR_JS: &str = r#"
(function() {
    try {
        var result = [];
        var titleNode = document.querySelector('.F9yp7e, .vQ43Ie, div[role="heading"][aria-level="1"]');
        var title = titleNode ? titleNode.innerText : document.title;

        var items = document.querySelectorAll('div[role="listitem"]');
        items.forEach(function(item, idx) {
            var cid = item.getAttribute('data-item-id') || ('qContainer_' + idx);
            item.id = cid;
            var qd = { index: idx, type: 'unknown', container_id: cid };

            var heading = item.querySelector('div[role="heading"]');
            qd.text = heading ? heading.innerText : item.innerText.split('\n')[0];

            var img = item.querySelector('img');
            if (img) qd.image_url = img.src;

            var links = [];
            item.querySelectorAll('a').forEach(function(a) { links.push({ text: a.innerText, href: a.href }); });
            if (links.length > 0) qd.links = links;

            var isAnswered = false;

            // --- GRID RADIO: multiple radiogroups = one per row in a matrix ---
            var radioGroups = Array.from(item.querySelectorAll('[role="radiogroup"]'));
            if (radioGroups.length > 1) {
                qd.type = 'grid_radio';
                qd.grid_rows = [];
                var colHeaders = [];
                radioGroups[0].querySelectorAll('[role="radio"]').forEach(function(opt) {
                    colHeaders.push(opt.getAttribute('aria-label') || '');
                });
                radioGroups.forEach(function(group, rowIdx) {
                    var rowLabel = group.getAttribute('aria-label') || '';
                    if (!rowLabel) {
                        var lby = group.getAttribute('aria-labelledby');
                        if (lby) { var lel = document.getElementById(lby); if (lel) rowLabel = lel.innerText.trim(); }
                    }
                    if (!rowLabel) rowLabel = 'Row ' + (rowIdx + 1);
                    var rowOpts = [];
                    group.querySelectorAll('[role="radio"]').forEach(function(opt, colIdx) {
                        if (opt.getAttribute('aria-checked') === 'true') isAnswered = true;
                        var optLabel = opt.getAttribute('aria-label') || colHeaders[colIdx] || ('Col ' + (colIdx + 1));
                        var oid = opt.id || (cid + '_r' + rowIdx + '_c' + colIdx);
                        opt.id = oid;
                        rowOpts.push({ text: optLabel, id: oid });
                    });
                    qd.grid_rows.push({ row_text: rowLabel, options: rowOpts });
                });
            }

            // --- RADIO: single radiogroup = standard multiple choice ---
            else if (radioGroups.length === 1) {
                qd.type = 'radio';
                qd.options = [];
                radioGroups[0].querySelectorAll('[role="radio"]').forEach(function(opt, optIdx) {
                    if (opt.getAttribute('aria-checked') === 'true') isAnswered = true;
                    var label = opt.getAttribute('aria-label') || opt.getAttribute('data-value') || opt.innerText;
                    var oid = opt.id || (cid + '_opt_' + optIdx);
                    opt.id = oid;
                    qd.options.push({ text: label, id: oid });
                });
            }

            else {
                var allCheckboxes = Array.from(item.querySelectorAll('[role="checkbox"]'));

                if (allCheckboxes.length > 0) {
                    // --- GRID CHECKBOX: multiple named groups each containing checkboxes ---
                    var cbGroups = Array.from(item.querySelectorAll('[role="group"][aria-labelledby], [role="group"][aria-label]'));
                    if (cbGroups.length > 1) {
                        qd.type = 'grid_checkbox';
                        qd.grid_rows = [];
                        cbGroups.forEach(function(group, rowIdx) {
                            var rowLabel = group.getAttribute('aria-label') || '';
                            if (!rowLabel) {
                                var lby = group.getAttribute('aria-labelledby');
                                if (lby) { var lel = document.getElementById(lby); if (lel) rowLabel = lel.innerText.trim(); }
                            }
                            if (!rowLabel) rowLabel = 'Row ' + (rowIdx + 1);
                            var rowOpts = [];
                            group.querySelectorAll('[role="checkbox"]').forEach(function(opt, colIdx) {
                                if (opt.getAttribute('aria-checked') === 'true') isAnswered = true;
                                var optLabel = opt.getAttribute('aria-label') || ('Col ' + (colIdx + 1));
                                var oid = opt.id || (cid + '_r' + rowIdx + '_c' + colIdx);
                                opt.id = oid;
                                rowOpts.push({ text: optLabel, id: oid });
                            });
                            qd.grid_rows.push({ row_text: rowLabel, options: rowOpts });
                        });
                    }
                    // --- CHECKBOX: flat multi-select list ---
                    else {
                        qd.type = 'checkbox';
                        qd.options = [];
                        allCheckboxes.forEach(function(opt, optIdx) {
                            if (opt.getAttribute('aria-checked') === 'true') isAnswered = true;
                            var label = opt.getAttribute('aria-label') || opt.getAttribute('data-value') || opt.innerText;
                            var oid = opt.id || (cid + '_opt_' + optIdx);
                            opt.id = oid;
                            qd.options.push({ text: label, id: oid });
                        });
                    }
                }

                // --- DROPDOWN: native <select> or custom [role="listbox"] ---
                else if (item.querySelector('[role="listbox"], select')) {
                    qd.type = 'dropdown';
                    qd.options = [];
                    var sel = item.querySelector('select');
                    if (sel) {
                        var sid = sel.id || (cid + '_select');
                        sel.id = sid;
                        qd.id = sid;
                        Array.from(sel.options).forEach(function(opt) {
                            if (opt.selected && opt.value !== '') isAnswered = true;
                            if (opt.value !== '') qd.options.push({ text: opt.text, value: opt.value });
                        });
                    } else {
                        var lb = item.querySelector('[role="listbox"]');
                        var trigger = item.querySelector('[role="button"]') || lb;
                        var tid = (trigger ? trigger.id : null) || (cid + '_trigger');
                        if (trigger) trigger.id = tid;
                        qd.trigger_id = tid;
                        if (lb && lb.getAttribute('aria-activedescendant')) isAnswered = true;
                        item.querySelectorAll('[role="option"]').forEach(function(opt, optIdx) {
                            var optLabel = opt.getAttribute('aria-label') || opt.innerText.trim();
                            var oid = opt.id || (cid + '_opt_' + optIdx);
                            opt.id = oid;
                            qd.options.push({ text: optLabel, id: oid });
                        });
                    }
                }

                else {
                    // --- DATE / TIME / DATETIME ---
                    var dateInps = Array.from(item.querySelectorAll(
                        'input[type="date"], input[aria-label*="year" i], input[aria-label*="month" i], input[aria-label*="day" i]'
                    ));
                    var timeInps = Array.from(item.querySelectorAll(
                        'input[type="time"], input[aria-label*="hour" i], input[aria-label*="minute" i]'
                    ));
                    if (dateInps.length > 0 || timeInps.length > 0) {
                        qd.type = (dateInps.length > 0 && timeInps.length > 0) ? 'datetime' : (dateInps.length > 0 ? 'date' : 'time');
                        qd.fields = [];
                        dateInps.concat(timeInps).forEach(function(inp, fIdx) {
                            var flabel = inp.getAttribute('aria-label') || inp.placeholder || ('field_' + fIdx);
                            var fid = inp.id || (cid + '_field_' + fIdx);
                            inp.id = fid;
                            if (inp.value) isAnswered = true;
                            qd.fields.push({ label: flabel, id: fid });
                        });
                        var ampmSel = item.querySelector('select[aria-label*="AM" i], select[aria-label*="PM" i]');
                        if (ampmSel) {
                            var amid = ampmSel.id || (cid + '_ampm');
                            ampmSel.id = amid;
                            qd.fields.push({ label: 'AM/PM', id: amid, is_select: true });
                        }
                    }
                    // --- TEXT: short answer or paragraph ---
                    else {
                        var textInput = item.querySelector(
                            'input[type="text"], input[type="url"], input[type="email"], input[type="number"], textarea'
                        );
                        if (textInput) {
                            if (textInput.value && textInput.value.trim() !== '') isAnswered = true;
                            qd.type = 'text';
                            var iid = textInput.id || (cid + '_input');
                            textInput.id = iid;
                            qd.id = iid;
                        }
                    }
                }
            }

            if (!isAnswered) { result.push(qd); }
        });

        var navButtons = [];
        document.querySelectorAll('div[role="button"]').forEach(function(btn) {
            var text = btn.innerText.trim().toLowerCase();
            var label = (btn.getAttribute('aria-label') || '').toLowerCase();
            if (text === 'next' || label === 'next') {
                var nbid = btn.id || 'nav_next_btn'; btn.id = nbid;
                navButtons.push({ type: 'next', id: nbid, text: btn.innerText.trim() });
            } else if (text === 'submit' || label === 'submit') {
                var sbid = btn.id || 'nav_submit_btn'; btn.id = sbid;
                navButtons.push({ type: 'submit', id: sbid, text: btn.innerText.trim() });
            }
        });

        return JSON.stringify({ title: title, questions: result, navigation: navButtons });
    } catch(e) {
        return 'ERROR: ' + e.toString();
    }
})();
"#;
```

- [ ] **Step 2: Run tests to verify extractor tests now pass**

```bash
cd shadow_prompt && cargo test -p shadow_prompt browser::injector::tests::test_extractor 2>&1 | tail -20
```

Expected: all `test_extractor_js_*` tests PASS. The `select_native` and `dropdown_select` build_injector tests still FAIL.

- [ ] **Step 3: Commit**

```bash
git add shadow_prompt/src/browser/injector.rs
git commit -m "feat: rewrite EXTRACTOR_JS to support grid, dropdown, date/time, multi-select checkbox"
```

---

### Task 3: Extend `build_injector_call` to async with new action types; update `mod.rs`

**Files:**
- Modify: `shadow_prompt/src/browser/injector.rs`
- Modify: `shadow_prompt/src/browser/mod.rs`

- [ ] **Step 1: Replace `build_injector_call` with async version**

Replace the entire `pub fn build_injector_call(raw_actions_json: &str) -> String` function:

```rust
pub fn build_injector_call(raw_actions_json: &str) -> String {
    format!(
        r#"
        (async function() {{
            try {{
                var actions = {actions};

                function findTarget(id) {{
                    var el = document.getElementById(id);
                    if (el) return el;
                    var all = document.querySelectorAll('[role="radio"],[role="checkbox"],[role="option"],select,input,textarea,[role="button"]');
                    for (var i = 0; i < all.length; i++) {{
                        if (all[i].getAttribute('data-value') === id || all[i].getAttribute('aria-label') === id) {{
                            return all[i];
                        }}
                    }}
                    return null;
                }}

                for (var i = 0; i < actions.length; i++) {{
                    var action = actions[i];
                    var target = findTarget(action.id);
                    if (!target) continue;

                    if (action.action === "click" || action.action === "check") {{
                        if (target.getAttribute('aria-checked') !== 'true') {{
                            target.click();
                        }}
                    }} else if (action.action === "type") {{
                        target.value = action.value || "";
                        target.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        target.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }} else if (action.action === "select_native") {{
                        target.value = action.value || "";
                        target.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }} else if (action.action === "select_option") {{
                        target.click();
                    }} else if (action.action === "dropdown_select") {{
                        target.click();
                        await new Promise(function(r) {{ setTimeout(r, 350); }});
                        var opts = document.querySelectorAll('[role="option"]');
                        for (var j = 0; j < opts.length; j++) {{
                            if (opts[j].innerText.trim() === (action.value || "").trim()) {{
                                opts[j].click();
                                break;
                            }}
                        }}
                    }}
                }}
                return "Success";
            }} catch(e) {{
                return "ERROR: " + e.toString();
            }}
        }})()
        "#,
        actions = raw_actions_json
    )
}
```

- [ ] **Step 2: Run all injector tests to verify they pass**

```bash
cd shadow_prompt && cargo test -p shadow_prompt browser::injector::tests 2>&1 | tail -20
```

Expected output (all 9 tests PASS):
```
test browser::injector::tests::test_build_injector_call_contains_click_handler ... ok
test browser::injector::tests::test_build_injector_call_contains_dropdown_select_handler ... ok
test browser::injector::tests::test_build_injector_call_contains_select_native_handler ... ok
test browser::injector::tests::test_build_injector_call_contains_type_handler ... ok
test browser::injector::tests::test_extractor_js_handles_date ... ok
test browser::injector::tests::test_extractor_js_handles_dropdown ... ok
test browser::injector::tests::test_extractor_js_handles_grid_checkbox ... ok
test browser::injector::tests::test_extractor_js_handles_grid_radio ... ok
test browser::injector::tests::test_extractor_js_handles_time ... ok
```

- [ ] **Step 3: Update `tab.evaluate` in `mod.rs` to `await_promise: true`**

In `shadow_prompt/src/browser/mod.rs`, find the injection evaluate call (line ~219):

```rust
        tab.evaluate(&injection_script, false)
            .map_err(|e| anyhow!("Injection Script Error: {}", e))?;
```

Change `false` to `true`:

```rust
        tab.evaluate(&injection_script, true)
            .map_err(|e| anyhow!("Injection Script Error: {}", e))?;
```

- [ ] **Step 4: Run Clippy**

```bash
cd shadow_prompt && cargo clippy 2>&1 | grep -E "^error|^warning" | head -30
```

Expected: no new warnings or errors introduced by this change.

- [ ] **Step 5: Run all tests**

```bash
cd shadow_prompt && cargo test 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 6: Commit both files**

```bash
git add shadow_prompt/src/browser/injector.rs shadow_prompt/src/browser/mod.rs
git commit -m "feat: async injector with select_native/dropdown_select/select_option, await_promise=true"
```

---

### Task 4: Update LLM prompts in `mod.rs`

**Files:**
- Modify: `shadow_prompt/src/browser/mod.rs`

- [ ] **Step 1: Replace the `let prompt = if is_auto { ... } else { ... }` block**

In `shadow_prompt/src/browser/mod.rs`, find and replace the entire prompt block (lines ~187–206, the `let prompt = if is_auto { format!(...) } else { format!(...) };`):

```rust
        let nav_rule = if is_auto {
            "CRITICAL RULE 1: If there is a `navigation` button of type `next`, include a click action for it as the VERY LAST item in your array after all question answers.\nCRITICAL RULE 2: NEVER click a button of type `submit`."
        } else {
            "CRITICAL RULE: SINGLE-PAGE MODE. Do NOT interact with ANY navigation buttons. Do not click `next` or `submit`."
        };

        let prompt = format!(
            "You are an automated quiz solver filling out a Google Form.
The JSON contains `questions` and `navigation` buttons. Answer every unanswered question.

QUESTION TYPES — use exactly these action formats:
- \"radio\": Click ONE option. Action: {{\"id\":\"<option_id>\",\"action\":\"click\"}}
- \"checkbox\": Multi-select — click ALL correct options (one action per option). Action: {{\"id\":\"<option_id>\",\"action\":\"click\"}}
- \"text\": Type answer. Action: {{\"id\":\"<input_id>\",\"action\":\"type\",\"value\":\"<answer>\"}}
- \"dropdown\" with \"id\" field (native select): Action: {{\"id\":\"<select_id>\",\"action\":\"select_native\",\"value\":\"<exact option text>\"}}
- \"dropdown\" with \"trigger_id\" field (custom dropdown): Action: {{\"id\":\"<trigger_id>\",\"action\":\"dropdown_select\",\"value\":\"<exact option text>\"}}
- \"grid_radio\": Matrix — one click per row. Each row in `grid_rows` needs exactly one selected column. Action: {{\"id\":\"<row_col_id>\",\"action\":\"click\"}}
- \"grid_checkbox\": Matrix with checkboxes — click all applicable options per row. Action: {{\"id\":\"<row_col_id>\",\"action\":\"click\"}}
- \"date\": Fill each field in `fields` by label (Month, Day, Year with numeric values). Action: {{\"id\":\"<field_id>\",\"action\":\"type\",\"value\":\"<number>\"}}
- \"time\": Fill each field in `fields` (Hour, Minute). Action: {{\"id\":\"<field_id>\",\"action\":\"type\",\"value\":\"<number>\"}}. For AM/PM field with is_select=true: {{\"id\":\"<ampm_id>\",\"action\":\"select_native\",\"value\":\"AM\"}}

{nav_rule}

Return ONLY a valid JSON array of actions. No markdown, no explanation, no extra text.
Form JSON:
{form_json}",
            nav_rule = nav_rule,
            form_json = form_json
        );
```

- [ ] **Step 2: Run Clippy**

```bash
cd shadow_prompt && cargo clippy 2>&1 | grep -E "^error|^warning" | head -30
```

Expected: no warnings.

- [ ] **Step 3: Run all tests**

```bash
cd shadow_prompt && cargo test 2>&1 | tail -20
```

Expected: all tests pass.

- [ ] **Step 4: Commit**

```bash
git add shadow_prompt/src/browser/mod.rs
git commit -m "feat: update Google Forms LLM prompt to handle all question types, grid, dropdown, date/time"
```

---

## Done

All changes are committed. The Google Forms automation now correctly handles:
- **radio** — single-choice (unchanged behavior)
- **checkbox** — multi-select (LLM now explicitly instructed to click all correct options)
- **grid_radio** — matrix with one radio selection per row (connect/matching questions)
- **grid_checkbox** — matrix with checkbox selections per row
- **dropdown** — native `<select>` via `select_native`, or custom listbox via `dropdown_select`
- **date / time / datetime** — field-by-field text entry
- **text** — short answer and paragraph (unchanged behavior)
