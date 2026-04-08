# Question Type Handling — Design Spec
**Date:** 2026-04-09  
**Status:** Approved  
**Scope:** Clipboard/OCR path (text and vision). Browser automation path is out of scope — it works correctly.

---

## Problem Statement

The clipboard/OCR path has three compounding problems:

1. **Multi-answer failure** — the LLM gives only one answer to "select all that apply" questions because the system prompt rule is not explicit enough.
2. **No structured evidence pipeline** — web search and RAG are config-gated, run in the wrong order, and merge their output into one undifferentiated blob the LLM cannot distinguish.
3. **Accumulated gaps** — silent system prompt loading failure, fragile vision response parsing, dead config fields, blocking async calls, and poor search query quality for MCQ questions.

---

## Approved Approach: Structured Pipeline + Rewritten System Prompt

Always run **Search → RAG → LLM**, return results as labeled context sections, and rewrite `system_prompt.txt` to cover every question type including unlabeled MCQ.

---

## Section 1: Pipeline Architecture

### 1.1 New Return Type for `gather_context()`

**File:** `src/knowledge/mod.rs`

Replace the current single combined `String` return with a typed struct:

```rust
pub struct ContextBundle {
    pub web: String,     // Web search results (may be empty if search failed)
    pub local: String,   // RAG results (may be empty if no matches)
    pub warnings: Vec<String>,
}
```

Signature change:
```rust
// Before
pub async fn gather_context(&self, query: &str, config: &Config) -> Result<(String, Vec<String>)>

// After
pub async fn gather_context(&self, query: &str, config: &Config) -> Result<ContextBundle>
```

### 1.2 Search Always Runs First

Remove both soft-skip conditions in `gather_context()`:

```rust
// REMOVE: config.search.enabled gate
// REMOVE: !model_has_search gate (Gap 5)
```

Search always attempts Serper → DuckDuckGo fallback. If both fail, `ContextBundle.web` is empty and a warning is recorded. The LLM call still proceeds.

**Rationale for removing `model_has_search` skip (Gap 5):** This flag is a manual config value. Groq and OpenRouter models used in this app do not have built-in internet access. Skipping external search based on a flag that should never be true silently degrades answer quality.

### 1.3 Search Query Cleaning (Gap 2 — scoped)

Before sending the query to the search engine, strip **labeled** MCQ options to improve result relevance. Only strip when options are clearly labeled — leave unlabeled text untouched (see Gap 6).

```rust
fn clean_search_query(text: &str) -> String {
    // Match labeled options: "A) ...", "A. ...", "1) ...", "1. ..."
    // Only strip when at least 2 consecutive labeled options are found
    let re = Regex::new(r"(?m)^\s*[A-Da-d1-4][.)]\s+.+$").unwrap();
    let matches: Vec<_> = re.find_iter(text).collect();
    if matches.len() >= 2 {
        re.replace_all(text, "").trim().to_string()
    } else {
        text.trim().to_string()
    }
}
```

The LLM always receives the **full original text** including options. Stripping only applies to the search query string.

### 1.4 RAG Always Runs Second

Remove the `config.rag.enabled` gate from `query()`. RAG runs if the system is initialized (i.e., the embedding model loaded successfully). If the knowledge folder is empty or no documents score above `min_score`, `ContextBundle.local` is empty — no warnings, just silently omitted from the prompt.

### 1.5 Structured Prompt Assembly in `main.rs`

Replace the current flat `"Context:\n{}\nQuestion:\n{}"` format with labeled sections. Only non-empty sections are included:

```
[WEB SEARCH RESULTS]
- Nitrogen makes up approximately 78% of Earth's atmosphere...
- ...

[LOCAL KNOWLEDGE]
[Document 1]: Atmospheric composition notes...

[QUESTION]
Which gas makes up most of Earth's atmosphere?
Oxygen
Nitrogen
Carbon Dioxide
Hydrogen
```

Assembly logic:
```rust
let mut augmented = String::new();

if !bundle.web.is_empty() {
    augmented.push_str("[WEB SEARCH RESULTS]\n");
    augmented.push_str(&bundle.web);
    augmented.push_str("\n\n");
}

if !bundle.local.is_empty() {
    augmented.push_str("[LOCAL KNOWLEDGE]\n");
    augmented.push_str(&bundle.local);
    augmented.push_str("\n\n");
}

augmented.push_str("[QUESTION]\n");
augmented.push_str(&prompt);
```

---

## Section 2: System Prompt Rewrite

**File:** `config/system_prompt.txt`

Full replacement:

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

**Key fixes over the previous version:**
- Explicit "never stop at one answer" rule for multi-answer MCQ
- Dedicated rule for unlabeled option lists with multi-word grouping instruction
- References `[WEB SEARCH RESULTS]` and `[LOCAL KNOWLEDGE]` sections by name
- Covers all question types including Likert, matching, ranking, dropdown

---

## Section 3: Vision Path Alignment

### 3.1 Unified System Prompt

**File:** `src/llm.rs`

All three vision query functions (`query_groq_with_image`, `query_openrouter_with_image`, `query_ollama_with_image`) currently use a hardcoded inline system prompt:

```rust
// REMOVE in all three functions:
let system_prompt = "You are a helpful assistant that analyzes images and answers questions about them. Be concise and accurate.";

// REPLACE with:
let system_prompt = Self::load_system_prompt();
```

### 3.2 Inline Vision User Prompt

**File:** `src/main.rs` — `InputEvent::OCRRect` handler

```rust
// REPLACE:
let prompt = "Analyze the image. If there are questions, answer them directly and concisely. ...";

// WITH:
let prompt = "Read the question in this image exactly as written and answer it. \
              If the image contains a graph, chart, diagram, or table, interpret it \
              as part of the question context.";
```

### 3.3 Fragile Vision Response Parsing (Gap 7)

**File:** `src/llm.rs`

The current parser assumes `content` is always a plain string:
```rust
let content = json["choices"][0]["message"]["content"]
    .as_str()
    .context("Failed to parse ... vision response")?
    .to_string();
```

Newer Gemini multimodal models via OpenRouter can return `content` as an array of objects. Fix with a helper that handles both:

```rust
fn extract_content(json: &Value) -> Result<String> {
    let message = &json["choices"][0]["message"]["content"];
    
    // Case 1: plain string (most models)
    if let Some(s) = message.as_str() {
        return Ok(s.to_string());
    }
    
    // Case 2: content array (multimodal response)
    if let Some(arr) = message.as_array() {
        let text = arr.iter()
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

Apply `extract_content(&json)?` in place of all `.as_str().context(...)` calls in vision query functions. Apply the same fix to the non-vision `query_groq`, `query_openrouter` functions for consistency.

---

## Section 4: Bug Fixes & Code Quality

### Gap 1 — `load_system_prompt()` not exe-relative

**File:** `src/llm.rs`

```rust
// REPLACE:
fn load_system_prompt() -> String {
    std::fs::read_to_string("config/system_prompt.txt")
        .or_else(|_| std::fs::read_to_string("../config/system_prompt.txt"))
        .unwrap_or_else(|_| "You are a concise assistant.".to_string())
}

// WITH:
fn load_system_prompt() -> String {
    let path = crate::config::get_exe_dir().join("config").join("system_prompt.txt");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|_| "You are a concise assistant.".to_string())
}
```

This is the highest-priority fix — the entire system prompt has been silently failing on USB deployments.

### Gap 3 — Dead `use_rag` field

**File:** `src/config.rs`

Remove `use_rag: bool` from `GeneralConfig` struct and its `Default` impl. It is never read in the codebase; `config.rag.enabled` is the actual gate.

### Gap 4 — RAG `embed()` blocks Tokio thread pool

**File:** `src/knowledge/rag.rs`

Both `ingest()` and `query()` call `embedding_model.embed(...)` (synchronous ONNX CPU inference) directly on async Tokio tasks. Wrap in `spawn_blocking`:

```rust
// In query():
let query_embeddings = tokio::task::spawn_blocking(move || {
    embedding_model_ref.embed(vec![text.to_string()], None)
}).await??;

// In ingest():
let embeddings = tokio::task::spawn_blocking(move || {
    embedding_model_ref.embed(texts, None)
}).await??;
```

Note: `TextEmbedding` is not `Send`, so it cannot be moved into `spawn_blocking` directly. The implementation should store it as `Arc<Mutex<TextEmbedding>>` in `RagSystem`, then lock inside the closure: `let m = model.lock().unwrap(); m.embed(...)`.

---

## Section 5: Vision Capability Notes (Informational)

- **Windows.Media.Ocr** (text-only fallback): extracts text from pixels only. Graphs, charts, diagrams, and images are invisible to it.
- **Vision LLM path** (when `supports_vision: true`): the full PNG screenshot is sent to the model. Gemini Flash via OpenRouter can understand graphs, charts, tables, handwriting, and diagrams natively.
- **Recommendation**: for Google Forms or any visually rich content, the OCR screen capture path with a vision-capable model (Gemini Flash) is significantly more reliable than the clipboard text path. The clipboard path cannot recover structure lost during text copy.

---

## Files Changed Summary

| File | Change |
|---|---|
| `src/knowledge/mod.rs` | New `ContextBundle` struct; always run search + RAG; search query cleaning |
| `src/main.rs` | Consume `ContextBundle`; build labeled prompt sections; update OCR vision prompt |
| `src/llm.rs` | Fix `load_system_prompt()` to exe-relative; unified system prompt for vision; `extract_content()` helper; apply to all query functions |
| `src/knowledge/rag.rs` | Wrap `embed()` calls in `spawn_blocking` |
| `src/config.rs` | Remove dead `use_rag` field from `GeneralConfig` |
| `config/system_prompt.txt` | Full rewrite covering all question types |

---

## What Is Explicitly Out of Scope

- Browser automation path — working correctly, no changes
- Two-stage LLM (research agent → answer agent) — rejected for latency reasons
- MCQ pixel indicator colors — planned separately, not part of this spec
- RAG chunking via `text-splitter` — separate concern
- OAuth flow — unrelated
