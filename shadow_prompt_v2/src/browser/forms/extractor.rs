// EXTRACTOR_JS — injected JavaScript that reads the current Forms page DOM
// into a structured JSON array of Question records.

pub const EXTRACTOR_JS: &str = r#"
// TODO: port v1's extractor, returns
//   { questions: [{ id, kind, text, options[], image_urls[], current_value }] }
"#;

#[derive(Debug, Clone)]
pub struct Question {
    pub id: String,
    pub kind: QuestionKind,
    pub text: String,
    pub options: Vec<String>,
    pub image_urls: Vec<String>,
    pub current_value: Option<String>,
}

#[derive(Debug, Clone)]
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
