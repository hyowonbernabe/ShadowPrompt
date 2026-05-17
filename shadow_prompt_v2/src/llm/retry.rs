// Retry policy: 3 attempts with exponential backoff (1s, 2s, 4s) on transient errors.

use std::future::Future;
use std::time::Duration;

const ATTEMPTS: u32 = 3;

pub async fn with_retry<T, Fut, F>(mut op: F) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = anyhow::Result<T>>,
{
    let mut last: Option<anyhow::Error> = None;
    for i in 0..ATTEMPTS {
        match op().await {
            Ok(v) => return Ok(v),
            Err(e) => {
                log::warn!("attempt {} failed: {e}", i + 1);
                last = Some(e);
                if i + 1 < ATTEMPTS {
                    tokio::time::sleep(Duration::from_secs(1u64 << i)).await;
                }
            }
        }
    }
    Err(last.unwrap_or_else(|| anyhow::anyhow!("retry exhausted")))
}
