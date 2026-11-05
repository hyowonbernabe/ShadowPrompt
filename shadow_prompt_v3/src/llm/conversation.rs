// The running conversation for Clipboard/Screenshot Query (design:
// docs/superpowers/specs/2026-11-05-conversation-memory-design.md). The model has no memory, so
// finished turns are kept here and resent with every request. In memory only: a daemon restart
// or the reset hotkey starts fresh.
//
// Only finished turns are stored, and only the question + final answer text of each — the
// tool-call/web-search rounds inside a turn are needed only within that turn's own loop.
//
// ponytail: no text-length cap. Sonnet 5.5 holds ~1M tokens; a request that is too long just
// errors and Reset fixes it. Add trimming of the oldest turns if that ever bites in practice.

use super::messages::{ContentPart, Message};

/// Past this many images Anthropic applies a stricter per-image size limit to every image.
pub const MAX_IMAGES: usize = 20;
pub const REMOVED_IMAGE_TEXT: &str = "(earlier screenshot removed)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnKind {
    /// Reference material added with the add-context hotkey — no answer.
    Context,
    /// A question and the model's final answer.
    Question,
}

#[derive(Debug, Clone)]
struct Turn {
    kind: TurnKind,
    user: Vec<ContentPart>,
    answer: Option<String>,
}

#[derive(Debug, Default)]
pub struct Conversation {
    turns: Vec<Turn>,
}

impl Conversation {
    pub fn push_context(&mut self, content: Vec<ContentPart>) {
        self.turns.push(Turn { kind: TurnKind::Context, user: content, answer: None });
        self.enforce_image_cap();
    }

    pub fn push_question(&mut self, question: Vec<ContentPart>, answer: String) {
        self.turns.push(Turn { kind: TurnKind::Question, user: question, answer: Some(answer) });
        self.enforce_image_cap();
    }

    pub fn to_messages(&self) -> Vec<Message> {
        let mut out = Vec::with_capacity(self.turns.len() * 2);
        for t in &self.turns {
            out.push(Message::User { content: t.user.clone() });
            if let Some(a) = &t.answer {
                out.push(Message::assistant_text(a.clone()));
            }
        }
        out
    }

    pub fn has_image(&self) -> bool {
        self.image_count() > 0
    }

    pub fn clear(&mut self) {
        self.turns.clear();
    }

    pub fn len(&self) -> usize {
        self.turns.len()
    }

    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    fn image_count(&self) -> usize {
        self.turns.iter().flat_map(|t| &t.user).filter(|p| matches!(p, ContentPart::Image { .. })).count()
    }

    /// Replaces the oldest images with a short note until at most `MAX_IMAGES` remain. Question
    /// screenshots go first; reference (context) screenshots only once no question image is
    /// left, since the reference picture is usually the oldest and the most important.
    fn enforce_image_cap(&mut self) {
        while self.image_count() > MAX_IMAGES {
            let victim = [TurnKind::Question, TurnKind::Context].into_iter().find_map(|kind| {
                self.turns.iter().enumerate().filter(|(_, t)| t.kind == kind).find_map(|(ti, t)| {
                    t.user.iter().position(|p| matches!(p, ContentPart::Image { .. })).map(|pi| (ti, pi))
                })
            });
            match victim {
                Some((ti, pi)) => self.turns[ti].user[pi] = ContentPart::text(REMOVED_IMAGE_TEXT),
                None => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::messages::ImageUrl;

    fn img(tag: &str) -> ContentPart {
        ContentPart::Image { image_url: ImageUrl { url: format!("data:image/png;base64,{tag}") } }
    }

    fn image_urls(c: &Conversation) -> Vec<String> {
        c.turns
            .iter()
            .flat_map(|t| &t.user)
            .filter_map(|p| match p {
                ContentPart::Image { image_url } => Some(image_url.url.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn question_turn_stores_question_and_answer_only() {
        let mut c = Conversation::default();
        c.push_question(vec![ContentPart::text("2+2?")], "4".into());
        let json = serde_json::to_value(c.to_messages()).unwrap();
        assert_eq!(
            json,
            serde_json::json!([
                {"role": "user", "content": [{"type": "text", "text": "2+2?"}]},
                {"role": "assistant", "content": "4"}
            ])
        );
    }

    #[test]
    fn context_turn_has_no_assistant_message() {
        let mut c = Conversation::default();
        c.push_context(vec![ContentPart::text("ref"), img("a")]);
        assert_eq!(c.to_messages().len(), 1);
        assert!(c.has_image());
    }

    #[test]
    fn cap_drops_question_images_before_context_images() {
        let mut c = Conversation::default();
        c.push_context(vec![img("ctx")]);
        for i in 0..MAX_IMAGES {
            c.push_question(vec![img(&format!("q{i}"))], "a".into());
        }
        let urls = image_urls(&c);
        assert_eq!(urls.len(), MAX_IMAGES);
        assert!(urls[0].ends_with("ctx"), "context image must survive");
        assert!(!urls.iter().any(|u| u.ends_with(",q0")), "oldest question image must go first");
        // the removed image's turn keeps its place, with a note instead of the picture
        assert_eq!(c.len(), MAX_IMAGES + 1);
    }

    #[test]
    fn cap_drops_oldest_context_image_when_only_context_images_remain() {
        let mut c = Conversation::default();
        for i in 0..=MAX_IMAGES {
            c.push_context(vec![img(&format!("c{i}"))]);
        }
        let urls = image_urls(&c);
        assert_eq!(urls.len(), MAX_IMAGES);
        assert!(urls[0].ends_with(",c1"));
    }

    #[test]
    fn clear_empties_history() {
        let mut c = Conversation::default();
        c.push_context(vec![img("a")]);
        c.clear();
        assert!(c.is_empty());
        assert!(!c.has_image());
    }
}
