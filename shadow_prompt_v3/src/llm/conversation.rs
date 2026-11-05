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
/// Once over the cap, drop down to `MAX_IMAGES - IMAGE_DROP_BATCH`, not just to the cap: every
/// removal edits an early message, which breaks the cached prefix from there on. Dropping several
/// at once means one cache rewrite per few screenshots instead of one per screenshot (Claude Code
/// clears old context in batches for the same reason).
pub const IMAGE_DROP_BATCH: usize = 5;
pub const REMOVED_IMAGE_TEXT: &str = "(earlier screenshot removed)";
/// Total base64 image bytes per request. Every question re-uploads the whole history, so this is
/// sized for slow connections: ~6 MB is ~30 typical JPEG screenshots (capture/image.rs), far more
/// than MAX_IMAGES, so the count cap normally bites first and this only stops a run of very busy
/// screens from making each question a multi-minute upload.
pub const MAX_IMAGE_BYTES: usize = 6 * 1024 * 1024;
/// Room kept free under MAX_IMAGE_BYTES for the question's own screenshot, which is sent on top
/// of the history (capture/image.rs keeps one at ≤1.2 MB, ~1.6 MB as base64).
const IMAGE_BYTES_RESERVE: usize = 1_600_000;

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
        self.insert_question(self.turns.len(), question, answer);
    }

    /// Inserts a finished question turn at `position` (clamped to the end).
    pub fn insert_question(&mut self, position: usize, question: Vec<ContentPart>, answer: String) {
        let at = position.min(self.turns.len());
        self.turns.insert(at, Turn { kind: TurnKind::Question, user: question, answer: Some(answer) });
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

    fn image_bytes(&self) -> usize {
        self.turns
            .iter()
            .flat_map(|t| &t.user)
            .map(|p| match p {
                ContentPart::Image { image_url } => image_url.url.len(),
                _ => 0,
            })
            .sum()
    }

    /// Replaces the oldest images with a short note until at most `MAX_IMAGES` remain. Question
    /// screenshots go first; reference (context) screenshots only once no question image is
    /// left, since the reference picture is usually the oldest and the most important.
    fn enforce_image_cap(&mut self) {
        // History stays at MAX_IMAGES - 1 or fewer: the question being asked adds its own
        // screenshot on top, and the request as a whole must stay within MAX_IMAGES.
        let byte_cap = MAX_IMAGE_BYTES - IMAGE_BYTES_RESERVE;
        if self.image_count() < MAX_IMAGES && self.image_bytes() <= byte_cap {
            return;
        }
        while self.image_count() > MAX_IMAGES - IMAGE_DROP_BATCH || self.image_bytes() > byte_cap {
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
        assert!(urls.len() < MAX_IMAGES, "room left for the next question's screenshot");
        assert!(urls.len() > MAX_IMAGES - IMAGE_DROP_BATCH, "drops a batch, not everything");
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
        assert_eq!(urls.len(), MAX_IMAGES - IMAGE_DROP_BATCH + 1);
        assert!(urls[0].ends_with(",c5"));
    }

    #[test]
    fn question_inserted_before_context_added_mid_flight() {
        let mut c = Conversation::default();
        let position = c.len();
        c.push_context(vec![ContentPart::text("ctx")]);
        c.insert_question(position, vec![ContentPart::text("q")], "a".into());
        let json = serde_json::to_value(c.to_messages()).unwrap();
        assert_eq!(json[0]["content"][0]["text"], "q");
        assert_eq!(json[2]["content"][0]["text"], "ctx");
    }

    #[test]
    fn byte_cap_drops_oldest_question_image() {
        let mut c = Conversation::default();
        let big = "x".repeat(MAX_IMAGE_BYTES / 2 + 1);
        c.push_context(vec![img(&big)]);
        c.push_question(vec![img(&format!("q{big}"))], "a".into());
        c.push_question(vec![img(&format!("r{big}"))], "a".into());
        assert!(c.image_bytes() <= MAX_IMAGE_BYTES);
        let urls = image_urls(&c);
        assert_eq!(urls.len(), 1, "context alone already fills half the budget");
        assert!(urls[0].contains(",x"), "context image kept");
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
