//! Traverse the public product sitemaps beyond the legacy search offset ceiling.
use super::{BRANCHES, Branch, INTERVAL, local_price, row, simulated_item};
use crate::{
    catalog::{self, ImportRow},
    now,
};
use anyhow::{Context, Result, ensure};
use reqwest::{Client, RequestBuilder, StatusCode, header::RETRY_AFTER};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::{collections::BTreeMap, time::Duration};
use url::Url;

const DISCOVERY_PER_ROUND: usize = 240;
const MAX_DISCOVERY_SKUS: usize = 960;
const REFRESH_PER_BRANCH: i64 = 600;
const CHECKOUT_BATCH: usize = 12;
const BASIS: &str = "PUBLIC_STOREFRONT: índice VTEX e retirada verificada em Sorriso-MT";

#[derive(Default, Deserialize, Serialize)]
#[serde(default)]
struct Checkpoint {
    shards: Vec<String>,
    shard: usize,
    products: Vec<String>,
    next: usize,
    next_sku: usize,
    sweeps: u64,
}

fn sitemap_urls(body: &str, host: &str, shards: bool) -> Vec<String> {
    let mut urls: Vec<_> = super::super::LOC
        .captures_iter(body)
        .filter_map(|entry| Url::parse(&entry[1]).ok())
        .filter(|url| {
            url.scheme() == "https"
                && url.host_str() == Some(host)
                && url.port().is_none()
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none()
                && if shards {
                    url.path()
                        .strip_prefix("/sitemap/product-")
                        .and_then(|name| name.strip_suffix(".xml"))
                        .is_some_and(|number| {
                            !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit())
                        })
                } else {
                    url.path()
                        .strip_suffix("/p")
                        .and_then(|path| path.strip_prefix('/'))
                        .is_some_and(|slug| !slug.is_empty() && !slug.contains('/'))
                }
        })
        .map(|url| url.path().to_owned())
        .collect();
    urls.sort_unstable();
    urls.dedup();
    urls
}

struct Store<'a> {
    pool: &'a SqlitePool,
    client: &'a Client,
    market: &'a str,
    host: &'a str,
    requests: i64,
}

impl Store<'_> {
    async fn read(&mut self, request: RequestBuilder) -> Result<Option<String>> {
        if self.requests > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        self.requests += 1;
        let response = request.send().await?;
        let status = response.status();
        if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::SERVICE_UNAVAILABLE {
            let retry = response
                .headers()
                .get(RETRY_AFTER)
                .and_then(|value| value.to_str().ok());
            let seconds = crate::adapters::retry_delay(retry, 0)
                .max(INTERVAL)
                .as_secs();
            let seconds = i64::try_from(seconds).unwrap_or(i64::MAX);
            sqlx::query(
                "UPDATE vtex_catalogue_scan SET next_at=MAX(next_at,?) WHERE supermarket_id=?",
            )
            .bind(now().saturating_add(seconds))
            .bind(self.market)
            .execute(self.pool)
            .await?;
        }
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        ensure!(status.is_success(), "vitrine VTEX retornou HTTP {status}");
        Ok(Some(super::super::read(response).await?))
    }

    async fn get(&mut self, path: &str) -> Result<Option<String>> {
        self.read(self.client.get(format!("https://{}{path}", self.host)))
            .await
    }

    async fn simulate(&mut self, branch: &Branch, skus: &[&str]) -> Result<Value> {
        let request = self.client.post(format!("https://{}/api/checkout/pub/orderForms/simulation", self.host))
            .json(&json!({
                "items":skus.iter().map(|sku| json!({"id":sku,"quantity":1,"seller":branch.seller})).collect::<Vec<_>>(),
                "country":"BRA","postalCode":branch.cep
            }));
        let data: Value =
            serde_json::from_str(&self.read(request).await?.context("checkout ausente")?)?;
        ensure!(
            data["items"].is_array() && data["logisticsInfo"].is_array(),
            "simulação VTEX inválida"
        );
        Ok(data)
    }
}

async fn current_rows(pool: &SqlitePool, branch: &Branch) -> Result<Vec<ImportRow>> {
    let rows: Vec<String> = sqlx::query_scalar(
        "SELECT json_object('source_id',s.source_id,'branch_id',o.branch_id,'name',s.original_name,
          'brand',p.brand,'quantity',p.quantity,'unit',p.unit,'presentation',p.presentation,
          'category',p.category,'gtin',p.gtin,'source_url',o.source_url,
          'photo_source_url',p.photo_source_url,'collected_at',o.collected_at,'offer_type','regular',
          'source_city','Sorriso','source_state','MT','conditions',o.conditions)
         FROM offers o JOIN source_products s ON s.id=o.source_product_id JOIN products p ON p.id=o.product_id
         LEFT JOIN vtex_offer_checks c ON c.branch_id=o.branch_id AND c.source_id=s.source_id
         WHERE o.branch_id=? AND s.source_kind='retail' AND s.source_id LIKE 'vtex-%'
           AND COALESCE(c.checked_at,o.collected_at)<=?-CASE WHEN o.available=1 THEN 600 ELSE 21600 END
         ORDER BY COALESCE(c.checked_at,o.collected_at),s.source_id LIMIT ?",
    ).bind(branch.id).bind(now()).bind(REFRESH_PER_BRANCH).fetch_all(pool).await?;
    rows.iter()
        .map(|json| Ok(serde_json::from_str(json)?))
        .collect()
}

async fn refresh(store: &mut Store<'_>, branch: &Branch) -> Result<usize> {
    let mut rows = current_rows(store.pool, branch).await?;
    let mut imported = 0;
    for batch in rows.chunks_mut(CHECKOUT_BATCH) {
        let ids: Vec<_> = batch
            .iter()
            .map(|r| r.source_id.trim_start_matches("vtex-"))
            .collect();
        let data = store.simulate(branch, &ids).await?;
        let mut updates = Vec::new();
        let mut tx = store.pool.begin_with("BEGIN IMMEDIATE").await?;
        let observed = now();
        for (index, saved) in batch.iter().enumerate() {
            sqlx::query("INSERT INTO vtex_offer_checks(branch_id,source_id,checked_at) VALUES(?,?,?) ON CONFLICT(branch_id,source_id) DO UPDATE SET checked_at=excluded.checked_at")
                .bind(branch.id).bind(&saved.source_id).bind(observed).execute(&mut *tx).await?;

            if let Some(price) = local_price(&data, branch, ids[index], index) {
                let mut row = saved.clone();
                row.price_cents = Some(price);
                row.available = Some(true);
                row.collected_at = observed;
                row.valid_until = Some(observed + 1800);
                updates.push(row);
            } else if simulated_item(&data, branch, ids[index], index)
                .is_some_and(|item| item["availability"] == "withoutStock")
            {
                sqlx::query("UPDATE offers SET available=0,collected_at=?,valid_until=? WHERE branch_id=? AND source_product_id=(SELECT id FROM source_products WHERE supermarket_id=? AND source_id=? AND source_kind='retail')")
                    .bind(observed).bind(observed+1800).bind(branch.id).bind(branch.market).bind(&saved.source_id)
                    .execute(&mut *tx).await?;
            }
        }
        if !updates.is_empty() {
            imported +=
                catalog::import_in_transaction(&mut tx, branch.market, &updates, false, "retail")
                    .await?;
        }
        tx.commit().await?;
    }
    Ok(imported)
}

async fn discover(
    store: &mut Store<'_>,
    checkpoint: &mut Checkpoint,
) -> Result<BTreeMap<String, Value>> {
    if checkpoint.shards.is_empty() {
        let body = store
            .get("/sitemap.xml")
            .await?
            .context("índice VTEX ausente")?;
        checkpoint.shards = sitemap_urls(&body, store.host, true);
        ensure!(
            !checkpoint.shards.is_empty(),
            "índice VTEX sem sitemaps de produto"
        );
    }
    if checkpoint.products.is_empty() {
        let body = store
            .get(&checkpoint.shards[checkpoint.shard])
            .await?
            .context("sitemap de produtos ausente")?;
        checkpoint.products = sitemap_urls(&body, store.host, false);
        ensure!(!checkpoint.products.is_empty(), "sitemap VTEX sem produtos");
    }

    let mut candidates = BTreeMap::new();
    for path in checkpoint
        .products
        .iter()
        .skip(checkpoint.next)
        .take(DISCOVERY_PER_ROUND)
    {
        let search = format!("/api/catalog_system/pub/products/search{path}");
        let mut variants = BTreeMap::new();
        if let Some(body) = store.get(&search).await? {
            let products: Vec<Value> = serde_json::from_str(&body)?;
            for product in products {
                if !product["link"]
                    .as_str()
                    .and_then(|link| Url::parse(link).ok())
                    .is_some_and(|url| {
                        url.scheme() == "https"
                            && url.host_str() == Some(store.host)
                            && url.path() == path
                    })
                {
                    continue;
                }
                if let Some(items) = product["items"].as_array() {
                    for item in items {
                        if let Some(sku) = item["itemId"].as_str().filter(|id| {
                            (1..=24).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit())
                        }) {
                            variants
                                .entry(sku.to_owned())
                                .or_insert_with(|| json!([product]));
                        }
                    }
                }
            }
        }
        let capacity = MAX_DISCOVERY_SKUS - candidates.len();
        let remaining = variants.len().saturating_sub(checkpoint.next_sku);
        candidates.extend(
            variants
                .into_iter()
                .skip(checkpoint.next_sku)
                .take(capacity),
        );
        if remaining > capacity {
            checkpoint.next_sku += capacity;
            break;
        }
        checkpoint.next_sku = 0;
        checkpoint.next += 1;
        if candidates.len() >= MAX_DISCOVERY_SKUS {
            break;
        }
    }
    if checkpoint.next == checkpoint.products.len() {
        checkpoint.products.clear();
        checkpoint.next = 0;
        checkpoint.shard += 1;
        if checkpoint.shard == checkpoint.shards.len() {
            checkpoint.shards.clear();
            checkpoint.shard = 0;
            checkpoint.sweeps += 1;
        }
    }
    Ok(candidates)
}

pub(super) async fn collect(pool: &SqlitePool, client: &Client, market: &str) -> Result<()> {
    let branches: Vec<_> = BRANCHES.iter().filter(|b| b.market == market).collect();
    let host = branches.first().context("rede VTEX não configurada")?.host;
    sqlx::query("INSERT INTO vtex_catalogue_scan(supermarket_id) VALUES(?) ON CONFLICT DO NOTHING")
        .bind(market)
        .execute(pool)
        .await?;
    let at = now();
    let saved: Option<String> = sqlx::query_scalar("UPDATE vtex_catalogue_scan SET next_at=? WHERE supermarket_id=? AND next_at<=? RETURNING checkpoint")
        .bind(at+INTERVAL.as_secs() as i64).bind(market).bind(at).fetch_optional(pool).await?;
    let Some(saved) = saved else { return Ok(()) };
    let mut checkpoint: Checkpoint = serde_json::from_str(&saved)?;
    let mut store = Store {
        pool,
        client,
        market,
        host,
        requests: 0,
    };
    let mut refreshed = 0;
    for branch in &branches {
        refreshed += refresh(&mut store, branch).await?;
    }
    let candidates = discover(&mut store, &mut checkpoint).await?;
    let ids: Vec<_> = candidates.keys().map(String::as_str).collect();
    let mut rows = Vec::new();
    let mut rejected: Vec<(String, String, String)> = Vec::new();
    for branch in &branches {
        for batch in ids.chunks(CHECKOUT_BATCH) {
            let data = store.simulate(branch, batch).await?;
            for (index, sku) in batch.iter().enumerate() {
                let Some(price) = local_price(&data, branch, sku, index) else {
                    continue;
                };
                match row(&candidates[*sku], branch, sku, price) {
                    Ok(row) => rows.push(row),
                    Err(error) => {
                        rejected.push((branch.id.to_owned(), (*sku).to_owned(), error.to_string()))
                    }
                }
            }
        }
    }
    let mut imported = 0;
    if !rows.is_empty() {
        match catalog::import_transaction(pool, market, &rows, false, "retail").await {
            Ok(count) => imported = count,
            Err(_) => {
                for row in &rows {
                    match catalog::import_transaction(
                        pool,
                        market,
                        std::slice::from_ref(row),
                        false,
                        "retail",
                    )
                    .await
                    {
                        Ok(count) => imported += count,
                        Err(error) => rejected.push((
                            row.branch_id.clone(),
                            row.source_id.trim_start_matches("vtex-").to_owned(),
                            error.to_string(),
                        )),
                    }
                }
            }
        }
    }
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let run: i64 = sqlx::query_scalar("INSERT INTO collection_runs(supermarket_id,started_at,finished_at,status,authorization_reference,requests,imported) VALUES(?,?,?,'success',?,?,?) RETURNING id")
        .bind(market).bind(at).bind(now()).bind(BASIS).bind(store.requests).bind((imported+refreshed) as i64)
        .fetch_one(&mut *tx).await?;
    for (branch, sku, reason) in rejected {
        sqlx::query("INSERT INTO source_catalog_items(supermarket_id,branch_id,source_id,raw_json,observed_at,run_id,normalized,reason) VALUES(?,?,?,?,?,?,0,?) ON CONFLICT(supermarket_id,branch_id,source_id) DO UPDATE SET raw_json=excluded.raw_json,observed_at=excluded.observed_at,run_id=excluded.run_id,normalized=0,reason=excluded.reason")
            .bind(market).bind(branch).bind(format!("vtex-{sku}")).bind(candidates.get(&sku).map(|value| value.to_string()).unwrap_or_default()).bind(now()).bind(run).bind(reason)
            .execute(&mut *tx).await?;
    }
    sqlx::query(
        "UPDATE vtex_catalogue_scan SET checkpoint=?,next_at=MAX(next_at,?) WHERE supermarket_id=?",
    )
    .bind(serde_json::to_string(&checkpoint)?)
    .bind(now() + INTERVAL.as_secs() as i64)
    .bind(market)
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds,status,last_started_at,last_finished_at,last_success_at,requests,raw_items,normalized_items) VALUES(?,'pharmacy-vtex',600,'success',?,?,?,?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET status='success',last_started_at=excluded.last_started_at,last_finished_at=excluded.last_finished_at,last_success_at=excluded.last_success_at,last_error=NULL,requests=excluded.requests,raw_items=excluded.raw_items,normalized_items=excluded.normalized_items")
        .bind(market).bind(at).bind(now()).bind(now()).bind(store.requests).bind(ids.len() as i64).bind((imported+refreshed) as i64).execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!(
        market,
        shard = checkpoint.shard,
        next = checkpoint.next,
        sweeps = checkpoint.sweeps,
        candidates = ids.len(),
        refreshed,
        imported = rows.len(),
        requests = store.requests,
        "pharmacy_vtex_batch_complete"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sitemap_scope_keeps_product_paths_on_the_verified_host() {
        let host = "www.example.com";
        let body = "<loc>https://www.example.com/sitemap/product-21.xml</loc>
                    <loc>https://www.example.com/sitemap/category-0.xml</loc>
                    <loc>https://www.example.com/creme/p</loc>
                    <loc>https://www.example.com/creme/p?elsewhere=1</loc>
                    <loc>https://example.org/preservativo/p</loc>";
        assert_eq!(sitemap_urls(body, host, true), ["/sitemap/product-21.xml"]);
        assert_eq!(sitemap_urls(body, host, false), ["/creme/p"]);
    }

    #[tokio::test]
    async fn refresh_queue_includes_all_known_offers_not_a_fixed_sample() {
        let dir = tempfile::tempdir().unwrap();
        let pool = crate::db::open(&format!(
            "sqlite://{}",
            dir.path().join("test.db").display()
        ))
        .await
        .unwrap();
        crate::db::seed_sources(&pool).await.unwrap();
        let branch = super::super::BRANCHES
            .iter()
            .find(|branch| branch.id == "pague-menos-brescansin")
            .unwrap();
        let rows: Vec<ImportRow> = (0..60)
            .map(|index| ImportRow {
                source_id: format!("vtex-{index}"),
                branch_id: branch.id.into(),
                name: format!("Produto de teste {index} 20g"),
                brand: "Teste".into(),
                quantity: 20,
                unit: "g".into(),
                presentation: "Embalagem".into(),
                category: "Farmácia".into(),
                gtin: None,
                price_cents: Some(1000 + index),
                promotional_price_cents: None,
                offer_type: "regular".into(),
                conditions: None,
                available: Some(true),
                source_url: None,
                photo_source_url: None,
                collected_at: now() - 601,
                valid_until: Some(now() + 1800),
                source_city: Some("Sorriso".into()),
                source_state: Some("MT".into()),
            })
            .collect();
        catalog::import(&pool, branch.market, &rows, Some("test fixture"))
            .await
            .unwrap();
        let due = current_rows(&pool, branch).await.unwrap();
        assert_eq!(due.len(), 60);
    }

    #[test]
    fn checkpoint_retains_position_and_nested_sku_offset() {
        let first = Checkpoint {
            shards: vec!["/sitemap/product-4.xml".into()],
            products: vec!["/creme/p".into(), "/shampoo/p".into()],
            next: 1,
            next_sku: 13,
            ..Checkpoint::default()
        };
        let decoded: Checkpoint =
            serde_json::from_str(&serde_json::to_string(&first).unwrap()).unwrap();
        assert_eq!(decoded.products[decoded.next], "/shampoo/p");
        assert_eq!(decoded.next_sku, 13);
    }
}
