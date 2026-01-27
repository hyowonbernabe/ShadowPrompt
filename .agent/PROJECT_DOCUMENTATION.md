# ShadowPrompt: Portable Stealth AI Architecture

**Version:** 1.0.0
**Target OS:** Windows 10/11
**Language:** Rust

---

## 1. Project Overview
ShadowPrompt is a "Zero-Install" AI assistant designed to run entirely from a USB drive. It prioritizes stealth, portability, and speed. The application runs as a background daemon with no visible window or taskbar icon. It interacts with the user solely through global hotkeys and clipboard manipulation.

### Core Philosophy
* **Contained:** All logic, dependencies (DLLs), authentication, and vector indices reside on the USB. No files are written to the host machine.
* **Invisible:** The UI is non-intrusive. Interactions occur via invisible overlays and clipboard injections.
* **Agnostic:** Capable of reading screen context (OCR) or clipboard content to query generic LLMs or project-specific RAG data.

---

## 2. Technical Stack
We utilize **Rust** for its memory safety, zero-dependency compilation (static binary), and direct access to Windows APIs.

| Component | Choice | Justification |
| :--- | :--- | :--- |
| **Language** | **Rust** | Compiles to a single `.exe`. Extremely performant and lightweight. |
| **OCR Engine** | **Windows.Media.Ocr** | Native Windows 10/11 API. Adds **0MB** to binary size. Privacy-friendly (local). |
| **Windowing/GUI** | **Windows API (Win32)** | Required for creating the "invisible" overlay and handling global keyboard hooks without a GUI framework. |
| **Vector DB** | **LanceDB** | Serverless, embedded vector database. |
| **LLM Provider** | **Groq (Primary) + OpenRouter (Fallback)** | **Justification:** Multi-tier fallback strategy for high availability and low latency. <br> 1. **Groq:** `llama-3.1-8b-instant` (Ultra-fast, Free). <br> 2. **OpenRouter:** `google/gemma-3-27b-it:free` (High-quality fallback). <br> 3. **Ollama:** `llama3` (Local offline fallback). |

---

## 3. Workflow & UX Strategy

### A. The "Stealth" Loop (Startup)
1.  **User Action:** Plug in USB, run `shadow_prompt.exe`.
2.  **State:** Background process starts.
3.  **Visual Feedback:** A single **Green Pixel** appears in the top-right (or very right) of the screen to indicate "Loaded/Ready".
4.  **Process Persistence:** If USB is removed, the process dies immediately.

### B. Scenario 1: Clipboard Trigger (The Main Way)
This is the primary method. The user supplies the text via standard system copy.

1.  **Action:** User selects text (e.g., quiz question) and presses `Ctrl + C`.
2.  **Trigger:** User presses `Model Key` (e.g., `Ctrl + Shift + V`).
3.  **Visual:** **Red Pixel** appears (Top Right) to indicate "Thinking/Processing".
4.  **Processing:**
    *   App reads current clipboard content.
    *   **Logic (Fallback Chain):** 
        1. **Try Groq:** Fast inference using `llama-3.1-8b-instant`.
        2. **Try OpenRouter:** If Groq fails (Rate Limit/Error), use `google/gemma-3-27b-it:free`.
        3. **Try Ollama:** Final local fallback if internet is down.
    *   *Note: If "Web Search" is strictly needed (live data), we will need to add a specialized search tool (e.g., DuckDuckGo API), but for now, we rely on the generic intelligence of the LLM.*
5.  **Output:** App overwrites system clipboard with the Answer.
6.  **Visual:** Red Pixel disappears.
7.  **User Action:** User checks Clipboard History (`Win + V`) or pastes (`Ctrl + V`) to see the answer.

### C. Scenario 2: OCR Trigger (Fallback)
Used when copy is disabled or text is an image.

1.  **Trigger:** User presses `OCR Key` (e.g., `Ctrl + Shift + Space`).
2.  **Visual:** **Red Pixel** appears (Top Right) to indicate "Waiting for Selection".
3.  **Action:** User clicks and drags to define a region.
    *   *Visuals:* **Completely Invisible Selection.** No border, no shading.
    *   *Logic:* Captures from the screen/monitor where the cursor is focused.
4.  **Processing:**
    *   Use `Windows.Media.Ocr` to extract text.
    *   **Result:** Extracted text is placed into System Clipboard.
5.  **Visual:** Red Pixel disappears (indicating text is ready).
6.  **Follow-up:** User proceeds to **Scenario 1** (Presses `Model Key` to process the now-copied text).

### D. Scenario 3: Multiple Choice Questions (MCQ)
ShadowPrompt automatically detects if the answer is a multiple choice option (A, B, C, D or 1, 2, 3, 4) and provides a secondary visual cue.

1.  **Trigger:** Standard Model Query (Scenario 1).
2.  **Detection:** If the LLM response starts with "A", "B", "C", "D" (or equivalent numbers/formats).
3.  **Visual Feedback:** A **Secondary Pixel** appears just below the main indicator:
    *   **Cyan:** Answer is **A** or **1**
    *   **Magenta:** Answer is **B** or **2**
    *   **Yellow:** Answer is **C** or **3**
    *   **Black:** Answer is **D** or **4**
4.  **Clipboard:** The full answer text is still copied to the clipboard.
5.  **Reset:** The secondary indicator resets when a new query is started.

---

## 4. Directory Structure (USB Layout)
The application assumes relative paths. The USB drive letter can change without breaking functionality.

```text
/ShadowPrompt
├── bin/
│   ├── shadow_prompt.exe       # Main executable
│   ├── onnxruntime.dll         # Required for Portability
│   └── ... (other DLLs)
├── config/
│   ├── config.toml             # Keys, Model IDs, Prompts
│   └── system_prompt.txt       # "You are a concise assistant..."
├── data/
│   ├── vectors/                # LanceDB data files
│   └── logs/
├── knowledge/                  # RAG Source
│   ├── documentation.pdf
│   └── cheat_sheet.txt
└── models/
    └── embedding_model.onnx    # Local quantization model
```

---

## 5. Security & Safety Mechanisms

### A. The Panic Button
* **Keybind:** `Ctrl + Shift + F12`
* **Behavior:**
    1. Terminate `shadow_prompt.exe` immediately.
    2. Clear system clipboard (Safety wipe).

### B. Cost & Limits
* **Daily Limit:** Track usage in `data/usage.json`.
* **RAG Timeout:** If RAG is too slow, skip it and use raw LLM knowledge/Search.

---

## 6. Risks & Mitigations

| Risk | Impact | Mitigation Strategy |
| :--- | :--- | :--- |
| **Missing Visual Feedback** | User unsure of state | **Green Pixel** (Loaded), **Red Pixel** (Active/Waiting). |
| **Missing DLLs** | App fails on guest PC | **Side-load DLLs:** Ship `onnxruntime.dll` in `/bin`. |
| **Firewall Blocks** | Network requests fail | **User Assumption:** User handles network/AV (mentioned in scenario). |
| **Clipboard Collisions** | User loses data | **Simplicity:** We overwrite. User accepts risk. |
| **USB Removal** | Process hangs | **Heartbeat:** Check drive existence; die if gone. |

---

## 7. Configuration Specification (`config.toml`)

```toml
[general]
mode = "stealth"
wake_key = "Ctrl+Shift+Space"
model_key = "Ctrl+Shift+V"
panic_key = "Ctrl+Shift+F12"
use_rag = true

[visuals]
indicator_color = "#FF0000" # Processing Color
ready_color = "#00FF00"
cursor_change = false

# New Configurable Options
position = "top-right" # Options: top-right, top-left, bottom-right, bottom-left
size = 5               # Pixel size (default 5)
offset = 0             # Distance from corner (e.g. 10 to move in)
x_axis = 0            # Add to move Right (Desktop coords)
y_axis = 0            # Subtract to move Down (User preference logic: -Y = Down)

# MCQ Colors
color_mcq_a = "#00FFFF" # Cyan
color_mcq_b = "#FF00FF" # Magenta
color_mcq_c = "#FFFF00" # Yellow
color_mcq_d = "#000000" # Black
color_processing = "#FF0000" # Red

[models]
# Options: "openrouter", "github_copilot", "ollama", "groq"
provider = "groq"

[models.groq]
api_key = "gsk_..."
model_id = "llama-3.1-8b-instant"

[models.openrouter]
api_key = "sk-or-..."
model_id = "google/gemma-3-27b-it:free"

[models.ollama]
base_url = "http://localhost:11434"
model_id = "llama3"

[models.github_copilot]
token_path = "config/copilot_token.json"
```
