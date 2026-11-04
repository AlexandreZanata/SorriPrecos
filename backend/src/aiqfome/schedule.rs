use super::{INTERVAL, Source, collect};
use crate::now;
use anyhow::{Result, ensure};
use sqlx::SqlitePool;
use std::{
    collections::HashSet,
    path::Path,
    time::{Duration, Instant},
};

const BATCH: usize = 12;
const BETWEEN_BATCHES: i64 = 120;
const LEASE_SECONDS: i64 = 420;

#[derive(Debug)]
pub(super) struct RateLimited(pub(super) Duration);

impl std::fmt::Display for RateLimited {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "origem solicitou adiamento por HTTP 429/503; aguardar {} segundos",
            self.0.as_secs()
        )
    }
}
impl std::error::Error for RateLimited {}

async fn round(pool: &SqlitePool, client: &reqwest::Client, sources: &[Source]) -> Result<bool> {
    if sources.is_empty() {
        return Ok(false);
    }
    sqlx::query("INSERT INTO local_catalogue_schedule(id) VALUES(1) ON CONFLICT DO NOTHING")
        .execute(pool)
        .await?;
    let at = now();
    let saved: Option<Option<String>> = sqlx::query_scalar(
        "UPDATE local_catalogue_schedule SET lease_until=? WHERE id=1 AND next_at<=? AND lease_until<=? RETURNING last_source",
    ).bind(at+LEASE_SECONDS).bind(at).bind(at).fetch_optional(pool).await?;
    let Some(saved) = saved else { return Ok(false) };
    let start = saved
        .and_then(|id| sources.iter().position(|s| s.supermarket_id == id))
        .map_or(0, |index| (index + 1) % sources.len());
    let mut attempted = 0;
    let mut failed = false;
    let started = Instant::now();
    let result: Result<()> = async {
        for source in sources.iter().cycle().skip(start).take(sources.len()) {
            if attempted >= BATCH || started.elapsed() >= Duration::from_secs(150) { break; }
            let origin = format!("local-food:{}", source.provider.adapter());
            let paused = crate::adapters::origin_paused(pool, &origin).await?;
            let due: bool = sqlx::query_scalar("SELECT NOT EXISTS(SELECT 1 FROM catalogue_collectors WHERE supermarket_id=? AND last_started_at>?)")
                .bind(&source.supermarket_id).bind(now()-INTERVAL).fetch_one(pool).await?;
            if paused || !due { continue; }
            attempted += 1;
            if let Err(error) = collect(pool, client, source).await {
                failed = true;
                tracing::warn!(source=%source.supermarket_id, %error, "public_catalog_failed");
                if let Some(limit) = error.downcast_ref::<RateLimited>() {
                    let retry = now().saturating_add(i64::try_from(limit.0.as_secs()).unwrap_or(i64::MAX));
                    crate::adapters::defer_origin(pool, &origin, retry).await?;
                }
            }
            sqlx::query("UPDATE local_catalogue_schedule SET last_source=? WHERE id=1")
                .bind(&source.supermarket_id).execute(pool).await?;
        }
        Ok(())
    }.await;
    sqlx::query("UPDATE local_catalogue_schedule SET lease_until=0,next_at=? WHERE id=1")
        .bind(now() + BETWEEN_BATCHES)
        .execute(pool)
        .await?;
    result?;
    tracing::info!(
        attempted,
        configured = sources.len(),
        "local_catalogue_batch_complete"
    );
    Ok(failed)
}

pub(super) async fn run(pool: &SqlitePool, path: &Path, watch: bool) -> Result<()> {
    let sources: Vec<Source> = serde_json::from_slice(&tokio::fs::read(path).await?)?;
    let mut ids = HashSet::new();
    ensure!(
        sources
            .iter()
            .all(|source| ids.insert(&source.supermarket_id)),
        "vendedor duplicado"
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(25))
        .redirect(reqwest::redirect::Policy::none())
        .user_agent("SorriPrecos/0.1 (public-storefront-catalog)")
        .build()?;
    loop {
        let failed = round(pool, &client, &sources).await?;
        if !watch {
            ensure!(
                !failed,
                "coleta de comércios locais incompleta; consulte /v1/collectors"
            );
            return Ok(());
        }
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(Duration::from_secs(30)) => {}
        }
    }
}
