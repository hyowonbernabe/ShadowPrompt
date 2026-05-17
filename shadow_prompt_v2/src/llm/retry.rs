// Retry policy: 3 attempts with exponential backoff (1s, 2s, 4s) on 429/5xx.

pub async fn with_retry<T, Fut>(_op: impl Fn() -> Fut) -> anyhow::Result<T>
where
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    anyhow::bail!("not yet implemented")
}
