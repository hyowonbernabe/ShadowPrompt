# Question Type Handling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the clipboard/OCR answer pipeline to always run Search → RAG → LLM with labeled context sections, fix all accumulated bugs, and rewrite the system prompt to correctly handle every question type including multi-answer and unlabeled MCQ.

**Architecture:** `gather_context()` returns a typed `ContextBundle { web, local, warnings }` instead of a single string blob. `main.rs` assembles these into `[WEB SEARCH RESULTS]` / `[LOCAL KNOWLEDGE]` / `[QUESTION]` sections before sending to the LLM. All paths (text, OCR, vision) share a single `system_prompt.txt` loaded via an exe-relative path.

**Tech Stack:** Rust 2021 · Tokio · `regex` crate (already in deps) · `fastembed` · `serde_json::Value` · Windows.Media.Ocr · `reqwest`

**Spec:** `docs/superpowers/specs/2026-04-09-question-type-handling-design.md`

**Crate root:** All `cargo` commands run from `shadow_prompt/` directory.

---

## Task 1: Remove dead `use_rag` config field

**Files:**
- Modify: `shadow_prompt/src/config.rs`

- [ ] **Step 1: Remove `use_rag` from `GeneralConfig` struct**

In `src/config.rs`, find the `GeneralConfig` struct (line ~47) and remove the `use_rag` field:

```rust
// REMOVE this line from GeneralConfig struct:
pub use_rag: bool,
```

- [ ] **Step 2: Remove `use_rag` from `GeneralConfig::default()`**

Find the `Default for GeneralConfig` impl (line ~72) and remove:

```rust
// REMOVE this line from Default impl:
use_rag: true,
```

- [ ] **Step 3: Verify it compiles**

```bash
cd shadow_prompt
cargo check 2>&1
```

Expected: no errors. If you see `use_rag` referenced anywhere else, remove those references too (search with `grep -r "use_rag" src/`).

- [ ] **Step 4: Commit**

```bash
git add shadow_prompt/src/config.rs
git commit -m "refactor: remove dead use_rag field from GeneralConfig"
```

---

## Task 2: Fix `load_system_prompt()` to use exe-relative path

**Files:**
- Modify: `shadow_prompt/src/llm.rs`

This is the highest-priority fix. On USB deployments the CWD is rarely the exe directory, so `config/system_prompt.txt` silently fails and falls back to `"You are a concise assistant."` — the entire system prompt has been ignored in production.

- [ ] **Step 1: Replace `load_system_prompt()` body**

Find `fn load_system_prompt()` in `src/llm.rs` (around line 206) and replace the entire function body:

```rust
fn load_system_prompt() -> String {
    let path = crate::config::get_exe_dir()
        .join("config")
        .join("system_prompt.txt");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| "You are a concise assistant.".to_string())
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd shadow_prompt
cargo check 2>&1
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add shadow_prompt/src/llm.rs
git commit -m "fix: load system_prompt.txt from exe-relative path (was silently failing on USB)"
```

---

## Task 3: Add `extract_content()` helper and unify vision system prompt

**Files:**
- Modify: `shadow_prompt/src/llm.rs`

Currently all five query functions use `.as_str().context(...)` to parse LLM responses. Gemini via OpenRouter can return `content` as an array of objects, which silently crashes this parse. This task adds a shared helper and unifies the vision system prompt.

- [ ] **Step 1: Write the failing test for `extract_content()`**

Add a `#[cfg(test)]` block at the bottom of `src/llm.rs` (before the closing `}`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extract_content_string() {
        let json = json!({
            "choices": [{"message": {"content": "Paris"}}]
        });
        assert_eq!(LlmClient::extract_content(&json).unwrap(), "Paris");
    }

    #[test]
    fn test_extract_content_array() {
        let json = json!({
            "choices": [{"message": {"content": [
                {"type": "text", "text": "The answer is"},
                {"type": "text", "text": "Nitrogen"}
            ]}}]
        });
        assert_eq!(LlmClient::extract_content(&json).unwrap(), "The answer is\nNitrogen");
    }

    #[test]
    fn test_extract_content_array_skips_non_text() {
        let json = json!({
            "choices": [{"message": {"content": [
                {"type": "image_url", "image_url": {"url": "data:..."}},
                {"type": "text", "text": "B) Nitrogen"}
            ]}}]
        });
        assert_eq!(LlmClient::extract_content(&json).unwrap(), "B) Nitrogen");
    }

    #[test]
    fn test_extract_content_missing_fails() {
        let json = json!({"choices": [{"message": {"content": null}}]});
        assert!(LlmClient::extract_content(&json).is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd shadow_prompt
cargo test llm::tests 2>&1
```

Expected: compile error — `extract_content` not found.

- [ ] **Step 3: Add `extract_content()` as a public method on `LlmClient`**

Add this method inside the `impl LlmClient` block in `src/llm.rs`, just before the closing `}` of the impl:

```rust
pub fn extract_content(json: &Value) -> Result<String> {
    let content = &json["choices"][0]["message"]["content"];

    // Case 1: plain string (most models)
    if let Some(s) = content.as_str() {
        return Ok(s.to_string());
    }

    // Case 2: content array (multimodal Gemini responses)
    if let Some(arr) = content.as_array() {
        let text = arr
            .iter()
            .filter_map(|part| {
                if part["type"].as_str() == Some("text") {
                    part["text"].as_str().map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        if !text.is_empty() {
            return Ok(text);
        }
    }

    anyhow::bail!("Could not extract text content from LLM response")
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cd shadow_prompt
cargo test llm::tests 2>&1
```

Expected: all 4 tests PASS.

- [ ] **Step 5: Replace all fragile `.as_str().context(...)` response parsing**

In `src/llm.rs`, replace the response parsing in all five functions. Each currently looks like:

```rust
let content = json["choices"][0]["message"]["content"]
    .as_str()
    .context("Failed to parse ... response")?
    .to_string();
Ok(content)
```

Replace **every instance** of that pattern with:

```rust
Self::extract_content(&json)
```

The five functions are: `query_groq`, `query_openrouter`, `query_groq_with_image`, `query_openrouter_with_image`. (`query_ollama` and `query_ollama_with_image` use a different response field `json["response"]` — leave those unchanged.)

- [ ] **Step 6: Unify vision system prompt in all three vision functions**

In `query_groq_with_image`, `query_openrouter_with_image`, and `query_ollama_with_image`, find the hardcoded system prompt line:

```rust
let system_prompt = "You are a helpful assistant that analyzes images and answers questions about them. Be concise and accurate.";
```

Replace it in **all three** with:

```rust
let system_prompt = Self::load_system_prompt();
```

- [ ] **Step 7: Verify it compiles and tests still pass**

```bash
cd shadow_prompt
cargo test llm::tests 2>&1
cargo check 2>&1
```

Expected: all tests pass, no compile errors.

- [ ] **Step 8: Commit**

```bash
git add shadow_prompt/src/llm.rs
git commit -m "fix: add extract_content() helper for string/array LLM responses, unify vision system prompt"
```

---

## Task 4: Rewrite `system_prompt.txt`

**Files:**
- Modify: `shadow_prompt/config/system_prompt.txt`

- [ ] **Step 1: Replace the entire file contents**

Overwrite `shadow_prompt/config/system_prompt.txt` with exactly:

```
You are a stealth AI assistant. Answer questions directly. Zero filler text.

EVIDENCE (when present):
- [WEB SEARCH RESULTS]: Live web data. Highest priority source.
- [LOCAL KNOWLEDGE]: User reference documents. Use when relevant.
Reason from the evidence before answering. If sources conflict, prefer web results.
If no evidence is provided, use your own knowledge.

OUTPUT RULES:
- No preamble. No "The answer is", "Sure", "Here is", etc.
- No markdown formatting unless the question explicitly asks for it.
- No quotes around answers unless requested.
- Concise. Every word must earn its place.

QUESTION TYPE RULES (detect from context and apply exactly):

TRUE / FALSE
Output only: True  or  False

MULTIPLE CHOICE — single correct answer (options labeled A/B/C/D or 1/2/3/4):
Output: A) option text
Example: B) Nitrogen

MULTIPLE CHOICE — multiple correct answers ("select all", "choose all that apply", "which of the following are correct", "select all that apply"):
Output ALL correct options, comma-separated. Never stop at one answer if multiple are correct.
A) option text, C) option text, D) option text

UNLABELED OPTIONS (options appear as a plain list after the question, common in Google Forms):
Use web search results and your knowledge to identify which items are the options and group
multi-word concepts correctly (e.g., "Carbon Dioxide" is one option, not two separate words).
Answer by naming the correct option(s) directly.
If multiple options are correct, list all of them.
Example output: Nitrogen  or  Nitrogen, Oxygen

SHORT ANSWER / IDENTIFICATION:
Output the direct answer only. No explanation unless the question explicitly asks for one.

FILL IN THE BLANK:
Output only the missing word or phrase. Nothing else.

NUMERIC / CALCULATION:
Output the number and unit. Show work only if the question asks to show work.

CODE:
Output only valid, runnable code. No explanation unless asked.

ESSAY / EXPLANATION:
Write one concise, accurate paragraph. No bullet points unless the question asks for a list.

MATCHING:
List each pairing on its own line:
1 → B
2 → D
3 → A

RANKING / ORDERING:
List items in the correct order, numbered from first to last.

LIKERT / RATING SCALE:
Output the scale label only (e.g., Strongly Agree).

DROPDOWN / SELECT ONE:
Output the correct option text only.
```

- [ ] **Step 2: Commit**

```bash
git add shadow_prompt/config/system_prompt.txt
git commit -m "feat: rewrite system_prompt.txt — all question types, multi-answer, unlabeled MCQ"
```

---

## Task 5: Wrap RAG `embed()` calls in `spawn_blocking`

**Files:**
- Modify: `shadow_prompt/src/knowledge/rag.rs`

`TextEmbedding::embed()` is synchronous ONNX CPU inference. Calling it directly inside async Tokio tasks blocks a worker thread for 50–200ms per call. Fix: store the model behind `Arc` and move it into `spawn_blocking`.

- [ ] **Step 1: Update the import block**

At the top of `src/knowledge/rag.rs`, the imports currently include `use std::collections::HashMap;`. Add `Arc` to the std imports:

```rust
use std::collections::HashMap;
use std::sync::Arc;
```

- [ ] **Step 2: Change `embedding_model` field type in `RagSystem`**

Find the `RagSystem` struct definition and change:

```rust
// BEFORE:
pub struct RagSystem {
    embedding_model: Option<TextEmbedding>,

// AFTER:
pub struct RagSystem {
    embedding_model: Option<Arc<TextEmbedding>>,
```

- [ ] **Step 3: Update `RagSystem::new()` to wrap model in `Arc`**

Find the line in `new()` that constructs the tuple on model init success:

```rust
// BEFORE:
let (model, is_operational, init_error) = match TextEmbedding::try_new(options) {
    Ok(m) => (Some(m), true, None),

// AFTER:
let (model, is_operational, init_error) = match TextEmbedding::try_new(options) {
    Ok(m) => (Some(Arc::new(m)), true, None),
```

- [ ] **Step 4: Update `ingest()` to use `spawn_blocking`**

In the `ingest()` method, find the embedding call:

```rust
// BEFORE:
let embeddings = embedding_model.embed(texts, None)?;
```

Replace it with:

```rust
// AFTER:
let model_arc = Arc::clone(embedding_model);
let embeddings = tokio::task::spawn_blocking(move || {
    model_arc.embed(texts, None)
})
.await
.map_err(|e| anyhow::anyhow!("spawn_blocking join error: {}", e))??;
```

Note: `embedding_model` at this point is `&Arc<TextEmbedding>` (extracted via `match &self.embedding_model`). Adjust the match arm if needed so `embedding_model` is a `&Arc<TextEmbedding>` reference you can clone.

The full updated match block in `ingest()` should look like:

```rust
let embedding_model = match &self.embedding_model {
    Some(m) => m,
    None => return Ok(0),
};
```

This gives you `embedding_model: &Arc<TextEmbedding>`, which you can `Arc::clone`.

- [ ] **Step 5: Update `query()` to use `spawn_blocking`**

In the `query()` method, find the query embedding call:

```rust
// BEFORE:
let query_embeddings = embedding_model.embed(vec![text.to_string()], None)?;
let query_vec = &query_embeddings[0];
```

Replace it with:

```rust
// AFTER:
let model_arc = Arc::clone(embedding_model);
let text_owned = text.to_string();
let query_embeddings = tokio::task::spawn_blocking(move || {
    model_arc.embed(vec![text_owned], None)
})
.await
.map_err(|e| anyhow::anyhow!("spawn_blocking join error: {}", e))??;
let query_vec = &query_embeddings[0];
```

The `embedding_model` variable is already `&Arc<TextEmbedding>` from the match above in `query()`:

```rust
let embedding_model = match &self.embedding_model {
    Some(m) => m,
    None => return Ok(vec![]),
};
```

- [ ] **Step 6: Update the four unit tests that construct `RagSystem` directly**

The tests in `rag.rs` construct `RagSystem` with `embedding_model: None`. These still compile fine since `None` works for `Option<Arc<TextEmbedding>>` too. Verify:

```bash
cd shadow_prompt
cargo test knowledge::rag::tests 2>&1
```

Expected: all 4 existing tests PASS. If `TextEmbedding` does not implement `Send`, you will get a compile error like `TextEmbedding cannot be sent between threads safely` — if this happens, report back rather than attempting a workaround.

- [ ] **Step 7: Commit**

```bash
git add shadow_prompt/src/knowledge/rag.rs
git commit -m "perf: wrap TextEmbedding::embed() in spawn_blocking to avoid blocking Tokio workers"
```

---

## Task 6: Add `ContextBundle` and restructure `gather_context()`

**Files:**
- Modify: `shadow_prompt/src/knowledge/mod.rs`

This is the core pipeline change. `gather_context()` now always runs search then RAG and returns a typed bundle instead of a single string.

- [ ] **Step 1: Write failing tests for `clean_search_query()`**

Add a `#[cfg(test)]` block at the bottom of `src/knowledge/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::clean_search_query;

    #[test]
    fn test_strips_labeled_abcd_options() {
        let input = "What is the capital of France?\nA) London\nB) Paris\nC) Berlin\nD) Rome";
        let result = clean_search_query(input);
        assert_eq!(result.trim(), "What is the capital of France?");
    }

    #[test]
    fn test_strips_labeled_numbered_options() {
        let input = "What is 2+2?\n1) 3\n2) 4\n3) 5\n4) 6";
        let result = clean_search_query(input);
        assert_eq!(result.trim(), "What is 2+2?");
    }

    #[test]
    fn test_leaves_unlabeled_options_untouched() {
        let input = "Which gas makes up most of Earth's atmosphere?\nOxygen\nNitrogen\nCarbon Dioxide\nHydrogen";
        let result = clean_search_query(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_leaves_single_labeled_match_untouched() {
        // Only 1 match — could be part of question text, don't strip
        let input = "What happened in A) 1776 during the revolution?";
        let result = clean_search_query(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_plain_question_untouched() {
        let input = "What is the boiling point of water?";
        let result = clean_search_query(input);
        assert_eq!(result, input);
    }
}
```

- [ ] **Step 2: Run tests to confirm they fail**

```bash
cd shadow_prompt
cargo test knowledge::tests 2>&1
```

Expected: compile error — `clean_search_query` not found.

- [ ] **Step 3: Add `ContextBundle` struct and `clean_search_query()` function**

Replace the **entire** import/use block at the top of `src/knowledge/mod.rs` with the following. This also removes the now-unused `ModelCapabilities` import (Gap 5 from spec — the `model_has_search` skip it powered is gone; search always runs now):

```rust
pub mod search;
pub mod rag;

use anyhow::Result;
use crate::config::Config;
use std::sync::Arc;
use regex::Regex;

/// Structured context returned by gather_context().
/// Each field is empty string if that source had no results.
pub struct ContextBundle {
    pub web: String,
    pub local: String,
    pub warnings: Vec<String>,
}

/// Strip clearly labeled MCQ options (A) B) C) D) or 1) 2) 3) 4)) from a search query.
/// Only strips when at least 2 consecutive labeled options are detected.
/// Leaves unlabeled option lists untouched — we cannot reliably parse them.
pub(crate) fn clean_search_query(text: &str) -> String {
    let re = match Regex::new(r"(?m)^\s*[A-Da-d1-4][.)]\s+.+$") {
        Ok(r) => r,
        Err(_) => return text.to_string(),
    };
    let matches: Vec<_> = re.find_iter(text).collect();
    if matches.len() >= 2 {
        re.replace_all(text, "").trim().to_string()
    } else {
        text.to_string()
    }
}
```

- [ ] **Step 4: Run tests to confirm they pass**

```bash
cd shadow_prompt
cargo test knowledge::tests 2>&1
```

Expected: all 5 tests PASS.

- [ ] **Step 5: Rewrite `KnowledgeProvider` struct and `gather_context()`**

Replace the `KnowledgeProvider` struct and all its impl methods with:

```rust
pub struct KnowledgeProvider {
    rag: Option<Arc<rag::RagSystem>>,
}

impl KnowledgeProvider {
    pub async fn new(config: &Config) -> Result<Self> {
        let rag = if config.rag.enabled {
            println!("[*] Initializing Local RAG System...");
            let sys = rag::RagSystem::new(config).await;
            Some(Arc::new(sys))
        } else {
            None
        };

        let provider = Self { rag };

        if let Some(rag_sys) = &provider.rag {
            let rag_clone = rag_sys.clone();
            tokio::spawn(async move {
                if let Err(e) = rag_clone.ingest().await {
                    eprintln!("[!] RAG Ingestion Failed: {}", e);
                }
            });
        }

        Ok(provider)
    }

    /// Always runs: web search first, then RAG.
    /// Returns a ContextBundle with separate web and local fields.
    /// The LLM receives the full original query — only the search engine
    /// gets the cleaned query (MCQ options stripped for better results).
    pub async fn gather_context(&self, query: &str, config: &Config) -> Result<ContextBundle> {
        let mut bundle = ContextBundle {
            web: String::new(),
            local: String::new(),
            warnings: Vec::new(),
        };

        // 1. Web Search — always runs, cleaned query for better results
        let search_query = clean_search_query(query);
        match search::perform_search(&search_query, &config.search).await {
            Ok(results) if !results.is_empty() && results != "No search results found." => {
                bundle.web = results;
            }
            Ok(_) => {
                // Empty or "No results" — leave bundle.web empty, no warning
            }
            Err(e) => {
                let msg = format!("Search failed: {}", e);
                eprintln!("{}", msg);
                bundle.warnings.push(msg);
            }
        }

        // 2. RAG — always runs if initialized
        if let Some(rag) = &self.rag {
            match rag.query(query).await {
                Ok(results) if !results.is_empty() => {
                    let mut local = String::new();
                    for (i, doc) in results.iter().enumerate() {
                        local.push_str(&format!("[Document {}]: {}\n", i + 1, doc));
                    }
                    bundle.local = local;
                }
                Ok(_) => {
                    // No matching docs — leave bundle.local empty, no warning
                }
                Err(e) => {
                    let msg = format!("RAG Query Failed: {}", e);
                    eprintln!("[!] {}", msg);
                    bundle.warnings.push(msg);
                }
            }
        }

        Ok(bundle)
    }
}
```

- [ ] **Step 6: Verify it compiles**

```bash
cd shadow_prompt
cargo check 2>&1
```

Expected: compile errors in `main.rs` because it still calls the old `gather_context()` signature. That is expected — Task 7 fixes `main.rs`. The errors should only be in `main.rs`, not in `mod.rs` itself.

- [ ] **Step 7: Run the knowledge tests again to confirm they still pass**

```bash
cd shadow_prompt
cargo test knowledge::tests 2>&1
```

Expected: all 5 PASS.

- [ ] **Step 8: Commit**

```bash
git add shadow_prompt/src/knowledge/mod.rs
git commit -m "feat: add ContextBundle, always-search-then-RAG pipeline, clean_search_query()"
```

---

## Task 7: Update `main.rs` to consume `ContextBundle` and build labeled prompt sections

**Files:**
- Modify: `shadow_prompt/src/main.rs`

This task wires everything together: consume the new `ContextBundle`, assemble the labeled prompt, and update the OCR vision user prompt.

- [ ] **Step 1: Update the `InputEvent::Model` handler to use `ContextBundle`**

In `src/main.rs`, find the `InputEvent::Model` tokio task (around line 275). Replace the context-gathering and prompt-assembly block:

```rust
// FIND and REPLACE this block (lines ~290-305):
let (context, warnings) = match kp_arc.gather_context(&prompt, &config_clone).await {
     Ok((ctx, warns)) => (ctx, warns),
     Err(e) => {
         let err_msg = format!("Knowledge System Error: {}", e);
         error!("{}", err_msg);
         (String::new(), vec![err_msg])
     }
};

let augmented_prompt = if !context.is_empty() {
    info!("[*] Context found. Augmenting prompt.");
    format!("Context:\n{}\nQuestion:\n{}", context, prompt)
} else {
    prompt.clone()
};
```

Replace with:

```rust
let bundle = match kp_arc.gather_context(&prompt, &config_clone).await {
    Ok(b) => b,
    Err(e) => {
        let err_msg = format!("Knowledge System Error: {}", e);
        error!("{}", err_msg);
        crate::knowledge::ContextBundle {
            web: String::new(),
            local: String::new(),
            warnings: vec![err_msg],
        }
    }
};

// Build structured prompt with labeled sections
let mut augmented_prompt = String::new();

if !bundle.web.is_empty() {
    info!("[*] Web context found ({} chars)", bundle.web.len());
    augmented_prompt.push_str("[WEB SEARCH RESULTS]\n");
    augmented_prompt.push_str(&bundle.web);
    augmented_prompt.push_str("\n\n");
}

if !bundle.local.is_empty() {
    info!("[*] Local RAG context found ({} chars)", bundle.local.len());
    augmented_prompt.push_str("[LOCAL KNOWLEDGE]\n");
    augmented_prompt.push_str(&bundle.local);
    augmented_prompt.push_str("\n\n");
}

augmented_prompt.push_str("[QUESTION]\n");
augmented_prompt.push_str(&prompt);
```

- [ ] **Step 2: Update the warnings block**

Just below the augmented_prompt assembly, find the warnings loop:

```rust
// FIND:
let mut final_output = String::new();

// Add warnings to output if any
for warning in warnings {
    final_output.push_str(&format!("[System Warning: {}]\n\n", warning));
}
```

Replace `warnings` with `bundle.warnings`:

```rust
let mut final_output = String::new();

for warning in bundle.warnings {
    final_output.push_str(&format!("[System Warning: {}]\n\n", warning));
}
```

- [ ] **Step 3: Update the OCR vision user prompt**

Find the `InputEvent::OCRRect` handler (around line 186). Find:

```rust
let prompt = "Analyze the image. If there are questions, answer them directly and concisely. Provide all correct options if it is a multiple-choice question. If it is a matching or matrix question, clearly provide all pairings and answers.";
```

Replace with:

```rust
let prompt = "Read the question in this image exactly as written and answer it. \
              If the image contains a graph, chart, diagram, or table, interpret it \
              as part of the question context.";
```

- [ ] **Step 4: Verify full compilation**

```bash
cd shadow_prompt
cargo check 2>&1
```

Expected: no errors. If there are any remaining references to the old `gather_context()` return type `(String, Vec<String>)`, fix them now.

- [ ] **Step 5: Run all tests**

```bash
cd shadow_prompt
cargo test 2>&1
```

Expected: all tests pass (the 4 RAG tests + 5 knowledge tests + 4 LLM tests = 13 total).

- [ ] **Step 6: Commit**

```bash
git add shadow_prompt/src/main.rs
git commit -m "feat: wire ContextBundle into main event loop, labeled prompt sections, updated OCR prompt"
```

---

## Task 8: Final integration check

**Files:** (read-only verification)

- [ ] **Step 1: Run full test suite**

```bash
cd shadow_prompt
cargo test 2>&1
```

Expected: all tests pass.

- [ ] **Step 2: Run Clippy**

```bash
cd shadow_prompt
cargo clippy 2>&1
```

Fix any warnings introduced by this PR (unused imports, unnecessary clones, etc.). Do not fix pre-existing warnings unrelated to this work.

- [ ] **Step 3: Verify `system_prompt.txt` is loaded correctly at runtime**

Add a temporary debug print in `load_system_prompt()` to confirm the path resolves:

```rust
fn load_system_prompt() -> String {
    let path = crate::config::get_exe_dir()
        .join("config")
        .join("system_prompt.txt");
    println!("[*] Loading system prompt from: {:?}", path);  // temp debug
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| "You are a concise assistant.".to_string())
}
```

Run once in debug mode to confirm the path is correct, then **remove the debug print** before committing:

```bash
cd shadow_prompt
cargo run -- --debug
```

Expected log line: `[*] Loading system prompt from: "C:\\...\\config\\system_prompt.txt"` and the file should be found (no fallback to "You are a concise assistant.").

- [ ] **Step 4: Remove the debug print and commit final cleanup**

After confirming the path is correct, remove the `println!` from `load_system_prompt()`.

```bash
git add shadow_prompt/src/llm.rs
git commit -m "chore: remove debug print from load_system_prompt"
```

- [ ] **Step 5: Final commit summary**

```bash
git log --oneline -8
```

You should see commits for each task in this plan. The branch is ready for review.
