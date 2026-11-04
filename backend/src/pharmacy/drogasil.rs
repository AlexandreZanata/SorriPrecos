//! Public pickup stock by store. The network's price endpoint is not scoped to a store.
use crate::{
    catalog::{self, ImportRow},
    normalize, now,
};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::{sync::LazyLock, time::Duration};

const ORIGIN: &str = "https://www.drogasil.com.br";
const CATEGORIES: &[&str] = &[
    "/saude/saude-sexual/preservativos.html",
    "/saude.html",
    "/medicamentos.html",
    "/vitaminas-e-suplementos.html",
    "/mamae-bebe.html",
    "/beleza.html",
    "/beleza-premium.html",
    "/cuidados-diarios.html",
    "/pet.html",
    "/marcas-exclusivas.html",
];
const BUDGET: usize = 8;
const SPACING: Duration = Duration::from_millis(1500);
const STOCK_QUERY: &str = r#"
  query GET_STOCK(
    $zipcode: String!
    $products: [StockNearbyBtZipCodeTypeInput!]!
    $logotype: String!
    $maxQuantityBranchSearch: String
  ) {
    getNearbyStockByZipCode(
      products: $products
      zipcode: $zipcode
      logotype: $logotype
      maxQuantityBranchSearch: $maxQuantityBranchSearch
    ) {
      branch {
        id
        businessName
        flag24hours
        distanceKMFromSearch
        address {
          district
          addressLocal
          addressNumber
          city
          sgState
        }
        branchService {
          hourOpenNormaly
          hourEndNormaly
        }
        logoType {
          id
          description
        }
      }
      stocks {
        sku
        quantity
      }
    }
  }
"#;

fn branch_id(branch: &Value) -> Option<&'static str> {
    let address = &branch["address"];
    if normalize::text(address["city"].as_str()?) != "sorriso"
        || address["sgState"] != "MT"
        || branch["logoType"]["description"] != "DROGASIL"
    {
        return None;
    }
    let street = normalize::text(address["addressLocal"].as_str()?);
    match (branch["id"].as_i64()?, address["addressNumber"].as_str()?) {
        (3852, "488") if street.contains("natalino joao brescansin") => Some("drogasil-brescansin"),
        (5635, "2942") if street.contains("tancredo neves") => Some("drogasil-tancredo"),
        // The official locator identifies 3034 as Blumenau 2995. The stock API
        // currently repeats another street, but publishes the same ID, name and number.
        (3034, "2995")
            if normalize::text(branch["businessName"].as_str()?) == "avenida blumenau"
                && (street.contains("blumenau")
                    || street == "avenida natalino joao brescansin") =>
        {
            Some("drogasil-blumenau")
        }
        _ => None,
    }
}

fn candidate(product: &Value, category: &str, observed: i64) -> Option<(i64, ImportRow)> {
    if product["is1P"] != true || product["isThirdPartner"] != false || product["isKit"] != false {
        return None;
    }
    let sku = product["sku"]
        .as_str()?
        .parse::<i64>()
        .ok()
        .filter(|n| *n > 0)?;
    let path = product["url"].as_str()?;
    static PRODUCT_PATH: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^/[a-z0-9][a-z0-9-]*\.html$").unwrap());
    if !PRODUCT_PATH.is_match(path) {
        return None;
    }
    let amount = product["amount"].as_str()?.trim();
    let normalized = normalize::text(amount);
    static COUNT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^(\d+) (?:comprimidos?|capsulas?)(?: revestidos?| de liberacao prolongada| orodispersiveis)?$").unwrap()
    });
    let (quantity, unit) = if let Some(matched) = COUNT.captures(&normalized) {
        (matched[1].parse().ok()?, "un")
    } else {
        super::vtex::package(amount, &normalized).ok()?
    };
    let photo = product["image"]["src"]
        .as_str()
        .filter(|url| url.starts_with("https://product-data.raiadrogasil.io/images/"));
    let row = ImportRow {
        source_id: format!("rd-stock-{sku}"),
        branch_id: "drogasil-brescansin".into(),
        name: product["name"].as_str()?.trim().into(),
        brand: product["brand"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Não informada")
            .into(),
        quantity,
        unit: unit.into(),
        presentation: amount.into(),
        category: category.into(),
        // Preserve the code previously verified against the public product detail.
        gtin: (sku == 1268912).then(|| "7509546654034".into()),
        price_cents: None,
        promotional_price_cents: None,
        offer_type: "regular".into(),
        conditions: Some("Estoque consultado para retirada. A filial não informou o preço.".into()),
        available: None,
        source_url: Some(format!("{ORIGIN}{path}")),
        photo_source_url: photo.map(str::to_owned),
        collected_at: observed,
        valid_until: Some(observed + 1800),
        source_city: Some("Sorriso".into()),
        source_state: Some("MT".into()),
    };
    catalog::validate_for_sector(&row, "pharmacy").ok()?;
    Some((sku, row))
}

fn rows(reply: &Value, sku: i64, template: &ImportRow) -> Result<Vec<ImportRow>> {
    if let Some(errors) = reply["errors"]
        .as_array()
        .filter(|errors| !errors.is_empty())
    {
        ensure!(
            errors.len() == 1
                && errors[0]["path"] == json!(["getNearbyStockByZipCode"])
                && errors[0]["extensions"]["exception"]["status"] == 404
                && errors[0]["extensions"]["exception"]["code"] == "01",
            "consulta de retirada Drogasil rejeitada"
        );
        return Ok(Vec::new());
    }
    let branches = reply["data"]["getNearbyStockByZipCode"]
        .as_array()
        .context("consulta de retirada Drogasil sem resultado válido")?;
    let mut rows = Vec::new();
    for entry in branches {
        let Some(branch) = branch_id(&entry["branch"]) else {
            continue;
        };
        let Some(quantity) = entry["stocks"]
            .as_array()
            .and_then(|stocks| stocks.iter().find(|stock| stock["sku"] == sku))
            .and_then(|stock| stock["quantity"].as_u64())
        else {
            continue;
        };
        let mut row = template.clone();
        row.branch_id = branch.into();
        row.available = Some(quantity > 0);
        rows.push(row);
    }
    Ok(rows)
}

async fn request(pool: &SqlitePool, url: &str, stock: Option<(i64, &str)>) -> Result<String> {
    // Standard public frontend requests, without cookies, credentials or redirects.
    let mut command = tokio::process::Command::new("curl");
    command.args([
        "--disable",
        "--silent",
        "--show-error",
        "--fail",
        "--compressed",
        "--write-out",
        "\n%{http_code}\n%header{retry-after}",
        "--max-time",
        "15",
        "--max-filesize",
        "5000000",
    ]);
    if let Some((sku, referer)) = stock {
        command.args([
            "--header", "Content-Type: application/json", "--header",
            "Origin: https://www.drogasil.com.br", "--referer", referer, "--data-binary",
        ]).arg(json!({
            "operationName": "GET_STOCK", "query": STOCK_QUERY,
            "variables": {"zipcode":"78890168","products":[{"sku":sku.to_string(),"quantity":1}],"logotype":"RD","maxQuantityBranchSearch":"3"}
        }).to_string());
    }
    let response = command
        .arg(url)
        .kill_on_drop(true)
        .output()
        .await
        .context("não foi possível executar curl para consultar a Drogasil")?;
    let output = String::from_utf8(response.stdout)?;
    let mut parts = output.rsplitn(3, '\n');
    let retry = parts.next();
    let status = parts.next().unwrap_or_default();
    let body = parts
        .next()
        .context("consulta Drogasil sem resposta HTTP")?;
    if matches!(status, "429" | "503") {
        let delay = crate::adapters::retry_delay(retry, 0).as_secs().max(600);
        let retry_at = now().saturating_add(i64::try_from(delay).unwrap_or(i64::MAX));
        crate::adapters::defer_origin(pool, ORIGIN, retry_at).await?;
    }
    ensure!(
        response.status.success() && status == "200",
        "consulta Drogasil HTTP {status}: {}",
        String::from_utf8_lossy(&response.stderr).trim()
    );
    Ok(body.to_owned())
}

fn catalogue(html: &str, page: i64) -> Result<(Vec<Value>, bool)> {
    static NEXT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r#"(?s)<script id="__NEXT_DATA__"[^>]*>(.*?)</script>"#).unwrap()
    });
    let state: Value = serde_json::from_str(
        NEXT.captures(html)
            .context("estado público Drogasil ausente")?
            .get(1)
            .unwrap()
            .as_str(),
    )?;
    let listing = &state["props"]["pageProps"]["pageProps"];
    let metadata = &listing["metadata"];
    let limit = metadata["limit"]
        .as_i64()
        .context("limite do catálogo ausente")?;
    let pages = metadata["pages"]
        .as_i64()
        .context("total de páginas ausente")?;
    ensure!(
        (1..=96).contains(&limit) && (0..=5000).contains(&pages),
        "paginação fora dos limites"
    );
    ensure!(
        metadata["offset"].as_i64() == (page - 1).checked_mul(limit),
        "página Drogasil divergente"
    );
    let products = listing["results"]["products"]
        .as_array()
        .context("produtos Drogasil ausentes")?;
    ensure!(
        products.len() <= limit as usize,
        "página Drogasil acima do limite anunciado"
    );
    Ok((products.clone(), page < pages))
}

async fn scan(pool: &SqlitePool) -> Result<(usize, usize, usize)> {
    for path in CATEGORIES {
        sqlx::query("INSERT OR IGNORE INTO drogasil_catalogue_scan(path) VALUES(?)")
            .bind(path)
            .execute(pool)
            .await?;
    }
    let (path, page, item): (String, i64, i64) = sqlx::query_as(
        "SELECT path,page,item FROM drogasil_catalogue_scan ORDER BY last_attempt_at,rowid LIMIT 1",
    )
    .fetch_one(pool)
    .await?;
    ensure!(
        CATEGORIES.contains(&path.as_str()),
        "categoria Drogasil desconhecida"
    );
    sqlx::query("UPDATE drogasil_catalogue_scan SET last_attempt_at=? WHERE path=?")
        .bind(now())
        .bind(&path)
        .execute(pool)
        .await?;
    let url = format!("{ORIGIN}{path}?p={page}");
    let (products, has_next) = catalogue(&request(pool, &url, None).await?, page)?;
    let mut requests = 1;
    let mut imported = 0;
    let mut examined = 0;
    let mut position = item as usize;
    while position < products.len() && requests <= BUDGET {
        let product = &products[position];
        let category = product["categories"]
            .as_array()
            .and_then(|values| values.last())
            .and_then(|value| value["name"].as_str())
            .unwrap_or("Farmácia");
        let mut observations = Vec::new();
        if let Some((sku, row)) = candidate(product, category, now()) {
            tokio::time::sleep(SPACING).await;
            let response = request(
                pool,
                &format!("{ORIGIN}/api/next/product-hub/graphql"),
                Some((sku, &url)),
            )
            .await?;
            requests += 1;
            observations = rows(&serde_json::from_str(&response)?, sku, &row)?;
        }
        position += 1;
        examined += 1;
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        if !observations.is_empty() {
            imported +=
                catalog::import_in_transaction(&mut tx, "drogasil", &observations, false, "retail")
                    .await?;
        }
        sqlx::query("UPDATE drogasil_catalogue_scan SET item=? WHERE path=?")
            .bind(position as i64)
            .bind(&path)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
    }
    if position >= products.len() {
        sqlx::query("UPDATE drogasil_catalogue_scan SET page=?,item=0 WHERE path=?")
            .bind(if has_next { page + 1 } else { 1 })
            .bind(&path)
            .execute(pool)
            .await?;
    }
    Ok((requests, examined, imported))
}

pub(super) async fn collect(pool: &SqlitePool) -> Result<usize> {
    if crate::adapters::origin_paused(pool, ORIGIN).await? {
        return Ok(0);
    }
    sqlx::query("INSERT OR IGNORE INTO catalogue_collectors(supermarket_id,adapter,interval_seconds) VALUES('drogasil','drogasil-public-stock',600)")
        .execute(pool).await?;
    let claim = sqlx::query("UPDATE catalogue_collectors SET status='running',last_started_at=?,last_error=NULL WHERE supermarket_id='drogasil' AND (last_started_at IS NULL OR last_started_at<=?)")
        .bind(now()).bind(now() - 600).execute(pool).await?;
    if claim.rows_affected() == 0 {
        return Ok(0);
    }
    match scan(pool).await {
        Ok((requests, examined, imported)) => {
            sqlx::query("UPDATE catalogue_collectors SET status='success',last_success_at=?,last_finished_at=?,requests=?,raw_items=?,normalized_items=? WHERE supermarket_id='drogasil'")
                .bind(now()).bind(now()).bind(requests as i64).bind(examined as i64).bind(imported as i64).execute(pool).await?;
            Ok(imported)
        }
        Err(error) => {
            sqlx::query("UPDATE catalogue_collectors SET status='failed',last_finished_at=?,last_error=? WHERE supermarket_id='drogasil'")
                .bind(now()).bind(error.to_string()).execute(pool).await?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_first_party_packs_and_only_accepts_the_requested_local_stock() {
        let mut listed = json!({"sku":"41397", "url":"/prudence.html",
            "name":"Camisinha Prudence Lubrificada com 8 unidades", "brand":"Prudence",
            "is1P":true,"isThirdPartner":false,"isKit":false,"amount":"8un",
            "image":{"src":"https://product-data.raiadrogasil.io/images/3454925.webp"}});
        let (sku, item) = candidate(&listed, "Preservativos", now()).unwrap();
        assert_eq!((sku, item.quantity, item.unit.as_str()), (41397, 8, "un"));
        assert!(item.price_cents.is_none());
        assert!(item.photo_source_url.is_some());
        let branch = json!({"id":3852, "address":{"addressLocal":"Av Natalino João Brescansin",
            "addressNumber":"488","city":"Sorriso","sgState":"MT"},"logoType":{"description":"DROGASIL"}});
        let mut foreign = branch.clone();
        foreign["address"]["city"] = json!("Sinop");
        let response = json!({"data":{"getNearbyStockByZipCode":[
            {"branch":branch,"stocks":[{"sku":555,"quantity":8},{"sku":41397,"quantity":2}]},
            {"branch":foreign,"stocks":[{"sku":41397,"quantity":2}]}
        ]}});
        let local = rows(&response, sku, &item).unwrap();
        assert_eq!(local.len(), 1);
        assert_eq!(local[0].available, Some(true));
        assert!(local[0].price_cents.is_none());
        listed["isThirdPartner"] = json!(true);
        assert!(candidate(&listed, "Preservativos", now()).is_none());
        let no_stock = json!({"errors":[{"path":["getNearbyStockByZipCode"],
            "extensions":{"exception":{"status":404,"code":"01"}}}]});
        assert!(rows(&no_stock, sku, &item).unwrap().is_empty());
        assert!(rows(&json!({"errors":[{"message":"upstream"}]}), sku, &item).is_err());
    }

    #[test]
    fn rejects_repeated_first_page_when_a_later_page_is_requested() {
        let page = r#"<script id="__NEXT_DATA__" type="application/json">{"props":{"pageProps":{"pageProps":{"metadata":{"offset":0,"limit":48,"pages":2},"results":{"products":[]}}}}}</script>"#;
        assert!(catalogue(page, 1).unwrap().1);
        assert!(catalogue(page, 2).is_err());
    }

    #[test]
    fn identifies_a_local_branch_by_store_id_and_address() {
        let mut branch = json!({"id":3852, "address":{"addressLocal":"Av Natalino João Brescansin","addressNumber":"488","city":"Sorriso","sgState":"MT"},"logoType":{"description":"DROGASIL"}});
        assert_eq!(branch_id(&branch), Some("drogasil-brescansin"));
        branch["address"]["city"] = json!("Sinop");
        assert_eq!(branch_id(&branch), None);
    }
}
