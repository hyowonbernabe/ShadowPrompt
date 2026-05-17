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

    const radios = item.querySelectorAll('[role="radio"]');
    if (radios.length > 0) {
      const options = Array.from(radios).map(r => r.getAttribute('aria-label') || r.innerText.trim());
      const checked = Array.from(radios).find(r => r.getAttribute('aria-checked') === 'true');
      out.push({
        id, kind: 'Radio', text,
        options,
        image_urls: imgs,
        current_value: checked ? (checked.getAttribute('aria-label') || checked.innerText.trim()) : null
      });
      return;
    }

    const checks = item.querySelectorAll('[role="checkbox"]');
    if (checks.length > 0) {
      const options = Array.from(checks).map(r => r.getAttribute('aria-label') || r.innerText.trim());
      const checked = Array.from(checks)
        .filter(r => r.getAttribute('aria-checked') === 'true')
        .map(r => r.getAttribute('aria-label') || r.innerText.trim());
      out.push({
        id, kind: 'Checkbox', text,
        options, image_urls: imgs,
        current_value: checked.length ? checked.join(',') : null
      });
      return;
    }

    const input = item.querySelector('input[type="text"], input[type="email"], input[type="number"]');
    const textarea = item.querySelector('textarea');
    if (textarea) {
      out.push({
        id, kind: 'LongText', text,
        options: [], image_urls: imgs,
        current_value: textarea.value || null
      });
      return;
    }
    if (input) {
      out.push({
        id, kind: 'ShortText', text,
        options: [], image_urls: imgs,
        current_value: input.value || null
      });
      return;
    }

    const listbox = item.querySelector('[role="listbox"]');
    if (listbox) {
      const sel = listbox.querySelector('[aria-selected="true"]');
      out.push({
        id, kind: 'Dropdown', text,
        options: [],
        image_urls: imgs,
        current_value: sel ? sel.innerText.trim() : null
      });
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
    Date,
    Time,
    Grid,
    Scale,
}
