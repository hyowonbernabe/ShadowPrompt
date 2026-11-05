// Clipboard Query — design doc §3. Text only. Tools: list_docs, read_doc (web_search is
// attached inside LlmClient::run_turn as the server tool, not passed in here).

use std::sync::Arc;

use super::{busy, deliver_answer, report_failure, ActionContext};
use crate::capture::clipboard;
use crate::knowledge::{ListDocsTool, ReadDocTool};
use crate::llm::messages::ContentPart;
use crate::llm::system_prompts::general_prompt;
use crate::llm::Tool;
use crate::ui::commands::{IndicatorState, UICommand};

/// Clipboard text beyond this is cut: it's stored in the conversation and resent with every
/// later question, so a huge accidental paste would slow and break the whole test.
const MAX_QUESTION_CHARS: usize = 100_000;

pub async fn execute(ctx: ActionContext) -> anyhow::Result<()> {
    let _ = ctx.ui_tx.send(UICommand::SetIndicatorState(IndicatorState::Processing));
    let result = async {
        let question = match tokio::task::spawn_blocking(clipboard::read_text).await? {
            Ok(q) => q,
            // No text (an image, a file) or locked by another app — both mean "nothing to ask".
            Err(e) => anyhow::bail!("clipboard is empty or unreadable: {e}"),
        };
        if question.trim().is_empty() {
            anyhow::bail!("clipboard is empty");
        }
        let question = truncate_question(question);
        let tools = memory_tools(&ctx);
        ask_in_conversation(&ctx, vec![ContentPart::text(question)], false, &tools).await
    }
    .await;
    match result {
        Ok(answer) => {
            deliver_answer(&ctx, answer).await;
            Ok(())
        }
        Err(e) => {
            report_failure(&ctx, &e);
            Err(e)
        }
    }
}

/// Keeps the start and the end and cuts the middle: in a pasted passage the question usually
/// comes last, so the end matters as much as the beginning.
fn truncate_question(q: String) -> String {
    let n = q.chars().count();
    if n <= MAX_QUESTION_CHARS {
        return q;
    }
    let head: String = q.chars().take(MAX_QUESTION_CHARS * 7 / 10).collect();
    let tail: String = q.chars().skip(n - MAX_QUESTION_CHARS * 3 / 10).collect();
    format!("{head}\n\n(... middle of the clipboard text cut: too long ...)\n\n{tail}")
}

/// Runs one question as the next turn of the running conversation (shared by Clipboard and
/// Screenshot Query). The turn is stored only once a final answer exists: an error returns
/// early, and an abort (new press / abort hotkey) drops this future before the push — either
/// way a failed turn never pollutes the history.
pub async fn ask_in_conversation(
    ctx: &ActionContext,
    content: Vec<ContentPart>,
    content_has_image: bool,
    tools: &[Arc<dyn Tool>],
) -> anyhow::Result<String> {
    let (history, history_has_image, position) = {
        let c = ctx.conversation.lock().unwrap();
        (c.to_messages(), c.has_image(), c.len())
    };
    let _busy = busy(ctx);
    let answer = ctx
        .llm
        .run_turn_with_history(&general_prompt(), history, content.clone(), tools, content_has_image || history_has_image, None)
        .await?;
    // Stored where the snapshot ended, so context added while this was in flight stays after
    // it — the history matches what the model actually saw.
    // A cut-off answer is shown, not kept: a truncated turn would mislead every later question.
    if !answer.ends_with(crate::llm::CUT_OFF_NOTE) {
        ctx.conversation.lock().unwrap().insert_question(position, content, answer.clone());
    }
    Ok(answer)
}

/// Shared by Clipboard/Screenshot Query — design doc §4: memory tools available to all three
/// entry points, sandboxed to the configured knowledge/ folder.
pub fn memory_tools(ctx: &ActionContext) -> Vec<Arc<dyn Tool>> {
    if !ctx.config.knowledge.enabled {
        return Vec::new();
    }
    vec![
        Arc::new(ListDocsTool(ctx.knowledge.clone())),
        Arc::new(ReadDocTool(ctx.knowledge.clone())),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::{
        Config, FormsConfig, HotkeysConfig, HttpConfig, KnowledgeConfig, OpenRouterConfig,
        VisualsConfig,
    };
    use crate::knowledge::KnowledgeStore;
    use crate::llm::LlmClient;
    use tokio::sync::Mutex;

    fn test_config(knowledge_enabled: bool) -> Config {
        Config {
            openrouter: OpenRouterConfig {
                api_key: "test-key".to_string(),
                models: vec!["google/gemini-3.5-flash-lite".to_string()],
            },
            hotkeys: HotkeysConfig {
                clipboard_query: "VK_F1".to_string(),
                screenshot_query: "VK_F2".to_string(),
                test_model: "VK_F3".to_string(),
                switch_model: "VK_F4".to_string(),
                add_context: "VK_F16".to_string(),
                reset_conversation: "VK_F17".to_string(),
                launch_debugger: "VK_F5".to_string(),
                forms_answer_page: "VK_F6".to_string(),
                forms_answer_all: "VK_F7".to_string(),
                forms_answer_page_axtree: "VK_F14".to_string(),
                forms_answer_all_axtree: "VK_F15".to_string(),
                abort: "VK_F8".to_string(),
                hide_toggle: "VK_F9".to_string(),
                help_toggle: "VK_F10".to_string(),
                restart_daemon: "VK_F11".to_string(),
                insta_delete: "VK_F12".to_string(),
                panic_kill: "VK_ESCAPE".to_string(),
                debug_open_tab: "VK_F13".to_string(),
                screen_answer: "VK_F14".to_string(),
                clear_dots: "VK_F15".to_string(),
            },
            visuals: VisualsConfig {
                indicator_corner: "top_right".to_string(),
                indicator_size: 4,
                indicator_offset: [0, 0],
                color_ready: "#00FF00".to_string(),
                color_processing: "#FF0000".to_string(),
                color_error: "#FFFF00".to_string(),
                form_indicator_corner: "top_left".to_string(),
                form_indicator_offset: [0, 0],
                form_color_running: "#00FFFF".to_string(),
                form_color_done: "#00FF00".to_string(),
                form_color_failed: "#FF00FF".to_string(),
                form_color_aborted: "#000000".to_string(),
                overlay_enabled: true,
                overlay_corner: "bottom_right".to_string(),
                overlay_offset: [0, 0],
                overlay_font_size: 14,
                crawl_trigger_lines: 2,
                crawl_ms_per_line: 2000,
                dot_color: "#FF0000".to_string(),
                dot_size: 3,
            },
            http: HttpConfig { connect_timeout_secs: 5, read_timeout_secs: 30 },
            forms: FormsConfig { max_pages: 20, max_images_per_request: 4, max_image_long_edge_px: 1600 },
            knowledge: KnowledgeConfig { enabled: knowledge_enabled, max_doc_bytes: 100_000 },
            screen_answer: Default::default(),
        }
    }

    fn test_ctx(knowledge_root: std::path::PathBuf, knowledge_enabled: bool) -> ActionContext {
        let (ui_tx, _ui_rx) = tokio::sync::mpsc::unbounded_channel();
        let llm = LlmClient::new("test-key".to_string(), vec!["google/gemini-3.5-flash-lite".to_string()], 5, 30)
            .expect("LlmClient::new should not fail on well-formed dummy config");
        ActionContext {
            config: Arc::new(test_config(knowledge_enabled)),
            llm: Arc::new(llm),
            knowledge: Arc::new(KnowledgeStore::new(knowledge_root, 100_000)),
            ui_tx,
            answer_task: Arc::new(Mutex::new(None)),
            forms_task: Arc::new(Mutex::new(None)),
            conversation: Default::default(),
            busy: Default::default(),
            screen_task: Arc::new(Mutex::new(None)),
        }
    }

    /// Unique-per-test scratch dir under the OS temp folder, cleaned up on drop.
    struct TempKnowledgeDir(std::path::PathBuf);

    impl TempKnowledgeDir {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "shadowprompt_v3_test_knowledge_{tag}_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&dir).expect("create temp knowledge dir");
            Self(dir)
        }
    }

    impl Drop for TempKnowledgeDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn huge_clipboard_text_is_cut_on_a_char_boundary() {
        let q = format!("{}QUESTION?", "é".repeat(MAX_QUESTION_CHARS + 10));
        let t = truncate_question(q);
        assert!(t.starts_with("éé") && t.contains("middle of the clipboard text cut"));
        assert!(t.ends_with("QUESTION?"), "the question at the end survives");
        assert!(t.chars().count() < MAX_QUESTION_CHARS + 100);
        assert_eq!(truncate_question("short".into()), "short");
    }

    #[tokio::test]
    async fn indicator_stays_processing_until_every_model_call_ends_even_on_abort() {
        use crate::actions::{abort_slot, busy, idle_indicator};
        use crate::ui::commands::IndicatorState;
        let ctx = test_ctx(std::env::temp_dir(), false);
        let a = busy(&ctx);
        let c2 = ctx.clone();
        let handle = tokio::spawn(async move {
            let _b = busy(&c2);
            std::future::pending::<()>().await;
        });
        tokio::task::yield_now().await;
        *ctx.screen_task.lock().await = Some(handle);
        drop(a);
        assert!(matches!(idle_indicator(&ctx), IndicatorState::Processing), "screen job still running");
        abort_slot(&ctx.screen_task).await;
        assert!(matches!(idle_indicator(&ctx), IndicatorState::Ready), "aborted job must release its count");
    }

    #[tokio::test]
    async fn memory_tools_round_trips_list_then_read() {
        let dir = TempKnowledgeDir::new("roundtrip");
        std::fs::write(dir.0.join("note.md"), "hello from a real knowledge doc").unwrap();

        let ctx = test_ctx(dir.0.clone(), true);
        let tools = memory_tools(&ctx);
        assert_eq!(tools.len(), 2, "expect list_docs + read_doc when knowledge is enabled");

        let list_tool = tools.iter().find(|t| t.name() == "list_docs").expect("list_docs tool present");
        let listed_json = list_tool.execute("{}").await.expect("list_docs should succeed");
        let listed: Vec<String> = serde_json::from_str(&listed_json).unwrap();
        assert_eq!(listed, vec!["note.md".to_string()]);

        let read_tool = tools.iter().find(|t| t.name() == "read_doc").expect("read_doc tool present");
        let content = read_tool
            .execute(r#"{"name":"note.md"}"#)
            .await
            .expect("read_doc should succeed for a name list_docs just returned");
        assert_eq!(content, "hello from a real knowledge doc");
    }

    #[tokio::test]
    async fn memory_tools_read_doc_rejects_sandbox_escape() {
        let dir = TempKnowledgeDir::new("escape");
        std::fs::write(dir.0.join("note.md"), "in sandbox").unwrap();

        let ctx = test_ctx(dir.0.clone(), true);
        let tools = memory_tools(&ctx);
        let read_tool = tools.iter().find(|t| t.name() == "read_doc").unwrap();

        let escape_attempt = read_tool.execute(r#"{"name":"../outside.txt"}"#).await;
        assert!(escape_attempt.is_err(), "escaping the knowledge/ sandbox must be rejected");
    }

    #[tokio::test]
    async fn memory_tools_empty_when_knowledge_disabled() {
        let dir = TempKnowledgeDir::new("disabled");
        let ctx = test_ctx(dir.0.clone(), false);
        assert!(memory_tools(&ctx).is_empty());
    }
}
