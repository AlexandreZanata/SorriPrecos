use crate::{normalize, now};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, SqlitePool};
use std::{collections::HashSet, path::Path};

pub const MAX_ROWS: usize = 10_000;
pub const MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportRow {
    pub source_id: String,
    pub branch_id: String,
    pub name: String,
    pub brand: String,
    pub quantity: i64,
    pub unit: String,
    pub presentation: String,
    pub category: String,
    #[serde(default)]
    pub gtin: Option<String>,
    #[serde(default)]
    pub price_cents: Option<i64>,
    #[serde(default)]
    pub promotional_price_cents: Option<i64>,
    #[serde(default = "regular")]
    pub offer_type: String,
    #[serde(default)]
    pub conditions: Option<String>,
    #[serde(default)]
    pub available: Option<bool>,
    #[serde(default)]
    pub source_url: Option<String>,
    pub collected_at: i64,
    #[serde(default)]
    pub valid_until: Option<i64>,
}
fn regular() -> String {
    "regular".into()
}

pub fn parse(data: &[u8], format: &str) -> Result<Vec<ImportRow>> {
    ensure!(data.len() <= MAX_BYTES, "arquivo excede 8 MiB");
    let rows: Vec<ImportRow> = match format {
        "json" => serde_json::from_slice(data)?,
        "csv" => csv::Reader::from_reader(data)
            .deserialize()
            .collect::<std::result::Result<_, _>>()?,
        _ => bail!("formato esperado: csv ou json"),
    };
    ensure!(
        !rows.is_empty() && rows.len() <= MAX_ROWS,
        "lote deve conter 1 a 10000 registros"
    );
    Ok(rows)
}

pub async fn read_file(path: &Path) -> Result<Vec<ImportRow>> {
    ensure!(
        tokio::fs::metadata(path).await?.len() <= MAX_BYTES as u64,
        "arquivo excede 8 MiB"
    );
    parse(
        &tokio::fs::read(path).await?,
        path.extension().and_then(|x| x.to_str()).unwrap_or(""),
    )
}

pub fn validate(row: &ImportRow) -> Result<()> {
    for (field, value, max) in [
        ("nome", row.name.as_str(), 180),
        ("marca", &row.brand, 80),
        ("código", &row.source_id, 120),
        ("filial", &row.branch_id, 100),
        ("apresentação", &row.presentation, 80),
        ("categoria", &row.category, 80),
    ] {
        ensure!(
            !value.trim().is_empty() && value.chars().count() <= max,
            "{field} inválido"
        );
    }
    ensure!(
        row.conditions.as_ref().is_none_or(|s| s.len() <= 500),
        "condição muito longa"
    );
    let category = normalize::text(&row.category);
    let name = normalize::text(&row.name);
    ensure!(
        !["medicamento", "farmacia", "farmaceutico", "medicamentos"]
            .iter()
            .any(|s| category.contains(s)),
        "categoria fora do MVP"
    );
    ensure!(
        !["dipirona", "paracetamol", "ibuprofeno", "amoxicilina"]
            .iter()
            .any(|s| name.contains(s)),
        "medicamento fora do MVP"
    );
    normalize::package(row.quantity, &row.unit)?;
    if let Some(g) = row.gtin.as_deref().filter(|s| !s.is_empty()) {
        ensure!(normalize::valid_gtin(g), "GTIN inválido: {g}");
    }
    for p in [row.price_cents, row.promotional_price_cents]
        .into_iter()
        .flatten()
    {
        ensure!((1..=100_000_000).contains(&p), "preço em centavos inválido");
    }
    ensure!(
        row.promotional_price_cents
            .is_none_or(|p| row.price_cents.is_some_and(|base| p <= base)),
        "promoção inválida"
    );
    ensure!(
        ["regular", "promotion", "club", "wholesale"].contains(&row.offer_type.as_str()),
        "tipo de oferta inválido"
    );
    if ["club", "wholesale"].contains(&row.offer_type.as_str()) {
        ensure!(
            row.conditions
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty()),
            "oferta condicional exige condições"
        );
    }
    ensure!(
        row.collected_at > 0 && row.collected_at <= now() + 300,
        "horário de observação inválido"
    );
    ensure!(
        row.valid_until.is_none_or(|t| t >= row.collected_at),
        "validade anterior à coleta"
    );
    if let Some(s) = &row.source_url {
        let u = url::Url::parse(s)?;
        ensure!(
            ["http", "https"].contains(&u.scheme())
                && u.username().is_empty()
                && u.password().is_none()
                && s.len() <= 2000,
            "URL inválida"
        );
    }
    Ok(())
}

pub async fn import(
    pool: &SqlitePool,
    source: &str,
    rows: &[ImportRow],
    authorization: Option<&str>,
) -> Result<usize> {
    ensure!(
        !rows.is_empty() && rows.len() <= MAX_ROWS,
        "lote deve conter 1 a 10000 registros"
    );
    let demo: bool = sqlx::query_scalar("SELECT is_demo FROM supermarkets WHERE id=?")
        .bind(source)
        .fetch_one(pool)
        .await
        .context("fonte não cadastrada")?;
    ensure!(
        demo || authorization.is_some_and(|s| s.trim().len() >= 8),
        "importação real exige referência de autorização (--authorization)"
    );
    let run:i64=sqlx::query_scalar("INSERT INTO collection_runs(supermarket_id,started_at,status,authorization_reference) VALUES(?,?,'running',?) RETURNING id").bind(source).bind(now()).bind(authorization).fetch_one(pool).await?;
    let result = import_transaction(pool, source, rows, demo).await;
    let (status, count, error) = match &result {
        Ok(n) => ("success", *n as i64, None),
        Err(e) => ("failed", 0, Some(e.to_string())),
    };
    sqlx::query("UPDATE collection_runs SET finished_at=?,status=?,imported=?,error=? WHERE id=?")
        .bind(now())
        .bind(status)
        .bind(count)
        .bind(error)
        .bind(run)
        .execute(pool)
        .await?;
    result
}

async fn import_transaction(
    pool: &SqlitePool,
    source: &str,
    rows: &[ImportRow],
    demo: bool,
) -> Result<usize> {
    let mut seen = HashSet::new();
    for r in rows {
        validate(r)?;
        ensure!(
            seen.insert((&r.source_id, &r.branch_id)),
            "código/filial duplicado no lote"
        );
    }
    let mut tx = pool.begin().await?;
    let branches: HashSet<String> =
        sqlx::query_scalar("SELECT id FROM branches WHERE supermarket_id=? AND active=1")
            .bind(source)
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .collect();
    let mut applied = 0;
    for r in rows {
        ensure!(
            branches.contains(&r.branch_id),
            "filial não pertence à fonte: {}",
            r.branch_id
        );
        let (q, unit) = normalize::package(r.quantity, &r.unit)?;
        let gtin = r
            .gtin
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("{:0>14}", s));
        let key = normalize::identity(
            &r.name,
            &r.brand,
            q,
            &unit,
            &r.presentation,
            gtin.as_deref(),
            demo,
        );
        sqlx::query("INSERT OR IGNORE INTO products(identity_key,gtin,name,normalized_name,brand,quantity,unit,presentation,category,is_demo) VALUES(?,?,?,?,?,?,?,?,?,?)")
            .bind(&key).bind(&gtin).bind(&r.name).bind(normalize::text(&r.name)).bind(&r.brand).bind(q).bind(&unit).bind(&r.presentation).bind(&r.category).bind(demo).execute(&mut *tx).await?;
        let product =
            sqlx::query("SELECT id,quantity,unit,presentation FROM products WHERE identity_key=?")
                .bind(&key)
                .fetch_one(&mut *tx)
                .await?;
        let pid: i64 = product.get("id");
        ensure!(
            product.get::<i64, _>("quantity") == q
                && product.get::<String, _>("unit") == unit
                && normalize::text(&product.get::<String, _>("presentation"))
                    == normalize::text(&r.presentation),
            "GTIN associado a embalagem diferente"
        );
        let indexed: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM products_fts WHERE rowid=?)")
                .bind(pid)
                .fetch_one(&mut *tx)
                .await?;
        if !indexed {
            sqlx::query("INSERT INTO products_fts(rowid,name,brand,pack) VALUES(?,?,?,?)")
                .bind(pid)
                .bind(&r.name)
                .bind(&r.brand)
                .bind(normalize::pack_tokens(q, &unit))
                .execute(&mut *tx)
                .await?;
        }
        // A source SKU must not silently change its canonical product.
        sqlx::query("INSERT OR IGNORE INTO source_products(supermarket_id,source_id,product_id,original_name,source_url) VALUES(?,?,?,?,?)").bind(source).bind(&r.source_id).bind(pid).bind(&r.name).bind(&r.source_url).execute(&mut *tx).await?;
        let sp = sqlx::query(
            "SELECT id,product_id FROM source_products WHERE supermarket_id=? AND source_id=?",
        )
        .bind(source)
        .bind(&r.source_id)
        .fetch_one(&mut *tx)
        .await?;
        ensure!(
            sp.get::<i64, _>("product_id") == pid,
            "código de origem mudou de produto; revisão manual necessária"
        );
        let sid: i64 = sp.get("id");
        let previous=sqlx::query("SELECT id,price_cents,promotional_price_cents,collected_at FROM offers WHERE source_product_id=? AND branch_id=?").bind(sid).bind(&r.branch_id).fetch_optional(&mut *tx).await?;
        if previous
            .as_ref()
            .is_some_and(|p| p.get::<i64, _>("collected_at") > r.collected_at)
        {
            continue;
        }
        let changed = previous.as_ref().is_none_or(|p| {
            p.get::<Option<i64>, _>("price_cents") != r.price_cents
                || p.get::<Option<i64>, _>("promotional_price_cents") != r.promotional_price_cents
        });
        let oid:i64=sqlx::query_scalar("INSERT INTO offers(source_product_id,product_id,supermarket_id,branch_id,price_cents,promotional_price_cents,offer_type,conditions,available,source_url,collected_at,valid_until) VALUES(?,?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(source_product_id,branch_id) DO UPDATE SET price_cents=excluded.price_cents,promotional_price_cents=excluded.promotional_price_cents,offer_type=excluded.offer_type,conditions=excluded.conditions,available=excluded.available,source_url=excluded.source_url,collected_at=excluded.collected_at,valid_until=excluded.valid_until RETURNING id")
            .bind(sid).bind(pid).bind(source).bind(&r.branch_id).bind(r.price_cents).bind(r.promotional_price_cents).bind(&r.offer_type).bind(&r.conditions).bind(r.available).bind(&r.source_url).bind(r.collected_at).bind(r.valid_until).fetch_one(&mut *tx).await?;
        if changed {
            sqlx::query("INSERT INTO price_history(offer_id,price_cents,promotional_price_cents,collected_at) VALUES(?,?,?,?)").bind(oid).bind(r.price_cents).bind(r.promotional_price_cents).bind(r.collected_at).execute(&mut *tx).await?;
        }
        applied += 1;
    }
    // An authorized import does not imply a working automatic network adapter.
    tx.commit().await?;
    Ok(applied)
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: i64,
    pub gtin: Option<String>,
    pub name: String,
    pub brand: String,
    pub quantity: i64,
    pub unit: String,
    pub presentation: String,
    pub category: String,
    pub is_demo: bool,
    pub min_price_cents: Option<i64>,
    pub last_collected_at: Option<i64>,
    pub branch_count: i64,
}

pub const PRODUCT_FIELDS: &str = "p.id,p.gtin,p.name,p.brand,p.quantity,p.unit,p.presentation,p.category,p.is_demo, MIN(CASE WHEN o.available=1 AND o.collected_at>=? AND (o.valid_until IS NULL OR o.valid_until>=?) THEN CASE WHEN o.offer_type IN ('regular','promotion') AND COALESCE(o.conditions,'')='' THEN COALESCE(o.promotional_price_cents,o.price_cents) ELSE o.price_cents END END) AS min_price_cents,MAX(o.collected_at) AS last_collected_at, COUNT(DISTINCT CASE WHEN o.available=1 AND o.price_cents IS NOT NULL AND o.collected_at>=? AND (o.valid_until IS NULL OR o.valid_until>=?) THEN o.branch_id END) AS branch_count";

pub async fn search(
    pool: &SqlitePool,
    q: &str,
    limit: i64,
    after: i64,
    demo: bool,
) -> Result<Vec<Product>> {
    let fts = normalize::fts_query(q);
    if fts.is_empty() {
        return Ok(vec![]);
    }
    let sql = format!(
        "WITH hits AS MATERIALIZED (SELECT f.rowid id FROM products_fts f JOIN products p ON p.id=f.rowid WHERE products_fts MATCH ? AND p.is_demo=? AND f.rowid>? ORDER BY f.rowid LIMIT ?) SELECT {PRODUCT_FIELDS} FROM hits h JOIN products p ON p.id=h.id LEFT JOIN offers o ON o.product_id=p.id GROUP BY p.id ORDER BY p.id"
    );
    Ok(sqlx::query_as(&sql)
        .bind(fts)
        .bind(demo)
        .bind(after)
        .bind(limit)
        .bind(now() - 172800)
        .bind(now())
        .bind(now() - 172800)
        .bind(now())
        .fetch_all(pool)
        .await?)
}

pub async fn product(pool: &SqlitePool, id: i64) -> Result<Option<Product>> {
    let sql = format!(
        "SELECT {PRODUCT_FIELDS} FROM products p LEFT JOIN offers o ON o.product_id=p.id WHERE p.id=? GROUP BY p.id"
    );
    Ok(sqlx::query_as(&sql)
        .bind(now() - 172800)
        .bind(now())
        .bind(now() - 172800)
        .bind(now())
        .bind(id)
        .fetch_optional(pool)
        .await?)
}
