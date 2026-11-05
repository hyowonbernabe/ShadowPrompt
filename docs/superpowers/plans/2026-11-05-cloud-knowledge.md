# Cloud Knowledge Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let every ShadowPrompt user's AI search and read an owner-published cloud doc set beside their local `knowledge/` folder, deduplicated.

**Architecture:** A stdlib Python script bundles `cloud_docs/materials/` into one `bundle.json`, which Cloudflare Pages serves. `KnowledgeStore` keeps that bundle in memory, refreshed in the background with an ETag. It merges the bundle with local docs (dedup by hash, then same name → newer mtime) and serves `list_docs`, `search_docs` and `read_doc`.

**Tech Stack:** Rust (tokio, reqwest+gzip, regex, sha2), Python 3 stdlib, Cloudflare Pages.

**Spec:** `docs/superpowers/specs/2026-11-05-cloud-knowledge-design.md`

## Global Constraints

- No cloud data written to disk. Memory only.
- A query waits at most 2s for cloud. Each request times out after 5s. Retries: 3 attempts via `llm::retry::with_retry`. After a failed refresh, wait 5 minutes before the next.
- `cloud_url = ""` turns cloud off. The built-in default is `DEFAULT_CLOUD_URL` (empty until the owner's Pages URL exists).
- Paths are prefixed `local/` and `cloud/`. Every tool output is capped at `max_doc_bytes`.
- Defaults: search `max_results` = 50, read `limit` = 200 lines, prompt listing capped at 300 entries.
- No new crates beyond `regex`, `sha2`, and reqwest's `gzip` feature.

## Review Focus

- A regex the model writes that is invalid (e.g. `C++`) → treat it as a literal, never error out. Tested in Task 2.
- `read_doc` with an offset past the end → returns an empty-range note, not a panic. Tested in Task 2.
- Cloud returns 500 or bad JSON → the tools still list local docs, plus the unavailable note. Tested in Task 3.
- Local file is not UTF-8 → skipped, never shown. Tested in Task 2.
- Path traversal (`local/../x`) → rejected, as today. Tested in Task 2.

---

### Task 1: Server bundle script + owner README

**Files:** Create `cloud_docs/build.py`, `cloud_docs/README.md`, `cloud_docs/materials/README.md` (placeholder so the folder exists; build.py skips it, since it is the folder's own README).

- [ ] Write `build.py`. If the repo is shallow, run `git fetch --unshallow` (ignore failure). Walk `materials/` for `.md` and `.txt`. For each file record `{text, sha256 (hex of bytes), modified}`, where `modified` comes from `git log -1 --format=%ct -- <file>` and falls back to build time. Write `public/bundle.json` with `built_at`.
- [ ] Run it locally on a sample file. Check the JSON has the right hash (`python -c` against hashlib) and the git date.
- [ ] Write the README: one-time Cloudflare setup (root `cloud_docs`, build `python build.py`, output `public`, watch path `cloud_docs/*`), how to upload on GitHub, and `markitdown x.pdf -o x.md`.
- [ ] Commit.

### Task 2: Merged store + tools (local + injected cloud bundle)

**Files:** Modify `shadow_prompt_v3/src/knowledge/mod.rs`, `Cargo.toml` (`regex`, `sha2`). The tests live in the same file.

**Produces:**
- `pub struct CloudDoc { pub text: String, pub sha256: String, pub modified: u64 }`
- `pub struct Bundle { pub docs: BTreeMap<String, CloudDoc> }` (serde)
- `KnowledgeStore::docs(&self) -> (Vec<Doc>, Option<String>)`: merged, deduped docs plus an optional cloud-unavailable note. `Doc { name: String, text: String }`.
- `KnowledgeStore::set_cloud_for_test(&self, Bundle)` (`cfg(test)`).
- Tools `ListDocsTool`, `SearchDocsTool`, `ReadDocTool`. `fn prompt_listing(&self) -> String`.

- [ ] Tests. Name each one and assert on it:
  - `dedup_same_hash_keeps_local`
  - `dedup_same_name_newer_wins_cloud`
  - `dedup_same_name_newer_wins_local`
  - `dedup_tie_local_wins`
  - `different_names_both_shown`
  - `non_utf8_local_skipped`
  - `search_invalid_regex_falls_back_to_literal`
  - `search_caps_results`
  - `read_doc_offset_limit_numbers_lines`
  - `read_doc_offset_past_end`
  - `rejects_traversal`
  - `prompt_listing_caps_at_300`
- [ ] Run them and watch them fail. Implement. Run `cargo test knowledge`, expect pass.
- [ ] Update the existing `clipboard_query` round-trip test, since names are now `local/…` and there are 3 tools.
- [ ] Commit.

### Task 3: Background cloud refresh

**Files:** Create `shadow_prompt_v3/src/knowledge/cloud.rs`. Modify `knowledge/mod.rs`, `config/schema.rs` (add `cloud_url` with `DEFAULT_CLOUD_URL`), `config/config.example.toml`, `lib.rs` (pass the URL, refresh at launch), `Cargo.toml` (reqwest `gzip`).

**Produces:** `KnowledgeStore::new(root, max_doc_bytes, cloud_url: String)`, `fn refresh_in_background(self: &Arc<Self>)`, `async fn wait_for_cloud(&self)` (at most 2s, only when there is no copy and a refresh is in flight).

- [ ] wiremock tests:
  - `fetch_200_then_304_keeps_copy` asserts the second request sent `If-None-Match`.
  - `fetch_500_tools_still_list_local_with_note`.
  - `failure_starts_cooldown` asserts that a second refresh within 5 minutes sends no request.
  - `empty_url_never_fetches`.
- [ ] Implement, run `cargo test knowledge`, commit.

### Task 4: Wire into queries + prompt listing

**Files:** Modify `shadow_prompt_v3/src/actions/clipboard_query.rs` (`memory_tools` adds `SearchDocsTool` and calls `refresh_in_background`; `ask_in_conversation` appends `prompt_listing()` to `general_prompt()` when knowledge is enabled).

- [ ] Run the full `cargo test`, `cargo build --release` and `cargo clippy`, and get them clean. Commit.
