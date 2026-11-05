// Usage analytics — our own small PostHog. Design: docs/2026-11-06-analytics-design.md.
// Server side: supabase/migrations/20261106120000_analytics.sql (`sp_track` only accepts events).
//
// Rules this module keeps:
// * `track` never blocks and never fails: it only pushes onto an in-memory queue. A background
//   thread (its own tiny runtime, separate from the app's) sends batches. If the server is down
//   the app behaves exactly as before; events just wait (capped) or are dropped.
// * Nothing the user typed, copied, captured or got as an answer is ever sent — only which
//   feature ran, how long it took, whether it worked, and technical facts (model, tokens, OS).
// * No files: the install id is derived from the machine's own MachineGuid, so a self-delete
//   or panic-wipe leaves no analytics trace behind.

use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const MAX_QUEUED: usize = 1000;
const BATCH: usize = 100;
const SEND_EVERY: Duration = Duration::from_secs(10);
/// Each batch: tried after 0s, 1s, 2s; then it goes back in the queue for the next round.
const ATTEMPTS: u32 = 3;

struct Ids {
    install: String,
    session: String,
    os_build: String,
}

static QUEUE: Mutex<VecDeque<Value>> = Mutex::new(VecDeque::new());
static ACCOUNT: Mutex<Option<String>> = Mutex::new(None);
static IDS: OnceLock<Ids> = OnceLock::new();
static SENDER: OnceLock<()> = OnceLock::new();

fn ids() -> &'static Ids {
    IDS.get_or_init(|| {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        Ids {
            install: machine_guid().map(|g| short_hash(&format!("shadowprompt:{g}"))).unwrap_or_else(|| "unknown".into()),
            session: short_hash(&format!("{}:{now}", std::process::id())),
            os_build: os_build(),
        }
    })
}

fn short_hash(s: &str) -> String {
    hex(&Sha256::digest(s.as_bytes()))[..16].to_string()
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Ties later events to an account. Stores the same SHA-256 the server keys accounts by
/// (`private.code_hash`), never the code itself. Empty code = not logged in.
pub fn set_account(code: &str) {
    let v = (!code.is_empty()).then(|| hex(&Sha256::digest(code.as_bytes())));
    if let Ok(mut a) = ACCOUNT.lock() {
        *a = v;
    }
}

/// Records one event. Never blocks on the network, never panics, never errors.
pub fn track(name: &str, props: Value) {
    let account = ACCOUNT.lock().ok().and_then(|a| a.clone());
    let event = build_event(ids(), name, account, props);
    if let Ok(mut q) = QUEUE.lock() {
        push(&mut q, event);
    }
    start_sender();
}

/// For the panic hook: the crashing thread may itself hold one of our locks (panic = abort runs
/// the hook with locks still held), so only try them, and skip the event rather than hang.
fn track_from_panic(name: &str, props: Value) {
    let Some(ids) = IDS.get() else { return };
    let account = ACCOUNT.try_lock().ok().and_then(|a| a.clone());
    let event = build_event(ids, name, account, props);
    if let Ok(mut q) = QUEUE.try_lock() {
        push(&mut q, event);
    }
}

fn push(q: &mut VecDeque<Value>, event: Value) {
    if q.len() >= MAX_QUEUED {
        q.pop_front();
    }
    q.push_back(event);
}

fn build_event(ids: &Ids, name: &str, account: Option<String>, props: Value) -> Value {
    let at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    json!({
        "name": name,
        "at": at,
        "install": ids.install,
        "session": ids.session,
        "account": account,
        "version": env!("CARGO_PKG_VERSION"),
        "build": if cfg!(feature = "admin") { "admin" } else { "standard" },
        "dev": cfg!(feature = "debug"),
        "os": ids.os_build,
        "props": props,
    })
}

/// Times one feature run. Reports `feature_done` with ok/error when `finish`ed, or
/// `feature_cancelled` if dropped first (aborted by the user or replaced by a newer run).
pub struct Run {
    name: &'static str,
    start: Instant,
    done: bool,
}

pub fn run(name: &'static str) -> Run {
    Run { name, start: Instant::now(), done: false }
}

impl Run {
    pub fn finish<T>(mut self, r: &anyhow::Result<T>) {
        self.done = true;
        let mut props = json!({ "feature": self.name, "ms": self.start.elapsed().as_millis() as u64, "ok": r.is_ok() });
        if let Err(e) = r {
            props["error"] = json!(error_kind(e));
        }
        track("feature_done", props);
    }
}

impl Drop for Run {
    fn drop(&mut self) {
        if !self.done {
            track("feature_cancelled", json!({ "feature": self.name, "ms": self.start.elapsed().as_millis() as u64 }));
        }
    }
}

/// A failure as a fixed category, never the raw error text: raw errors can quote the model's
/// answer, OpenRouter echoing the prompt, form contents or the page address.
pub fn error_kind(e: &anyhow::Error) -> String {
    let text = format!("{e:#}");
    let reason = crate::llm::user_facing_error(e);
    match text.find("openrouter ").and_then(|i| text.get(i + 11..i + 14)).filter(|c| c.bytes().all(|b| b.is_ascii_digit())) {
        Some(code) => format!("{reason} (HTTP {code})"),
        None => reason.to_string(),
    }
}

/// Tries to send what's queued before a normal exit, waiting at most `max`. Only for quiet exits
/// (launch dialog closed): never call it on panic-kill or self-delete, which must stay instant.
pub fn flush_before_exit(max: Duration) {
    let batch: Vec<Value> = match QUEUE.lock() {
        Ok(mut q) => q.drain(..).collect(),
        Err(_) => return,
    };
    send_detached(batch, max);
}

/// Crash report from the panic hook: where it crashed (file:line only; the panic text can
/// contain user data). Never waits on the input hook's own thread: while it waits, Windows'
/// system-wide keyboard/mouse hook would go unanswered and lag the whole PC.
pub fn report_crash(at: Option<String>) {
    track_from_panic("crash", json!({ "at": at }));
    if std::thread::current().name() == Some("rdev-listener") {
        return;
    }
    let batch: Vec<Value> = match QUEUE.try_lock() {
        Ok(mut q) => q.drain(..).collect(),
        Err(_) => return,
    };
    send_detached(batch, Duration::from_secs(1));
}

/// Sends `batch` on a fresh thread and waits for it at most `max` (a real deadline: the thread
/// is left behind, not joined, and dies with the process).
fn send_detached(batch: Vec<Value>, max: Duration) {
    if cfg!(test) || batch.is_empty() {
        return;
    }
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let spawned = std::thread::Builder::new().name("analytics-exit".into()).spawn(move || {
        if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
            rt.block_on(send(&batch));
        }
        let _ = done_tx.send(());
    });
    if spawned.is_ok() {
        let _ = done_rx.recv_timeout(max);
    }
}

fn start_sender() {
    // Unit tests never talk to the real server.
    if cfg!(test) {
        return;
    }
    SENDER.get_or_init(|| {
        let _ = std::thread::Builder::new().name("analytics".into()).spawn(sender_loop);
    });
}

fn sender_loop() {
    let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() else { return };
    loop {
        std::thread::sleep(SEND_EVERY);
        loop {
            let batch: Vec<Value> = match QUEUE.lock() {
                Ok(mut q) => {
                    let n = q.len().min(BATCH);
                    q.drain(..n).collect()
                }
                Err(_) => return,
            };
            if batch.is_empty() {
                break;
            }
            let full = batch.len() == BATCH;
            if !rt.block_on(send_with_retry(&batch)) {
                // Server unreachable: put the batch back (oldest dropped past the cap) and wait
                // for the next round.
                if let Ok(mut q) = QUEUE.lock() {
                    for e in batch.into_iter().rev() {
                        q.push_front(e);
                    }
                    q.truncate(MAX_QUEUED);
                }
                break;
            }
            if !full {
                break;
            }
        }
    }
}

async fn send_with_retry(batch: &[Value]) -> bool {
    for attempt in 0..ATTEMPTS {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_secs(1 << (attempt - 1))).await;
        }
        if send(batch).await {
            return true;
        }
    }
    false
}

async fn send(batch: &[Value]) -> bool {
    let Ok(http) = reqwest::Client::builder().connect_timeout(Duration::from_secs(8)).timeout(Duration::from_secs(15)).build() else {
        return false;
    };
    let resp = http
        .post(format!("{}/rest/v1/rpc/sp_track", crate::account::SUPABASE_URL))
        .header("apikey", crate::account::SUPABASE_PUBLISHABLE_KEY)
        .json(&json!({ "p_events": batch }))
        .send()
        .await;
    // A 4xx means the server rejected the batch itself; resending it won't help, so count it as
    // handled (dropped) rather than retrying forever.
    matches!(resp, Ok(r) if !r.status().is_server_error())
}

/// HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid — stable per Windows install, readable
/// without admin. Only ever sent hashed.
fn machine_guid() -> Option<String> {
    reg_string(windows::core::w!("SOFTWARE\\Microsoft\\Cryptography"), windows::core::w!("MachineGuid"))
}

fn os_build() -> String {
    let key = windows::core::w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
    let name = reg_string(key, windows::core::w!("ProductName")).unwrap_or_default();
    let display = reg_string(key, windows::core::w!("DisplayVersion")).unwrap_or_default();
    let build = reg_string(key, windows::core::w!("CurrentBuild")).unwrap_or_default();
    format!("{name} {display} ({build})").trim().to_string()
}

fn reg_string(subkey: windows::core::PCWSTR, value: windows::core::PCWSTR) -> Option<String> {
    use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};
    let mut buf = [0u16; 256];
    let mut size = std::mem::size_of_val(&buf) as u32;
    // SAFETY: buf/size describe a valid writable buffer; RegGetValueW null-terminates within it.
    let status = unsafe {
        RegGetValueW(HKEY_LOCAL_MACHINE, subkey, value, RRF_RT_REG_SZ, None, Some(buf.as_mut_ptr().cast()), Some(&mut size))
    };
    if status.is_err() {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..len]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_queue_without_network_and_cap_holds() {
        set_account("2244644");
        for i in 0..(MAX_QUEUED + 5) {
            track("test_event", json!({ "i": i }));
        }
        // Other tests may track events in parallel, so look for ours rather than at positions.
        let q = QUEUE.lock().unwrap();
        assert!(q.len() <= MAX_QUEUED);
        let ours = q.iter().rev().find(|e| e["name"] == "test_event").unwrap();
        // Hash only — the code itself never appears.
        assert_eq!(ours["account"].as_str().map(str::len), Some(64));
        assert!(!q.iter().any(|e| e.to_string().contains("2244644")));
    }

    #[test]
    fn errors_are_categories_not_raw_text() {
        let e = anyhow::anyhow!("openrouter 401: {{\"error\":\"my secret question text\"}}");
        assert_eq!(error_kind(&e), "API key invalid or expired: log in again (HTTP 401)");
        let e = anyhow::anyhow!("parse SSE chunk: bad; data=the model's private answer");
        assert_eq!(error_kind(&e), "Request failed");
    }

    #[test]
    fn os_and_install_ids_are_filled() {
        assert!(!os_build().is_empty());
        assert_eq!(ids().install.len(), 16);
    }
}
