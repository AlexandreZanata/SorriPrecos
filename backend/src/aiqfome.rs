//! Read the same public, embedded catalogue that the aiqfome storefront renders.
use crate::{
    catalog::{self, ImportRow},
    normalize, now, vip,
};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::{collections::HashSet, path::Path, sync::LazyLock, time::Duration};

const INTERVAL: i64 = 600;
const BASIS: &str = "PUBLIC_STOREFRONT: catálogo público aiqfome com identidade local verificada";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    supermarket_id: String,
    branch_id: String,
    slug: String,
    seller_id: i64,
    cnpj: String,
    street: String,
    number: String,
    postcode: String,
    categories: Vec<String>,
}

fn digits(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

fn verify(page: &Value, source: &Source) -> Result<()> {
    ensure!(
        page["id"] == source.seller_id
            && page["menu"]["restaurante_id"] == source.seller_id
            && page["slug"] == source.slug
            && page["cidade_id"] == 4430
            && page["cidade"]["id"] == 4430
            && page["cidade"]["estado_id"] == 13
            && page["cidade"]["nome"] == "Sorriso"
            && digits(page["cnpj"].as_str().unwrap_or_default()) == source.cnpj
            && normalize::text(page["logradouro"].as_str().unwrap_or_default())
                == normalize::text(&source.street)
            && page["numero"] == source.number
            && digits(page["cep"].as_str().unwrap_or_default()) == source.postcode,
        "identidade ou endereço do vendedor diverge da filial de Sorriso-MT"
    );
    Ok(())
}

fn entries(value: &Value) -> Result<Box<dyn Iterator<Item = &Value> + '_>> {
    match value {
        Value::Array(items) => Ok(Box::new(items.iter())),
        Value::Object(items) => Ok(Box::new(items.values())),
        _ => anyhow::bail!("lista de catálogo inválida"),
    }
}

static SIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*(kg|g|ml|l)\b").unwrap());
static EXACT_SIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^\s*(\d+(?:[.,]\d+)?)\s*(kg|g|ml|l)\s*$").unwrap());

static MULTIPACK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\d\s*[x×]\s*\d").unwrap());

fn measure(capture: &regex::Captures<'_>) -> Result<(i64, &'static str)> {
    let (factor, unit) = match capture[2].to_ascii_lowercase().as_str() {
        "kg" => (1000, "g"),
        "l" => (1000, "ml"),
        "g" => (1, "g"),
        _ => (1, "ml"),
    };
    Ok((vip::scaled(&capture[1], factor)?, unit))
}

fn package(label: &str, name: &str) -> Result<(i64, String)> {
    let mut title_sizes = SIZE.captures_iter(name);
    let title_size = title_sizes.next().as_ref().map(measure).transpose()?;
    ensure!(
        title_sizes.next().is_none() && !MULTIPACK.is_match(name),
        "embalagem múltipla ou ambígua"
    );
    let size = match label.trim().to_ascii_lowercase().as_str() {
        "un" | "unidade" => title_size.context("unidade sem peso ou volume confirmado")?,
        _ => measure(
            &EXACT_SIZE
                .captures(label)
                .context("variante sem peso ou volume confirmado")?,
        )?,
    };
    ensure!(
        title_size.is_none_or(|title_size| title_size == size),
        "peso ou volume do título diverge da variante"
    );
    Ok((size.0, size.1.into()))
}

fn required_cost(data: &Value) -> bool {
    data["obrigatorios_grupos"]
        .as_array()
        .is_some_and(|groups| {
            groups.iter().any(|group| {
                group["limite_minimo_obrigatorio"].as_i64().unwrap_or(0) > 0
                    && group["obrigatorios"].as_array().is_none_or(|options| {
                        options.is_empty()
                            || options.iter().any(|option| {
                                !option["valor"].is_null() && option["valor"].as_f64() != Some(0.0)
                            })
                    })
            })
        })
}

#[derive(Serialize)]
struct Observation {
    id: String,
    name: String,
    size: String,
    category: String,
    price: Value,
    available: bool,
    reason: Option<String>,
}

fn normalize_menu(
    page: &Value,
    source: &Source,
    url: &str,
    observed: i64,
) -> Result<(Vec<Observation>, Vec<ImportRow>)> {
    let mut observations = Vec::new();
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    for category in entries(&page["menu"]["cardapios_categorias"])? {
        let name = category["nome"]
            .as_str()
            .context("categoria sem nome")?
            .trim();
        if !source
            .categories
            .iter()
            .any(|value| *value == normalize::text(name))
        {
            continue;
        }
        for item in entries(&category["itens"])? {
            let title = item["nome"].as_str().context("produto sem nome")?.trim();
            for variant in entries(&item["itens_tamanhos"])? {
                ensure!(
                    variant["restaurante_id"] == source.seller_id
                        && variant["item_id"] == item["id"],
                    "variante vinculada a outro vendedor ou produto"
                );
                let id = format!("aiq-{}", variant["id"].as_i64().context("variante sem ID")?);
                ensure!(seen.insert(id.clone()), "variante repetida no catálogo");
                let size = variant["tamanho"]["nome"].as_str().unwrap_or_default();
                let available = category["disponivel"] == true
                    && item["disponivel"] == true
                    && variant["disponivel"] == true;
                let parsed = (|| -> Result<ImportRow> {
                    ensure!(item["is_free_gift"] != true, "brinde");
                    ensure!(
                        category["taxa_embalagem"].as_f64() == Some(0.0)
                            && ![category, item, variant, &variant["tamanho"]]
                                .iter()
                                .any(|value| required_cost(value)),
                        "preço depende de embalagem ou complemento obrigatório"
                    );
                    let (quantity, unit) = package(size, title)?;
                    let gtin = variant["ean_code"]
                        .as_str()
                        .filter(|gtin| normalize::valid_gtin(gtin))
                        .map(str::to_owned);
                    let presentation = if gtin.is_some() {
                        "unidade de venda".into()
                    } else {
                        format!("variante aiqfome {}", source.seller_id)
                    };
                    let row = ImportRow {
                        source_id: id.clone(),
                        branch_id: source.branch_id.clone(),
                        name: title.into(),
                        brand: variant["brand"]
                            .as_str()
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or("Não informada")
                            .into(),
                        quantity,
                        unit,
                        presentation,
                        category: name.into(),
                        gtin,
                        price_cents: Some(vip::cents(&variant["valor"])?),
                        promotional_price_cents: None,
                        offer_type: "regular".into(),
                        conditions: None,
                        available: Some(available),
                        source_url: Some(url.into()),
                        collected_at: observed,
                        valid_until: Some(observed + 1800),
                        source_city: Some("Sorriso".into()),
                        source_state: Some("MT".into()),
                    };
                    catalog::validate(&row)?;
                    Ok(row)
                })();
                let reason = match parsed {
                    Ok(row) => {
                        rows.push(row);
                        None
                    }
                    Err(error) => Some(error.to_string()),
                };
                observations.push(Observation {
                    id,
                    name: title.into(),
                    size: size.into(),
                    category: name.into(),
                    price: variant["valor"].clone(),
                    available,
                    reason,
                });
            }
        }
    }
    Ok((observations, rows))
}

async fn fetch(client: &reqwest::Client, url: &str) -> Result<Value> {
    let mut response = client.get(url).send().await?.error_for_status()?;
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            body.len() + chunk.len() <= catalog::MAX_BYTES,
            "página excede 8 MiB"
        );
        body.extend_from_slice(&chunk);
    }
    let script = std::str::from_utf8(&body)?
        .split_once("const restaurantenovo = ")
        .context("catálogo público ausente")?
        .1;
    serde_json::Deserializer::from_str(script)
        .into_iter::<Value>()
        .next()
        .context("objeto de catálogo ausente")?
        .context("objeto de catálogo inválido")
}

async fn collect(pool: &SqlitePool, client: &reqwest::Client, source: &Source) -> Result<()> {
    ensure!(
        !source.slug.is_empty()
            && source
                .slug
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
        "slug aiqfome inválido"
    );
    let branch_ok: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branches WHERE id=? AND supermarket_id=? AND city='Sorriso' AND state='MT' AND active=1)")
        .bind(&source.branch_id).bind(&source.supermarket_id).fetch_one(pool).await?;
    ensure!(branch_ok, "filial local não cadastrada");

    sqlx::query("INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds) VALUES(?,'aiqfome',?) ON CONFLICT(supermarket_id) DO NOTHING")
        .bind(&source.supermarket_id).bind(INTERVAL).execute(pool).await?;
    let started = now();
    let claimed = sqlx::query("UPDATE catalogue_collectors SET status='running',last_started_at=?,last_error=NULL WHERE supermarket_id=? AND adapter='aiqfome' AND (last_started_at IS NULL OR last_started_at<=?) AND (status!='running' OR last_started_at<?)")
        .bind(started).bind(&source.supermarket_id).bind(started - INTERVAL).bind(started - 1800).execute(pool).await?;
    if claimed.rows_affected() == 0 {
        return Ok(());
    }

    let url = format!("https://aiqfome.com/MT/sorriso/{}", source.slug);
    let mut raw_count = 0;
    let mut imported = 0;
    let mut scope_rejected = false;
    let result: Result<()> = async {
        let page = fetch(client, &url).await?;
        if let Err(error) = verify(&page, source) {
            scope_rejected = true;
            return Err(error);
        }
        let observed = now();
        let (observations, rows) = normalize_menu(&page, source, &url, observed)?;
        raw_count = observations.len() as i64;
        ensure!(!rows.is_empty(), "catálogo sem variantes de mercearia utilizáveis");
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        let count = catalog::import_in_transaction(&mut tx, &source.supermarket_id, &rows, false, "retail").await? as i64;
        let run: i64 = sqlx::query_scalar("INSERT INTO collection_runs(supermarket_id,started_at,finished_at,status,authorization_reference,requests,imported) VALUES(?,?,?,'success',?,1,?) RETURNING id")
            .bind(&source.supermarket_id).bind(started).bind(now()).bind(BASIS).bind(count).fetch_one(&mut *tx).await?;
        for observation in observations {
            sqlx::query("INSERT INTO source_catalog_items(supermarket_id,branch_id,source_id,raw_json,observed_at,run_id,normalized,reason) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(supermarket_id,branch_id,source_id) DO UPDATE SET raw_json=excluded.raw_json,observed_at=excluded.observed_at,run_id=excluded.run_id,normalized=excluded.normalized,reason=excluded.reason")
                .bind(&source.supermarket_id).bind(&source.branch_id).bind(&observation.id)
                .bind(serde_json::to_string(&observation)?).bind(observed).bind(run)
                .bind(observation.reason.is_none()).bind(&observation.reason).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE offers SET available=NULL WHERE branch_id=? AND source_product_id IN (SELECT id FROM source_products sp WHERE sp.supermarket_id=? AND sp.source_id LIKE 'aiq-%' AND NOT EXISTS (SELECT 1 FROM source_catalog_items i WHERE i.supermarket_id=sp.supermarket_id AND i.branch_id=? AND i.source_id=sp.source_id AND i.run_id=? AND i.normalized=1))")
            .bind(&source.branch_id).bind(&source.supermarket_id).bind(&source.branch_id).bind(run).execute(&mut *tx).await?;
        sqlx::query("UPDATE supermarkets SET integration_status='authorized' WHERE id=?")
            .bind(&source.supermarket_id).execute(&mut *tx).await?;
        tx.commit().await?;
        imported = count;
        Ok(())
    }.await;
    let error = result.as_ref().err().map(ToString::to_string);
    sqlx::query("UPDATE catalogue_collectors SET status=?,last_finished_at=?,last_error=?,requests=1,raw_items=?,normalized_items=?,scope_rejected=?,last_success_at=CASE WHEN ? THEN ? ELSE last_success_at END WHERE supermarket_id=?")
        .bind(if result.is_ok() { "success" } else { "failed" })
        .bind(now()).bind(error).bind(raw_count).bind(imported).bind(i64::from(scope_rejected))
        .bind(result.is_ok()).bind(now()).bind(&source.supermarket_id).execute(pool).await?;
    result
}

pub async fn run(pool: &SqlitePool, path: &Path, watch: bool) -> Result<()> {
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
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);
    loop {
        let mut failed = false;
        for source in &sources {
            if let Err(error) = collect(pool, &client, source).await {
                failed = true;
                tracing::error!(source=%source.supermarket_id, %error, "public_catalog_failed");
            }
        }
        if !watch {
            ensure!(
                !failed,
                "coleta aiqfome incompleta; consulte /v1/collectors"
            );
            return Ok(());
        }
        tokio::select! {
            _ = &mut shutdown => return Ok(()),
            _ = tokio::time::sleep(Duration::from_secs(15)) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_checks_variant_against_title() {
        assert_eq!(package("2KG", "picanha").unwrap(), (2000, "g".into()));
        assert_eq!(package("UN", "água 1,5l").unwrap(), (1500, "ml".into()));
        assert!(package("500ml", "suco pet 300ml").is_err());
        assert!(package("UN", "água 12 x 350ml").is_err());
        assert!(package("UN", "refrigerante").is_err());
    }
}
