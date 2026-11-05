//! Read the same public, embedded catalogue that the aiqfome storefront renders.
mod schedule;
use crate::{
    catalog::{self, ImportRow},
    normalize, now, vip,
};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use schedule::RateLimited;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqlitePool;
use std::{collections::HashSet, path::Path, sync::LazyLock, time::Duration};

const INTERVAL: i64 = 600;
const MAX_PAGE_BYTES: usize = 16 * 1024 * 1024;
const EGAO_ORIGIN: &str = "https://pedido.egaosushi.com.br";
const DMACENA_MENU: &str = "https://pizzariadmacena.meia7solucoes.com.br/cardapio/pedidos";

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum Provider {
    #[default]
    Aiqfome,
    Egao,
    Dmacena,
    Ola,
}

impl Provider {
    fn adapter(self) -> &'static str {
        match self {
            Self::Aiqfome => "aiqfome",
            Self::Egao => "egao",
            Self::Dmacena => "dmacena",
            Self::Ola => "ola",
        }
    }

    fn sku_pattern(self) -> &'static str {
        match self {
            Self::Aiqfome => "aiq-%",
            Self::Egao => "egao-%",
            Self::Dmacena => "dmacena-%",
            Self::Ola => "ola-%",
        }
    }
}
const BASIS: &str = "PUBLIC_STOREFRONT: catálogo público com identidade local verificada";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    #[serde(default)]
    provider: Provider,
    supermarket_id: String,
    branch_id: String,
    #[serde(default)]
    slug: String,
    #[serde(default)]
    storefront_id: String,
    #[serde(default)]
    seller_id: i64,
    #[serde(default)]
    cnpj: String,
    street: String,
    #[serde(default)]
    number: String,
    #[serde(default)]
    postcode: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    allow_portions: bool,
}

fn digits(value: &str) -> String {
    value.chars().filter(char::is_ascii_digit).collect()
}

fn verify(page: &Value, source: &Source) -> Result<()> {
    let cnpj = digits(page["cnpj"].as_str().unwrap_or_default());
    let street = normalize::text(page["logradouro"].as_str().unwrap_or_default());
    let postcode = digits(page["cep"].as_str().unwrap_or_default());
    ensure!(
        page["id"] == source.seller_id
            && page["menu"]["restaurante_id"] == source.seller_id
            && page["slug"] == source.slug
            && page["cidade_id"] == 4430
            && page["cidade"]["id"] == 4430
            && page["cidade"]["estado_id"] == 13
            && page["cidade"]["nome"] == "Sorriso"
            && cnpj == source.cnpj
            && street == normalize::text(&source.street)
            && page["numero"] == source.number
            && postcode == source.postcode,
        "identidade ou endereço do vendedor diverge da filial de Sorriso-MT: seller={} menu={} slug={} cidade_id={} cidade={} estado={} nome={} cnpj={} rua={} numero={} cep={}",
        page["id"],
        page["menu"]["restaurante_id"],
        page["slug"],
        page["cidade_id"],
        page["cidade"]["id"],
        page["cidade"]["estado_id"],
        page["cidade"]["nome"],
        cnpj,
        street,
        page["numero"],
        postcode
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

static SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(\d+(?:[.,]\d+)?)\s*(kg|gramas?|g|ml|litros?|lt|l)\b").unwrap()
});
static EXACT_SIZE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(\d+(?:[.,]\d+)?)\s*(kg|gramas?|g|ml|litros?|lt|l)\s*$").unwrap()
});

static MULTIPACK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(?i)\d\s*[x×]\s*\d").unwrap());
static MULTIPLIER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^\s*(?:cx\s*c?\s*-?\s*|caixa\s+(?:com\s+)?|grade\s+(?:com\s+)?|fardo\s+(?:com\s+)?)?(\d+)\s*(?:uni(?:d(?:ades?)?)?)?\s*$").unwrap()
});

fn measure(capture: &regex::Captures<'_>) -> Result<(i64, &'static str)> {
    let (factor, unit) = match capture[2].to_ascii_lowercase().as_str() {
        "kg" => (1000, "g"),
        "l" | "lt" | "litro" | "litros" => (1000, "ml"),
        "g" | "grama" | "gramas" => (1, "g"),
        _ => (1, "ml"),
    };
    Ok((normalize::scaled(&capture[1], factor)?, unit))
}

fn package(label: &str, name: &str) -> Result<(i64, String)> {
    let mut title_sizes = SIZE.captures_iter(name);
    let title_size = title_sizes.next().as_ref().map(measure).transpose()?;
    ensure!(
        title_sizes.next().is_none() && !MULTIPACK.is_match(name),
        "embalagem múltipla ou ambígua"
    );
    let label = label.trim();
    if let Some(capture) = MULTIPLIER.captures(label) {
        let units: i64 = capture[1].parse()?;
        ensure!(
            (1..=100).contains(&units),
            "quantidade de embalagem inválida"
        );
        let (quantity, unit) = title_size.context("caixa sem peso ou volume confirmado")?;
        let quantity = quantity
            .checked_mul(units)
            .context("quantidade de embalagem inválida")?;
        return Ok((quantity, unit.into()));
    }
    let size = if let Some(capture) = EXACT_SIZE.captures(label) {
        measure(&capture)?
    } else if !label.bytes().any(|byte| byte.is_ascii_digit()) {
        title_size.context("unidade sem peso ou volume confirmado")?
    } else {
        let mut matches = SIZE.find_iter(label);
        let matched = matches
            .next()
            .filter(|_| matches.next().is_none())
            .context("variante sem peso ou volume confirmado")?;
        measure(&SIZE.captures(matched.as_str()).unwrap())?
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
    available: Option<bool>,
    reason: Option<String>,
}

#[derive(Default, Serialize)]
struct Coverage {
    total_categories: usize,
    selected_categories: usize,
    total_items: usize,
    selected_items: usize,
    total_variants: usize,
    selected_variants: usize,
    skipped_categories: Vec<String>,
}

fn normalize_menu(
    page: &Value,
    source: &Source,
    url: &str,
    observed: i64,
) -> Result<(Vec<Observation>, Vec<ImportRow>, i64, Coverage)> {
    let mut rejected_scope = 0;
    let mut observations = Vec::new();
    let mut rows = Vec::new();
    let mut seen = HashSet::new();
    let mut coverage = Coverage::default();
    for category in entries(&page["menu"]["cardapios_categorias"])? {
        let name = category["nome"]
            .as_str()
            .context("categoria sem nome")?
            .trim();
        let items: Vec<_> = entries(&category["itens"])?.collect();
        let variants: usize = items
            .iter()
            .map(|item| entries(&item["itens_tamanhos"]).map(Iterator::count))
            .try_fold(0usize, |total, count| count.map(|count| total + count))?;
        coverage.total_categories += 1;
        coverage.total_items += items.len();
        coverage.total_variants += variants;
        if !source.categories.is_empty()
            && !source
                .categories
                .iter()
                .any(|value| *value == normalize::text(name))
        {
            coverage.skipped_categories.push(name.to_owned());
            continue;
        }
        coverage.selected_categories += 1;
        coverage.selected_items += items.len();
        coverage.selected_variants += variants;
        for item in items {
            let title = item["nome"].as_str().context("produto sem nome")?.trim();
            for variant in entries(&item["itens_tamanhos"])? {
                if variant["restaurante_id"] != source.seller_id || variant["item_id"] != item["id"]
                {
                    rejected_scope += 1;
                    continue;
                }
                let id = format!("aiq-{}", variant["id"].as_i64().context("variante sem ID")?);
                if !seen.insert(id.clone()) {
                    continue;
                }
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
                    let portion = source.allow_portions;
                    let mut size_matches = SIZE.find_iter(size);
                    let measured_label = if portion && !MULTIPACK.is_match(size) {
                        size_matches
                            .next()
                            .filter(|_| size_matches.next().is_none())
                            .map(|matched| matched.as_str())
                            .unwrap_or(size)
                    } else {
                        size
                    };
                    let (quantity, unit) = match package(measured_label, title) {
                        Ok(package) => package,
                        Err(_)
                            if portion
                                && !size.trim().is_empty()
                                && !SIZE.is_match(size)
                                && !SIZE.is_match(title) =>
                        {
                            // One order of the named variant, with no inferred weight.
                            (1, "un".into())
                        }
                        Err(error) => return Err(error),
                    };
                    let gtin = variant["ean_code"]
                        .as_str()
                        .filter(|gtin| !portion && normalize::valid_gtin(gtin))
                        .map(str::to_owned);
                    let presentation = if portion {
                        format!("Porção · {}", source.seller_id)
                    } else if gtin.is_some() {
                        "unidade de venda".into()
                    } else {
                        format!("variante aiqfome {}", source.seller_id)
                    };
                    let row = ImportRow {
                        source_id: id.clone(),
                        branch_id: source.branch_id.clone(),
                        name: if portion {
                            format!("{title} · {}", size.trim())
                        } else {
                            title.into()
                        },
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
                        photo_source_url: page["item_dir"]
                            .as_str()
                            .zip(item["imagem"].as_str())
                            .filter(|(dir, file)| {
                                !dir.is_empty()
                                    && !file.is_empty()
                                    && dir.bytes().all(|b| b.is_ascii_alphanumeric())
                                    && file
                                        .bytes()
                                        .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                                    && item["imagem_oculta"] != true
                            })
                            .map(|(dir, file)| {
                                format!("https://files.aiqfome.com/itens/{dir}/{file}")
                            }),
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
                    available: Some(available),
                    reason,
                });
            }
        }
    }
    Ok((observations, rows, rejected_scope, coverage))
}

async fn fetch_body(client: &reqwest::Client, url: &str) -> Result<Vec<u8>> {
    tokio::time::sleep(Duration::from_secs(1)).await;
    let response = client.get(url).send().await?;
    if matches!(
        response.status(),
        reqwest::StatusCode::TOO_MANY_REQUESTS | reqwest::StatusCode::SERVICE_UNAVAILABLE
    ) {
        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok());
        let delay =
            crate::adapters::retry_delay(retry_after, 0).max(Duration::from_secs(INTERVAL as u64));
        return Err(RateLimited(delay).into());
    }
    let mut response = response.error_for_status()?;
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        ensure!(
            body.len() + chunk.len() <= MAX_PAGE_BYTES,
            "página excede 16 MiB"
        );
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn fetch(client: &reqwest::Client, url: &str) -> Result<Value> {
    let body = fetch_body(client, url).await?;
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

fn normalize_options(
    menu: &Value,
    source: &Source,
    observed: i64,
) -> Result<(Vec<Observation>, Vec<ImportRow>, Coverage)> {
    let mut observations = Vec::new();
    let mut rows = Vec::new();
    let mut coverage = if let Some(value) = menu.get("cobertura") {
        Coverage {
            total_categories: value["total_categories"].as_u64().unwrap_or(0) as usize,
            selected_categories: value["selected_categories"].as_u64().unwrap_or(0) as usize,
            total_items: value["total_items"].as_u64().unwrap_or(0) as usize,
            selected_items: value["selected_items"].as_u64().unwrap_or(0) as usize,
            total_variants: value["total_variants"].as_u64().unwrap_or(0) as usize,
            selected_variants: value["selected_variants"].as_u64().unwrap_or(0) as usize,
            skipped_categories: value["skipped_categories"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
        }
    } else {
        Coverage::default()
    };
    let categories: Vec<_> = entries(&menu["categorias"])?.collect();
    let items: Vec<_> = entries(&menu["itens"])?.collect();
    let variants: usize = items
        .iter()
        .map(|item| entries(&item["opcoes"]).map(Iterator::count))
        .try_fold(0usize, |total, count| count.map(|count| total + count))?;
    coverage.total_categories = coverage.total_categories.max(categories.len());
    coverage.selected_categories = coverage.selected_categories.max(categories.len());
    coverage.total_items = coverage.total_items.max(items.len());
    coverage.selected_items = coverage.selected_items.max(items.len());
    coverage.total_variants = coverage.total_variants.max(variants);
    coverage.selected_variants = coverage.selected_variants.max(variants);
    let (brand, url) = match source.provider {
        Provider::Egao => ("Egao Sushi".to_owned(), EGAO_ORIGIN.to_owned()),
        Provider::Dmacena => ("Pizzaria D’Macena".to_owned(), DMACENA_MENU.to_owned()),
        Provider::Ola => (
            source.supermarket_id.clone(),
            format!("https://{}.ola.click/products", source.slug),
        ),
        Provider::Aiqfome => anyhow::bail!("vitrine com formato incompatível"),
    };
    let mut ids = HashSet::new();
    for item in entries(&menu["itens"])? {
        let item_id = item["id"].as_str().context("item sem ID")?;
        let title = item["nome"].as_str().context("item sem nome")?;
        let category_slug = item["categoria"].as_str().context("item sem categoria")?;
        let category = entries(&menu["categorias"])?
            .find(|entry| entry["slug"] == category_slug)
            .and_then(|entry| entry["nome"].as_str())
            .context("categoria desconhecida")?;
        for option in entries(&item["opcoes"])? {
            let label = option["rotulo"]
                .as_str()
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let id = format!(
                "{}-{item_id}-{}",
                source.provider.adapter(),
                label.map(normalize::text).unwrap_or_default()
            );
            if !ids.insert(id.clone()) {
                continue;
            }
            let available = item["disponivel"].as_bool();
            let parsed = (|| -> Result<ImportRow> {
                ensure!(
                    option["dependent_price"] != true,
                    "preço depende de complemento ou condição"
                );
                let price = vip::cents(&option["preco"])?;
                let name =
                    label.map_or_else(|| title.to_owned(), |size| format!("{title} · {size}"));
                let row = ImportRow {
                    source_id: id.clone(),
                    branch_id: source.branch_id.clone(),
                    name,
                    brand: brand.clone(),
                    quantity: 1,
                    unit: "un".into(),
                    presentation: format!("Porção · {brand}"),
                    category: category.into(),
                    gtin: None,
                    price_cents: Some(price),
                    promotional_price_cents: None,
                    offer_type: "regular".into(),
                    conditions: None,
                    available,
                    source_url: Some(url.clone()),
                    photo_source_url: item["foto"].as_str().map(|photo| {
                        if matches!(source.provider, Provider::Egao)
                            && photo.starts_with("/api/fotos/")
                        {
                            format!("{EGAO_ORIGIN}{photo}")
                        } else {
                            photo.to_owned()
                        }
                    }),
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
                size: label.unwrap_or("porção").into(),
                category: category.into(),
                price: option["preco"].clone(),
                available,
                reason,
            });
        }
    }
    Ok((observations, rows, coverage))
}

fn dmacena_options(catalog: &Value) -> Result<Value> {
    let mut categories = Vec::new();
    let mut items = Vec::new();
    for category in entries(&catalog["categorias"])? {
        let id = category["id"]
            .as_i64()
            .context("categoria sem ID")?
            .to_string();
        categories.push(serde_json::json!({"slug": id, "nome": category["nome"]}));
        for item in entries(&category["itens"])? {
            let item_id = item["id"].as_i64().context("item sem ID")?.to_string();
            ensure!(
                item["categoria_id"] == category["id"],
                "item em categoria divergente"
            );
            let configurable = entries(&item["categorias_complementos"])?
                .any(|group| group["obrigatorio"] != false);
            let fixed_price = !configurable
                && item["somente_por_pontos"] == false
                && item["qtd_minima"]
                    .as_i64()
                    .is_some_and(|minimum| minimum <= 1);
            let listed = item["status"] == "disponivel"
                && category["status"] == "disponivel"
                && item["disponivel_delivery"] == true;
            let all_days = item["disponibilidade"]["sempre_disponivel"] == true
                && category["disponibilidade"]["sempre_disponivel"] == true;
            let availability = if !listed {
                Some(false)
            } else {
                all_days.then_some(true)
            };
            items.push(serde_json::json!({
                "id": item_id, "nome": item["nome"], "categoria": id, "disponivel": availability,
                "foto": item["foto_principal"],
                "opcoes": [{"preco": item["preco"], "dependent_price": !fixed_price}]
            }));
        }
    }
    Ok(serde_json::json!({"categorias": categories, "itens": items}))
}

async fn collect(pool: &SqlitePool, client: &reqwest::Client, source: &Source) -> Result<()> {
    ensure!(
        !matches!(source.provider, Provider::Aiqfome | Provider::Ola)
            || (!source.slug.is_empty()
                && source
                    .slug
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')),
        "slug aiqfome inválido"
    );
    let branch_ok: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branches WHERE id=? AND supermarket_id=? AND city='Sorriso' AND state='MT' AND active=1)")
        .bind(&source.branch_id).bind(&source.supermarket_id).fetch_one(pool).await?;
    ensure!(branch_ok, "filial local não cadastrada");

    sqlx::query("INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds) VALUES(?,?,?) ON CONFLICT(supermarket_id) DO NOTHING")
        .bind(&source.supermarket_id).bind(source.provider.adapter()).bind(INTERVAL).execute(pool).await?;
    let started = now();
    let claimed = sqlx::query("UPDATE catalogue_collectors SET status='running',last_started_at=?,last_error=NULL WHERE supermarket_id=? AND adapter=? AND (last_started_at IS NULL OR last_started_at<=?) AND (status!='running' OR last_started_at<?)")
        .bind(started).bind(&source.supermarket_id).bind(source.provider.adapter()).bind(started - INTERVAL).bind(started - 1800).execute(pool).await?;
    if claimed.rows_affected() == 0 {
        return Ok(());
    }

    let url = format!("https://aiqfome.com/MT/sorriso/{}", source.slug);
    let mut requests = 0;
    let mut raw_count = 0;
    let mut imported = 0;
    let mut scope_rejected = 0;
    let result: Result<()> = async {
        let observed = now();
        let (observations, rows, coverage) = match source.provider {
            Provider::Aiqfome => {
                requests += 1;
                let page = fetch(client, &url).await?;
                if let Err(error) = verify(&page, source) {
                    scope_rejected = 1;
                    return Err(error);
                }
                let (observations, rows, rejected, coverage) =
                    normalize_menu(&page, source, &url, observed)?;
                scope_rejected = rejected;
                (observations, rows, coverage)
            }
            Provider::Egao => {
                requests += 1;
                let shop: Value = serde_json::from_slice(
                    &fetch_body(client, &format!("{EGAO_ORIGIN}/api/config")).await?,
                )?;
                let address = normalize::text(shop["endereco"].as_str().unwrap_or_default());
                if address != normalize::text(&source.street) || !address.ends_with("sorriso mt") {
                    scope_rejected = 1;
                    anyhow::bail!("endereço da vitrine Egao diverge da filial de Sorriso-MT");
                }
                let open = shop["aberta"].as_bool().context("horário da loja ausente")?;
                requests += 1;
                let mut menu: Value = serde_json::from_slice(
                    &fetch_body(client, &format!("{EGAO_ORIGIN}/api/cardapio")).await?,
                )?;
                if !open {
                    for item in menu["itens"].as_array_mut().context("lista de itens inválida")? {
                        item["disponivel"] = Value::Bool(false);
                    }
                }
                normalize_options(&menu, source, observed)?
            }
            Provider::Dmacena => {
                requests += 1;
                let body = fetch_body(client, DMACENA_MENU).await?;
                let script = std::str::from_utf8(&body)?.split_once("window.__INITIAL_DATA__ = ")
                    .context("catálogo D’Macena ausente")?.1;
                let data = serde_json::Deserializer::from_str(script).into_iter::<Value>()
                    .next().context("catálogo D’Macena ausente")??;
                let shop = &data["loja"];
                let address = &shop["endereco"];
                if shop["id"] != source.seller_id
                    || digits(shop["cnpj"].as_str().unwrap_or_default()) != source.cnpj
                    || address["cidade"] != "Sorriso" || address["uf"] != "MT"
                    || address["numero"] != source.number
                    || normalize::text(address["logradouro"].as_str().unwrap_or_default()) != normalize::text(&source.street)
                    || digits(address["cep"].as_str().unwrap_or_default()) != source.postcode
                {
                    scope_rejected = 1;
                    anyhow::bail!("identidade ou endereço D’Macena diverge da filial de Sorriso-MT");
                }
                let menu = dmacena_options(&data["catalogo"])?;
                normalize_options(&menu, source, observed)?
            }
            Provider::Ola => {
                requests += 1;
                let url = format!("https://{}.ola.click/products", source.slug);
                let body = fetch_body(client, &url).await?;
                let menu = crate::ola::parse(&body, &source.storefront_id, &source.street, &source.categories)?;
                normalize_options(&menu, source, observed)?
            }
        };
        raw_count = observations.len() as i64;
        ensure!(!rows.is_empty(), "catálogo sem variantes de mercearia utilizáveis");
        let mut tx = pool.begin_with("BEGIN IMMEDIATE").await?;
        let count = catalog::import_in_transaction(&mut tx, &source.supermarket_id, &rows, false, "retail").await? as i64;
        sqlx::query("INSERT INTO source_coverage(supermarket_id,branch_id,adapter,observed_at,total_categories,selected_categories,total_items,selected_items,total_variants,selected_variants,skipped_categories) VALUES(?,?,?,?,?,?,?,?,?,?,?) ON CONFLICT(supermarket_id,branch_id) DO UPDATE SET adapter=excluded.adapter,observed_at=excluded.observed_at,total_categories=excluded.total_categories,selected_categories=excluded.selected_categories,total_items=excluded.total_items,selected_items=excluded.selected_items,total_variants=excluded.total_variants,selected_variants=excluded.selected_variants,skipped_categories=excluded.skipped_categories")
            .bind(&source.supermarket_id).bind(&source.branch_id).bind(source.provider.adapter()).bind(observed)
            .bind(coverage.total_categories as i64).bind(coverage.selected_categories as i64)
            .bind(coverage.total_items as i64).bind(coverage.selected_items as i64)
            .bind(coverage.total_variants as i64).bind(coverage.selected_variants as i64)
            .bind(serde_json::to_string(&coverage.skipped_categories)?)
            .execute(&mut *tx).await?;
        let run: i64 = sqlx::query_scalar("INSERT INTO collection_runs(supermarket_id,started_at,finished_at,status,authorization_reference,requests,imported) VALUES(?,?,?,'success',?,?,?) RETURNING id")
            .bind(&source.supermarket_id).bind(started).bind(now()).bind(BASIS).bind(requests).bind(count).fetch_one(&mut *tx).await?;
        for observation in observations {
            sqlx::query("INSERT INTO source_catalog_items(supermarket_id,branch_id,source_id,raw_json,observed_at,run_id,normalized,reason) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(supermarket_id,branch_id,source_id) DO UPDATE SET raw_json=excluded.raw_json,observed_at=excluded.observed_at,run_id=excluded.run_id,normalized=excluded.normalized,reason=excluded.reason")
                .bind(&source.supermarket_id).bind(&source.branch_id).bind(&observation.id)
                .bind(serde_json::to_string(&observation)?).bind(observed).bind(run)
                .bind(observation.reason.is_none()).bind(&observation.reason).execute(&mut *tx).await?;
        }
        sqlx::query("UPDATE offers SET available=NULL WHERE branch_id=? AND source_product_id IN (SELECT id FROM source_products sp WHERE sp.supermarket_id=? AND sp.source_id LIKE ? AND NOT EXISTS (SELECT 1 FROM source_catalog_items i WHERE i.supermarket_id=sp.supermarket_id AND i.branch_id=? AND i.source_id=sp.source_id AND i.run_id=? AND i.normalized=1))")
            .bind(&source.branch_id).bind(&source.supermarket_id).bind(source.provider.sku_pattern()).bind(&source.branch_id).bind(run).execute(&mut *tx).await?;
        sqlx::query("UPDATE supermarkets SET integration_status='authorized' WHERE id=?")
            .bind(&source.supermarket_id).execute(&mut *tx).await?;
        tx.commit().await?;
        imported = count;
        Ok(())
    }.await;
    let error = result.as_ref().err().map(ToString::to_string);
    sqlx::query("UPDATE catalogue_collectors SET status=?,last_finished_at=?,last_error=?,requests=?,raw_items=?,normalized_items=?,scope_rejected=?,last_success_at=CASE WHEN ? THEN ? ELSE last_success_at END WHERE supermarket_id=?")
        .bind(if result.is_ok() { "success" } else { "failed" })
        .bind(now()).bind(error).bind(requests).bind(raw_count).bind(imported).bind(scope_rejected)
        .bind(result.is_ok()).bind(now()).bind(&source.supermarket_id).execute(pool).await?;
    result
}

pub async fn run(pool: &SqlitePool, path: &Path, watch: bool) -> Result<()> {
    schedule::run(pool, path, watch).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_checks_variant_against_title() {
        assert_eq!(package("2KG", "picanha").unwrap(), (2000, "g".into()));
        assert_eq!(package("UN", "água 1,5l").unwrap(), (1500, "ml".into()));
        assert_eq!(
            package("CX C-6", "budweiser 330ml").unwrap(),
            (1980, "ml".into())
        );
        assert_eq!(
            package("CAIXA COM 12", "água 500ml").unwrap(),
            (6000, "ml".into())
        );
        assert_eq!(
            package("12uni", "cerveja 350ml").unwrap(),
            (4200, "ml".into())
        );
        assert_eq!(
            package("GARRAFA", "vinho 750ml").unwrap(),
            (750, "ml".into())
        );
        assert_eq!(
            package("Lata 250 Ml", "energético").unwrap(),
            (250, "ml".into())
        );
        assert!(package("ML", "energético").is_err());
        assert!(package("500ml", "suco pet 300ml").is_err());
        assert!(package("UN", "água 12 x 350ml").is_err());
        assert!(package("UN", "refrigerante").is_err());
    }

    #[test]
    fn coverage_records_skipped_categories_and_all_source_counts() {
        let source: Source = serde_json::from_value(serde_json::json!({
            "provider":"aiqfome", "supermarket_id":"demo", "branch_id":"demo-1",
            "slug":"demo", "seller_id":1, "cnpj":"", "street":"Rua A",
            "number":"1", "postcode":"78890000", "categories":["alimentos"]
        }))
        .unwrap();
        let page = serde_json::json!({
            "menu":{"cardapios_categorias":[
                {"nome":"Alimentos","taxa_embalagem":0.0,"itens":[
                    {"id":1,"nome":"Arroz 5kg","disponivel":true,"is_free_gift":false,
                     "itens_tamanhos":[{"id":10,"item_id":1,"restaurante_id":1,"disponivel":true,
                        "valor":10,"tamanho":{"nome":"5kg"},"ean_code":null}]}
                ]},
                {"nome":"Tabacaria","taxa_embalagem":0.0,"itens":[
                    {"id":2,"nome":"Cigarro","disponivel":true,"is_free_gift":false,
                     "itens_tamanhos":[{"id":20,"item_id":2,"restaurante_id":1,"disponivel":true,
                        "valor":5,"tamanho":{"nome":"UN"},"ean_code":null}]}
                ]}
            ]}
        });
        let (_, rows, _, coverage) =
            normalize_menu(&page, &source, "https://example.test/demo", now()).unwrap();
        assert_eq!(coverage.total_categories, 2);
        assert_eq!(coverage.selected_categories, 1);
        assert_eq!(coverage.total_items, 2);
        assert_eq!(coverage.selected_items, 1);
        assert_eq!(coverage.total_variants, 2);
        assert_eq!(coverage.selected_variants, 1);
        assert_eq!(coverage.skipped_categories, ["Tabacaria"]);
        assert_eq!(rows.len(), 1);
    }

    #[test]
    fn repeated_variant_across_categories_is_deduplicated() {
        let source: Source = serde_json::from_value(serde_json::json!({
            "provider":"aiqfome", "supermarket_id":"demo", "branch_id":"demo-1",
            "slug":"demo", "seller_id":1, "street":"Rua A", "number":"1",
            "postcode":"78890000", "categories":["alimentos","destaques"]
        }))
        .unwrap();
        let item = serde_json::json!({
            "id":1,"nome":"Arroz 5kg","disponivel":true,"is_free_gift":false,
            "itens_tamanhos":[{"id":10,"item_id":1,"restaurante_id":1,"disponivel":true,
                "valor":10,"tamanho":{"nome":"5kg"},"ean_code":null}]
        });
        let page = serde_json::json!({"menu":{"cardapios_categorias":[
            {"nome":"Alimentos","taxa_embalagem":0.0,"itens":[item.clone()]},
            {"nome":"Destaques","taxa_embalagem":0.0,"itens":[item]}
        ]}});
        let (_, rows, _, coverage) =
            normalize_menu(&page, &source, "https://example.test/demo", now()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(coverage.total_variants, 2);
        assert_eq!(coverage.selected_variants, 2);
    }

    #[test]
    fn unknown_stock_is_preserved_in_offer_and_audit() {
        let source: Source = serde_json::from_value(serde_json::json!({
            "provider":"ola", "supermarket_id":"store", "branch_id":"branch",
            "slug":"store", "street":"Sorriso"
        }))
        .unwrap();
        let menu = serde_json::json!({
            "categorias":[{"slug":"food","nome":"Alimentos"}],
            "itens":[{"id":"variant","nome":"Bolo","categoria":"food",
                "disponivel":null,"opcoes":[{"preco":12.5}]}]
        });
        let (observations, rows, coverage) = normalize_options(&menu, &source, now()).unwrap();
        assert_eq!(coverage.selected_items, 1);
        assert_eq!(rows[0].available, None);
        assert!(serde_json::to_value(&observations[0]).unwrap()["available"].is_null());
    }

    #[tokio::test]
    async fn rate_limit_preserves_retry_after_for_the_whole_cycle() {
        use axum::{Router, http::StatusCode, routing::get};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new().route(
            "/",
            get(|| async { (StatusCode::TOO_MANY_REQUESTS, [("Retry-After", "720")]) }),
        );
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let error = fetch_body(&reqwest::Client::new(), &format!("http://{address}/"))
            .await
            .unwrap_err();
        assert_eq!(
            error.downcast_ref::<RateLimited>().unwrap().0,
            Duration::from_secs(720)
        );
        server.abort();
    }
}
