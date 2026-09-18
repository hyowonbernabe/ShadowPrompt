// Retry policy for transport-level failures only (network errors before any HTTP response
// reaches us). OpenRouter's own `models` array already handles retrying across the fallback
// chain server-side on error/429/downtime (design doc §8) — this is NOT duplicating that; it's
// the layer below it, for when the request never got a response at all.
//
// This distinction is enforced via `CallError`, not just stated in a comment: a real HTTP
// response (any status, even a 4xx/5xx/429) means the request *did* reach OpenRouter and get an
// answer — retrying the exact same request bytes won't change a deterministic outcome, and
// retrying here would just duplicate/race the server-side `models`-array fallback that already
// owns that failure mode. Only a failure that happens before any HTTP response is obtained
// (`reqwest`'s own connect/send/body-read errors) is retried with backoff.
//
// TODO: port the typed error taxonomy from docs/OPENCODE.md §5.5 (Auth / RateLimit / QuotaExceeded
// / InvalidRequest / ContentPolicy / ServerError / Transport) and the secret-redaction pass
// before logging any request/response body — design doc §2 calls this out as cheap and worth
// stealing regardless of provider count. Not yet implemented in this scaffold pass.

use std::future::Future;
use std::time::Duration;

const ATTEMPTS: u32 = 3;

/// A `call()` attempt's outcome, distinguishing *why* it failed so `with_retry` knows whether
/// retrying could plausibly help.
pub enum CallError {
    /// No HTTP response was obtained at all (connection refused/reset, DNS failure, body
    /// dropped mid-read, etc.) — worth retrying, the next attempt might reach the server.
    Transport(anyhow::Error),
    /// A real HTTP response came back (non-2xx status, or a 2xx with an unparseable body) —
    /// retrying the identical request is very unlikely to change the outcome. Not retried.
    Fatal(anyhow::Error),
}

impl CallError {
    pub fn into_error(self) -> anyhow::Error {
        match self {
            CallError::Transport(e) => e,
            CallError::Fatal(e) => e,
        }
    }
}

pub async fn with_retry<T, Fut, F>(mut op: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, CallError>>,
{
    let mut last: Option<anyhow::Error> = None;
    for i in 0..ATTEMPTS {
        match op().await {
            Ok(v) => return Ok(v),
            Err(CallError::Fatal(e)) => {
                log::warn!("fatal (non-retryable) error, not retrying: {e}");
                return Err(e);
            }
            Err(CallError::Transport(e)) => {
                log::warn!("attempt {} failed (transport): {e}", i + 1);
                last = Some(e);
                if i + 1 < ATTEMPTS {
                    tokio::time::sleep(Duration::from_secs(1u64 << i)).await;
                }
            }
        }
    }
    Err(last.unwrap_or_else(|| anyhow::anyhow!("retry exhausted")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Pure logic test (no HTTP involved): confirms `with_retry` actually retries a transport
    /// failure and returns the eventual success, rather than giving up after the first error.
    #[tokio::test]
    async fn retries_transport_failure_then_succeeds() {
        let attempts = AtomicU32::new(0);
        let result = with_retry(|| {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if n == 0 {
                    Err(CallError::Transport(anyhow::anyhow!("simulated connection refused")))
                } else {
                    Ok(42)
                }
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 2, "expected exactly one retry before success");
    }

    /// A fatal (non-transport) error must NOT be retried — it should return immediately on the
    /// first attempt, since a real HTTP response already came back.
    #[tokio::test]
    async fn does_not_retry_fatal_error() {
        let attempts = AtomicU32::new(0);
        let result: anyhow::Result<()> = with_retry(|| {
            attempts.fetch_add(1, Ordering::SeqCst);
            async move { Err(CallError::Fatal(anyhow::anyhow!("openrouter 400: bad request"))) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 1, "fatal error must not be retried");
    }

    /// Exhausting all transport-failure retries surfaces the last error, not a panic/hang.
    #[tokio::test]
    async fn exhausts_attempts_on_persistent_transport_failure() {
        let attempts = AtomicU32::new(0);
        let result: anyhow::Result<()> = with_retry(|| {
            attempts.fetch_add(1, Ordering::SeqCst);
            async move { Err(CallError::Transport(anyhow::anyhow!("still refused"))) }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), ATTEMPTS, "should try exactly ATTEMPTS times");
    }
}
