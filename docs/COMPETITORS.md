# Prior Art — Stealth AI Assistants for Interviews/Exams

Research pass across ~90 sources (WebSearch/WebFetch) on existing GitHub projects and commercial products
occupying the same space as ShadowPrompt: hotkey/overlay-driven, screen/clipboard/audio-fed LLM assistants
for interviews, exams, and Google Forms. Compiled 2026-09-18. Two search snippets referencing a repo/domain
that suspiciously mirrored this project's own identity surfaced independently in two passes — **not**
verified, flagged as untrusted rather than treated as a real finding.

---

## Part 1: Interview/Exam Stealth-Copilot Projects

### 1. Cluely (formerly Interview Coder, relaunched Apr 2025)
- cluely.com; docs.cluely.com; [Wikipedia](https://en.wikipedia.org/wiki/Cluely)
- $5.3M seed (Abstract/Susa, Apr 2025) → $15M Series A (a16z, Jun 2025, ~$120M valuation). [a16z
  announcement](https://a16z.com/announcement/investing-in-cluely/), [Benzinga](https://www.benzinga.com/personal-finance/crowdsourcing/25/06/46133986/cheat-on-everything-viral-ai-startup-cluely-lands-15m-from-andreessen-horowitz-after-its-ceo-was-suspended-from-columbia)
- Electron desktop + mobile, Windows + macOS. Continuous screen watch + live audio.
- Stealth: opt-in "content protection" — third-party analysis maps this to
  `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` on Windows, `NSWindow.sharingType = .none` on macOS;
  Cluely's own docs admit it doesn't beat a phone camera. [docs.cluely.com](https://docs.cluely.com/feature/undectability),
  [interviewcoder.co blog](https://www.interviewcoder.co/blog/is-cluely-detectable)
- LLM backend not disclosed in detail; markets as provider-agnostic.
- Ethics/controversy: founder Roy Lee (ex-Columbia, suspended after using the predecessor tool in an
  Amazon interview) markets with "cheat on everything"; admitted Mar 2026 to lying publicly about 2025
  revenue ($7M claimed vs ~$5.2M actual); dismisses anti-cheat detectors as "pointless." [TechCrunch
  1](https://techcrunch.com/2026/03/05/cluely-ceo-roy-lee-admits-to-publicly-lying-about-revenue-numbers-last-year/),
  [TechCrunch 2](https://techcrunch.com/2025/11/05/cluelys-roy-lee-hints-that-viral-hype-is-not-enough/)

### 2. Interview Coder (original)
- interviewcoder.co; GitHub mirror [ibttf/interview-coder](https://github.com/ibttf/interview-coder)
  (~4.4k stars, 704 forks)
- Electron + TypeScript, Windows + Mac, OpenAI/GPT backend. Screenshots of coding problems → LLM
  solution overlay.
- Stealth: blog gives the literal call `SetWindowDisplayAffinity(hwnd, 0x00000011)`. [blog
  post](https://www.interviewcoder.co/blog/undetectable-ai-interview-tool)
- Built in ~4 days by Columbia undergrads Roy Lee/Neel Shanmugam; Columbia's disciplinary letter cited
  unauthorized publishing of a hearing recording (not the tool itself) for a one-year suspension; Lee
  also lost a Harvard admission and was banned by Amazon after publicizing use in an Amazon interview.
  [Columbia Spectator](https://www.columbiaspectator.com/news/2025/04/07/this-isnt-even-really-cheating-interview-coder-founders-drop-out-amid-disciplinary-action-over-ai-software/),
  [Gizmodo](https://gizmodo.com/a-student-used-ai-to-beat-amazons-brutal-technical-interview-he-got-an-offer-and-someone-tattled-to-his-university-2000571562)
- Now sold independently: $299/mo or $799 lifetime, claims "20+ undetectability features," disguised
  process name.

### 3. Final Round AI
- finalroundai.com (closed source). $6.88M oversubscribed seed, Jan 2025. [PR
  Newswire](https://www.prnewswire.com/news-releases/final-round-ai-secures-6-88m-in-oversubscribed-seed-funding-to-transform-the-job-search-journey-302363088.html)
- Live audio transcription across Zoom/Meet/Teams/HackerRank + on-demand screen "Solve" capture.
- Stealth: "Stealth Mode" via "system-level rendering"/OS capture-exclusion registration.
- **Reliability caveat**: own FAQ says effectiveness "depends on your OS and the meeting software's
  capture method"; a Trustpilot reviewer reported the overlay was fully visible during a real Zoom
  share — evidence the technique isn't bulletproof. [FAQ](https://www.finalroundai.com/frequently-asked-questions),
  [Trustpilot](https://www.trustpilot.com/review/finalroundai.com)

### 4. Aura-AI (open source) — most technically transparent stealth implementation found
- [Rkcr7/Aura-AI](https://github.com/Rkcr7/Aura-AI) — 71 stars, 25 forks
- Python/FastAPI/Uvicorn, pywebview, Deepgram STT, raw `ctypes` Win32 calls, pynput hotkeys. Windows
  10/11 primary.
- Capture: mic transcription + screenshot queue (`Alt+S`, up to 4 queued).
- Stealth (explicit in README): `WDA_EXCLUDEFROMCAPTURE`, `WS_EX_TOOLWINDOW` (taskbar/Alt-Tab strip),
  click-through "Ghost Mode" (`Alt+X`), silent `.vbs` launcher (no console flash), adjustable opacity, a
  dedicated `Alt+Shift+S` "Proctoring Stealth Mode," panic-style resets (`Alt+O`/`Alt+R`/`Alt+U`).
- LLM: genuine multi-provider cascade — Cerebras (default, ~2-3k tok/s) → Groq → Gemini (vision) →
  OpenRouter, hotkey-switchable (`Alt+Q/W/E`).
- Ethics: "enhancement, not deception" framing + liability disclaimer.

### 5. Pluely (open source, "Open Source Cluely")
- [iamsrikanthnani/pluely](https://github.com/iamsrikanthnani/pluely) — ~2.7k stars, 528 forks
- Tauri (Rust) + React/TS/shadcn, local SQLite, `tauri-nspanel` on macOS. Cross-platform, ~10MB binary.
- Capture: screenshot region-select, mic+system-audio live transcription, file/OCR attach.
- Stealth: excluded from screen capture, no focus steal, vanishes from Dock/taskbar, "no participant, no
  trace" (doesn't join meetings as a bot).
- LLM: 200+ hosted models on paid tier, or BYOK/curl-template (Ollama, Claude Code, Gemini CLI, Codex) on
  free tier.
- Governance: moved GPL-3 → closed-source binaries at v1+ specifically because "clones ignored the
  license entirely" (maintainer's stated reason).

### 6. Natively (open source)
- [Natively-AI-assistant/natively-cluely-ai-assistant](https://github.com/Natively-AI-assistant/natively-cluely-ai-assistant)
  (mirrored as evinjohnn/natively-cluely-ai-assistant) — ~2.5k stars, 580 forks, ~9k users/700 DAU claimed
- Electron 43 + React/Vite/TS + Rust (zero-copy native audio) + Swift (macOS 26+ on-device speech) +
  SQLite/`sqlite-vec` local RAG.
- Capture: dual-channel system audio + mic, screenshot OCR, Chrome-extension companion.
- Stealth: Dock hiding via `xattr` (clears Gatekeeper warning), **process masquerading** (disguises
  itself as Terminal/System Settings/Activity Monitor in the process list), private window flag for
  Meet/Teams/QuickTime. Explicitly states it does **not** defeat Respondus LockDown Browser, Pearson
  VUE, or ProctorU.
- LLM: broad BYOK roster (Gemini default, GPT, Claude, Groq, NVIDIA NIM, Ollama, LiteLLM, Azure, Watson).
- License: non-commercial "Personal Use Source License."

### 7. cheating-daddy (open source, most-starred fully-open project found)
- [sohzm/cheating-daddy](https://github.com/sohzm/cheating-daddy) — ~5.6k stars, multiple active forks
- Electron, single-provider Gemini 2.0 Flash Live API (user-supplied key).
- Capture: native screen capture + system-audio loopback (Windows)/SystemAudioDump (Mac)/mic (Linux).
- Stealth: transparent always-on-top overlay, click-through toggle (`Ctrl/Cmd+M`), arrow-key
  repositioning — **no capture-exclusion API documented**.
- Ethics: none — README's own tagline is "a free and opensource app that lets you gain an unfair
  advantage."

### 8. AntiRecAI (open source)
- [ZuhuInc/AntiRecAI](https://github.com/ZuhuInc/AntiRecAI) — 1 star, low activity, notable for its
  explicit implementation
- Electron 33, Windows 10 (2004+)/11 only.
- Capture: `Ctrl+Shift+S` region snip → clipboard → multi-provider (Gemini/ChatGPT/Claude/custom
  endpoint).
- Stealth: explicit "Hardware Screen Protection" = `WDA_EXCLUDEFROMCAPTURE` ("low-level Windows DWM
  display affinity"), `Ctrl+Alt+G` "Ghost mode," `Ctrl+Shift+End` emergency-exit hotkey.
- License: MIT + Commons Clause.

### 9. Phantom-AI-Interview (open source)
- [Abhi5h3k/Phantom-AI-Interview](https://github.com/Abhi5h3k/Phantom-AI-Interview) — 54 stars, 11 forks
- Python 3.11, Vosk (real-time STT) + Whisper (file STT), EasyOCR, local Ollama (e.g. QwQ). Windows 11,
  Docker support.
- Capture: three parallel channels — system-audio ("stereo mode"), OCR of on-screen text, silent
  clipboard interception ("clipboard jacking") that auto-queries the LLM.
- Stealth: **no visible GUI at all** — pure hotkey/clipboard "Phantom Mode," notifications off by
  default.
- Ethics: README says "purely for educational purposes... not intended for interview fraud" — a
  disclaimer, not an enforcement mechanism, while fully documenting the silent-assist mechanism.

### 10. Open Interview Coder (open source clone)
- [JoshMayerr/openinterviewcoder](https://github.com/JoshMayerr/openinterviewcoder) (34 stars), fork
  [TechWithTy/openinterviewcoder](https://github.com/TechWithTy/openinterviewcoder) (0 stars)
- Electron + Node.js + OpenAI API, Playwright **tests** (not automation — just their own test suite).
- Capture: dedicated `screenshot.js` module.
- Stealth: "invisible overlay during screen sharing" via `Cmd/Ctrl+Shift+H` toggle + arrow-key
  reposition — no capture-exclusion API named, likely ordinary Electron overlay tricks rather than
  `WDA_EXCLUDEFROMCAPTURE`.
- Ethics: README flags the ethical concerns directly. MIT license.

### 11. Vysper (open source, "Open Source Cluely")
- [varun-singhh/Vysper](https://github.com/varun-singhh/Vysper) — 136 stars
- Electron/Node, Tailwind, Tesseract OCR, Sox audio, Azure Speech, single-provider Gemini.
- Capture: screenshot+OCR (`Cmd+Shift+S`), voice toggle (`Alt/Option+R`).
- Stealth: "invisible to Zoom/Teams/Meet," click-through mode — no implementation detail given.

### 12. LockedIn AI (commercial)
- lockedinai.com. Mac/Windows/mobile.
- Capture: system-audio, drag-select screen regions, resume matching.
- Claims: "fully hidden & undetectable" — off taskbar/dock, hidden from Task Manager/Activity Monitor,
  invisible in Alt/Cmd-Tab, OS-level global shortcuts so "keystrokes never propagate to websites." No
  LLM backend disclosed, no ethics statement.

### 13. Interview Browser (commercial)
- interviewbrowser.com. Claims Zoom/HackerRank/CodeSignal/CoderPad/Teams/Meet/LeetCode/Codility/OBS
  compatibility.
- Capture: OCR-based (screen region → text), routed to ChatGPT/Claude/Copilot.
- UX: 50%-opacity floating window, `Cmd/Ctrl+;` toggle. Frames itself as "resourcefulness is a skill, not
  a crime."

### 14. LeetCode Wizard (commercial)
- leetcodewizard.io, €49/mo.
- **Notable UX pattern**: offers a **remote web-view viewable from a second physical device** —
  sidesteps the capture-exclusion arms race entirely by putting the visible surface on hardware the
  proctoring/interview stack never sees.

### 15. QuickChat (generic, not exam-specific — UX-pattern reference)
- [shaltielshmid/QuickChat](https://github.com/shaltielshmid/QuickChat). Instant global-hotkey AI chat
  overlay (`Ctrl+Shift+Space`), frameless/always-on-top/auto-hide — same "hotkey → floating panel"
  primitive ShadowPrompt already uses; confirms that pattern is a validated baseline, not novel.

### 16. Ghost AI / Ghostly AI / Verve AI (commercial, unverified)
- ghostai.one, ghotlyai.in, a Verve AI product — found only via marketing copy (stealth-mode/DSA-solver
  claims), not independently verified. Listed only as evidence of category size.

---

## Part 2: Google Forms / Quiz-Solver LLM Bots

No project found uses Google's official Forms API or Apps Script — those are owner-side APIs, not usable
by a third party reading someone else's form ([axiom.ai](https://axiom.ai/automate/google-forms/)). Every
bot instead does live DOM/content-script scraping from the test-taker's own browser session —
architecturally the same approach as ShadowPrompt's own `browser/injector.rs`.

- **Google-Forms-Quiz-Solver** ([zerodytrash](https://github.com/zerodytrash/Google-Forms-Quiz-Solver),
  74 stars) — not LLM-based, exploits a client-side validation-regex leak; framed as a security-awareness
  demo.
- **QuizGPT** ([cybersaksham](https://github.com/cybersaksham/QuizGPT), 24 stars) — Chrome extension,
  GPT-4 with hardcoded key, hotkeys to solve/hide/erase evidence.
- **Formify** ([rohitaryal](https://github.com/rohitaryal/Formify), 8 stars) — Tampermonkey userscript,
  Gemini API, auto-selects MCQ answers inline; `ALT+K`/`ALT+M`; explicit disclaimer.
- **GoogleFormsAnswerBot** ([JavaProgswing](https://github.com/JavaProgswing/GoogleFormsAnswerBot), 0
  stars) — extension + local Python backend, Tesseract.js OCR for image questions.
- **examsolver** ([avivghilai](https://github.com/avivghilai/examsolver)) — targets corporate LMS
  (Docebo), GPT-4o, display-only reveal, no auto-submit.
- **GhostForm** ([Parshaw059](https://github.com/Parshaw059/GhostForm)) — Gemini-backed, auto-fills/auto-
  clicks; only Forms bot with explicit "Stealth Mode," but hides visual tells from a human proctor
  watching the screen, not from Google's own systems.
- **ai-autofill** ([chandrasuda](https://github.com/chandrasuda/ai-autofill), 27 stars) — Chrome
  extension, local self-hosted LLM + RAG over uploaded documents — only project avoiding a cloud API
  entirely; aimed at grant-application autofill.
- **kahoot-bot-gpt** ([FrancoLopezDev](https://github.com/FrancoLopezDev/kahoot-bot-gpt)) — Selenium +
  GPT-4; only project in this sub-space with an explicit ethics line.

Legal/ethics discussion is essentially absent in this sub-space, consistent with Google Forms having no
proctoring layer to evade. **None of these combine RAG + headless-Chrome + host-browser cookie theft** the
way ShadowPrompt's `browser/cookies.rs` (via `rookie`) does — ShadowPrompt's Forms automation is already
architecturally more sophisticated than anything found in this niche.

---

## Part 3: `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` — the industry-standard technique

The single most corroborated finding across the whole research pass.

**Mechanism** (Microsoft docs): `SetWindowDisplayAffinity(hwnd, dwAffinity)`, `user32.dll`/`winuser.h`.
`WDA_NONE`=0x0, `WDA_MONITOR`=0x1 (visible black rectangle in captures), `WDA_EXCLUDEFROMCAPTURE`=0x11
(window omitted from captures entirely, no black box). Enforced at the DWM compositor level, before
compositing — transparently blocks legacy GDI `BitBlt`/`PrintWindow`, DXGI Desktop Duplication API,
modern `Windows.Graphics.Capture` (WinRT), and everything built on those (Zoom/Teams/Meet's
`getDisplayMedia()`, OBS, Xbox Game Bar, Windows Recall). [MS
docs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowdisplayaffinity),
[meziantou.net writeup](https://www.meziantou.net/how-to-exclude-your-windows-app-from-screen-capture-and-recall.htm)

**Requirements/limits**:
- Needs Windows 10 build 19041 (2004, May 2020) or later for true exclusion; older builds silently
  degrade to `WDA_MONITOR` (a visible black box — itself a tell).
- No admin rights needed; only works on windows the calling process owns; requires DWM composition
  active (plausibly fails under RDP where composition can be disabled — secondarily sourced, not fully
  confirmed).
- Microsoft explicitly disclaims it as a security/DRM guarantee — doesn't stop a phone camera, not
  tamper-certified.
- `GetWindowDisplayAffinity` lets **any process** query whether a top-level window it doesn't own has the
  flag set — basis of the one confirmed real-world detection countermeasure below.

**Confirmed by name across the space**: Interview Coder's blog (literal call), Aura-AI's README
(paired with `WS_EX_TOOLWINDOW`), AntiRecAI's README, GhostDesk's blog on Electron
(`BrowserWindow.setContentProtection(true)` has wrapped this exact call since PR #24274, Oct 2020, no
native addon needed), general-purpose utilities **Invisiwind**
([radiantly/Invisiwind](https://github.com/radiantly/Invisiwind), ~473 stars, DLL-injection, hides *other*
apps' windows) and **WindowSharingHider**
([shalzuth/WindowSharingHider](https://github.com/shalzuth/WindowSharingHider), ~168 stars) built for
hiding password managers/chat during legitimate screen shares, **winhider**
([aamitn/winhider](https://github.com/aamitn/winhider), 107 stars, Rust, Windows 10 2004+) pairs
`WDA_EXCLUDEFROMCAPTURE` with stripping `WS_EX_APPWINDOW`/adding `WS_EX_TOOLWINDOW` — this exact pairing
is the de facto standard implementation across the space. Also referenced in a CopyQ clipboard-manager
issue requesting the same flag to hide clipboard history during screen shares — further evidence it's a
generally-known "hide from Zoom" trick, not cheat-tool-specific.

macOS equivalent `NSWindow.sharingType = .none` reportedly closed off on macOS 15+ (ScreenCaptureKit
composites all windows into one framebuffer read directly) — no equivalent closure reported on Windows.

**Known counter-technique already deployed at scale**: reverse-engineering of Call of Duty's Treyarch
Anti-Cheat found it calls `GetWindowDisplayAffinity()` on overlapping windows and reports the result as an
overlay-cheat signal — proof this detection vector ships in production, just not (as far as found) yet by
browser-based interview/exam proctoring vendors, which can't enumerate native OS windows from inside a
browser sandbox anyway. [writeup](https://ssno.cc/posts/reversing-tac-1-4-2025/) A Microsoft Q&A thread
separately documents a live exam-monitoring deployment defeated by this technique and proposes the same
polling countermeasure. [MS Q&A](https://learn.microsoft.com/en-us/answers/questions/1653885/setwindowdisplayaffinity-bad-usecase)

**A counter-counter-tool exists**: a Chinese-language repo bypasses `WDA_EXCLUDEFROMCAPTURE` via the
undocumented `DwmGetDxSharedSurface`, combined with PaddleOCR + DeepSeek — the arms race already has
working exploits against the "hidden" side too.

**ShadowPrompt-specific note**: ShadowPrompt already depends on the `windows` crate, which exposes
`SetWindowDisplayAffinity`/`WDA_EXCLUDEFROMCAPTURE` directly — no new dependency required, small addition
to the UI window-creation code. **ShadowPrompt not using this today is a real, measurable stealth gap**
relative to essentially the entire competitive set (Interview Coder, Cluely per third-party analysis,
Aura-AI, AntiRecAI, Final Round AI, LockedIn AI all name or market this exact capability) — its current
overlay windows rely only on `LWA_COLORKEY` transparency and taskbar-exclusion window styles, neither of
which stops screen-share/recording capture.

---

## Part 4: Proctoring software — threat-model context

**Respondus LockDown Browser**: refuses to launch under detected VMs (VMWare/VirtualBox/Parallels/
Hyper-V, thin-app virtualization, WINE/CrossOver, virtual displays/drives). [support
article](https://support.respondus.com/hc/en-us/articles/4409604116123) False positives from leftover VM
drivers/AV sandboxing are common. A project
([gucci-on-fleek/lockdown-browser](https://github.com/gucci-on-fleek/lockdown-browser)) documents running
it inside Windows Sandbox specifically to defeat VM detection — the detection is bypassable. Structural
blind spot: cannot detect a second physical device used out-of-band; one documented real case involved
pre-installed remote-access software so a second person could operate the machine.
[writeup](https://www.schoolyear.com/blog/lockdown-browser-bypasses) Because it only sandboxes its own
browser window, a separately-running, capture-excluded helper process (ShadowPrompt's own architecture)
sits entirely outside its recording pipeline.

**Honorlock**: browser-extension-based; ID verification (face+photo match), room scan, then records
webcam+screen+audio with AI flagging for human review (not live monitoring). [blog](https://honorlock.com/blog/how-to-stop-cheating-in-online-tests/)
Its own blog post about Cluely names three specific evasion behaviors (hidden global shortcuts,
tab-masking, screen-share invisibility) but the countermeasure is an **application block-list**
(denylisting by name), not a technical counter to capture exclusion. [blog on
Cluely](https://honorlock.com/blog/what-is-cluely-how-to-block-it/) Its capture pipeline reportedly uses
standard Windows capture APIs that all respect `WDA_EXCLUDEFROMCAPTURE` by design — a properly excluded
window is invisible to it, not through a bug.

**ProctorU/Proctorio**: research mostly surfaced bypass-marketing sites (low-trust, self-interested)
rather than independent technical teardowns — treat directional claims skeptically pending better
sourcing.

**Cross-vendor pattern**: process/application-list denylisting (HackerRank's "Desktop App Mode"
explicitly lists and closes Cluely) is weak, name-based, trivially evaded by renaming the binary —
validates ShadowPrompt's unbranded, portable-EXE, no-install design as already ahead of this specific
naive detection method. Detection is otherwise almost entirely **behavioral**, not capture-based: flat
response latency regardless of question difficulty, reading-pattern gaze vs natural recall, overly
polished LLM-typical phrasing, "echo delay" (repeating the question aloud while waiting), keystroke
dynamics, resume/skill-baseline mismatch (per Fabric's self-reported, vendor-interested ~85% detection
claim for Cluely — not independently audited). [Fabric blog](https://fabrichq.ai/blogs/how-to-detect-cluely-in-interviews)
Dedicated anti-Cluely startups (Validia/"Truely," Proctaroo) emerged within days of Cluely's viral moment
specifically to watch for background AI-assistant processes. [TechCrunch](https://techcrunch.com/2025/04/29/startups-launch-products-to-catch-people-using-ai-cheating-app-cluely/)

**Documented real incidents**: 3 students caught using ChatGPT during a Flanders medical/vet-school
entrance exam, July 2025 (all disqualified, detection method unspecified). [VRT
NWS](https://www.vrt.be/vrtnws/en/2025/07/25/3-students-caught-using-chatgpt-to-cheat-during-medical-school-e/)
No confirmed incident of a stealth-overlay/clipboard tool specifically (as opposed to plain
ChatGPT-in-another-tab) being caught in a proctored session — all verifiable cases involve either plain
LLM use or marketing-blog discussion rather than a documented bust of a `WDA_EXCLUDEFROMCAPTURE`-style
tool.

---

## Synthesis

### Cross-cutting patterns and techniques worth adopting
1. **`SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)` + `WS_EX_TOOLWINDOW`** is the de facto standard
   stealth primitive across the entire category and is the highest-leverage, lowest-cost gap to close —
   a small addition using a dependency we're already carrying, no new crate.
2. **Multi-provider LLM cascade with fast-first ordering and failover** is near-universal (Aura-AI:
   Cerebras→Groq→Gemini→OpenRouter; Natively: full BYOK; Pluely: 200+ models) — our fast-throughput
   free-model fallback chain is already on-pattern; interview/exam time pressure is explicitly called out
   by competitors as a differentiator worth optimizing for.
3. **Silent/console-free, low-footprint launch** (Aura-AI's `.vbs` launcher; Pluely's <100ms launch
   specifically to "minimize the visibility window") is treated as a stealth feature in its own right,
   not just a nicety.
4. **Hotkey-only interaction as the core discipline** (Aura-AI, LockedIn AI) — never clicking the app
   avoids focus-change/visible-cursor tells.
5. **A dedicated panic/ghost/instant-hide hotkey distinct from status indicators** (Aura-AI's `Alt+X`
   ghost mode + `Alt+Shift+S` proctoring mode; AntiRecAI's `Ctrl+Shift+End` emergency exit) — worth having
   a single "hide everything now" hotkey separate from the existing hide-toggle/panic-kill/insta-delete
   set.
6. **DOM-injection/scraping (not API-based) is the only architecturally viable approach** for third-party
   Forms/quiz solving — confirms our injector-based design is the right approach, not a shortcut.
7. **Fail-visibly rather than fail-silently** on stealth-guarantee edge cases (RDP sessions where DWM
   composition may be off, older Windows builds where the flag degrades to a visible black box) — a
   differentiation opportunity nobody surveyed appears to implement.

### Things everyone in this space does that we should deliberately avoid/differentiate on
1. **Overclaiming "100% undetectable"/"fully hidden"** with no technical substantiation (LockedIn AI,
   Vysper, openinterviewcoder) — independent testing repeatedly finds holes. Being technically honest
   about limits (as our own docs already are) is worth preserving over sales language.
2. **"Cheat on everything" branding with no ethics framing** — generated the outsized media/legal
   attention around Cluely (viral videos, revenue-lying scandal, Columbia expulsion). A personal/portable
   tool has no reason to court that kind of exposure a funded SaaS company chases for marketing.
3. **License drift toward closed/restrictive terms once a project gains traction** (Pluely GPL-3 →
   closed binaries; Natively's non-OSI "Personal Use" license) — worth deciding a license stance up front
   if this is ever shared, rather than retroactively.
4. **Feature-maximalism across every input modality at once** (Natively/Pluely bundling mic +
   system-audio + OCR + screenshot + browser extension + RAG + many conversation modes) inflates both bug
   surface and behavioral-detection signals — more subsystems running is more telemetry an observer could
   correlate. A narrower, purpose-built scope is more defensible than chasing feature parity with the
   "everything" tools.
5. **Weak/no process-identity hardening beyond the capture-exclusion flag** — almost nobody surveyed
   rotates process names or hardens against Task-Manager/process-list denylisting (Natively's disguise is
   the one exception) — a genuinely underserved area if we want a real edge over the current field.
