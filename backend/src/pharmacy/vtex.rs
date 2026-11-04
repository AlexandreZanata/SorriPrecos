//! Small, verified pharmacy catalogue. Only the public simulation proves a local offer.
use crate::{
    catalog::{self, ImportRow},
    normalize, now, photos,
};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use reqwest::{Client, StatusCode, Url};
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::{sync::LazyLock, time::Duration};

const INTERVAL: Duration = Duration::from_secs(600);
const DSP_SKUS: &[&str] = &["49697", "895083", "886513"];
const PM_SKUS: &[&str] = &["52572", "6958", "164998", "164188", "171834"];

struct Branch {
    market: &'static str,
    id: &'static str,
    host: &'static str,
    seller: &'static str,
    street: &'static str,
    number: &'static str,
    cep: &'static str,
    skus: &'static [&'static str],
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
        skus: DSP_SKUS,
    },
    Branch {
        market: "pague-menos",
        id: "pague-menos-brescansin",
        host: "www.paguemenos.com.br",
        seller: "paguemenos00980",
        street: "natalino joao brescansin",
        number: "1314",
        cep: "78890-134",
        skus: PM_SKUS,
    },
    Branch {
        market: "pague-menos",
        id: "pague-menos-tancredo",
        host: "www.paguemenos.com.br",
        seller: "paguemenos01406",
        street: "tancredo neves",
        number: "2198",
        cep: "78890-143",
        skus: PM_SKUS,
    },
];

fn local_price(body: &Value, branch: &Branch, sku: &str) -> Option<i64> {
    let items = body["items"].as_array()?;
    let [item] = items.as_slice() else {
        return None;
    };
    if item["id"] != sku
        || item["availability"] != "available"
        || item["sellerChain"].as_array()?.last()?.as_str()? != branch.seller
    {
        return None;
    }
    let point_prefix = format!("{}_", branch.seller);
    let slas = body["logisticsInfo"]
        .as_array()?
        .iter()
        .filter(|line| line["itemIndex"] == 0)
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
    static WEIGHT: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*g\s*$").unwrap());
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
    ensure!(
        normalized.starts_with("creme dental colgate ") && !normalized.contains("kit"),
        "produto fora da seleção de higiene"
    );
    let quantity = normalize::scaled(&WEIGHT.captures(name).context("peso ausente")?[1], 1)?;
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
        unit: "g".into(),
        presentation: "Tubo".into(),
        category: "Higiene bucal".into(),
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

async fn observe(client: &Client, branch: &Branch, sku: &str) -> Result<Option<ImportRow>> {
    let base = format!("https://{}", branch.host);
    let request = json!({"items":[{"id":sku,"quantity":1,"seller":branch.seller}],"country":"BRA","postalCode":branch.cep});
    let response = client
        .post(format!("{base}/api/checkout/pub/orderForms/simulation"))
        .json(&request)
        .send()
        .await?;
    let simulation: Value = serde_json::from_str(&super::read(response).await?)?;
    let Some(price) = local_price(&simulation, branch, sku) else {
        return Ok(None);
    };
    let response = client
        .get(format!("{base}/api/catalog_system/pub/products/search"))
        .query(&[("fq", format!("skuId:{sku}"))])
        .send()
        .await?;
    let metadata: Value = serde_json::from_str(&super::read(response).await?)?;
    Ok(Some(row(&metadata, branch, sku, price)?))
}

pub async fn run(pool: &SqlitePool, watch: bool) -> Result<()> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(12))
        .user_agent("SorriPrecos/0.1")
        .build()?;
    loop {
        let mut limited_host = None;
        let mut total = 0;
        for branch in BRANCHES {
            if limited_host == Some(branch.host) {
                continue;
            }
            let mut rows = Vec::new();
            for sku in branch.skus {
                tokio::time::sleep(Duration::from_millis(750)).await;
                match observe(&client, branch, sku).await {
                    Ok(Some(row)) => rows.push(row),
                    Ok(None) => {}
                    Err(error) => {
                        tracing::warn!(branch = branch.id, sku, %error, "pharmacy_vtex_observation_failed");
                        if error
                            .downcast_ref::<reqwest::Error>()
                            .and_then(|e| e.status())
                            == Some(StatusCode::TOO_MANY_REQUESTS)
                        {
                            limited_host = Some(branch.host);
                            break;
                        }
                    }
                }
            }
            if !rows.is_empty() {
                total += catalog::import(
                    pool,
                    branch.market,
                    &rows,
                    Some(
                        "PUBLIC_STOREFRONT: simulação de retirada com vendedor e filial Sorriso-MT",
                    ),
                )
                .await?;
            }
            tracing::info!(
                branch = branch.id,
                verified = rows.len(),
                "pharmacy_vtex_branch_complete"
            );
        }
        match super::drogasil::collect(pool).await {
            Ok(verified) => {
                total += verified;
                tracing::info!(verified, "pharmacy_drogasil_stock_complete");
            }
            Err(error) => tracing::warn!(%error, "pharmacy_drogasil_stock_failed"),
        }
        if !watch {
            ensure!(
                total > 0,
                "nenhuma oferta VTEX local confirmada nesta rodada"
            );
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

    #[test]
    fn online_price_requires_the_matching_seller_and_sorriso_pickup() {
        let branch = &BRANCHES[0];
        let mut reply = json!({
            "items": [{"id":"49697","availability":"available","sellerChain":["drogariaspsorriso"],"sellingPrice":559}],
            "logisticsInfo": [{"itemIndex":0,"slas":[{"deliveryChannel":"pickup-in-point","pickupPointId":"drogariaspsorriso_2436","price":0}]}],
            "pickupPoints":[{"id":"drogariaspsorriso_2436","address":{"city":"Sorriso","state":"MT","street":"Avenida Natalino João Brescansin","number":"1722"}}]
        });
        assert_eq!(local_price(&reply, branch, "49697"), Some(559));
        assert_eq!(local_price(&reply, branch, "99999"), None);
        reply["items"][0]["sellerChain"] = json!(["1", "drogariaspcuiaba"]);
        assert_eq!(local_price(&reply, branch, "49697"), None);
        reply["items"][0]["sellerChain"] = json!(["drogariaspsorriso"]);
        reply["pickupPoints"][0]["address"]["city"] = json!("Sinop");
        assert_eq!(local_price(&reply, branch, "49697"), None);
    }
}
