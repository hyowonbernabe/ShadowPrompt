# Unified Knowledge Pipeline Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make OCR Box and Google Forms methods use the same knowledge pipeline (Search + RAG) as the Clipboard method, with native search support for capable models.

**Architecture:** Route all input methods through KnowledgeProvider::gather_context() before querying LLM. Add native search capability detection to skip pre-search for models that have built-in search tools.

**Tech Stack:** Rust, Tokio async runtime, Serper/DuckDuckGo fallback, FastEmbed RAG

---

## File Structure

| File | Responsibility |
|------|----------------|
| `src/main.rs` | Fix OCRRect handler to use gather_context |
| `src/browser/mod.rs` | Pass KnowledgeProvider to execute_form_flow |
| `src/knowledge/mod.rs` | Add native search capability detection |
| `src/config/config.rs` | Add has_native_search config option |
| `config/system_prompt.txt` | Update to force search behavior |
| `config/config.example.toml` | Add has_native_search field |

---

## Chunk 1: Knowledge Provider - Native Search Support

**Files:**
- Modify: `shadow_prompt/src/knowledge/mod.rs:44-87`
- Modify: `shadow_prompt/src/config.rs` (check for model config field)

- [ ] **Step 1: Add native search capability detection to KnowledgeProvider**

In `knowledge/mod.rs`, add a method to determine if the current model has native search:

```rust
/// Check if the configured model should use native search instead of pre-search
pub fn should_use_native_search(&self, config: &Config) -> bool {
    // Models with native tool use / search capabilities
    // Check based on model ID patterns or config flag
    let provider = config.models.provider.as_str();
    let model_id = match provider {
        "groq" => config.models.groq.as_ref().and_then(|c| Some(&c.model_id)),
        "openrouter" => config.models.openrouter.as_ref().and_then(|c| Some(&c.model_id)),
        "ollama" => config.models.ollama.as_ref().and_then(|c| Some(&c.model_id)),
        _ => None,
    };
    
    // Pattern match for known native-search models
    // Groq's deepseek-r1 and certain models have tool use
    if let Some(m) = model_id {
        let m_lower = m.to_lowercase();
        // Known models with native search/tool capability
        return m_lower.contains("deepseek-r1") || 
               m_lower.contains("tool") || 
               config.models.force_native_search;
    }
    
    // Default: use pre-search fallback
    false
}
```

- [ ] **Step 2: Modify gather_context to skip pre-search for native models**

Update the `gather_context` method to conditionally skip Serper search:

```rust
pub async fn gather_context(&self, query: &str, config: &Config) -> Result<ContextBundle> {
    let mut bundle = ContextBundle {
        web: String::new(),
        local: String::new(),
        warnings: Vec::new(),
    };

    // 1. Check if model has native search - if so, skip pre-search
    // Let the model handle its own search based on system prompt instructions
    if !self.should_use_native_search(config) {
        // Use fallback search (Serper → DuckDuckGo)
        let search_query = clean_search_query(query);
        match search::perform_search(&search_query, &config.search).await {
            Ok(results) => {
                if !results.is_empty() {
                    bundle.web = results;
                }
            }
            Err(e) => {
                let msg = format!("Search failed: {}", e);
                eprintln!("{}", msg);
                bundle.warnings.push(msg);
            }
        }
    } else {
        // Native search model - add warning that model will search itself
        bundle.warnings.push("Model has native search - will search itself".to_string());
    }

    // 2. Local RAG — always runs
    if let Some(rag) = &self.rag {
        match rag.query(query).await {
            Ok(results) => {
                if !results.is_empty() {
                    let mut local = String::new();
                    for (i, doc) in results.iter().enumerate() {
                        local.push_str(&format!("[Document {}]: {}\n", i + 1, doc));
                    }
                    bundle.local = local;
                }
            },
            Err(e) => {
                let msg = format!("RAG Query Failed: {}", e);
                eprintln!("[!] {}", msg);
                bundle.warnings.push(msg);
            }
        }
    }

    Ok(bundle)
}
```

- [ ] **Step 3: Add force_native_search to Config struct**

In `config.rs`, find the model config section and add:

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    // ... existing fields ...
    
    /// Force native search even if model pattern doesn't match
    #[serde(default)]
    pub force_native_search: bool,
}
```

- [ ] **Step 4: Commit**

```bash
git add shadow_prompt/src/knowledge/mod.rs shadow_prompt/src/config.rs
git commit -m "feat: add native search capability detection to KnowledgeProvider"
```

---

## Chunk 2: Main.rs - OCR Rect Pipeline Fix

**Files:**
- Modify: `shadow_prompt/src/main.rs:186-280`

- [ ] **Step 1: Pass knowledge_provider to OCRRect handler**

In the `InputEvent::OCRRect` match arm (around line 186), add the knowledge provider clone:

```rust
let kp_arc = knowledge_provider.clone();  // Add this line
```

- [ ] **Step 2: After OCR extraction, call gather_context**

After line 249 where OCR text is extracted, add:

```rust
// 2. Gather Context (Search/RAG)
let bundle = match kp_arc.gather_context(&text, &config_clone).await {
    Ok(b) => b,
    Err(e) => {
        let err_msg = format!("Knowledge System Error: {}", e);
        error!("{}", err_msg);
        ContextBundle {
            web: String::new(),
            local: String::new(),
            warnings: vec![err_msg],
        }
    }
};

let mut augmented_prompt = String::new();
if !bundle.web.is_empty() {
    info!("[*] Web context found. Augmenting prompt.");
    augmented_prompt.push_str("[WEB SEARCH RESULTS]\n");
    augmented_prompt.push_str(&bundle.web);
    augmented_prompt.push_str("\n\n");
}
if !bundle.local.is_empty() {
    info!("[*] Local knowledge found. Augmenting prompt.");
    augmented_prompt.push_str("[LOCAL KNOWLEDGE]\n");
    augmented_prompt.push_str(&bundle.local);
    augmented_prompt.push_str("\n\n");
}
augmented_prompt.push_str("[QUESTION]\n");
augmented_prompt.push_str(&text);
```

- [ ] **Step 3: Use augmented_prompt in LlmClient::query**

Replace line 253 `LlmClient::query(&text, ...` with:

```rust
match LlmClient::query(&augmented_prompt, &config_clone, ModelUseCase::General).await {
```

- [ ] **Step 4: Do the same for the vision fallback path**

Around line 220 in the fallback section, apply the same gather_context call.

- [ ] **Step 5: Commit**

```bash
git add shadow_prompt/src/main.rs
git commit -m "feat: add knowledge pipeline to OCR Box method"
```

---

## Chunk 3: Browser Module - Add Knowledge Pipeline

**Files:**
- Modify: `shadow_prompt/src/browser/mod.rs:50-272`
- Modify: `shadow_prompt/src/main.rs:412-443` (caller site)

- [ ] **Step 1: Update execute_form_flow signature**

Add knowledge_provider parameter:

```rust
pub async fn execute_form_flow(
    url: Option<&str>,
    _password: Option<&str>,
    config: Arc<Config>,
    ui_tx: Sender<UICommand>,
    is_auto: bool,
    knowledge_provider: Arc<KnowledgeProvider>,  // NEW PARAM
) -> Result<()> {
```

- [ ] **Step 2: After JSON extraction, gather context**

After line 182 where `form_json` is extracted, add:

```rust
// Extract a summary of questions for search/RAG
let questions_summary = extract_questions_for_context(form_json);

// Gather knowledge context
let bundle = match knowledge_provider.gather_context(&questions_summary, &config).await {
    Ok(b) => b,
    Err(e) => {
        let msg = format!("Knowledge System Error: {}", e);
        eprintln!("[!] {}", msg);
        ContextBundle {
            web: String::new(),
            local: String::new(),
            warnings: vec![msg],
        }
    }
};

// Build context string for prompt injection
let mut context_section = String::new();
if !bundle.web.is_empty() {
    context_section.push_str("[WEB SEARCH RESULTS]\n");
    context_section.push_str(&bundle.web);
    context_section.push_str("\n\n");
}
if !bundle.local.is_empty() {
    context_section.push_str("[LOCAL KNOWLEDGE]\n");
    context_section.push_str(&bundle.local);
    context_section.push_str("\n\n");
}
```

- [ ] **Step 3: Inject context into the prompt**

Around line 193-217 where the prompt is built, insert the context at the top:

```rust
let prompt = format!(
    "CONTEXT (use this information to answer questions more accurately):
    {}

    ---

    You are an automated quiz solver filling out a Google Form.
    The JSON contains `questions` and `navigation` buttons. Answer every unanswered question.

    QUESTION TYPES — use exactly these action formats:
    // ... rest of prompt ...",
    context_section = context_section,
    // ... rest of format args ...
);
```

- [ ] **Step 4: Add helper function for question extraction**

Add this function at the bottom of browser/mod.rs:

```rust
fn extract_questions_for_context(form_json: &str) -> String {
    // Parse the JSON and extract question text for search/RAG
    // This helps find relevant context for the form questions
    let mut questions = Vec::new();
    
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(form_json) {
        if let Some(arr) = json.get("questions").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(question) = item.get("question").and_then(|v| v.as_str()) {
                    questions.push(question.to_string());
                }
            }
        }
    }
    
    // Join questions into a search-friendly string
    questions.join(" ")
}
```

- [ ] **Step 5: Update main.rs caller to pass knowledge_provider**

In main.rs around line 432 where `execute_form_flow` is called:

```rust
active_browser_task = Some(tokio::spawn(async move {
    if let Err(e) = crate::browser::execute_form_flow(
        url.as_deref(), 
        p_clone.as_deref(), 
        c_clone.clone(), 
        tx_clone.clone(), 
        is_auto,
        knowledge_provider.clone(),  // NEW ARG
    ).await {
```

- [ ] **Step 6: Commit**

```bash
git add shadow_prompt/src/browser/mod.rs shadow_prompt/src/main.rs
git commit -m "feat: add knowledge pipeline to Google Forms method"
```

---

## Chunk 4: System Prompt - Force Search Behavior

**Files:**
- Modify: `shadow_prompt/config/system_prompt.txt`

- [ ] **Step 1: Update system prompt to enforce search**

Add explicit instructions to search even for easy questions:

```
IMPORTANT - SEARCH BEHAVIOR:
- ALWAYS search the web for relevant information before answering ANY question
- Even for questions that seem simple or obvious, perform a search to verify your answer
- If your model has native search capability, use it to find current/relevant information
- If you do not have native search, rely on the provided [WEB SEARCH RESULTS] context
- Use any [LOCAL KNOWLEDGE] notes available for this topic
- Search is critical for: factual questions, current events, technical answers, math/science problems
```

- [ ] **Step 2: Commit**

```bash
git add shadow_prompt/config/system_prompt.txt
git commit -m "chore: update system prompt to force search behavior"
```

---

## Chunk 5: Config Example - Add Native Search Field

**Files:**
- Modify: `shadow_prompt/config/config.example.toml`

- [ ] **Step 1: Add has_native_search example**

Add under model section:

```toml
[models.groq]
model_id = "deepseek-r1-distill-llama-70b"
# Models like deepseek-r1 have native search/tool capability
# When enabled, skip pre-search and let model search itself
# has_native_search = true
```

- [ ] **Step 2: Commit**

```bash
git add shadow_prompt/config/config.example.toml
git commit -m "docs: add native search config example"
```

---

## Chunk 6: Verification

**Files:**
- Run: `cargo build` in shadow_prompt/
- Run: `cargo test` in shadow_prompt/

- [ ] **Step 1: Build the project**

```bash
cd shadow_prompt && cargo build
```

- [ ] **Step 2: Run tests**

```bash
cd shadow_prompt && cargo test
```

- [ ] **Step 3: Commit final changes**

```bash
git add -A && git commit -m "feat: unified knowledge pipeline for all input methods"
```

---

## Success Criteria

- [ ] OCR Box method logs show "Web context found" / "Local knowledge found"
- [ ] Google Forms logs show search/RAG context being gathered
- [ ] Clipboard method unchanged (existing behavior preserved)
- [ ] Native search models skip pre-search
- [ ] All three methods follow identical pipeline: system → search → rag → llm
- [ ] Build passes without errors
- [ ] Tests pass