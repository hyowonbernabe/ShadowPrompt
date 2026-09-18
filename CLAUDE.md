# ShadowPrompt — CLAUDE.md

Rules only. No architecture description, no file tree, no version numbers, no
feature list here — those live in code and belong in docs/ that get
regenerated or deleted, never hand-maintained here where they rot. If you're
tempted to write "currently the project has X modules" or "as of vN.N" in
this file, stop — that belongs in a dated doc under `docs/`, not here.

---

## Non-negotiables (apply regardless of version/architecture)

- **Never auto-submit** anything on the user's behalf (forms, purchases,
  irreversible network actions) unless the user explicitly triggers that
  exact action. Fill/prepare, don't commit.
- **Never overwrite** a field/answer/value the user already set. Detect
  "already has a value" before touching anything.
- **Stealth is a requirement, not a feature**: no taskbar entry, no visible
  window, no console in release builds. Any new UI surface must justify
  itself against this.
- **Exe-relative paths only.** Nothing hardcodes an absolute path or assumes
  a fixed install location — the whole point is USB/portable deployment.
- **Secrets never committed.** Config templates ship with no keys. Real
  config is gitignored.
- **Destructive lifecycle actions** (self-delete, panic-wipe, kill-other-
  instance) must be intentional and hard to trigger by accident (double-tap,
  explicit flag) — but once triggered, they run to completion without
  prompting, since the whole point is speed under pressure.

## Working rules for this repo

- **Docs describe intent and decisions, not current file contents.**
  Anything a `grep`/`ls`/`git log` can answer doesn't belong in a doc.
  Architecture docs describe *why* a boundary exists, not *what* is on each
  side of it today — that drifts the moment a file moves.
- **One doc per decision that isn't obvious from the code**, dated, under
  `docs/`. Superseded docs get deleted or marked superseded, not left to
  silently rot next to the current one.
- **Don't maintain a repo tree or symbol table by hand anywhere.** If you
  need one for a task, generate it (`git ls-files`, a search) — don't write
  it into a markdown file that will be wrong in a month.
- **Before adding a capability**, check whether it's a variant of something
  that already exists (another hotkey action, another tool) — prefer
  extending a shared mechanism over adding a parallel one-off.
- **When multiple implementations of the "same idea" exist in the tree**
  (e.g. an old and new crate, a stub module shadowed by a real one), that's
  a defect to resolve, not a state to document around. Flag it, don't build
  on top of it.

## Conventions

- Rust 2021, single EXE, Windows-only. `unsafe` confined to Win32/WinRT FFI
  boundaries — never used for convenience elsewhere.
- Config is TOML + `serde`, exe-relative, with a committed `*.example.toml`
  template and a gitignored real file.
- Prefer a typed enum + exhaustive `match` over string-keyed dispatch for
  anything internal (hotkeys → events → actions). String keys are fine at
  external boundaries (tool names sent to an LLM, config field names).
- Retries get exponential backoff and a fixed attempt cap — no unbounded
  retry loops.
- Every destructive or irreversible code path gets a comment stating the
  guard that prevents accidental triggering, next to the trigger itself, not
  in a separate doc.

## Build & verify

Run the project's own build/test/lint commands (check `Cargo.toml` /
`package.json` / CI workflow for the actual invocations — don't hardcode
them here either). A change isn't done until it builds clean and existing
tests pass; say so plainly, and say plainly if you skipped that.
