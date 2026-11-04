//! VTEX storefront catalogue and branch-specific checkout validation.
mod scan;
use crate::{
    catalog::{self, ImportRow},
    normalize, now, photos,
};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use reqwest::{Client, Url};
use serde_json::Value;
use sqlx::SqlitePool;
use std::{sync::LazyLock, time::Duration};

const INTERVAL: Duration = Duration::from_secs(600);

struct Branch {
    market: &'static str,
    id: &'static str,
    host: &'static str,
    seller: &'static str,
    street: &'static str,
    number: &'static str,
    cep: &'static str,
}

const BRANCHES: &[Branch] = &[
    Branch {
        market: "drogaria-sao-paulo",
        id: "drogaria-sao-paulo-brescansin",
        host: "www.drogariasaopaulo.com.br",
        seller: "drogariaspsorriso",
        street: "natalino joao brescansin",
        number: "1722",
        cep: "78890-179",
    },
    Branch {
        market: "pague-menos",
        id: "pague-menos-brescansin",
        host: "www.paguemenos.com.br",
        seller: "paguemenos00980",
        street: "natalino joao brescansin",
        number: "1314",
        cep: "78890-134",
    },
    Branch {
        market: "pague-menos",
        id: "pague-menos-tancredo",
        host: "www.paguemenos.com.br",
        seller: "paguemenos01406",
        street: "tancredo neves",
        number: "2198",
        cep: "78890-143",
    },
];

fn simulated_item<'a>(
    body: &'a Value,
    branch: &Branch,
    sku: &str,
    index: usize,
) -> Option<&'a Value> {
    let items = body["items"].as_array()?;
    let item = items.iter().find(|item| {
        item["id"] == sku
            && (item["requestIndex"] == index
                || (items.len() == 1 && index == 0 && item["requestIndex"].is_null()))
    })?;
    (item["sellerChain"].as_array()?.last()?.as_str()? == branch.seller).then_some(item)
}

fn local_price(body: &Value, branch: &Branch, sku: &str, index: usize) -> Option<i64> {
    let item = simulated_item(body, branch, sku, index)?;
    if item["availability"] != "available"
        || item["quantity"]
            .as_i64()
            .is_some_and(|quantity| quantity != 1)
        || (body["items"].as_array()?.len() > 1
            && (item["priceTags"]
                .as_array()
                .is_none_or(|tags| !tags.is_empty())
                || item["price"] != item["sellingPrice"]))
    {
        return None;
    }
    let point_prefix = format!("{}_", branch.seller);
    let slas = body["logisticsInfo"]
        .as_array()?
        .iter()
        .filter(|line| line["itemIndex"] == index)
        .flat_map(|line| line["slas"].as_array().into_iter().flatten());
    let points = body["pickupPoints"].as_array()?;
    let matched = slas
        .filter(|sla| sla["deliveryChannel"] == "pickup-in-point")
        .any(|sla| {
            let Some(id) = sla["pickupPointId"].as_str() else {
                return false;
            };
            id.starts_with(&point_prefix)
                && points.iter().any(|p| {
                    let address = &p["address"];
                    p["id"] == id
                        && normalize::text(address["city"].as_str().unwrap_or_default())
                            == "sorriso"
                        && address["state"] == "MT"
                        && address["number"] == branch.number
                        && normalize::text(address["street"].as_str().unwrap_or_default())
                            .contains(branch.street)
                })
        });
    matched
        .then(|| item["sellingPrice"].as_i64().filter(|&p| p > 0))
        .flatten()
}

fn row(body: &Value, branch: &Branch, sku: &str, price: i64) -> Result<ImportRow> {
    let product = body
        .as_array()
        .context("catálogo inválido")?
        .iter()
        .find(|p| {
            p["items"]
                .as_array()
                .is_some_and(|items| items.iter().any(|item| item["itemId"] == sku))
        })
        .context("SKU ausente no catálogo")?;
    let item = product["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["itemId"] == sku)
        .unwrap();
    let name = item["name"].as_str().context("SKU sem nome")?;
    let normalized = normalize::text(name);
    let (quantity, unit) = package(name, &normalized)?;
    let (presentation, category) = if normalized.starts_with("creme dental colgate ") {
        ("Tubo", "Higiene bucal")
    } else if normalized.starts_with("preservativo ") || normalized.starts_with("camisinha ") {
        ("Pacote", "Saúde sexual")
    } else {
        (
            "Embalagem",
            product["categories"][0]
                .as_str()
                .and_then(|path| {
                    path.split('/')
                        .filter(|name| !name.trim().is_empty())
                        .next_back()
                })
                .unwrap_or("Farmácia"),
        )
    };
    let source = Url::parse(product["link"].as_str().context("SKU sem URL")?)?;
    ensure!(
        source.scheme() == "https"
            && source.host_str() == Some(branch.host)
            && source.path().ends_with("/p"),
        "origem do produto divergente"
    );
    let gtin = item["ean"]
        .as_str()
        .filter(|s| normalize::valid_gtin(s))
        .map(str::to_owned);
    let photo = item["images"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v["imageUrl"].as_str())
        .find_map(photos::trusted_url)
        .map(|url| url.to_string());
    let observed = now();
    let result = ImportRow {
        source_id: format!("vtex-{sku}"),
        branch_id: branch.id.into(),
        name: name.into(),
        brand: product["brand"].as_str().context("marca ausente")?.into(),
        quantity,
        unit: unit.into(),
        presentation: presentation.into(),
        category: category.chars().take(80).collect(),
        gtin,
        price_cents: Some(price),
        promotional_price_cents: None,
        offer_type: "regular".into(),
        conditions: Some("Preço online de uma unidade para retirada na filial indicada.".into()),
        available: Some(true),
        source_url: Some(source.to_string()),
        photo_source_url: photo,
        collected_at: observed,
        valid_until: Some(observed + 1800),
        source_city: Some("Sorriso".into()),
        source_state: Some("MT".into()),
    };
    catalog::validate_for_sector(&result, "pharmacy")?;
    Ok(result)
}

pub(super) fn package(name: &str, normalized: &str) -> Result<(i64, &'static str)> {
    static UNITS: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\b(\d+)\s*(?:un|uni|unidades?|comprimidos?|capsulas?|saches?|doses?|ampolas?|frascos?|supositorios?)$").unwrap()
    });
    static MEASURE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*(kg|g|ml|l)\s*$").unwrap());
    static MULTIPACK: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b\d+\s*[x×]\s*\d+").unwrap());
    ensure!(
        !normalized
            .split_whitespace()
            .any(|word| matches!(word, "kit" | "gratis" | "combo"))
            && !MULTIPACK.is_match(name),
        "embalagem composta exige revisão"
    );
    if let Some(count) = UNITS.captures(normalized) {
        return Ok((count[1].parse()?, "un"));
    }
    let measure = MEASURE
        .captures(name)
        .context("medida ou contagem explícita ausente")?;
    let (factor, unit) = match measure[2].to_ascii_lowercase().as_str() {
        "kg" => (1000, "g"),
        "l" => (1000, "ml"),
        "ml" => (1, "ml"),
        _ => (1, "g"),
    };
    Ok((normalize::scaled(&measure[1], factor)?, unit))
}

pub async fn run(pool: &SqlitePool, watch: bool) -> Result<()> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(12))
        .user_agent("SorriPrecos/0.1")
        .build()?;
    loop {
        let mut failed = false;
        for market in ["drogaria-sao-paulo", "pague-menos"] {
            if let Err(error) = scan::collect(pool, &client, market).await {
                failed = true;
                tracing::warn!(market, %error, "pharmacy_vtex_scan_failed");
                sqlx::query("INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds,status,last_finished_at,last_error) VALUES(?,'pharmacy-vtex',600,'failed',?,?) ON CONFLICT(supermarket_id) DO UPDATE SET status='failed',last_finished_at=excluded.last_finished_at,last_error=excluded.last_error")
                    .bind(market).bind(now()).bind(error.to_string()).execute(pool).await?;
            }
        }
        match super::drogasil::collect(pool).await {
            Ok(verified) => {
                tracing::info!(verified, "pharmacy_drogasil_stock_complete");
            }
            Err(error) => tracing::warn!(%error, "pharmacy_drogasil_stock_failed"),
        }
        if !watch {
            ensure!(!failed, "uma ou mais consultas VTEX falharam");
            return Ok(());
        }
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(INTERVAL) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_the_quantity_on_the_condom_package() {
        let branch = &BRANCHES[0];
        let mut body = json!([{
            "link": "https://www.drogariasaopaulo.com.br/preservativo/p",
            "brand": "Olla",
            "items": [{"itemId":"85332","name":"Preservativo Olla Lubrificado 3 Unidades","ean":"7896222716279"}]
        }]);
        let result = row(&body, branch, "85332", 1029).unwrap();
        assert_eq!((result.quantity, result.unit.as_str()), (3, "un"));
        assert_eq!(result.category, "Saúde sexual");
        body[0]["items"][0]["name"] = json!("Preservativo Olla Lubrificado");
        assert!(row(&body, branch, "85332", 1029).is_err());
    }

    #[test]
    fn batch_prices_require_item_specific_pickup_and_no_basket_discount() {
        let branch = &BRANCHES[0];
        let mut reply = json!({
            "items": [
                {"id":"85332","requestIndex":0,"quantity":1,"availability":"available","sellerChain":["drogariaspsorriso"],"price":1029,"sellingPrice":1029,"priceTags":[]},
                {"id":"18171","requestIndex":1,"quantity":1,"availability":"available","sellerChain":["drogariaspsorriso"],"price":599,"sellingPrice":499,"priceTags":[{"value":-100}]}
            ],
            "logisticsInfo": [{"itemIndex":0,"slas":[{"deliveryChannel":"pickup-in-point","pickupPointId":"drogariaspsorriso_2436"}]},
                              {"itemIndex":1,"slas":[{"deliveryChannel":"pickup-in-point","pickupPointId":"drogariaspsorriso_2436"}]}],
            "pickupPoints": [{"id":"drogariaspsorriso_2436","address":{"city":"Sorriso","state":"MT","street":"Avenida Natalino João Brescansin","number":"1722"}}]
        });
        assert_eq!(local_price(&reply, branch, "85332", 0), Some(1029));
        assert_eq!(local_price(&reply, branch, "18171", 1), None);
        reply["logisticsInfo"][0]["itemIndex"] = json!(1);
        assert_eq!(local_price(&reply, branch, "85332", 0), None);
    }

    #[test]
    fn online_price_requires_the_matching_seller_and_sorriso_pickup() {
        let branch = &BRANCHES[0];
        let mut reply = json!({
            "items": [{"id":"49697","availability":"available","sellerChain":["drogariaspsorriso"],"sellingPrice":559}],
            "logisticsInfo": [{"itemIndex":0,"slas":[{"deliveryChannel":"pickup-in-point","pickupPointId":"drogariaspsorriso_2436","price":0}]}],
            "pickupPoints":[{"id":"drogariaspsorriso_2436","address":{"city":"Sorriso","state":"MT","street":"Avenida Natalino João Brescansin","number":"1722"}}]
        });
        assert_eq!(local_price(&reply, branch, "49697", 0), Some(559));
        assert_eq!(local_price(&reply, branch, "99999", 0), None);
        reply["items"][0]["sellerChain"] = json!(["1", "drogariaspcuiaba"]);
        assert_eq!(local_price(&reply, branch, "49697", 0), None);
        reply["items"][0]["sellerChain"] = json!(["drogariaspsorriso"]);
        reply["pickupPoints"][0]["address"]["city"] = json!("Sinop");
        assert_eq!(local_price(&reply, branch, "49697", 0), None);
    }
}
