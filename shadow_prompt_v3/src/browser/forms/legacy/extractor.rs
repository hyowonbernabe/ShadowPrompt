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
(async () => {
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
  // Top-level questions only. Confirmed live: Google Forms also tags each individual
  // checkbox/radio *option* inside a group with its own role="listitem" (nested inside the
  // group's own listitem) — without the `:not()` guard, a 5-option checkbox question was read as
  // 1 real question plus 5 phantom one-option ones, and the model answering those phantoms
  // overwrote an already-correct multi-select with wrong extra picks. This selector must match
  // the one in `injector.rs`'s `build_injector_js`/`select_dropdown_option` exactly, or their
  // `qN` ids drift apart from what this file assigns.
  const items = document.querySelectorAll('[role="listitem"]:not([role="listitem"] [role="listitem"])');

  // Confirmed live: current Google Forms grid/checkbox-grid markup uses no
  // role="columnheader"/"rowheader" at all, so both were always dead code here, silently falling
  // through to fallback heuristics. The old column fallback (reading the first row's own option
  // aria-labels) bakes the row name into the text (e.g. "Never heard of it, response for
  // Algebra"), and there was no row-label fallback at all for CheckboxGrid specifically (its
  // groups carry no aria-label at all, unlike Grid's, which happens to still have one) — every
  // CheckboxGrid row read as "", indistinguishable from every other row. Replaced with structural
  // detection that depends on neither ARIA roles nor Google's own (unstable) class names.
  const gridColumnHeaders = (firstGroup) => {
    // The real header row is a plain sibling of every row's own group, shared identically across
    // all rows, rendered as one element whose text joins every column name with a tab character.
    let p = firstGroup.parentElement;
    for (let i = 0; i < 4 && p; i++) {
      for (const c of Array.from(p.children)) {
        if (c === firstGroup || c.contains(firstGroup)) continue;
        const t = (c.innerText || '');
        if (t.includes('\t')) return t.split('\t').map(s => s.trim()).filter(Boolean);
      }
      p = p.parentElement;
    }
    return [];
  };
  const gridRowLabel = (group) => {
    const al = (group.getAttribute('aria-label') || '').trim();
    if (al) return al;
    // Every option's own label lives only in its aria-label, never as inner text, so the row's
    // own single non-control leaf-text div is unambiguous where present.
    const leaf = Array.from(group.querySelectorAll('div')).find(d => d.children.length === 0 && (d.innerText || '').trim());
    return leaf ? leaf.innerText.trim() : '';
  };

  // Confirmed live: a Google Forms question image's URL is a signed, ephemeral, per-load token —
  // handing that URL to OpenRouter makes every provider (and this app's own separate fetch of it)
  // fetch it independently over the network a second time, which is both an unwanted extra point
  // of failure on a bad connection and, worse, sometimes gets redirected by Google's own CDN to a
  // mirror URL that's already dead by the time anything follows it. The browser has *already*
  // downloaded and decoded these bytes once, just to render them on screen — reusing that via a
  // canvas instead needs no network call at all. Works because the image and the page are both
  // served from docs.google.com (same-origin), so the canvas is never cross-origin-tainted.
  // Falls back to the raw URL (the old behavior) if the browser ever declines the canvas read.
  //
  // Confirmed live: that fallback was firing on a real exam form, leaking the signed CDN URL to
  // OpenRouter anyway (every provider then failed to fetch it a second time, exactly what this
  // was built to avoid) — not a same-origin/taint problem, but a timing one. Right after a page
  // load/navigation, an `<img>` below the fold can still have `naturalWidth === 0` because the
  // browser hasn't finished decoding it yet, so the old synchronous version bailed out before the
  // image was ever actually ready. Waiting for `load`/`error` (or a 2s cap so one stuck image
  // can't hang the whole page's extraction) gives the browser the chance to finish first.
  const imgToDataUrl = async (imgEl) => {
    if (!imgEl.complete || !imgEl.naturalWidth) {
      await new Promise(res => {
        imgEl.addEventListener('load', res, { once: true });
        imgEl.addEventListener('error', res, { once: true });
        setTimeout(res, 2000);
      });
    }
    try {
      const canvas = document.createElement('canvas');
      canvas.width = imgEl.naturalWidth || imgEl.width;
      canvas.height = imgEl.naturalHeight || imgEl.height;
      if (!canvas.width || !canvas.height) return null;
      canvas.getContext('2d').drawImage(imgEl, 0, 0);
      return canvas.toDataURL('image/png');
    } catch (e) {
      return null;
    }
  };

  // A plain `.forEach` can't `await` per item — needs a real loop now that `imgToDataUrl` does.
  for (const [idx, item] of Array.from(items).entries()) {
    const titleEl = item.querySelector('[role="heading"]');
    const text = titleEl ? titleEl.innerText.trim() : '';
    const id = 'q' + idx;
    const imgEls = Array.from(item.querySelectorAll('img'));
    const imgs = (await Promise.all(imgEls.map(imgToDataUrl)))
      .map((d, i2) => d || imgEls[i2].src)
      .filter(Boolean);

    const dateInput = item.querySelector('input[type="date"]');
    if (dateInput) {
      out.push({ id, kind: 'Date', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: dateInput.value || null });
      continue;
    }
    const timeInput = item.querySelector('input[type="time"]');
    if (timeInput) {
      out.push({ id, kind: 'Time', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: timeInput.value || null });
      continue;
    }

    // Confirmed live: current Google Forms Time questions render as a hour/minute pair of text
    // inputs (aria-label "Hour"/"Minute", 12-hour range) plus a separate AM/PM `role="listbox"` —
    // never the native `input[type="time"]` checked above (that branch is now legacy/dead for any
    // form built recently, kept only in case an older form still uses it). Must be checked before
    // the generic listbox/Dropdown branch below, or this item's own AM/PM picker gets mistaken for
    // the whole question being a 2-option Dropdown, silently hiding the Hour/Minute inputs from
    // ever being read at all — confirmed live to be exactly why a real Time question came back
    // blank end to end. Google's AM/PM listbox defaults to showing "AM" pre-selected with a real
    // (non-placeholder) value even on a totally untouched question, unlike Dropdown's own
    // placeholder — so hour/minute both being empty, not the listbox's selection state, is the
    // only reliable "still blank" signal here (see injector.rs's matching comment).
    const hourInput = item.querySelector('input[aria-label="Hour"]');
    const minuteInput = item.querySelector('input[aria-label="Minute"]');
    if (hourInput && minuteInput) {
      const meridiemSel = item.querySelector('[role="listbox"] [aria-selected="true"]');
      const meridiemText = meridiemSel ? meridiemSel.innerText.trim() : '';
      const filled = hourInput.value && minuteInput.value;
      out.push({
        id, kind: 'Time', text, options: [], rows: [], blank_rows: [], image_urls: imgs,
        current_value: filled ? `${hourInput.value}:${minuteInput.value} ${meridiemText}`.trim() : null
      });
      continue;
    }

    const radioGroups = item.querySelectorAll('[role="radiogroup"]');
    if (radioGroups.length > 1) {
      let columns = gridColumnHeaders(radioGroups[0]);
      if (columns.length === 0) {
        const firstRadios = radioGroups[0].querySelectorAll('[role="radio"]');
        columns = Array.from(firstRadios).map(r => (r.getAttribute('aria-label') || '').trim());
      }
      const rows = [];
      const chosen = {};
      const blankRows = [];
      radioGroups.forEach(g => {
        const label = gridRowLabel(g);
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
      continue;
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
      continue;
    }

    const checkGroups = Array.from(item.querySelectorAll('[role="group"]'))
      .filter(g => g.querySelector('[role="checkbox"]'));
    if (checkGroups.length > 1) {
      let columns = gridColumnHeaders(checkGroups[0]);
      if (columns.length === 0) {
        const firstChecks = checkGroups[0].querySelectorAll('[role="checkbox"]');
        columns = Array.from(firstChecks).map(c => (c.getAttribute('aria-label') || '').trim());
      }
      const rows = [];
      const chosen = {};
      const blankRows = [];
      checkGroups.forEach(g => {
        const label = gridRowLabel(g);
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
      continue;
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
      continue;
    }

    const listbox = item.querySelector('[role="listbox"]');
    if (listbox) {
      // Confirmed live: the unselected placeholder option ("Choose"/"Pumili"/etc. — text is
      // locale-dependent) always carries an empty `data-value`, unlike every real option, and by
      // default *is itself* marked `aria-selected="true"` before anything is picked. Filtering by
      // that empty value (not by matching English placeholder text, which silently breaks on
      // other UI languages) is what tells "nothing chosen yet" apart from a real selection.
      const allOpts = Array.from(listbox.querySelectorAll('[role="option"]'));
      const opts = allOpts
        .filter(o => (o.getAttribute('data-value') || '').trim() !== '')
        .map(o => (o.getAttribute('data-value') || o.innerText || '').trim())
        .filter(s => s && s !== '...');
      const sel = allOpts.find(o => o.getAttribute('aria-selected') === 'true' && (o.getAttribute('data-value') || '').trim() !== '');
      out.push({
        id, kind: 'Dropdown', text, options: opts, rows: [], blank_rows: [], image_urls: imgs,
        current_value: sel ? sel.innerText.trim() : null
      });
      continue;
    }

    const textarea = item.querySelector('textarea');
    if (textarea) {
      out.push({ id, kind: 'LongText', text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: textarea.value || null });
      continue;
    }

    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"], input[type="tel"], input[type="url"]');
    if (input) {
      const kind = input.getAttribute('type') === 'number' ? 'Numeric' : 'ShortText';
      out.push({ id, kind, text, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: input.value || null });
      continue;
    }

    // Patch 1 (design doc §7.4): no recognized control at all (e.g. a picture-only question) —
    // still produces a field, never silently dropped the way v2's `if (!text) return` did.
    //
    // Patch (confirmed live): this also covers a section-break item — a heading plus a
    // paragraph of instructions/context (e.g. "Reading Comprehension" + "A train travels 120km
    // in 2 hours...") with no input control of its own. Without the block below, that paragraph
    // was silently dropped entirely: the page-level description capture above explicitly skips
    // anything inside a listitem (on the assumption this per-item branch would catch it), but
    // this branch previously only ever kept the heading. The model was answering the very next
    // question with zero knowledge of the passage it depends on. Same leaf-text heuristic as the
    // page-level capture, just scoped to this item instead of the whole page.
    const descParts = [];
    Array.from(item.querySelectorAll('div, span, p')).forEach(el => {
      if (el.closest('[role="heading"]')) return;
      if (el.children.length > 0) return;
      const t = (el.innerText || '').trim();
      if (t && t !== text && !descParts.includes(t)) descParts.push(t);
    });
    const fullText = descParts.length ? `${text}\n${descParts.join('\n')}` : text;
    out.push({ id, kind: 'Unknown', text: fullText, options: [], rows: [], blank_rows: [], image_urls: imgs, current_value: null });
  }
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
            // Confirmed live: a plain section-header item ("Reading Comprehension" + a passage,
            // no image, no input control) is `Unknown` kind with `current_value` hardcoded to
            // null, so without this branch it read as "unanswered" forever — forcing a real
            // OpenRouter call every single time the page was revisited, for an item the injector
            // can never write an answer into either way (no branch there handles `Unknown` at
            // all). An image-bearing `Unknown` item is different: that's a genuine picture-only
            // question (the scenario this kind was built for, design doc §7.4 patch 1) and still
            // needs a best-effort attempt.
            QuestionKind::Unknown => !self.image_urls.is_empty(),
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
    fn text_only_unknown_item_does_not_need_attention() {
        // A section header/divider ("Reading Comprehension" + a passage) — Unknown kind, no
        // image, nothing the injector can ever fill in. Must not force a repeated LLM call.
        let q = Question {
            id: "q0".into(), kind: QuestionKind::Unknown, text: "Reading Comprehension".into(),
            options: vec![], rows: vec![], blank_rows: vec![], image_urls: vec![], current_value: None,
        };
        assert!(!q.needs_attention());
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
