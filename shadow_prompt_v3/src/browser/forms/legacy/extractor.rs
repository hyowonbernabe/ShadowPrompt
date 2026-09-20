// Page reading for v3 legacy Forms — design doc §7.4. Ported from shadow_prompt_v2's
// `EXTRACTOR_JS` (per-type selector JS, run via `Page::evaluate()` instead of headless_chrome's
// sync `tab.evaluate()`), patched for the four gaps design doc §7.4 lists:
//   1. No `if (!text) return` — a heading-less (image-only) field still produces a `Question`
//      (reported as kind `Unknown` when no recognized input control is found either).
//   2. No `data:`-prefix filter on collected image URLs — inline images survive.
//   3. Page-level heading/paragraph text (outside any `[role="listitem"]`) is collected into
//      `page_context`, returned alongside the questions — v2 never read this at all.
//   4. Grid/CheckboxGrid report each row's blank/filled state individually (`blank_rows`), not
//      one collective `current_value` JSON blob — a partially-filled grid no longer reads as
//      wholly answered just because one row has a pick.
//
// Honest limitation, same as the rest of this project's Forms work: the page-level description
// text capture below (as opposed to the title, which reliably carries `role="heading"`) is a
// best-effort DOM heuristic, unverified against a live form — there is no live Chrome available in
// this environment to confirm Google's exact markup for the description block.

pub const EXTRACTOR_JS: &str = r#"
(() => {
  const pageContextParts = [];
  document.querySelectorAll('[role="heading"]').forEach(h => {
    if (h.closest('[role="listitem"]')) return;
    const t = (h.innerText || '').trim();
    if (t) pageContextParts.push(t);
  });
  // Best-effort description capture (see module doc): a leaf text node sitting alongside the
  // title, outside any listitem, with no reliable ARIA role of its own.
  document.querySelectorAll('[role="heading"]').forEach(h => {
    if (h.closest('[role="listitem"]')) return;
    const container = h.parentElement;
    if (!container) return;
    Array.from(container.querySelectorAll('div, span, p')).forEach(el => {
      if (el.closest('[role="heading"]')) return;
      if (el.children.length > 0) return;
      const t = (el.innerText || '').trim();
      if (t && !pageContextParts.includes(t)) pageContextParts.push(t);
    });
  });

  const out = [];
  const items = document.querySelectorAll('[role="listitem"]');
  items.forEach((item, idx) => {
    const titleEl = item.querySelector('[role="heading"]');
    const text = titleEl ? titleEl.innerText.trim() : '';
    const id = 'q' + idx;
    const imgs = Array.from(item.querySelectorAll('img')).map(i => i.src).filter(Boolean);

    const dateInput = item.querySelector('input[type="date"]');
    if (dateInput) {
      out.push({ id, kind: 'Date', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: dateInput.value || null });
      return;
    }
    const timeInput = item.querySelector('input[type="time"]');
    if (timeInput) {
      out.push({ id, kind: 'Time', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: timeInput.value || null });
      return;
    }

    const radioGroups = item.querySelectorAll('[role="radiogroup"]');
    if (radioGroups.length > 1) {
      const headerCells = item.querySelectorAll('[role="columnheader"]');
      let columns = Array.from(headerCells).map(h => h.innerText.trim()).filter(Boolean);
      if (columns.length === 0) {
        const firstRadios = radioGroups[0].querySelectorAll('[role="radio"]');
        columns = Array.from(firstRadios).map(r => (r.getAttribute('aria-label') || '').trim());
      }
      const rows = [];
      const chosen = {};
      const blankRows = [];
      radioGroups.forEach(g => {
        const label = (g.getAttribute('aria-label') || '').trim();
        rows.push(label);
        const checked = Array.from(g.querySelectorAll('[role="radio"]'))
          .find(r => r.getAttribute('aria-checked') === 'true');
        if (checked) { chosen[label] = (checked.getAttribute('aria-label') || '').trim(); }
        else { blankRows.push(label); }
      });
      out.push({
        id, kind: 'Grid', text, options: columns, rows, blank_rows: blankRows, image_urls: imgs,
        current_value: Object.keys(chosen).length ? JSON.stringify(chosen) : null
      });
      return;
    }

    const radios = item.querySelectorAll('[role="radio"]');
    if (radios.length > 0) {
      const options = Array.from(radios).map(r => (r.getAttribute('aria-label') || r.innerText.trim()));
      const checked = Array.from(radios).find(r => r.getAttribute('aria-checked') === 'true');
      const allNumeric = options.length > 0 && options.every(o => /^\s*\d+\s*$/.test(o));
      out.push({
        id, kind: allNumeric ? 'Scale' : 'Radio', text, options, rows: [], blank_rows: [], image_urls: imgs,
        current_value: checked ? (checked.getAttribute('aria-label') || checked.innerText.trim()) : null
      });
      return;
    }

    const checkGroups = Array.from(item.querySelectorAll('[role="group"]'))
      .filter(g => g.querySelector('[role="checkbox"]'));
    if (checkGroups.length > 1) {
      const headerCells = item.querySelectorAll('[role="columnheader"]');
      let columns = Array.from(headerCells).map(h => h.innerText.trim()).filter(Boolean);
      if (columns.length === 0) {
        const firstChecks = checkGroups[0].querySelectorAll('[role="checkbox"]');
        columns = Array.from(firstChecks).map(c => (c.getAttribute('aria-label') || '').trim());
      }
      const rows = [];
      const chosen = {};
      const blankRows = [];
      checkGroups.forEach(g => {
        const label = (g.getAttribute('aria-label') || '').trim();
        rows.push(label);
        const picks = Array.from(g.querySelectorAll('[role="checkbox"]'))
          .filter(c => c.getAttribute('aria-checked') === 'true')
          .map(c => (c.getAttribute('aria-label') || '').trim());
        if (picks.length > 0) { chosen[label] = picks.join(', '); } else { blankRows.push(label); }
      });
      out.push({
        id, kind: 'CheckboxGrid', text, options: columns, rows, blank_rows: blankRows, image_urls: imgs,
        current_value: Object.keys(chosen).length ? JSON.stringify(chosen) : null
      });
      return;
    }

    const checks = item.querySelectorAll('[role="checkbox"]');
    if (checks.length > 0) {
      const options = Array.from(checks).map(r => (r.getAttribute('aria-label') || r.innerText.trim()));
      const picks = Array.from(checks)
        .filter(r => r.getAttribute('aria-checked') === 'true')
        .map(r => (r.getAttribute('aria-label') || r.innerText.trim()));
      out.push({
        id, kind: 'Checkbox', text, options, rows: [], blank_rows: [], image_urls: imgs,
        current_value: picks.length ? picks.join(', ') : null
      });
      return;
    }

    const listbox = item.querySelector('[role="listbox"]');
    if (listbox) {
      const opts = Array.from(listbox.querySelectorAll('[role="option"]'))
        .map(o => (o.getAttribute('data-value') || o.innerText || '').trim())
        .filter(s => s && s !== '...' && !s.toLowerCase().startsWith('choose') && !s.toLowerCase().startsWith('select'));
      const sel = listbox.querySelector('[aria-selected="true"]');
      out.push({
        id, kind: 'Dropdown', text, options: opts, rows: [], blank_rows: [], image_urls: imgs,
        current_value: sel ? sel.innerText.trim() : null
      });
      return;
    }

    const textarea = item.querySelector('textarea');
    if (textarea) {
      out.push({ id, kind: 'LongText', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: textarea.value || null });
      return;
    }

    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"], input[type="tel"], input[type="url"]');
    if (input) {
      const kind = input.getAttribute('type') === 'number' ? 'Numeric' : 'ShortText';
      out.push({ id, kind, text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: input.value || null });
      return;
    }

    // Patch 1 (design doc §7.4): no recognized control at all (e.g. a picture-only question) —
    // still produces a field, never silently dropped the way v2's `if (!text) return` did.
    out.push({ id, kind: 'Unknown', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: null });
  });
  return JSON.stringify({ page_context: pageContextParts.join('\n'), questions: out });
})()
"#;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExtractedPage {
    pub page_context: String,
    pub questions: Vec<Question>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Question {
    pub id: String,
    pub kind: QuestionKind,
    pub text: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub rows: Vec<String>,
    /// Grid/CheckboxGrid only: row labels with no current value. Empty for every other kind.
    /// This is design doc §7.4's fix for the grid already-answered gap: a partially-filled
    /// grid's `current_value` (the whole row→choice map) is non-empty the instant *any* row has
    /// a pick, so `current_value` alone can't distinguish "fully answered" from "partially
    /// answered." `blank_rows` gives row-level truth independent of that.
    #[serde(default)]
    pub blank_rows: Vec<String>,
    #[serde(default)]
    pub image_urls: Vec<String>,
    pub current_value: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum QuestionKind {
    Radio,
    Checkbox,
    Dropdown,
    ShortText,
    LongText,
    Numeric,
    Date,
    Time,
    Grid,
    CheckboxGrid,
    Scale,
    /// A field with no recognized input control (e.g. a picture-only question) — surfaced rather
    /// than silently dropped (design doc §7.4 patch 1).
    Unknown,
}

impl Question {
    /// True if this question still needs an answer this turn. Grid/CheckboxGrid are judged by
    /// `blank_rows` (row-level); every other kind by whether `current_value` is empty
    /// (whole-field — including multi-select checkboxes, same conservative rule as v3 new: no
    /// reliable way to tell "stopped partway" from "chose fewer on purpose").
    pub fn needs_attention(&self) -> bool {
        match self.kind {
            QuestionKind::Grid | QuestionKind::CheckboxGrid => !self.blank_rows.is_empty(),
            _ => self.current_value.as_deref().unwrap_or("").trim().is_empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Hand-built JSON matching exactly what `EXTRACTOR_JS`'s `JSON.stringify({page_context,
    /// questions})` produces — the only part of this file testable without a live browser (the
    /// JS string itself can't run outside Chrome; see design doc §13's testing-strategy posture,
    /// same honest limitation as v3 new's read.rs/fill.rs). Covers: page-level context present,
    /// a heading-less image-only field surviving as `Unknown` (patch 1), a partially-filled grid
    /// reporting per-row blank state (patch 4).
    fn fixture_json() -> &'static str {
        r#"{
            "page_context": "Midterm Exam\nAnswer every question below.",
            "questions": [
                { "id": "q0", "kind": "Radio", "text": "Favorite color?",
                  "options": ["Red", "Blue"], "rows": [], "blank_rows": [],
                  "image_urls": [], "current_value": "Blue" },
                { "id": "q1", "kind": "Unknown", "text": "",
                  "options": [], "rows": [], "blank_rows": [],
                  "image_urls": ["https://example.com/diagram.png"], "current_value": null },
                { "id": "q2", "kind": "Grid", "text": "Rate each item",
                  "options": ["Always", "Never"], "rows": ["Row 1", "Row 2"],
                  "blank_rows": ["Row 2"], "image_urls": [],
                  "current_value": "{\"Row 1\":\"Always\"}" },
                { "id": "q3", "kind": "ShortText", "text": "Identify the chart below",
                  "options": [], "rows": [], "blank_rows": [],
                  "image_urls": ["data:image/png;base64,iVBORw0KGgoAAAANSUhEUg=="],
                  "current_value": null }
            ]
        }"#
    }

    #[test]
    fn page_level_context_is_parsed() {
        let page: ExtractedPage = serde_json::from_str(fixture_json()).unwrap();
        assert!(page.page_context.contains("Midterm Exam"));
        assert!(page.page_context.contains("Answer every question below."));
    }

    #[test]
    fn heading_less_image_only_question_survives_as_unknown() {
        let page: ExtractedPage = serde_json::from_str(fixture_json()).unwrap();
        let q1 = page.questions.iter().find(|q| q.id == "q1").expect("q1 must be present");
        assert_eq!(q1.kind, QuestionKind::Unknown);
        assert_eq!(q1.image_urls, vec!["https://example.com/diagram.png".to_string()]);
        assert!(q1.needs_attention(), "an unanswered Unknown-kind field still needs attention");
    }

    #[test]
    fn inline_data_uri_image_survives_unfiltered() {
        let page: ExtractedPage = serde_json::from_str(fixture_json()).unwrap();
        let q3 = page.questions.iter().find(|q| q.id == "q3").unwrap();
        assert_eq!(q3.image_urls, vec!["data:image/png;base64,iVBORw0KGgoAAAANSUhEUg==".to_string()]);
    }

    #[test]
    fn partially_filled_grid_needs_attention_for_its_blank_row_only() {
        let page: ExtractedPage = serde_json::from_str(fixture_json()).unwrap();
        let q2 = page.questions.iter().find(|q| q.id == "q2").unwrap();
        assert!(q2.needs_attention(), "grid with one blank row must still need attention");
        assert_eq!(q2.blank_rows, vec!["Row 2".to_string()]);
        assert!(q2.current_value.as_ref().unwrap().contains("Row 1"), "already-answered row 1 stays visible as context");
    }

    #[test]
    fn fully_answered_radio_does_not_need_attention() {
        let page: ExtractedPage = serde_json::from_str(fixture_json()).unwrap();
        let q0 = page.questions.iter().find(|q| q.id == "q0").unwrap();
        assert!(!q0.needs_attention());
    }

    #[test]
    fn grid_with_no_blank_rows_does_not_need_attention() {
        let mut q = Question {
            id: "g".into(), kind: QuestionKind::Grid, text: "t".into(),
            options: vec![], rows: vec!["Row 1".into()], blank_rows: vec![],
            image_urls: vec![], current_value: Some("{\"Row 1\":\"Always\"}".into()),
        };
        assert!(!q.needs_attention());
        q.blank_rows.push("Row 1".into());
        assert!(q.needs_attention());
    }
}
