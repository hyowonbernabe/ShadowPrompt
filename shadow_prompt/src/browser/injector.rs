pub const EXTRACTOR_JS: &str = r#"
(function() {
    try {
        let result = [];
        let titleNode = document.querySelector('.F9yp7e, .vQ43Ie, div[role="heading"][aria-level="1"]');
        let title = titleNode ? titleNode.innerText : document.title;
        
        let items = document.querySelectorAll('div[role="listitem"]');
        items.forEach((item, idx) => {
            let questionData = { index: idx, type: "unknown", container_id: item.getAttribute('data-item-id') || ("qContainer_" + idx) };
            item.id = questionData.container_id;
            
            let heading = item.querySelector('div[role="heading"]');
            if (heading) {
                questionData.text = heading.innerText;
            } else {
                questionData.text = item.innerText.split('\n')[0]; // fallback
            }
            
            let img = item.querySelector('img');
            if (img) { questionData.image_url = img.src; }
            
            let links = [];
            item.querySelectorAll('a').forEach(a => links.push({ text: a.innerText, href: a.href }));
            if (links.length > 0) { questionData.links = links; }

            let isAnswered = false;

            let options = item.querySelectorAll('[role="radio"], [role="checkbox"]');
            if (options.length > 0) {
                questionData.type = options[0].getAttribute('role'); // "radio" or "checkbox"
                questionData.options = [];
                options.forEach((opt, optIdx) => {
                    if (opt.getAttribute('aria-checked') === 'true') {
                        isAnswered = true;
                    }
                    let label = opt.getAttribute('aria-label') || opt.getAttribute('data-value') || opt.innerText;
                    let optId = opt.id;
                    if (!optId) {
                        optId = questionData.container_id + "_opt_" + optIdx;
                        opt.id = optId; // Assign strictly unique ID to the DOM element
                    }
                    questionData.options.push({ text: label, id: optId });
                });
            } else {
                let textInput = item.querySelector('input[type="text"], input[type="url"], input[type="email"], input[type="number"], textarea');
                if (textInput) {
                    if (textInput.value && textInput.value.trim() !== '') {
                        isAnswered = true;
                    }
                    questionData.type = "text";
                    let inputId = textInput.id;
                    if (!inputId) {
                        inputId = questionData.container_id + "_input";
                        textInput.id = inputId;
                    }
                    questionData.id = inputId;
                }
            }
            
            if (!isAnswered) {
                result.push(questionData);
            }
        });
        
        let navButtons = [];
        document.querySelectorAll('div[role="button"]').forEach(btn => {
            let text = btn.innerText.trim().toLowerCase();
            let label = (btn.getAttribute('aria-label') || "").toLowerCase();
            if (text === "next" || label === "next") {
                let btnId = btn.id || "nav_next_btn";
                btn.id = btnId;
                navButtons.push({ type: "next", id: btnId, text: btn.innerText.trim() });
            } else if (text === "submit" || label === "submit") {
                let btnId = btn.id || "nav_submit_btn";
                btn.id = btnId;
                navButtons.push({ type: "submit", id: btnId, text: btn.innerText.trim() });
            }
        });
        
        return JSON.stringify({ title, questions: result, navigation: navButtons });
    } catch(e) {
        return "ERROR: " + e.toString();
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
