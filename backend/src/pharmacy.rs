//! Read the local Therapeutica storefront's sitemap and published Product JSON-LD.
use crate::{
    catalog::{self, ImportRow},
    normalize, now, vip,
};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use reqwest::{StatusCode, Url};
use serde_json::Value;
use sqlx::SqlitePool;
use std::{sync::LazyLock, time::Duration};

const ORIGIN: &str = "https://www.therapeuticafarmacia.com.br";
const SOURCE: &str = "therapeutica-farmacia";
const BRANCH: &str = "therapeutica-farmacia-sorriso";
const INTERVAL: u64 = 600;
static LOC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<loc>\s*([^<]+?)\s*</loc>").unwrap());
static ADDRESS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<address\b[^>]*>(.*?)</address>").unwrap());
static PRODUCT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?is)<script\b[^>]*\bid=["']product-schema["'][^>]*>(.*?)</script>"#).unwrap()
});

fn local_seller(html: &str) -> bool {
    ADDRESS.captures_iter(html).any(|a| {
        normalize::text(&a[1]).contains(
            "avenida curitiba n 2734 bela vista sorriso mt cep 78896 002 cnpj 01 780 384 0001 75",
        )
    })
}

fn pages(sitemap: &str) -> Vec<Url> {
    let mut result: Vec<_> = LOC
        .captures_iter(sitemap)
        .filter_map(|capture| {
            let url = Url::parse(&capture[1]).ok()?;
            let slug = url.path().strip_prefix('/')?;
            (url.origin().ascii_serialization() == ORIGIN
                && url.query().is_none()
                && url.fragment().is_none()
                && url.username().is_empty()
                && url.password().is_none()
                && !slug.is_empty()
                && slug
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                && !matches!(
                    slug,
                    "beleza"
                        | "saude"
                        | "emagrecimento"
                        | "aromathera"
                        | "florais"
                        | "homeopatia"
                        | "gestante-kids"
                        | "chas-fitoterapicos"
                        | "sobre-nos"
                        | "nossas-lojas"
                ))
            .then_some(url)
        })
        .collect();
    result.sort_unstable_by(|a, b| {
        let key = |url: &Url| url.path() != "/protetor-solar-bi-gel-fps-30";
        key(a).cmp(&key(b)).then(a.path().cmp(b.path()))
    });
    result.dedup();
    result
}

fn parse(html: &str, url: &Url, observed: i64) -> Result<Vec<ImportRow>> {
    ensure!(
        local_seller(html),
        "CNPJ ou endereço da loja virtual diverge de Sorriso"
    );
    let Some(schema) = PRODUCT.captures(html) else {
        return Ok(Vec::new());
    };
    let product: Value = serde_json::from_str(&schema[1])?;
    ensure!(product["@type"] == "Product", "schema de produto inválido");
    let name = product["name"].as_str().context("produto sem nome")?;
    let brand = product["brand"]["name"].as_str().unwrap_or("Não informada");
    let offers = product["offers"]["offers"]
        .as_array()
        .context("ofertas da vitrine ausentes")?;
    // The public schema does not identify each variant's presentation.
    ensure!(
        offers.len() == 1,
        "produto com variantes sem embalagem discriminada"
    );
    let offer = &offers[0];
    ensure!(
        offer["url"].as_str() == Some(url.as_str()) && offer["priceCurrency"] == "BRL",
        "oferta de outro produto ou moeda"
    );
    let sku = offer["sku"]
        .as_str()
        .filter(|s| !s.is_empty())
        .context("oferta sem SKU")?;
    let available = match offer["availability"].as_str() {
        Some("https://schema.org/InStock") => Some(true),
        Some("https://schema.org/OutOfStock" | "https://schema.org/SoldOut") => Some(false),
        _ => None,
    };
    let price = vip::cents(&offer["price"])?;
    ensure!(
        price > 100 && !html.to_lowercase().contains("preço sob encomenda"),
        "preço simbólico ou sob orçamento"
    );
    let row = ImportRow {
        source_id: format!("thera-{sku}"),
        branch_id: BRANCH.into(),
        name: name.into(),
        brand: brand.into(),
        quantity: 1,
        unit: "un".into(),
        presentation: format!("Unidade da vitrine · {sku}"),
        category: "Farmácia".into(),
        gtin: None,
        price_cents: Some(price),
        promotional_price_cents: None,
        offer_type: "regular".into(),
        conditions: None,
        available,
        source_url: Some(url.to_string()),
        photo_source_url: product["image"][0].as_str().map(str::to_owned),
        collected_at: observed,
        valid_until: Some(observed + 4 * 3600),
        source_city: Some("Sorriso".into()),
        source_state: Some("MT".into()),
    };
    catalog::validate_for_sector(&row, "pharmacy")?;
    Ok(vec![row])
}

async fn read(response: reqwest::Response) -> Result<String> {
    let mut response = response.error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            bytes.len() + chunk.len() <= catalog::MAX_BYTES,
            "resposta acima de 8 MiB"
        );
        bytes.extend_from_slice(&chunk);
    }
    Ok(String::from_utf8(bytes)?)
}

pub async fn run(pool: &SqlitePool, limit: usize, watch: bool) -> Result<()> {
    ensure!((1..=40).contains(&limit), "lote de 1 a 40 páginas");
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .user_agent("SorriPrecos/0.1")
        .build()?;
    loop {
        let delay = match scan(pool, &client, limit).await {
            Ok(delay) => delay,
            Err(error) if watch => {
                tracing::warn!(%error, "pharmacy_scan_failed");
                Duration::from_secs(INTERVAL)
            }
            Err(error) => return Err(error),
        };
        if !watch {
            return Ok(());
        }
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(delay) => {}
        }
    }
}

fn retry_after(response: &reqwest::Response) -> Duration {
    let value = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok());
    crate::adapters::retry_delay(value, 0).max(Duration::from_secs(INTERVAL))
}

async fn scan(pool: &SqlitePool, client: &reqwest::Client, limit: usize) -> Result<Duration> {
    let retry: Option<i64> =
        sqlx::query_scalar("SELECT retry_at FROM pharmacy_scan WHERE supermarket_id=?")
            .bind(SOURCE)
            .fetch_optional(pool)
            .await?
            .flatten();
    if let Some(retry) = retry.filter(|retry| *retry > now()) {
        return Ok(Duration::from_secs((retry - now()).max(1) as u64));
    }
    let manifest = client.get(format!("{ORIGIN}/sitemap.xml")).send().await?;
    if manifest.status() == StatusCode::TOO_MANY_REQUESTS {
        let delay = retry_after(&manifest);
        sqlx::query("INSERT INTO pharmacy_scan(supermarket_id,next_page,retry_at) VALUES(?,0,?) ON CONFLICT(supermarket_id) DO UPDATE SET retry_at=excluded.retry_at")
            .bind(SOURCE).bind(now() + delay.as_secs() as i64).execute(pool).await?;
        tracing::warn!(seconds = delay.as_secs(), "pharmacy_rate_limited");
        return Ok(delay);
    }
    let pages = pages(&read(manifest).await?);
    ensure!(!pages.is_empty(), "sitemap sem páginas de produto");
    let start: i64 =
        sqlx::query_scalar("SELECT next_page FROM pharmacy_scan WHERE supermarket_id=?")
            .bind(SOURCE)
            .fetch_optional(pool)
            .await?
            .unwrap_or(0);
    let start = (start as usize) % pages.len();
    let mut next = start;
    let mut delay = Duration::from_secs(INTERVAL);
    let mut retry = None;
    let mut rows = Vec::new();
    let mut requests = 1;
    for url in pages.iter().skip(start).take(limit) {
        tokio::time::sleep(Duration::from_millis(500)).await;
        requests += 1;
        let response = client.get(url.clone()).send().await;
        if let Ok(response) = &response
            && response.status() == StatusCode::TOO_MANY_REQUESTS
        {
            delay = retry_after(response);
            retry = Some(now() + delay.as_secs() as i64);
            tracing::warn!(seconds = delay.as_secs(), "pharmacy_rate_limited");
            break;
        }
        let result = async { parse(&read(response?).await?, url, now()) }.await;
        match result {
            Ok(items) => rows.extend(items),
            Err(error) => tracing::warn!(path = url.path(), %error, "pharmacy_page_pending"),
        }
        next = (next + 1) % pages.len();
    }
    let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
    let mut seen = std::collections::HashSet::new();
    rows.retain(|r| seen.insert(r.source_id.clone()));
    if !rows.is_empty() {
        catalog::import_in_transaction(&mut tx, SOURCE, &rows, false, "retail").await?;
        sqlx::query("INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds,status,last_success_at,requests,raw_items,normalized_items) VALUES(?,'therapeutica',600,'success',?,?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET status='success',last_success_at=excluded.last_success_at,requests=excluded.requests,raw_items=excluded.raw_items,normalized_items=excluded.normalized_items")
            .bind(SOURCE).bind(now()).bind(requests).bind(rows.len() as i64).bind(rows.len() as i64).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO pharmacy_scan(supermarket_id,next_page,retry_at) VALUES(?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET next_page=excluded.next_page,retry_at=excluded.retry_at")
        .bind(SOURCE).bind(next as i64).bind(retry).execute(&mut *tx).await?;
    tx.commit().await?;
    tracing::info!(
        start,
        next,
        candidates = pages.len(),
        imported = rows.len(),
        "pharmacy_batch_complete"
    );
    Ok(delay)
}
