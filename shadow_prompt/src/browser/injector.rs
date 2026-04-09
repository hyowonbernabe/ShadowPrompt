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
            var isLabeledGrid = radioGroups.length > 1 &&
                radioGroups.every(function(g) {
                    return g.getAttribute('aria-label') || g.getAttribute('aria-labelledby');
                });
            if (isLabeledGrid) {
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
            else if (radioGroups.length >= 1) {
                qd.type = 'radio';
                qd.options = [];
                Array.from(item.querySelectorAll('[role="radio"]')).forEach(function(opt, optIdx) {
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
                        var trigger = item.querySelector('[role="button"]');
                        if (trigger) {
                            var tid = trigger.id || (cid + '_trigger');
                            trigger.id = tid;
                            qd.trigger_id = tid;
                        } else {
                            qd.trigger_id = null;
                        }
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

            if (!isAnswered && qd.type !== 'unknown') { result.push(qd); }
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

pub fn build_injector_call(raw_actions_json: &str) -> String {
    format!(
        r#"
        (function() {{
            try {{
                let actions = {actions};
                for (let action of actions) {{
                    let target = document.getElementById(action.id);

                    if (!target && action.id) {{
                        let all = document.querySelectorAll('[role="radio"], [role="checkbox"]');
                        for(let i=0; i<all.length; i++) {{
                            if (all[i].getAttribute('data-value') == action.id || all[i].getAttribute('aria-label') == action.id) {{
                                target = all[i];
                                break;
                            }}
                        }}
                    }}

                    if (target) {{
                        if (action.action === "click" || action.action === "check") {{
                            if (target.getAttribute('aria-checked') !== 'true') {{
                                target.click();
                            }}
                        }} else if (action.action === "type") {{
                            target.value = action.value || "";
                            target.dispatchEvent(new Event('input', {{ bubbles: true }}));
                            target.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        }}
                    }}
                }}
                return "Success";
            }} catch(e) {{
                return "ERROR: " + e.toString();
            }}
        }})();
        "#,
        actions = raw_actions_json
    )
}

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
        assert!(EXTRACTOR_JS.contains("'dropdown'") || EXTRACTOR_JS.contains("\"dropdown\""));
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
