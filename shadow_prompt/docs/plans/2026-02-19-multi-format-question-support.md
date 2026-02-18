# Multi-Format Question Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add support for True/False and Identification question formats, improve answer display with text overlay, fix Google Forms copy issue via LLM prompt, and add hide graphics hotkey.

**Architecture:** Extend QuestionType enum in utils.rs, modify LLM prompts for type detection, add text overlay window in ui.rs, add hide toggle in input.rs, update config.rs with new fields.

**Tech Stack:** Rust, Win32 API for UI, toml config

---

## Task 1: Add QuestionType Enum and Parsing

**Files:**
- Modify: `.worktrees/multi-format-support/shadow_prompt/src/utils.rs`

**Step 1: Add QuestionType enum and helper functions**

Add after line 8 (after McqAnswer enum):

```rust
#[derive(Debug, PartialEq, Clone)]
pub enum QuestionType {
    MultipleChoice(McqAnswer),
    TrueFalse(bool),
    Identification(String),
    Unknown,
}
```

**Step 2: Add parse_question_type function**

Add at end of utils.rs (before final closing brace or after parse_mcq_with_context):

```rust
pub fn parse_question_type(input: &str, output: &str) -> QuestionType {
    let output_lower = output.to_lowercase();
    
    // Check for TYPE: prefix from LLM
    if let Some(start) = output_lower.find("type:") {
        let remainder = &output_lower[start..];
        if remainder.starts_with("type:mcq") {
            if let Some(ans) = parse_mcq_answer(output) {
                return QuestionType::MultipleChoice(ans);
            }
        } else if remainder.starts_with("type:tf") {
            if remainder.contains("answer:true") || remainder.contains("answer: t") {
                return QuestionType::TrueFalse(true);
            } else if remainder.contains("answer:false") || remainder.contains("answer: f") {
                return QuestionType::TrueFalse(false);
            }
        } else if remainder.starts_with("type:id") {
            if let Some(ans_start) = remainder.find("answer:") {
                let answer_text = remainder[ans_start + 7..].trim().to_string();
                if !answer_text.is_empty() {
                    return QuestionType::Identification(answer_text);
                }
            }
        }
    }
    
    // Fallback: Try existing MCQ parsing with context
    if let Some(ans) = parse_mcq_with_context(input, output) {
        return QuestionType::MultipleChoice(ans);
    }
    
    // Fallback: Try simple True/False detection
    let output_trimmed = output.trim().to_lowercase();
    if output_trimmed == "true" || output_trimmed == "t" || output_trimmed == "false" || output_trimmed == "f" {
        return QuestionType::TrueFalse(output_trimmed == "true" || output_trimmed == "t");
    }
    
    // Fallback: If there's a substantial answer text, treat as Identification
    let answer_only = output.trim();
    if answer_only.len() > 0 && answer_only.len() < 100 {
        return QuestionType::Identification(answer_only.to_string());
    }
    
    QuestionType::Unknown
}

pub fn question_type_to_display_text(qt: &QuestionType, input: &str) -> String {
    match qt {
        QuestionType::MultipleChoice(ans) => {
            // Try to get the answer text from choices
            let choices = extract_mcq_choices(input);
            for (letter, value) in &choices {
                if *letter == ans.clone() {
                    return format!("{:?}: {}", ans, value);
                }
            }
            format!("{:?}", ans)
        }
        QuestionType::TrueFalse(true) => "True".to_string(),
        QuestionType::TrueFalse(false) => "False".to_string(),
        QuestionType::Identification(text) => text.clone(),
        QuestionType::Unknown => String::new(),
    }
}
```

**Step 3: Add tests**

Add to tests module in utils.rs:

```rust
#[test]
fn test_question_type_mcq() {
    let result = parse_question_type("What is 2+2? A. 4 B. 3", "TYPE:MCQ ANSWER:A");
    assert!(matches!(result, QuestionType::MultipleChoice(McqAnswer::A)));
}

#[test]
fn test_question_type_true_false() {
    let result = parse_question_type("True or False: The sky is blue.", "TYPE:TF ANSWER:TRUE");
    assert!(matches!(result, QuestionType::TrueFalse(true)));
}

#[test]
fn test_question_type_identification() {
    let result = parse_question_type("What is the capital of France?", "TYPE:ID ANSWER:Paris");
    assert!(matches!(result, QuestionType::Identification(x) if x == "paris"));
}
```

**Step 4: Run tests**

```bash
cd .worktrees/multi-format-support/shadow_prompt && cargo test
```

Expected: Tests pass

**Step 5: Commit**

```bash
cd .worktrees/multi-format-support && git add shadow_prompt/src/utils.rs && git commit -m "feat: add QuestionType enum and parsing"
```

---

## Task 2: Update Config with New Fields

**Files:**
- Modify: `.worktrees/multi-format-support/shadow_prompt/src/config.rs`

**Step 1: Add new config fields to VisualsConfig struct**

Add after line 109 (after cursor_change field):

```rust
#[serde(default = "default_color_true")]
pub color_true: String,

#[serde(default = "default_color_false")]
pub color_false: String,

#[serde(default = "default_true")]
pub text_overlay_enabled: bool,

#[serde(default = "default_position")]
pub text_overlay_position: String,

#[serde(default = "default_text_size")]
pub text_overlay_font_size: i32,

#[serde(default = "default_text_opacity")]
pub text_overlay_opacity: u8,

#[serde(default = "default_hide_key")]
pub hide_key: String,
```

**Step 2: Add default functions**

Add after line 156 (after default_processing function):

```rust
fn default_color_true() -> String {
    "#00FF00".to_string()  // Lime Green
}

fn default_color_false() -> String {
    "#800000".to_string()  // Maroon
}

fn default_text_size() -> i32 {
    12
}

fn default_text_opacity() -> u8 {
    200
}

fn default_hide_key() -> String {
    "Ctrl+Shift+H".to_string()
}
```

**Step 3: Update Default impl**

Update the Default impl for VisualsConfig to include new fields:

```rust
impl Default for VisualsConfig {
    fn default() -> Self {
        Self {
            indicator_color: "#FF0000".to_string(),
            ready_color: "#00FF00".to_string(),
            position: default_position(),
            size: default_size(),
            offset: 0,
            x_axis: 0,
            y_axis: 0,
            color_mcq_a: default_mcq_a(),
            color_mcq_b: default_mcq_b(),
            color_mcq_c: default_mcq_c(),
            color_mcq_d: default_mcq_d(),
            color_mcq_none: default_mcq_none(),
            color_processing: default_processing(),
            cursor_change: false,
            color_true: default_color_true(),
            color_false: default_color_false(),
            text_overlay_enabled: true,
            text_overlay_position: default_position(),
            text_overlay_font_size: default_text_size(),
            text_overlay_opacity: default_text_opacity(),
            hide_key: default_hide_key(),
        }
    }
}
```

**Step 4: Build to verify**

```bash
cd .worktrees/multi-format-support/shadow_prompt && cargo check
```

Expected: Compiles successfully

**Step 5: Commit**

```bash
cd .worktrees/multi-format-support && git add shadow_prompt/src/config.rs && git commit -m "feat: add new config fields for question types and text overlay"
```

---

## Task 3: Update Input Events for Hide Toggle

**Files:**
- Modify: `.worktrees/multi-format-support/shadow_prompt/src/input.rs`

**Step 1: Add Hide event to InputEvent enum**

Read input.rs first to find the enum, then add:

```rust
#[derive(Debug, Clone)]
pub enum InputEvent {
    Wake,
    OCRClick1,
    OCRRect(i32, i32, i32, i32),
    Model,
    Panic,
    HideToggle,  // New event
}
```

**Step 2: Add hide key handling in InputManager::start**

Need to parse hide_key from config and add it to the key handler. Read input.rs to see how other keys are handled, then add:

- Parse hide_keys similar to wake/model/panic keys
- Add HideToggle event when hide key is pressed

**Step 3: Commit**

```bash
cd .worktrees/multi-format-support && git add shadow_prompt/src/input.rs && git commit -m "feat: add hide toggle input event"
```

---

## Task 4: Update UI Manager for Text Overlay and Visibility

**Files:**
- Modify: `.worktrees/multi-format-support/shadow_prompt/src/ui.rs`

**Step 1: Add new UICommand variants**

Update UICommand enum:

```rust
pub enum UICommand {
    SetColor(u32), 
    SetSecondaryColor(u32),
    DrawDebugRect(i32, i32, i32, i32),
    ClearDebugRect,
    SetText(String),        // New: Show text at bottom right
    ClearText,               // New: Clear text display
    SetVisibility(bool),     // New: Show/hide all graphics
    Quit,
}
```

**Step 2: Add visibility and text state**

Add after CURRENT_COLOR and SECONDARY_COLOR:

```rust
static mut VISIBLE: bool = true;
static mut CURRENT_TEXT: String = String::new();
```

**Step 3: Add text overlay window creation**

In UIManager::start, after creating debug window, add text overlay window creation:

```rust
// Text Overlay Window Class
let text_class_name = w!("ShadowPromptTextOverlay");
let wc_text = WNDCLASSW {
    hCursor: HCURSOR::default(),
    hIcon: HICON::default(),
    lpszClassName: text_class_name,
    hInstance: instance,
    lpfnWndProc: Some(text_wnd_proc),
    ..Default::default()
};
RegisterClassW(&wc_text);

// Calculate text overlay position
let text_x = screen_w - 200 - offset;  // bottom-right, 200px wide
let text_y = screen_h - 30 - offset;   // above taskbar

let hwnd_text = CreateWindowExW(
    WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
    text_class_name,
    w!(""),
    WS_POPUP | WS_VISIBLE,
    text_x, text_y, 200, 30,
    HWND::default(),
    HMENU::default(),
    instance,
    None,
).unwrap_or(HWND::default());
```

**Step 4: Add text window procedure**

Add after debug_wnd_proc:

```rust
unsafe extern "system" fn text_wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            
            // Background
            let bg_brush = CreateSolidBrush(COLORREF(0x00202020));  // Semi-transparent dark
            FillRect(hdc, &ps.rcPaint, bg_brush);
            let _ = DeleteObject(bg_brush);
            
            // Text
            let color = COLORREF(0x00FFFFFF);  // White
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, color);
            
            let text = CURRENT_TEXT.clone();
            if !text.is_empty() {
                use std::ptr::null_mut;
                let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
                DrawTextW(hdc, wide.as_ptr(), -1, &mut std::mem::zeroed(), DT_LEFT | DT_VCENTER | DT_SINGLELINE);
            }
            
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
```

**Step 5: Handle new commands in loop**

Update the command handling:

```rust
if let Ok(cmd) = rx.try_recv() {
    match cmd {
        // ... existing ...
        UICommand::SetText(text) => {
            CURRENT_TEXT = text;
            let _ = InvalidateRect(hwnd_text, None, false);
            let _ = ShowWindow(hwnd_text, SW_SHOW);
        },
        UICommand::ClearText => {
            CURRENT_TEXT = String::new();
            let _ = InvalidateRect(hwnd_text, None, false);
            let _ = ShowWindow(hwnd_text, SW_HIDE);
        },
        UICommand::SetVisibility(visible) => {
            VISIBLE = visible;
            if visible {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = ShowWindow(hwnd_sec, SW_SHOW);
            } else {
                let _ = ShowWindow(hwnd, SW_HIDE);
                let _ = ShowWindow(hwnd_sec, SW_HIDE);
                let _ = ShowWindow(hwnd_text, SW_HIDE);
            }
        },
    }
}
```

**Step 6: Commit**

```bash
cd .worktrees/multi-format-support && git add shadow_prompt/src/ui.rs && git commit -m "feat: add text overlay and visibility toggle to UI"
```

---

## Task 5: Update Main.rs to Wire Everything

**Files:**
- Modify: `.worktrees/multi-format-support/shadow_prompt/src/main.rs`

**Step 1: Update imports**

Add to imports from utils:

```rust
use crate::utils::{parse_mcq_with_context, McqAnswer, parse_hex_color, parse_keys, QuestionType, question_type_to_display_text};
```

**Step 2: Update Model event handling**

In the Model event handler, replace the MCQ color logic with:

```rust
// 4. Check for Question Type
let question_type = parse_question_type(&prompt, &response);
let (pixel_color, display_text) = match &question_type {
    QuestionType::MultipleChoice(ans) => {
        let hex = match ans {
            McqAnswer::A => &config_clone.visuals.color_mcq_a,
            McqAnswer::B => &config_clone.visuals.color_mcq_b,
            McqAnswer::C => &config_clone.visuals.color_mcq_c,
            McqAnswer::D => &config_clone.visuals.color_mcq_d,
        };
        let color = parse_hex_color(hex);
        let text = question_type_to_display_text(&question_type, &prompt);
        (color, text)
    }
    QuestionType::TrueFalse(true) => {
        let color = parse_hex_color(&config_clone.visuals.color_true);
        ("True".to_string())
    }
    (color, text)
};
```

**Step 3: Send text to UI**

After setting pixel color, add:

```rust
let _ = ui_tx_clone.send(UICommand::SetText(display_text));
```

**Step 4: Handle Hide event**

In the main loop, add:

```rust
InputEvent::HideToggle => {
    // Toggle visibility - get current state and flip
    // This requires a shared state - use a channel or atomic
    let _ = ui_tx.send(UICommand::SetVisibility(!visible_state)); // Need to track state
}
```

**Step 5: Commit**

```bash
cd .worktrees/multi-format-support && git add shadow_prompt/src/main.rs && git commit -m "feat: wire up question type detection and text overlay"
```

---

## Task 6: Update System Prompt for Type Detection

**Files:**
- Modify: `.worktrees/multi-format-support/shadow_prompt/release/ShadowPrompt/config/system_prompt.txt`

**Step 1: Read current system prompt**

**Step 2: Add question type detection instructions**

Add at the beginning or end of the system prompt:

```
When answering questions, first identify the question type and include it in your response:
- If it's a Multiple Choice question (options A/B/C/D or 1/2/3/4), start with "TYPE:MCQ ANSWER:X" where X is the letter.
- If it's a True/False question, start with "TYPE:TF ANSWER:TRUE" or "TYPE:TF ANSWER:FALSE".
- If it's an Identification/Short Answer question, start with "TYPE:ID ANSWER:your answer".

Then provide your complete answer below.
```

**Step 3: Commit**

```bash
cd .worktrees/multi-format-support && git add release/ShadowPrompt/config/system_prompt.txt && git commit -m "feat: add question type detection to system prompt"
```

---

## Task 7: Final Build and Test

**Step 1: Full build**

```bash
cd .worktrees/multi-format-support/shadow_prompt && cargo build --release
```

**Step 2: Run tests**

```bash
cd .worktrees/multi-format-support/shadow_prompt && cargo test
```

**Step 3: Commit final**

```bash
cd .worktrees/multi-format-support && git add . && git commit -m "feat: implement multi-format question support"
```

---

## Plan Complete

**Implementation plan saved to:** `.worktrees/multi-format-support/shadow_prompt/docs/plans/2026-02-19-multi-format-question-support.md`

**Two execution options:**

1. **Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

2. **Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

**Which approach?**
