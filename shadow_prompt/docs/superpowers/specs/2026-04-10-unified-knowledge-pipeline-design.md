# Unified Knowledge Pipeline Design
**Date:** 2026-04-10  
**Status:** Draft  

## Problem Statement

Three input methods exist in ShadowPrompt, but only one (Clipboard) uses the full knowledge pipeline:

1. **Clipboard Method** (Ctrl+Shift+V) - ✅ Uses gather_context() → Search + RAG
2. **OCR Box Method** (Wake Key + Drag) - ❌ Direct LLM query, no Search/RAG
3. **Google Forms Method** (Browser Exec) - ❌ Direct LLM query, no Search/RAG

Additionally, the current implementation relies solely on Serper/DuckDuckGo for web search. Models with native search capabilities (like Groq's `deepseek-r1-distill-llama-70b` with built-in tool use) should use their native tools first.

## Goals

1. All three methods must use the same knowledge pipeline (system prompt → search → RAG → LLM)
2. Models with native search tools should use them first before falling back to Serper/DuckDuckGo
3. Force explicit search usage even for seemingly easy questions
4. Unified approach, different execution methods

---

## Architecture

### Current Flow (Clipboard - Working)

```
InputEvent::Model
    ↓
ClipboardManager::read() → prompt
    ↓
KnowledgeProvider::gather_context(prompt)
    ├── search::perform_search(prompt) → Serper/DuckDuckGo
    └── rag.query(prompt) → Local embeddings
    ↓
Augmented prompt: [WEB SEARCH] + [LOCAL KNOWLEDGE] + [QUESTION]
    ↓
LlmClient::query(augmented_prompt)
    ↓
ClipboardManager::write(response)
```

### Target Flow (All Methods)

```
┌─────────────────────────────────────────────────────────────────┐
│                    UNIFIED PIPELINE                             │
├─────────────────────────────────────────────────────────────────┤
│  Input (Any Method)                                             │
│    ├── Clipboard text                                          │
│    ├── OCR extracted text                                      │
│    └── Google Form questions (extracted)                       │
│                          ↓                                      │
│  1. Check Model Capabilities                                    │
│     ├── Model has native search? → Use model's tool            │
│     └── No native search → Use Serper → DuckDuckGo fallback   │
│                          ↓                                      │
│  2. knowledge::gather_context(input)                          │
│     ├── Web Search (via capability-appropriate method)         │
│     └── RAG Query (local knowledge)                            │
│                          ↓                                      │
│  3. Build Augmented Prompt                                      │
│     [SYSTEM PROMPT]                                            │
│     [WEB SEARCH RESULTS] (if any)                             │
│     [LOCAL KNOWLEDGE] (if any)                                 │
│     [USER QUESTION]                                           │
│                          ↓                                      │
│  4. LlmClient::query()                                         │
│                          ↓                                      │
│  Output (Method-specific)                                      │
│     ├── Clipboard: write + overlay                             │
│     ├── OCR Box: write + overlay                               │
│     └── Forms: inject actions into browser                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Component Changes

### 1. knowledge/mod.rs - Add Native Search Support

Add method to check model capabilities and route search accordingly:

```rust
impl KnowledgeProvider {
    /// Decide whether to use native model search or fallback search
    fn should_use_native_search(&self, config: &Config) -> bool {
        // Check if model supports tool use / native search
        // Based on model ID patterns or config flag
    }
    
    /// Perform search using appropriate method
    pub async fn perform_search(&self, query: &str, config: &Config) -> Result<String> {
        if self.should_use_native_search(config) {
            // Let model handle search via its native capabilities
            // The model will be instructed to search in system prompt
            Ok(String::new()) // Skip pre-search, let LLM do it
        } else {
            // Use Serper → DuckDuckGo fallback
            search::perform_search(query, &config.search).await
        }
    }
}
```

### 2. knowledge/search.rs - Keep Existing Fallback

The existing Serper/DuckDuckGo implementation stays as the fallback for models without native search.

### 3. system_prompt.txt - Force Search Behavior

Update system prompt to explicitly instruct model to search:

```
INSTRUCTIONS:
- ALWAYS search the web for relevant information before answering
- Even for simple questions, perform a quick search to verify your answer
- Use your built-in search capability if available
- If no native search, rely on the provided web context (which was pre-searched)
- Use local knowledge base notes if available for this topic
```

### 4. main.rs - Fix OCR Rect Pipeline

Update `InputEvent::OCRRect` handler (lines 186-280) to use gather_context:

```rust
InputEvent::OCRRect(x, y, w, h) => {
    // ... existing OCR capture code ...
    
    // NEW: Add knowledge pipeline
    let kp_arc = knowledge_provider.clone();
    let bundle = kp_arc.gather_context(&text, &config_clone).await?;
    
    // Build augmented prompt (same as clipboard method)
    let augmented = build_augmented_prompt(&text, bundle);
    
    // Query LLM with augmented prompt
    let response = LlmClient::query(&augmented, &config_clone, ModelUseCase::General).await?;
    
    // ... existing output handling ...
}
```

### 5. browser/mod.rs - Add Knowledge Pipeline

Pass `Arc<KnowledgeProvider>` to `execute_form_flow()` and use it:

```rust
pub async fn execute_form_flow(
    url: Option<&str>,
    _password: Option<&str>,
    config: Arc<Config>,
    ui_tx: Sender<UICommand>,
    is_auto: bool,
    knowledge_provider: Arc<KnowledgeProvider>,  // NEW PARAM
) -> Result<()> {
    // ... existing extraction code ...
    
    // Extract questions from form_json for search/RAG
    let questions_summary = extract_questions_summary(form_json);
    
    // Get knowledge context
    let bundle = knowledge_provider.gather_context(&questions_summary, &config).await?;
    
    // Inject context into the instruction prompt
    let prompt = format!(
        "CONTEXT PROVIDED (use this to inform answers):
        Web Search: {}
        Local Knowledge: {}
        
        [Existing prompt...]",
        bundle.web, bundle.local
    );
    
    // ... rest of execution ...
}
```

---

## Configuration Changes

### config.rs - Add native search flag

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelConfig {
    // ... existing fields ...
    
    /// Model has native search/tool capability
    #[serde(default)]
    pub has_native_search: bool,
}
```

---

## Testing Plan

1. **Clipboard Method** - Verify existing behavior unchanged
2. **OCR Box Method** - Verify search+RAG now used (check logs for "Web context found", "Local knowledge found")
3. **Google Forms** - Verify search+RAG now used (check logs during form execution)
4. **Native Search Models** - If configured, verify model performs its own search
5. **Fallback Models** - Verify Serper search still works when native not available

---

## Success Criteria

- [ ] OCR Box method shows "Web context found" / "Local knowledge found" in logs
- [ ] Google Forms shows search/RAG context being gathered
- [ ] Both methods produce same quality answers as clipboard method
- [ ] Models with native search skip pre-search (or pre-search provides backup)
- [ ] All three methods follow identical pipeline: system → search → rag → llm