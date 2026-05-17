// Detect which questions the user has already answered. Skip those when calling
// the model and never overwrite them when injecting.

use super::extractor::Question;

pub fn filter_unanswered(questions: &[Question]) -> Vec<&Question> {
    questions
        .iter()
        .filter(|q| q.current_value.is_none() || q.current_value.as_deref() == Some(""))
        .collect()
}
