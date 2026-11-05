# Cloud knowledge — design

Date: 2026-11-05. Builds on `feat/conversation-memory`. Approved by owner 2026-11-05.

## Intent

The owner keeps one shared, online set of study materials. During a query, every
ShadowPrompt user's AI can browse that set, and nothing is stored on the user's
disk. The point is reliability: a user who forgot to copy the docs locally still
has them, and the owner fixes or adds a doc once for everyone.

The cloud set sits **beside** each user's local `knowledge/` folder. It does not
replace that folder.

## Goals

- Anyone running ShadowPrompt can read the cloud docs, and only the owner can
  change them. The owner accepts that anyone on the internet who has the URL can
  read them.
- Everyone sees all cloud docs. There are no per-test selections.
- The AI searches first, then reads only a line range, the way Claude Code does.
  It does not pull whole files into context.
- A copy that exists both locally and in the cloud is shown to the AI once.
- Hosting is free, always up, and fast worldwide, with no account beyond GitHub.

## Non-goals

- Embeddings, vector DBs or a RAG service. Agentic text search is enough.
- An upload website. GitHub's web UI ("Add file → Upload files") is the upload page.
- Server-side PDF/Office conversion. The owner stores Markdown/TXT only and
  converts PDFs once before uploading (`markitdown file.pdf -o file.md`).
- OpenRouter's Files API, Files tool or shell containers. They are scoped to one
  OpenRouter workspace, and each user calls OpenRouter with their own key.
  (Verified against OpenRouter docs, 2026-11-05.)

## Anti-goals

- Cloud data must never be written to the user's disk. It lives in memory only.
- A fetch or a failure must never show a window, a console or a prompt.
- Cloud being down or slow must never break a query, or stall it beyond 2s.
- Secrets must never ship. The URL is public and read-only by design.
- Local files are never modified, synced or deleted.

## Server side

- The material lives in **this repo**, under `cloud_docs/materials/`, as `.md`/`.txt`
  only. The owner chose one repo. Text diffs well, so it adds little weight.
- `cloud_docs/build.py` uses only the standard library. It walks `materials/` and
  writes `cloud_docs/public/bundle.json`:

  ```json
  { "built_at": 1762300000,
    "docs": { "biology/cells.md": { "text": "…", "sha256": "…hex of file bytes…",
                                    "modified": 1762200000 } } }
  ```

  `modified` is the unix time of the last git commit that touched the file. When
  the clone is shallow, the script runs `git fetch --unshallow` first. If git has
  no date for a file, it falls back to the build time.
- **GitHub Pages**, built by `.github/workflows/cloud_docs.yml`. It runs on pushes to
  `main` that touch `cloud_docs/**` and does a full-history checkout. Pages and Actions
  are both free for this public repo, which is why it replaced the original Cloudflare
  Pages plan on 2026-11-05 (owner wanted no third-party account). Pages serves gzip and
  ETag. Its cache can add up to 10 minutes before users see an update. The URL is
  `https://hyowonbernabe.github.io/ShadowPrompt/bundle.json`.
- `cloud_docs/README.md` holds the owner's one-time setup clicks and the upload steps.

## Client side

- **Config:** `[knowledge]` gains `cloud_url`. The built-in default is the owner's
  URL, so existing configs get cloud without being edited. An empty string turns
  cloud off.
- **Fetching never blocks a query.**
  - A background refresh starts at launch and at the start of each query. It sends
    a GET with `If-None-Match`. On 304 the app keeps its copy; on 200 it replaces it.
  - Retries happen only on transport errors: 3 attempts with exponential backoff
    (the existing `llm::retry` helper). Each request times out after 5s.
  - After a failed refresh, the app waits 5 minutes before starting another.
  - A tool that finds no copy while a refresh is in flight waits up to 2s. After
    that it carries on with local docs only and notes `[cloud docs unavailable]`.
- reqwest gets its `gzip` feature.

### Deduplication (owner decisions)

Each doc has a hash (SHA-256 of its bytes) and a modified time. For cloud docs
these come from the bundle. For local docs they are the file mtime and a hash
computed when needed. Two checks run in order:

1. **Same hash, any name:** the docs are identical, so keep the local one.
2. **Same file name (case-insensitive, last path part), different hash:** the
   newer modified time wins. On a tie, local wins.

The losing copy is removed from every tool and from the prompt listing.
Known limit, accepted by the owner: a fresh browser download gets today's
mtime. If the download is identical, check 1 catches it. An old version
downloaded today from somewhere else would wrongly win.

Local files that are not valid UTF-8, such as stray PDFs, are skipped. They were
unreadable garbage to the model before this change anyway.

### Tools (extends the existing pair; no parallel cloud tools)

| Tool | Args | Returns |
|---|---|---|
| `list_docs` | none | `local/…` and `cloud/…` paths, with line counts |
| `search_docs` | `pattern` (case-insensitive regex; falls back to a literal match if invalid), `max_results` (default 50) | `path:line: text` lines |
| `read_doc` | `name`, `offset` (1-based, default 1), `limit` (default 200) | numbered lines plus a "more lines" note |

Every result is capped at `max_doc_bytes`.

### Prompt listing (saves a round trip)

When knowledge is enabled, the general system prompt gets the doc list appended,
capped at 300 entries (beyond that, the model is told to use `list_docs`). The
model can then search immediately without first calling `list_docs`.

## Calls made by the designer

- GitHub Pages, with a single `bundle.json`. This holds up to roughly 20MB of
  text; beyond that, split it into one file per doc plus an index.
- No hash cache for local files. Local folders are small and hashing them costs
  milliseconds.
- `local/` and `cloud/` prefixes, a 2s wait and a 5-minute cooldown.
