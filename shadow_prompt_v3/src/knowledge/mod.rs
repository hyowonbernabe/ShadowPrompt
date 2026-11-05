// Memory capability — design doc §4, extended by docs/superpowers/specs/2026-11-05-cloud-knowledge-design.md.
// Two sources merged into one view: the sandboxed local `knowledge/` folder (`local/…`) and the
// owner's cloud bundle (`cloud/…`), held in memory only and refreshed in the background. The
// model never names a filesystem path directly: every tool looks names up in the merged list,
// so nothing outside the sandbox (or outside the bundle) is reachable.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, UNIX_EPOCH};

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::llm::messages::{ToolDef, ToolFunctionDef};
use crate::llm::retry::{with_retry, CallError};
use crate::llm::Tool;

/// Connect, and each read of the body: a slow-but-moving download of a big bundle still finishes.
const FETCH_TIMEOUT: Duration = Duration::from_secs(5);
/// Hard ceiling on one whole download attempt.
const FETCH_TOTAL_CAP: Duration = Duration::from_secs(120);
const FAILURE_COOLDOWN: Duration = Duration::from_secs(5 * 60);
const MAX_CLOUD_WAIT: Duration = Duration::from_secs(2);
const DEFAULT_MAX_RESULTS: usize = 50;
const DEFAULT_READ_LIMIT: usize = 200;
const PROMPT_LISTING_CAP: usize = 300;

#[derive(Debug, Clone, Deserialize)]
pub struct CloudDoc {
    pub text: String,
    /// sha256 hex of `text` with CRLF normalised to LF — cloud_docs/build.py computes the same.
    pub sha256: String,
    /// Unix seconds of the last git commit touching the file.
    pub modified: u64,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Bundle {
    pub docs: BTreeMap<String, CloudDoc>,
}

#[derive(Default)]
struct CloudState {
    bundle: Option<Bundle>,
    etag: Option<String>,
    in_flight: bool,
    retry_after: Option<Instant>,
    last_error: Option<String>,
    /// A tool already waited the full `MAX_CLOUD_WAIT` on the current refresh: later tool calls
    /// (several per query) don't wait again, so a slow network costs a query 2s at most.
    waited_out: bool,
}

pub struct Doc {
    pub name: String,
    pub text: String,
}

struct LocalDoc {
    rel: String,
    text: String,
    sha256: String,
    modified: u64,
}

pub struct KnowledgeStore {
    root: PathBuf,
    max_doc_bytes: usize,
    cloud_url: String,
    http: reqwest::Client,
    cloud: Mutex<CloudState>,
}

impl KnowledgeStore {
    pub fn new(root: PathBuf, max_doc_bytes: usize, cloud_url: String) -> anyhow::Result<Self> {
        let http = reqwest::Client::builder()
            .connect_timeout(FETCH_TIMEOUT)
            .read_timeout(FETCH_TIMEOUT)
            .timeout(FETCH_TOTAL_CAP)
            .build()?;
        Ok(Self { root, max_doc_bytes, cloud_url, http, cloud: Mutex::new(CloudState::default()) })
    }

    /// Starts a background refresh of the cloud bundle unless cloud is off, one is already
    /// running, or the last one failed less than `FAILURE_COOLDOWN` ago. Never blocks.
    pub fn refresh_in_background(self: &Arc<Self>) {
        if self.cloud_url.is_empty() {
            return;
        }
        let etag = {
            let mut s = self.cloud.lock().unwrap();
            if s.in_flight || s.retry_after.is_some_and(|t| Instant::now() < t) {
                return;
            }
            s.in_flight = true;
            s.waited_out = false;
            s.etag.clone()
        };
        let me = self.clone();
        tokio::spawn(async move {
            let result = me.fetch(etag).await;
            let mut s = me.cloud.lock().unwrap();
            s.in_flight = false;
            match result {
                Ok(Some((bundle, etag))) => {
                    log::info!("cloud knowledge: {} docs", bundle.docs.len());
                    s.bundle = Some(bundle);
                    s.etag = etag;
                    s.last_error = None;
                }
                Ok(None) => s.last_error = None, // 304: in-memory copy is current
                Err(e) => {
                    log::warn!("cloud knowledge fetch failed: {e}");
                    s.retry_after = Some(Instant::now() + FAILURE_COOLDOWN);
                    s.last_error = Some(e.to_string());
                }
            }
        });
    }

    /// `Ok(None)` means 304 Not Modified.
    async fn fetch(&self, etag: Option<String>) -> anyhow::Result<Option<(Bundle, Option<String>)>> {
        with_retry(|| {
            let etag = etag.clone();
            async move {
                let mut req = self.http.get(&self.cloud_url);
                if let Some(e) = &etag {
                    req = req.header(reqwest::header::IF_NONE_MATCH, e);
                }
                let resp = req.send().await.map_err(|e| CallError::Transport(e.into()))?;
                if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
                    return Ok(None);
                }
                if !resp.status().is_success() {
                    return Err(CallError::Fatal(anyhow::anyhow!("HTTP {}", resp.status())));
                }
                let new_etag =
                    resp.headers().get(reqwest::header::ETAG).and_then(|v| v.to_str().ok()).map(str::to_string);
                let body = resp.bytes().await.map_err(|e| CallError::Transport(e.into()))?;
                let bundle: Bundle =
                    serde_json::from_slice(&body).map_err(|e| CallError::Fatal(anyhow::anyhow!("bad bundle: {e}")))?;
                Ok(Some((bundle, new_etag)))
            }
        })
        .await
    }

    /// Waits up to `MAX_CLOUD_WAIT`, and only when there's no copy yet and a refresh is running.
    pub async fn wait_for_cloud(&self) {
        let deadline = Instant::now() + MAX_CLOUD_WAIT;
        // ponytail: 50ms poll instead of a Notify — race-free and only ever runs during the first
        // ~second after launch.
        while Instant::now() < deadline {
            {
                let s = self.cloud.lock().unwrap();
                if s.bundle.is_some() || !s.in_flight || s.waited_out {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        self.cloud.lock().unwrap().waited_out = true;
    }

    #[cfg(test)]
    pub fn set_cloud_for_test(&self, bundle: Bundle) {
        self.cloud.lock().unwrap().bundle = Some(bundle);
    }

    /// The merged, deduplicated doc list plus a note when cloud is on but has no copy.
    pub fn docs(&self) -> (Vec<Doc>, Option<String>) {
        let local = self.local_docs();
        let s = self.cloud.lock().unwrap();
        let note = match (&s.bundle, self.cloud_url.is_empty()) {
            (None, false) => Some(format!(
                "[cloud docs unavailable: {}]",
                s.last_error.as_deref().unwrap_or("still loading")
            )),
            _ => None,
        };
        let empty = BTreeMap::new();
        let cloud = s.bundle.as_ref().map_or(&empty, |b| &b.docs);

        // Dedup — spec "Deduplication": (1) same hash, any name → identical, keep local;
        // (2) same file name, different hash → newer modified wins, tie → local.
        // ponytail: O(local × cloud) pairwise scan; fine for hundreds of docs, index by hash/name
        // if collections ever reach the thousands.
        let mut keep_local = vec![true; local.len()];
        let mut out = Vec::new();
        for (cname, c) in cloud {
            let mut keep_cloud = true;
            for (i, l) in local.iter().enumerate() {
                if l.sha256 == c.sha256 {
                    keep_cloud = false;
                } else if file_name_lower(&l.rel) == file_name_lower(cname) {
                    if c.modified > l.modified {
                        keep_local[i] = false;
                    } else {
                        keep_cloud = false;
                    }
                }
            }
            if keep_cloud {
                out.push(Doc { name: format!("cloud/{cname}"), text: c.text.clone() });
            }
        }
        drop(s);
        for (l, keep) in local.into_iter().zip(keep_local) {
            if keep {
                out.push(Doc { name: format!("local/{}", l.rel), text: l.text });
            }
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        (out, note)
    }

    fn local_docs(&self) -> Vec<LocalDoc> {
        let mut paths = Vec::new();
        walk(&self.root, &mut paths);
        let mut out = Vec::new();
        for path in paths {
            let Ok(rel) = path.strip_prefix(&self.root) else { continue };
            let Ok(bytes) = std::fs::read(&path) else { continue };
            // Non-UTF-8 (stray PDFs etc.) were unreadable garbage to the model anyway — skip.
            let Ok(text) = String::from_utf8(bytes) else { continue };
            let text = text.replace("\r\n", "\n");
            let modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            out.push(LocalDoc {
                rel: rel.to_string_lossy().replace('\\', "/"),
                sha256: format!("{:x}", Sha256::digest(text.as_bytes())),
                text,
                modified,
            });
        }
        out
    }

    /// Appended to the system prompt so the model can search without a `list_docs` round trip.
    pub fn prompt_listing(&self) -> String {
        let (docs, note) = self.docs();
        if docs.is_empty() {
            return note.map_or_else(String::new, |n| format!("

KNOWLEDGE DOCS: none yet {n}; list_docs may find them shortly.
"));
        }
        let mut s = String::from(
            "\n\nKNOWLEDGE DOCS (search with search_docs, then read only the relevant lines with read_doc):\n",
        );
        for d in docs.iter().take(PROMPT_LISTING_CAP) {
            s.push_str(&format!("- {} ({} lines)\n", d.name, d.text.lines().count()));
        }
        if docs.len() > PROMPT_LISTING_CAP {
            s.push_str(&format!("…and {} more; call list_docs for all.\n", docs.len() - PROMPT_LISTING_CAP));
        }
        s
    }

    fn cap(&self, mut s: String) -> String {
        if s.len() > self.max_doc_bytes {
            let mut end = self.max_doc_bytes;
            while !s.is_char_boundary(end) {
                end -= 1;
            }
            let cut = s.len() - end;
            s.truncate(end);
            s.push_str(&format!("\n[...{cut} bytes truncated at the {}-byte cap]", self.max_doc_bytes));
        }
        s
    }
}

fn file_name_lower(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_lowercase()
}

/// Regular files under `dir`, recursively. Symlinks are skipped so nothing outside the sandbox
/// is reachable through one.
fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else { continue };
        if ft.is_dir() {
            walk(&entry.path(), out);
        } else if ft.is_file() {
            out.push(entry.path());
        }
    }
}

fn tool_def(name: &str, description: &str, parameters: serde_json::Value) -> ToolDef {
    ToolDef { kind: "function", function: ToolFunctionDef { name: name.to_string(), description: description.to_string(), parameters } }
}

fn with_note(body: String, note: Option<String>) -> String {
    match note {
        Some(n) => format!("{body}\n{n}"),
        None => body,
    }
}

pub struct ListDocsTool(pub Arc<KnowledgeStore>);
pub struct SearchDocsTool(pub Arc<KnowledgeStore>);
pub struct ReadDocTool(pub Arc<KnowledgeStore>);

#[async_trait]
impl Tool for ListDocsTool {
    fn name(&self) -> &'static str {
        "list_docs"
    }

    fn definition(&self) -> ToolDef {
        tool_def(self.name(), "List every knowledge document (local/… and cloud/…) with its line count.", json!({"type": "object", "properties": {}}))
    }

    async fn execute(&self, _args_json: &str) -> anyhow::Result<String> {
        self.0.wait_for_cloud().await;
        let (docs, note) = self.0.docs();
        let body: Vec<String> = docs.iter().map(|d| format!("{} ({} lines)", d.name, d.text.lines().count())).collect();
        Ok(self.0.cap(with_note(body.join("\n"), note)))
    }
}

#[async_trait]
impl Tool for SearchDocsTool {
    fn name(&self) -> &'static str {
        "search_docs"
    }

    fn definition(&self) -> ToolDef {
        tool_def(
            self.name(),
            "Search all knowledge documents for a case-insensitive regex. Returns `name:line: text` matches. \
             Use this first, then read_doc around the matching lines.",
            json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string" },
                    "max_results": { "type": "integer", "description": "default 50" }
                },
                "required": ["pattern"]
            }),
        )
    }

    async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Args {
            pattern: String,
            max_results: Option<usize>,
        }
        let args: Args = serde_json::from_str(args_json).map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))?;
        let max = args.max_results.unwrap_or(DEFAULT_MAX_RESULTS).max(1);
        // A model-written pattern that isn't valid regex (e.g. `C++`) is searched literally.
        let re = regex::RegexBuilder::new(&args.pattern)
            .case_insensitive(true)
            .build()
            .or_else(|_| regex::RegexBuilder::new(&regex::escape(&args.pattern)).case_insensitive(true).build())?;
        self.0.wait_for_cloud().await;
        let (docs, note) = self.0.docs();
        let mut hits = Vec::new();
        let mut total = 0;
        for d in &docs {
            for (i, line) in d.text.lines().enumerate() {
                if re.is_match(line) {
                    total += 1;
                    if hits.len() < max {
                        hits.push(format!("{}:{}: {}", d.name, i + 1, line));
                    }
                }
            }
        }
        let mut body = if hits.is_empty() { "no matches".to_string() } else { hits.join("\n") };
        if total > hits.len() {
            body.push_str(&format!("\n[{} more matches not shown; narrow the pattern]", total - hits.len()));
        }
        Ok(self.0.cap(with_note(body, note)))
    }
}

#[async_trait]
impl Tool for ReadDocTool {
    fn name(&self) -> &'static str {
        "read_doc"
    }

    fn definition(&self) -> ToolDef {
        tool_def(
            self.name(),
            "Read lines of one knowledge document by its exact name (as listed). Returns numbered lines.",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "offset": { "type": "integer", "description": "1-based first line, default 1" },
                    "limit": { "type": "integer", "description": "number of lines, default 200" }
                },
                "required": ["name"]
            }),
        )
    }

    async fn execute(&self, args_json: &str) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Args {
            name: String,
            offset: Option<usize>,
            limit: Option<usize>,
        }
        let args: Args = serde_json::from_str(args_json).map_err(|e| anyhow::anyhow!("invalid arguments: {e}"))?;
        self.0.wait_for_cloud().await;
        let (docs, _) = self.0.docs();
        let doc = docs.iter().find(|d| d.name == args.name).ok_or_else(|| anyhow::anyhow!("no such document: {}", args.name))?;
        let lines: Vec<&str> = doc.text.lines().collect();
        let start = args.offset.unwrap_or(1).max(1);
        if start > lines.len() {
            return Ok(format!("[offset {start} is past the end; {} has {} lines]", doc.name, lines.len()));
        }
        let end = (start - 1).saturating_add(args.limit.unwrap_or(DEFAULT_READ_LIMIT)).min(lines.len());
        let mut out: Vec<String> = (start..=end).map(|n| format!("{n:>6}\t{}", lines[n - 1])).collect();
        if end < lines.len() {
            out.push(format!("[{} more lines; call read_doc with offset={}]", lines.len() - end, end + 1));
        }
        Ok(self.0.cap(out.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct TempDir(PathBuf);
    impl TempDir {
        fn new(tag: &str) -> Self {
            let nanos = std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
            let p = std::env::temp_dir().join(format!("sp_knowledge_{tag}_{}_{nanos}", std::process::id()));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn write(&self, rel: &str, bytes: &[u8], mtime: u64) {
            let p = self.0.join(rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, bytes).unwrap();
            let f = std::fs::File::options().write(true).open(&p).unwrap();
            f.set_modified(UNIX_EPOCH + Duration::from_secs(mtime)).unwrap();
        }
    }
    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn a_stuck_refresh_costs_one_two_second_wait_not_one_per_tool_call() {
        let dir = TempDir::new("waitonce");
        let k = KnowledgeStore::new(dir.0.clone(), 100_000, "http://unused".into()).unwrap();
        k.cloud.lock().unwrap().in_flight = true; // a fetch that never finishes
        let t = Instant::now();
        k.wait_for_cloud().await;
        assert!(t.elapsed() >= MAX_CLOUD_WAIT);
        let t = Instant::now();
        k.wait_for_cloud().await;
        k.wait_for_cloud().await;
        assert!(t.elapsed() < Duration::from_millis(200), "later calls on the same refresh don't wait");
    }

    fn sha(t: &str) -> String {
        format!("{:x}", Sha256::digest(t.as_bytes()))
    }

    fn cloud(docs: &[(&str, &str, u64)]) -> Bundle {
        Bundle {
            docs: docs
                .iter()
                .map(|(n, t, m)| (n.to_string(), CloudDoc { text: t.to_string(), sha256: sha(t), modified: *m }))
                .collect(),
        }
    }

    fn store(dir: &TempDir, url: &str, max: usize) -> Arc<KnowledgeStore> {
        Arc::new(KnowledgeStore::new(dir.0.clone(), max, url.to_string()).unwrap())
    }

    fn names(s: &KnowledgeStore) -> Vec<String> {
        s.docs().0.into_iter().map(|d| d.name).collect()
    }

    #[test]
    fn dedup_same_hash_keeps_local_even_with_crlf_and_other_name() {
        let d = TempDir::new("hash");
        d.write("mine.md", b"a\r\nb\r\n", 100);
        let s = store(&d, "", 100_000);
        s.set_cloud_for_test(cloud(&[("theirs.md", "a\nb\n", 999)]));
        assert_eq!(names(&s), vec!["local/mine.md"]);
    }

    #[test]
    fn dedup_same_name_newer_wins() {
        let d = TempDir::new("newer");
        d.write("cells.md", b"old", 100);
        d.write("x/Genes.MD", b"new local", 300);
        let s = store(&d, "", 100_000);
        s.set_cloud_for_test(cloud(&[("bio/cells.md", "new cloud", 200), ("genes.md", "old cloud", 200)]));
        assert_eq!(names(&s), vec!["cloud/bio/cells.md", "local/x/Genes.MD"]);
    }

    #[test]
    fn dedup_tie_local_wins_and_different_names_both_shown() {
        let d = TempDir::new("tie");
        d.write("a.md", b"local a", 200);
        let s = store(&d, "", 100_000);
        s.set_cloud_for_test(cloud(&[("a.md", "cloud a", 200), ("b.md", "cloud b", 1)]));
        assert_eq!(names(&s), vec!["cloud/b.md", "local/a.md"]);
    }

    #[test]
    fn non_utf8_local_skipped() {
        let d = TempDir::new("bin");
        d.write("x.pdf", &[0xff, 0xfe, 0x00, 0x80], 1);
        d.write("ok.txt", b"fine", 1);
        assert_eq!(names(&store(&d, "", 100_000)), vec!["local/ok.txt"]);
    }

    #[tokio::test]
    async fn search_invalid_regex_falls_back_to_literal_and_caps_results() {
        let d = TempDir::new("search");
        d.write("c.md", b"I like C++\nc++ again\nnothing\nC++ thrice", 1);
        let s = store(&d, "", 100_000);
        let out = SearchDocsTool(s.clone()).execute(r#"{"pattern":"C++","max_results":2}"#).await.unwrap();
        assert_eq!(out, "local/c.md:1: I like C++\nlocal/c.md:2: c++ again\n[1 more matches not shown; narrow the pattern]");
    }

    #[tokio::test]
    async fn read_doc_offset_limit_and_past_end() {
        let d = TempDir::new("read");
        d.write("n.md", b"one\ntwo\nthree\nfour", 1);
        let s = store(&d, "", 100_000);
        let r = ReadDocTool(s.clone());
        let out = r.execute(r#"{"name":"local/n.md","offset":2,"limit":2}"#).await.unwrap();
        assert_eq!(out, "     2\ttwo\n     3\tthree\n[1 more lines; call read_doc with offset=4]");
        let past = r.execute(r#"{"name":"local/n.md","offset":9}"#).await.unwrap();
        assert_eq!(past, "[offset 9 is past the end; local/n.md has 4 lines]");
    }

    #[tokio::test]
    async fn rejects_names_not_in_the_list() {
        let d = TempDir::new("trav");
        d.write("n.md", b"x", 1);
        let r = ReadDocTool(store(&d, "", 100_000));
        assert!(r.execute(r#"{"name":"local/../n.md"}"#).await.is_err());
        assert!(r.execute(r#"{"name":"n.md"}"#).await.is_err());
    }

    #[test]
    fn prompt_listing_caps_at_300() {
        let d = TempDir::new("listing");
        for i in 0..302 {
            d.write(&format!("d{i:03}.md"), format!("doc {i}").as_bytes(), 1);
        }
        let l = store(&d, "", 100_000).prompt_listing();
        assert!(l.contains("- local/d000.md (1 lines)"));
        assert!(!l.contains("d300.md"));
        assert!(l.ends_with("…and 2 more; call list_docs for all.\n"));
        assert_eq!(store(&TempDir::new("empty"), "", 100_000).prompt_listing(), "");
    }

    #[tokio::test]
    async fn output_is_capped_at_max_doc_bytes() {
        let d = TempDir::new("cap");
        d.write("big.md", "y".repeat(500).as_bytes(), 1);
        let out = ReadDocTool(store(&d, "", 100)).execute(r#"{"name":"local/big.md"}"#).await.unwrap();
        assert!(out.starts_with("     1\t"));
        assert!(out.contains("bytes truncated at the 100-byte cap"));
    }

    async fn settle(s: &KnowledgeStore) {
        for _ in 0..200 {
            if !s.cloud.lock().unwrap().in_flight {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("refresh never finished");
    }

    #[tokio::test]
    async fn fetch_200_then_304_keeps_copy() {
        let server = MockServer::start().await;
        let body = json!({"built_at": 1, "docs": {"g.md": {"text": "cloud text", "sha256": sha("cloud text"), "modified": 5}}});
        Mock::given(method("GET"))
            .and(header("if-none-match", "\"v1\""))
            .respond_with(ResponseTemplate::new(304))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body).insert_header("etag", "\"v1\""))
            .expect(1)
            .mount(&server)
            .await;
        let d = TempDir::new("fetch");
        let s = store(&d, &server.uri(), 100_000);
        s.refresh_in_background();
        settle(&s).await;
        assert_eq!(names(&s), vec!["cloud/g.md"]);
        s.refresh_in_background();
        settle(&s).await;
        assert_eq!(names(&s), vec!["cloud/g.md"], "304 keeps the in-memory copy");
    }

    #[tokio::test]
    async fn fetch_failure_keeps_local_working_and_starts_cooldown() {
        let server = MockServer::start().await;
        Mock::given(method("GET")).respond_with(ResponseTemplate::new(500)).expect(1).mount(&server).await;
        let d = TempDir::new("fail");
        d.write("l.md", b"local", 1);
        let s = store(&d, &server.uri(), 100_000);
        s.refresh_in_background();
        settle(&s).await;
        let out = ListDocsTool(s.clone()).execute("{}").await.unwrap();
        assert_eq!(out, "local/l.md (1 lines)\n[cloud docs unavailable: HTTP 500 Internal Server Error]");
        s.refresh_in_background(); // within cooldown: must not hit the server again (expect(1))
        settle(&s).await;
    }

    #[tokio::test]
    async fn empty_url_never_fetches_and_has_no_note() {
        let d = TempDir::new("off");
        let s = store(&d, "", 100_000);
        s.refresh_in_background();
        assert!(!s.cloud.lock().unwrap().in_flight);
        assert!(s.docs().1.is_none());
    }
}
