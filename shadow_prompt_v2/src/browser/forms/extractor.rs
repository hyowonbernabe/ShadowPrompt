// EXTRACTOR_JS — reads the current Forms page DOM into a JSON array of Question records.

pub const EXTRACTOR_JS: &str = r#"
(() => {
  const out = [];
  const items = document.querySelectorAll('[role="listitem"]');
  items.forEach((item, idx) => {
    const titleEl = item.querySelector('[role="heading"]');
    const text = titleEl ? titleEl.innerText.trim() : '';
    if (!text) return;
    const id = 'q' + idx;
    const imgs = Array.from(item.querySelectorAll('img'))
      .map(i => i.src)
      .filter(s => s && !s.startsWith('data:'));

    // Date / Time
    const dateInput = item.querySelector('input[type="date"]');
    if (dateInput) {
      out.push({ id, kind: 'Date', text, options: [], rows: [], image_urls: imgs, current_value: dateInput.value || null });
      return;
    }
    const timeInput = item.querySelector('input[type="time"]');
    if (timeInput) {
      out.push({ id, kind: 'Time', text, options: [], rows: [], image_urls: imgs, current_value: timeInput.value || null });
      return;
    }

    // Grid (multiple radiogroups; one per row)
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
      radioGroups.forEach(g => {
        const label = (g.getAttribute('aria-label') || '').trim();
        rows.push(label);
        const checked = Array.from(g.querySelectorAll('[role="radio"]'))
          .find(r => r.getAttribute('aria-checked') === 'true');
        if (checked) {
          chosen[label] = (checked.getAttribute('aria-label') || '').trim();
        }
      });
      out.push({
        id, kind: 'Grid', text,
        options: columns, rows,
        image_urls: imgs,
        current_value: Object.keys(chosen).length ? JSON.stringify(chosen) : null
      });
      return;
    }

    // Single radio set (Radio or Scale)
    const radios = item.querySelectorAll('[role="radio"]');
    if (radios.length > 0) {
      const options = Array.from(radios).map(r => (r.getAttribute('aria-label') || r.innerText.trim()));
      const checked = Array.from(radios).find(r => r.getAttribute('aria-checked') === 'true');
      const allNumeric = options.length > 0 && options.every(o => /^\s*\d+\s*$/.test(o));
      out.push({
        id, kind: allNumeric ? 'Scale' : 'Radio', text,
        options, rows: [],
        image_urls: imgs,
        current_value: checked ? (checked.getAttribute('aria-label') || checked.innerText.trim()) : null
      });
      return;
    }

    // Checkbox grid (multiple checkbox groups)
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
      checkGroups.forEach(g => {
        const label = (g.getAttribute('aria-label') || '').trim();
        rows.push(label);
        const picks = Array.from(g.querySelectorAll('[role="checkbox"]'))
          .filter(c => c.getAttribute('aria-checked') === 'true')
          .map(c => (c.getAttribute('aria-label') || '').trim());
        if (picks.length > 0) chosen[label] = picks.join(', ');
      });
      out.push({
        id, kind: 'CheckboxGrid', text,
        options: columns, rows,
        image_urls: imgs,
        current_value: Object.keys(chosen).length ? JSON.stringify(chosen) : null
      });
      return;
    }

    // Single checkbox set
    const checks = item.querySelectorAll('[role="checkbox"]');
    if (checks.length > 0) {
      const options = Array.from(checks).map(r => (r.getAttribute('aria-label') || r.innerText.trim()));
      const picks = Array.from(checks)
        .filter(r => r.getAttribute('aria-checked') === 'true')
        .map(r => (r.getAttribute('aria-label') || r.innerText.trim()));
      out.push({
        id, kind: 'Checkbox', text,
        options, rows: [],
        image_urls: imgs,
        current_value: picks.length ? picks.join(', ') : null
      });
      return;
    }

    // Dropdown
    const listbox = item.querySelector('[role="listbox"]');
    if (listbox) {
      const opts = Array.from(listbox.querySelectorAll('[role="option"]'))
        .map(o => (o.getAttribute('data-value') || o.innerText || '').trim())
        .filter(s => s && s !== '...' && !s.toLowerCase().startsWith('choose') && !s.toLowerCase().startsWith('select'));
      const sel = listbox.querySelector('[aria-selected="true"]');
      out.push({
        id, kind: 'Dropdown', text,
        options: opts, rows: [],
        image_urls: imgs,
        current_value: sel ? sel.innerText.trim() : null
      });
      return;
    }

    // Long text
    const textarea = item.querySelector('textarea');
    if (textarea) {
      out.push({ id, kind: 'LongText', text, options: [], rows: [], image_urls: imgs, current_value: textarea.value || null });
      return;
    }

    // Short text variants
    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"], input[type="tel"], input[type="url"]');
    if (input) {
      const kind = input.getAttribute('type') === 'number' ? 'Numeric' : 'ShortText';
      out.push({ id, kind, text, options: [], rows: [], image_urls: imgs, current_value: input.value || null });
      return;
    }
  });
  return JSON.stringify(out);
})()
"#;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Question {
    pub id: String,
    pub kind: QuestionKind,
    pub text: String,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub rows: Vec<String>,
    #[serde(default)]
    pub image_urls: Vec<String>,
    pub current_value: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
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
}
