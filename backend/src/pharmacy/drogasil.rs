//! Public pickup stock by store. The network's price endpoint is not scoped to a store.
use crate::{
    catalog::{self, ImportRow},
    normalize, now,
};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use sqlx::SqlitePool;

const ORIGIN: &str = "https://www.drogasil.com.br";
const PRODUCT: &str =
    "https://www.drogasil.com.br/colgate-creme-dental-total-anti-tartaro-90g-1268912.html";
const SKU: i64 = 1268912;
// Identity from the public product page's JSON-LD, verified for this exact SKU.
const NAME: &str = "Creme Dental Colgate Total Prevenção Ativa Antitártaro 90g";
const GTIN: &str = "7509546654034";
const PHOTO: &str = "https://product-data.raiadrogasil.io/images/18425022.webp";
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

fn rows(reply: &Value, observed: i64) -> Result<Vec<ImportRow>> {
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
            .and_then(|stocks| stocks.iter().find(|stock| stock["sku"] == SKU))
            .and_then(|stock| stock["quantity"].as_u64())
        else {
            continue;
        };
        rows.push(ImportRow {
            source_id: format!("rd-stock-{SKU}"),
            branch_id: branch.into(),
            name: NAME.into(),
            brand: "Colgate".into(),
            quantity: 90,
            unit: "g".into(),
            presentation: "Tubo".into(),
            category: "Higiene bucal".into(),
            gtin: Some(GTIN.into()),
            price_cents: None,
            promotional_price_cents: None,
            offer_type: "regular".into(),
            conditions: Some(
                "Estoque consultado para retirada. A filial não informou o preço.".into(),
            ),
            available: Some(quantity > 0),
            source_url: Some(PRODUCT.into()),
            photo_source_url: Some(PHOTO.into()),
            collected_at: observed,
            valid_until: Some(observed + 1800),
            source_city: Some("Sorriso".into()),
            source_state: Some("MT".into()),
        });
    }
    Ok(rows)
}

pub(super) async fn collect(pool: &SqlitePool) -> Result<usize> {
    let request = json!({
        "operationName": "GET_STOCK", "query": STOCK_QUERY,
        "variables": {"zipcode":"78890168","products":[{"sku":SKU.to_string(),"quantity":1}],"logotype":"RD","maxQuantityBranchSearch":"3"}
    });
    // The public endpoint accepts standard curl requests but rejects the shared
    // VTEX HTTP client. Use curl directly, without cookies, credentials or redirects.
    let response = tokio::process::Command::new("curl")
        .args([
            "--disable",
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            "12",
            "--max-filesize",
            "262144",
            "--header",
            "Content-Type: application/json",
            "--header",
            "Origin: https://www.drogasil.com.br",
            "--referer",
            PRODUCT,
            "--data-binary",
        ])
        .arg(request.to_string())
        .arg(format!("{ORIGIN}/api/next/product-hub/graphql"))
        .kill_on_drop(true)
        .output()
        .await
        .context("não foi possível executar curl para consultar a Drogasil")?;
    anyhow::ensure!(
        response.status.success(),
        "consulta de estoque Drogasil: {}",
        String::from_utf8_lossy(&response.stderr).trim()
    );
    let stock: Value = serde_json::from_slice(&response.stdout)?;
    let rows = rows(&stock, now())?;
    if rows.is_empty() {
        return Ok(0);
    }
    catalog::import(
        pool,
        "drogasil",
        &rows,
        Some("PUBLIC_STOREFRONT: estoque de retirada consultado por filial Drogasil Sorriso-MT"),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_a_local_branch_by_store_id_and_address() {
        let mut branch = json!({"id":3852, "address":{"addressLocal":"Av Natalino João Brescansin","addressNumber":"488","city":"Sorriso","sgState":"MT"},"logoType":{"description":"DROGASIL"}});
        assert_eq!(branch_id(&branch), Some("drogasil-brescansin"));
        branch["address"]["city"] = json!("Sinop");
        assert_eq!(branch_id(&branch), None);
    }
}
