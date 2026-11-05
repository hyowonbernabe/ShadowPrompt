# Knowledge docs, Claude Code style — design

Date: 2026-11-05. Supersedes the "Tools" and "Prompt listing" sections of
`2026-11-05-cloud-knowledge-design.md` (fetching, dedup and hosting there still stand).
Owner decision: make doc lookup work as close as possible to how Claude Code finds things in a
codebase.

## Why this approach

- Claude Code does not preload a codebase and does not use embeddings/RAG. It keeps a small,
  always-loaded memory (CLAUDE.md, skill name+description) and finds everything else with
  agentic search: Glob (by name), Grep (by content), Read (by line range). Anthropic chose this
  over RAG because it was simpler and worked better.
- We considered "preload everything when small". Measured on the owner's real docs (~82k tokens):
  ~1.5–3s per answer once cached, but a $0.33 cold write on Sonnet every hour-idle / model switch /
  doc change, and it stops scaling as the shared cloud collection grows. Search costs roughly
  the same at 5 or 500 docs. Owner chose the Claude Code model.

## What changes

**Tools** — same names, parameters and output shapes as Claude Code's (verified against
code.claude.com/docs/en/tools-reference and the live tool schemas), scoped to the merged
local+cloud doc set so the model's trained habits apply. They replace `list_docs`/`search_docs`/
`read_doc`.

| Tool | Args | Behaviour (as Claude Code) |
|---|---|---|
| `Glob` | `pattern`, `path` | Matching doc names, newest first, max 100 (+ truncation note). "No files found" when none. |
| `Grep` | `pattern`, `path`, `glob`, `output_mode` (`files_with_matches` default / `content` / `count`), `-i`, `-n`, `-A`, `-B`, `-C`/`context`, `multiline`, `head_limit` (default 250, 0 = all), `offset` | ripgrep-style output: `name:line:text` for hits, `name-line-text` for context, `--` between groups. |
| `Read` | `file_path`, `offset` (1-based), `limit` (default 2000) | Tab-numbered lines; empty-file and past-the-end notices; partial-view note. |

Names only ever resolve against the merged list, so nothing outside the sandbox/bundle is
reachable (unchanged). Every result stays capped at `max_doc_bytes` (100 KB ≈ Claude Code's
25k-token Read cap).

Deliberate deviations (builder's calls):
- Doc names carry a `local/` or `cloud/` root. Glob/Grep patterns and paths match either from the
  top or from inside either root (`biology/*.md` finds both copies), so the roots never get in
  the way. A pattern with no `/` (`*.md`) matches the bare file name at any depth, as ripgrep's
  globs do. Read also accepts a name without its root when that's unambiguous.
- Loosely typed tool arguments (`"true"`, `"10"`, `10.0`, `-1`) are accepted rather than
  rejected, and a pattern that isn't valid regex is searched as plain text with a note saying so.
- Grep is case-insensitive by default (`-i: false` makes it exact). Claude Code's isn't, but
  study notes are prose, where a missed capital silently loses the answer.
- Grep output lines are cut at 500 chars, Read lines at 2000, so one giant line can't flood
  the context.

**Always-loaded part** (end of the system prompt, cached; Claude Code's order is
instructions → project memory → conversation):
- Search instructions in Claude Code's spirit: go straight to a section via the index, else
  Grep then Read; independent calls in parallel; read generously; stop once answered. Claude
  Code's own wording isn't published, so this text is ours.
- `CLAUDE.md` at the root of the local folder and/or the cloud set, included in full every
  request (Claude Code loads the root CLAUDE.md at launch; guidance: under 200 lines). Capped
  at 40,000 chars total. Nested CLAUDE.md files are ordinary docs (Claude Code loads those
  lazily; not worth the machinery for a docs folder).
- A doc index — the analogue of Claude Code's skill descriptions: each doc's name, line count
  and `#`/`##` headings with line numbers, each entry cut at 1,536 chars (Claude Code's skill
  description cap). Whole index capped at 24,000 chars, then names only, then "use Glob" past
  300 docs.
- The same block goes to every action that has doc tools (clipboard, screenshot, screen answer,
  forms), not just clipboard.

**Caching:** the system message gets an explicit 1-hour cache breakpoint, so the stable prefix
(tools + instructions + index + memory files) is read from cache on every question, not only
within one conversation. Verified live: without it, every question re-paid the whole prefix;
with it, Sonnet and Gemini both hit. The docs are always emitted in sorted order so the
prefix only changes when a doc actually changes.

## Not changed
Fetching, background refresh, dedup and hosting (`2026-11-05-cloud-knowledge-design.md`).

## Measured (2026-11-05, owner's 4 course docs + test cloud docs, live via the hotkey)
- Sonnet 5.5: 1.6–4.4 s per answer, all correct. The heading index sent the model straight to a
  single `Read` at the right offset (one tool round). Cache read ~8.2k tokens from the second
  request on. Before this change: ~9 s with 2–3 search rounds.
- Gemini 3.8 Flash backup: 4.5–8.2 s, all correct (was ~14 s reading whole docs). Its ~3.5k-token
  prompt is below Gemini's caching minimum, so no cache hits there.
- Free fallback models accept the system-part `cache_control` (tested each one).
