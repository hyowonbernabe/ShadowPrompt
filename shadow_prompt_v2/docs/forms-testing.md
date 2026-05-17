# Google Forms automation — testing guide

End-to-end verification procedure. Run each scenario and confirm the daemon log shows the expected pattern.

## Prerequisites

1. Build:
   ```
   cd shadow_prompt_v2
   cargo build --features debug
   ```
2. Set a valid OpenRouter API key in `target/debug/config/config.toml`.
3. Have Chrome installed (path auto-detected from standard locations).

## Test setup

1. Launch daemon: `./target/debug/shadowprompt.exe --debug`
2. Press `Ctrl+Shift+I` → spawns Chrome incognito on `:9222` with marker profile.
3. Log line: `daemon loop running; awaiting input events`.

## Scenarios

### S1 — Empty form (basic smoke)

- Form: any Google Forms with 1-2 text questions, no submit required.
- Action: navigate to form URL in the debug Chrome window. Press `Ctrl+Shift+7` (single page).
- Expected log:
  ```
  event: FormsSingle
  page 1: extractor found N question(s)
  cache: write=... read=...      (only if knowledge enabled)
  page 1: answered N questions
  ```
- Visual: fields filled in browser. Form indicator (top-left pixel) yellow during run, hidden after.

### S2 — MCQ (radio buttons)

- Form: 1 page with 3+ multiple-choice questions.
- Action: `Ctrl+Shift+7`.
- Verify each radio is selected (visible bullet).
- Watch the `looseMatch` logic in injector: if option text in the form differs from LLM output (e.g. trailing whitespace), it still matches.

### S3 — Checkbox (multi-select)

- Form: a question like "Select all that apply: Apple, Banana, Cherry".
- Verify multiple boxes get ticked.
- Daemon log should show the answer JSON value as comma-separated text.

### S4 — Dropdown

- Form: a Google Forms dropdown question.
- Verify dropdown clicks open, scrolls if needed, then the right option is picked.
- This path uses an async injector with `setTimeout(250ms)` for menu open.

### S5 — Short text + long text

- Form: short answer + paragraph (long answer) questions.
- Verify input + textarea both receive text via `setNative` (React-aware setter).

### S6 — Date / Time

- Form: a date question + a time question.
- Verify the date input shows the chosen date; time input shows the chosen time.
- LLM must return ISO `YYYY-MM-DD` for date, `HH:MM` 24-hour for time.

### S7 — Linear scale (Scale)

- Form: a 1-to-5 rating question.
- Detector triggers because all option labels are pure digits → kind = Scale.
- Verify the chosen integer is clicked.

### S8 — Grid (multiple-choice grid)

- Form: a 3-row × 3-column matrix (e.g. "Rate these on a scale of...").
- LLM returns a stringified JSON object as the question's value, e.g. `{"Row 1": "Always", "Row 2": "Never"}`.
- Injector loops `[role="radiogroup"]` (one per row), matches the chosen column.
- Verify one radio per row is selected.

### S9 — CheckboxGrid (multi-select grid)

- Form: "Tick all that apply" grid with multiple selections per row allowed.
- LLM returns `{"Row 1": "Always, Often"}`.
- Verify multiple boxes per row tick.

### S10 — Image questions

- Form: a question containing an embedded image (uploaded by the form author).
- Extractor harvests image URLs from `<img>` tags whose src isn't `data:`.
- LLM receives the image as an `image_url` part alongside the question text.
- Verify the answer makes sense for what's in the picture.

### S11 — Multi-page auto-paginate

- Form: 3+ pages with a Next button between each.
- Action: `Ctrl+Shift+9` (auto-paginate).
- Log per page:
  ```
  page 1: extractor found ...
  page 1: answered N questions
  page 2: extractor found ...
  ...
  reached Submit page; halting (never auto-submit)
  ```
- Verify all pages got filled. Verify daemon STOPPED at the submit page — final page must remain on screen with Submit button visible but not clicked.

### S12 — Multi-page with many images (image pruning)

- Form: 5+ pages, each with at least one image question.
- Run `Ctrl+Shift+9`.
- Confirm in log: `forms: dropped N earlier image(s) from history` (or similar text from `prune_images_from_history`).
- Verify total request size stays bounded; daemon doesn't slow down or hit context limits.

### S13 — Skip already-answered questions

- Form: any form, manually fill 1-2 questions yourself first, then run automation.
- Extractor reads `aria-checked` / input.value to detect existing answers.
- Verify those questions stay as-is; only unanswered ones get filled.
- Log: `page 1: all questions already answered` if you pre-fill everything.

### S14 — Abort mid-run

- Action: start a long auto-paginate, then press `Ctrl+Shift+0` (abort).
- Verify form indicator goes to aborted color (orange) and the daemon stops touching the page.
- Active task in slot is cancelled; you can start a new run after.

### S15 — Bad Chrome state

- Edge case: launch debugger, close all tabs except `about:blank`. Press `Ctrl+Shift+9`.
- Expected: error message listing actually-open tab URLs, helping you debug.

## Known limitations

- **Custom date/time pickers**: some Google Forms use a JS-driven custom date picker instead of `<input type="date">`. Current injector uses native setter which works for standard inputs only. If a real-world form has a custom picker, the date question may not fill — flag this and we extend the injector.
- **File upload questions**: not implemented. Form expects a file, automation skips.
- **CAPTCHA / re-auth prompts**: if Google asks for a sign-in or CAPTCHA mid-run, automation cannot proceed.
- **Required questions on submit page**: daemon never clicks Submit. User must manually click.
- **JS-heavy forms with delayed render**: 900ms post-Next wait might be too short for forms on slow connections. Bump if needed.

## Verifying everything

Run scenarios S1, S2, S3, S5, S11, S13 at minimum. They cover the most common production cases. Add S6-S9 and S10 if your real exams use those types.

## What to report if it breaks

For any scenario that fails:
1. Daemon log lines around the failure (entire `event: FormsAuto` block).
2. Screenshot of the form question that didn't fill correctly.
3. The form URL if shareable.

Filed under: project root, `data/logs/app.log`.
