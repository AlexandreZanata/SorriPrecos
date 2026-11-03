//! Read-only storefront catalogue, using only the same anonymous store session as the website.
//! Customer login, orders, checkout and administrative operations are deliberately absent.
use crate::{
    catalog::{self, ImportRow, MAX_BYTES},
    normalize, now,
};
use anyhow::{Context, Result, bail, ensure};
use base64::Engine;
use regex::Regex;
use reqwest::{
    Client, Method,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::SqlitePool;
use std::{
    collections::{HashMap, HashSet},
    path::Path,
    sync::LazyLock,
    time::{Duration, Instant},
};
use url::Url;

const ORIGIN: &str = "https://services.vipcommerce.com.br";
const BASIS: &str = "PUBLIC_STOREFRONT: consulta do catálogo público por sessão anônima de loja; sem login de cliente";
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Branch {
    pub distribution_id: i64,
    pub branch_id: String,
    pub cnpj: String,
    pub street: String,
    pub number: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub supermarket_id: String,
    pub enabled: bool,
    pub website: String,
    pub domain: String,
    pub organization_id: i64,
    pub storefront_id: i64,
    pub filial_id: i64,
    pub configuration_path: String,
    pub interval_seconds: u64,
    pub page_size: usize,
    pub max_pages: usize,
    pub delay_ms: u64,
    pub branches: Vec<Branch>,
}
impl Config {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.interval_seconds >= 600 && self.interval_seconds <= 86400,
            "intervalo: 600 a 86400 segundos"
        );
        ensure!(
            (1..=100).contains(&self.page_size) && (1..=1000).contains(&self.max_pages),
            "limites de paginação inválidos"
        );
        ensure!(
            self.delay_ms >= 250 && self.delay_ms <= 10000,
            "pausa mínima: 250 ms"
        );
        let u = Url::parse(&self.website)?;
        ensure!(
            u.scheme() == "https"
                && u.username().is_empty()
                && u.password().is_none()
                && u.query().is_none(),
            "website inválido"
        );
        ensure!(
            u.host_str() == Some(self.domain.as_str())
                || u.host_str() == Some(format!("www.{}", self.domain).as_str()),
            "domínio diverge do website"
        );
        ensure!(
            self.configuration_path.starts_with("/chunk-")
                && self.configuration_path.ends_with(".js")
                && !self.configuration_path.contains(".."),
            "configuração deve ser um módulo público do site"
        );
        ensure!(
            !self.branches.is_empty() && self.branches.len() <= 20,
            "filiais explícitas ausentes"
        );
        ensure!(
            self.organization_id > 0 && self.storefront_id > 0 && self.filial_id > 0,
            "identidade inválida"
        );
        Ok(())
    }
}

pub struct Http {
    client: Client,
    origin: String,
    headers: HeaderMap,
    requests: i64,
    delay: Duration,
    token: Option<HeaderValue>,
}
impl Http {
    fn new(c: &Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert("domainkey", c.domain.parse()?);
        headers.insert("organizationid", c.organization_id.to_string().parse()?);
        Ok(Self {
            client: Client::builder()
                .timeout(Duration::from_secs(25))
                .connect_timeout(Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .user_agent("SorriPrecos/0.1 (public-storefront-catalog)")
                .gzip(true)
                .build()?,
            origin: ORIGIN.into(),
            headers,
            requests: 0,
            delay: Duration::from_millis(c.delay_ms),
            token: None,
        })
    }
    async fn bytes(&mut self, url: &str, body: Option<Value>, use_token: bool) -> Result<Vec<u8>> {
        let url = Url::parse(url)?;
        if use_token {
            ensure!(
                url.origin() == Url::parse(&self.origin)?.origin(),
                "sessão não pode sair da origem emissora"
            );
        }
        for attempt in 0..3u32 {
            if self.requests > 0 {
                tokio::time::sleep(self.delay).await;
            }
            self.requests += 1;
            let mut req = self.client.request(
                if body.is_some() {
                    Method::POST
                } else {
                    Method::GET
                },
                url.clone(),
            );
            if url.origin() == Url::parse(&self.origin)?.origin() {
                req = req.headers(self.headers.clone());
            }
            if use_token {
                req = req.header(
                    AUTHORIZATION,
                    self.token.clone().context("sessão pública ausente")?,
                );
            }
            if let Some(v) = &body {
                req = req.json(v);
            }
            let mut response = match req.send().await {
                Ok(r) => r,
                Err(_) if attempt < 2 => {
                    tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                    continue;
                }
                Err(_) => bail!("origem sem resposta dentro do prazo"),
            };
            if response.status().as_u16() == 429 || response.status().is_server_error() {
                let delay = crate::adapters::retry_delay(
                    response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok()),
                    attempt,
                );
                ensure!(
                    attempt < 2 && delay <= Duration::from_secs(30),
                    "origem solicitou adiamento; HTTP {}",
                    response.status()
                );
                tokio::time::sleep(delay).await;
                continue;
            }
            ensure!(
                response.status().is_success(),
                "origem retornou HTTP {}; nenhuma credencial de cliente será solicitada",
                response.status()
            );
            ensure!(
                response
                    .content_length()
                    .is_none_or(|n| n <= MAX_BYTES as u64),
                "resposta maior que 8 MiB"
            );
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await? {
                ensure!(
                    bytes.len() + chunk.len() <= MAX_BYTES,
                    "resposta maior que 8 MiB"
                );
                bytes.extend(chunk);
            }
            return Ok(bytes);
        }
        bail!("origem indisponível")
    }
    async fn json(&mut self, path: &str, token: bool) -> Result<Value> {
        ensure!(
            path.starts_with("/api-admin/v1/") && !path.contains(".."),
            "rota fora do catálogo"
        );
        let data = self
            .bytes(&format!("{}{}", self.origin, path), None, token)
            .await?;
        let body: Value = serde_json::from_slice(&data).context("resposta não é JSON válido")?;
        ensure!(body["success"] == true, "origem não confirmou a consulta");
        Ok(body)
    }
}

fn keys(script: &str) -> Result<(String, String)> {
    let get = |field: &str| -> Result<String> {
        let re = Regex::new(&format!(r#"{}:\s*"([^"\\]+)""#, field))?;
        Ok(re
            .captures(script)
            .and_then(|c| c.get(1))
            .context("configuração pública mudou")?
            .as_str()
            .to_owned())
    };
    Ok((get("lojaUser")?, get("lojaAuthJWT")?))
}
async fn bootstrap_config(http: &mut Http, c: &Config) -> Result<(String, String)> {
    let website = Url::parse(&c.website)?;
    let known = website.join(&c.configuration_path)?;
    match http.bytes(known.as_str(), None, false).await {
        Ok(script) => {
            if let Ok(value) = keys(std::str::from_utf8(&script)?) {
                return Ok(value);
            }
        }
        Err(e) if !e.to_string().contains("HTTP 404") => return Err(e),
        Err(_) => {} // A removed hash may be replaced by another script published on the homepage.
    }
    // A deployment may change the hash. Follow only script references actually published by the website.
    let home = String::from_utf8(http.bytes(website.as_str(), None, false).await?)?;
    let entry = Regex::new(r#"src="((?:main|chunk)-[A-Za-z0-9]+\.js)""#)?;
    let main = entry
        .captures_iter(&home)
        .filter_map(|m| m.get(1))
        .find(|m| m.as_str().starts_with("main-"))
        .context("script principal não encontrado")?
        .as_str();
    let main = String::from_utf8(
        http.bytes(website.join(main)?.as_str(), None, false)
            .await?,
    )?;
    let imports = Regex::new(r#"import\{([^}]+)\}from"\./(chunk-[A-Za-z0-9]+\.js)""#)?;
    let mut referenced: Vec<_> = imports
        .captures_iter(&main)
        .map(|m| (m[1].len(), m[2].to_owned()))
        .collect();
    referenced.sort_by_key(|r| std::cmp::Reverse(r.0));
    for (_, path) in referenced.into_iter().take(8) {
        let js = http
            .bytes(website.join(&path)?.as_str(), None, false)
            .await?;
        if let Ok(value) = keys(std::str::from_utf8(&js)?) {
            return Ok(value);
        }
    }
    bail!("configuração de visitante não encontrada; fonte precisa de revisão")
}
fn check_visitor(token: &str, org: i64) -> Result<()> {
    let payload = token.split('.').nth(1).context("sessão não é JWT")?;
    let data = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload)?;
    let claims: Value = serde_json::from_slice(&data)?;
    ensure!(
        claims["org"].as_i64() == Some(org)
            || claims["org"].as_str() == Some(org.to_string().as_str()),
        "sessão pertence a outra organização"
    );
    for key in ["client", "operator", "vipcommerceClienteId"] {
        let v = &claims[key];
        ensure!(
            v.is_null() || v == false || v == 0 || v == "",
            "sessão contém identidade de cliente/operador; uso recusado"
        );
    }
    Ok(())
}
pub fn verify_branch(data: &Value, b: &Branch) -> Result<()> {
    let a = &data["endereco"];
    ensure!(
        normalize::text(a["cidade"].as_str().unwrap_or("")) == "sorriso"
            && matches!(
                normalize::text(a["estado"].as_str().unwrap_or("")).as_str(),
                "mt" | "mato grosso"
            ),
        "filial fora de Sorriso-MT"
    );
    ensure!(
        data["id"].as_i64() == Some(b.distribution_id)
            && data["cnpj"].as_str() == Some(b.cnpj.as_str()),
        "CNPJ ou centro de distribuição divergente"
    );
    ensure!(
        normalize::text(a["logradouro"].as_str().unwrap_or(""))
            .contains(&normalize::text(&b.street))
            && a["numero"].as_str() == Some(b.number.as_str()),
        "endereço de filial divergente"
    );
    ensure!(
        data["ativo"] == true && data["exclusivo_televendas"] != true,
        "unidade não atende a vitrine pública"
    );
    Ok(())
}
fn number(v: &Value) -> Option<String> {
    v.as_str()
        .map(str::to_owned)
        .or_else(|| v.as_number().map(ToString::to_string))
}
pub fn cents(v: &Value) -> Result<i64> {
    let n = normalize::scaled(&number(v).context("preço ausente")?, 100)?;
    ensure!(n <= 100_000_000, "preço inválido");
    Ok(n)
}
static PACK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(\d+(?:[.,]\d+)?)\s*(kg|gramas?|grs?|g|ml|litros?|lts?|l|unidades?|und|un|rolos?)(?:\b|$)").unwrap()
});
static MULTIPACK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\d+\s*[x×]\s*\d+\s*(?:g|kg|ml|l)\b").unwrap());
fn package(p: &Value) -> Result<(i64, String)> {
    let name = p["descricao"].as_str().context("nome ausente")?;
    let unit = p["unidade_sigla"].as_str().unwrap_or("").to_uppercase();
    if unit == "KG" || unit == "L" || unit == "LT" {
        let amount = if p["possui_unidade_diferente"] == true {
            number(&p["quantidade_unidade_diferente"]).context("porção sem peso")?
        } else {
            "1".into()
        };
        return Ok((
            normalize::scaled(&amount, 1000)?,
            if unit == "KG" { "g" } else { "ml" }.into(),
        ));
    }
    ensure!(
        !MULTIPACK.is_match(name),
        "multipack exige revisão de embalagem"
    );
    let mut sizes = HashSet::new();
    for m in PACK.captures_iter(name) {
        let u = m[2].to_lowercase();
        let (factor, unit) = if u == "kg" {
            (1000, "g")
        } else if u.starts_with('g') {
            (1, "g")
        } else if u == "ml" {
            (1, "ml")
        } else if u.starts_with('l') {
            (1000, "ml")
        } else {
            (1, "un")
        };
        sizes.insert((normalize::scaled(&m[1], factor)?, unit.to_string()));
    }
    ensure!(sizes.len() == 1, "embalagem ausente ou ambígua");
    Ok(sizes.into_iter().next().unwrap())
}
pub fn normalize_product(
    p: &Value,
    branch: &Branch,
    website: &str,
    category: &str,
    observed: i64,
) -> Result<ImportRow> {
    let (quantity, unit) = package(p)?;
    let name = p["descricao"].as_str().context("produto sem descrição")?;
    let brand = p["marca"]
        .as_str()
        .or_else(|| p["marca"]["descricao"].as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Não informada");
    let id = p["produto_id"]
        .as_i64()
        .context("produto sem identificador")?;
    let barcode = p["codigo_barras"]
        .as_str()
        .filter(|s| normalize::valid_gtin(s))
        .map(str::to_owned);
    let base = cents(&p["preco"])?;
    let mut promo = None;
    let mut kind = "regular";
    let mut conditions = None;
    let offer = &p["oferta"];
    if offer.is_object() {
        let candidate = cents(&offer["preco_oferta"]).ok().filter(|v| *v <= base);
        let label = offer["nome"]
            .as_str()
            .unwrap_or("Condições da oferta não especificadas");
        let label_normalized = normalize::text(label);
        let club = ["clube", "cliente", "fidelidade", "exclusiv"]
            .iter()
            .any(|s| label_normalized.contains(s));
        let single = offer["tipo_oferta_id"] == 1
            && offer["categoria"] == "G"
            && number(&offer["quantidade_minima"])
                .as_deref()
                .is_some_and(|s| normalize::scaled(s, 1).ok() == Some(1))
            && number(&offer["quantidade_pagar"])
                .as_deref()
                .is_some_and(|s| normalize::scaled(s, 1).ok() == Some(1))
            && number(&offer["quantidade_oferta"])
                .as_deref()
                .is_some_and(|s| normalize::scaled(s, 1).ok() == Some(1));
        promo = candidate;
        if single && !club {
            kind = "promotion";
        } else {
            kind = if club { "club" } else { "wholesale" };
            conditions = Some(format!(
                "{} · quantidade mínima: {}. Confira as regras na origem.",
                label,
                number(&offer["quantidade_minima"]).unwrap_or_else(|| "não informada".into())
            ));
        }
    }
    let slug = p["link"].as_str().context("produto sem link")?;
    let mut source = Url::parse(website)?;
    {
        let mut seg = source
            .path_segments_mut()
            .map_err(|_| anyhow::anyhow!("URL inválida"))?;
        seg.clear().push("produto").push(&id.to_string()).push(slug);
    }
    let row = ImportRow {
        source_id: format!("vip-{id}"),
        branch_id: branch.branch_id.clone(),
        name: name.into(),
        brand: brand.into(),
        quantity,
        unit,
        presentation: "unidade de venda".into(),
        category: category.into(),
        gtin: barcode,
        price_cents: Some(base),
        promotional_price_cents: promo,
        offer_type: kind.into(),
        conditions,
        available: p["disponivel"].as_bool(),
        source_url: Some(source.to_string()),
        collected_at: observed,
        valid_until: None,
        source_city: Some("Sorriso".into()),
        source_state: Some("MT".into()),
    };
    catalog::validate(&row)?;
    Ok(row)
}
fn category_map(tree: &[Value], prefix: &str, map: &mut HashMap<i64, String>) {
    for item in tree {
        let name = item["descricao"].as_str().unwrap_or("Supermercado");
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{prefix} / {name}")
        };
        if let Some(id) = item["classificacao_mercadologica_id"].as_i64() {
            map.insert(id, path.chars().take(80).collect());
        }
        if let Some(children) = item["children"].as_array() {
            category_map(children, &path, map);
        }
    }
}

pub async fn collect(pool: &SqlitePool, c: &Config) -> Result<()> {
    c.validate()?;
    sqlx::query("INSERT INTO catalogue_collectors(supermarket_id,adapter,interval_seconds) VALUES(?,'vipcommerce',?) ON CONFLICT(supermarket_id) DO UPDATE SET interval_seconds=excluded.interval_seconds").bind(&c.supermarket_id).bind(c.interval_seconds as i64).execute(pool).await?;
    let claim=sqlx::query("UPDATE catalogue_collectors SET status='running',last_started_at=?,last_error=NULL WHERE supermarket_id=? AND (last_started_at IS NULL OR last_started_at<=?) AND (status!='running' OR last_started_at<?)").bind(now()).bind(&c.supermarket_id).bind(now()-c.interval_seconds as i64).bind(now()-1800).execute(pool).await?;
    if claim.rows_affected() == 0 {
        return Ok(());
    }
    let run:i64=sqlx::query_scalar("INSERT INTO collection_runs(supermarket_id,started_at,status,authorization_reference) VALUES(?,?,'running',?) RETURNING id").bind(&c.supermarket_id).bind(now()).bind(BASIS).fetch_one(pool).await?;
    let mut http = Http::new(c)?;
    let mut raw_count = 0i64;
    let mut normalized_count = 0i64;
    let mut rejected_cities = 0i64;
    let started = Instant::now();
    let result:Result<()>=async {
        let identity=http.json(&format!("/api-admin/v1/organizacoes/filiais/dominio/{}",c.domain),false).await?;
        ensure!(identity["data"]["id"]==c.storefront_id && identity["data"]["organizacao"]["id"]==c.organization_id && identity["data"]["organizacao"]["enderecoServidor"]==c.domain,"identidade do domínio mudou");
        ensure!(identity["data"]["ativo"]==true && identity["data"]["em_manutencao"]!=true,"vitrine em manutenção");
        let (username,key)=bootstrap_config(&mut http,c).await?;
        let bytes=http.bytes(&format!("{ORIGIN}/api-admin/v1/org/{}/auth/loja/login",c.organization_id),Some(json!({"domain":c.domain,"username":username,"key":key})),false).await?;
        let login:Value=serde_json::from_slice(&bytes)?;
        ensure!(login["success"]==true,"sessão de loja não iniciada");let token=login["data"].as_str().context("sessão pública inválida")?;
        check_visitor(token,c.organization_id)?;
        let mut header=HeaderValue::from_str(&format!("Bearer {token}"))?;header.set_sensitive(true);http.token=Some(header);
        let org=format!("/api-admin/v1/org/{}",c.organization_id);
        let branch=http.json(&format!("{org}/loja/filiais/{}",c.filial_id),true).await?;
        ensure!(branch["data"]["vipcommerce_filial_id"]==c.storefront_id && branch["data"]["em_manutencao"]!=true,"filial da vitrine divergente");
        let pickups=http.json(&format!("{org}/filial/{}/loja/centros_distribuicoes/retiradas",c.filial_id),true).await?;
        let units=pickups["data"].as_array().context("lista de filiais inválida")?;
        rejected_cities=units.iter().filter(|v|normalize::text(v["endereco"]["cidade"].as_str().unwrap_or(""))!="sorriso").count() as i64;
        for b in &c.branches {
            let data=units.iter().find(|v|v["id"]==b.distribution_id).context("unidade não aparece na vitrine")?;verify_branch(data,b)?;
            let valid:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM branches WHERE id=? AND supermarket_id=? AND active=1 AND city='Sorriso' AND state='MT')").bind(&b.branch_id).bind(&c.supermarket_id).fetch_one(pool).await?;ensure!(valid,"filial local não pertence à rede");
            let root=format!("{org}/filial/{}/centro_distribuicao/{}/loja",c.filial_id,b.distribution_id);
            let tree=http.json(&format!("{root}/classificacoes_mercadologicas/departamentos/arvore"),true).await?;
            let departments=tree["data"].as_array().context("departamentos inválidos")?;ensure!(!departments.is_empty(),"catálogo sem departamentos");
            let mut categories=HashMap::new();category_map(departments,"",&mut categories);
            let mut seen=HashSet::new();let mut total_pages=0usize;
            for d in departments {
                let dep=d["classificacao_mercadologica_id"].as_i64().context("departamento sem ID")?;
                let mut pages=1usize;let mut count=0usize;let mut expected=0usize;
                for page in 1..=c.max_pages {
                    ensure!(started.elapsed()<Duration::from_secs(540),"catálogo excedeu nove minutos; continuar na próxima execução sem sobreposição");
                    total_pages+=1;ensure!(total_pages<=c.max_pages,"limite total de páginas atingido");
                    let body=http.json(&format!("{root}/classificacoes_mercadologicas/departamentos/{dep}/produtos?page={page}&limit={}",c.page_size),true).await?;
                    let paging=&body["paginator"];let reported=paging["total_items"].as_u64().context("paginação sem total")? as usize;
                    let declared=paging["total_pages"].as_u64().context("paginação sem páginas")? as usize;
                    ensure!(paging["page"]==page as u64 && declared<=c.max_pages,"paginação inválida");
                    if page==1 {pages=declared;expected=reported;} else {ensure!(expected==reported && pages==declared,"catálogo mudou durante a paginação; nova coleta necessária");}
                    let entries=body["data"].as_array().or_else(||body["data"]["produtos"].as_array()).context("página sem produtos")?;
                    count+=entries.len();let observed=now();let mut normalized=Vec::new();
                    for p in entries {
                        let id=p["produto_id"].as_i64().context("item sem ID")?;if !seen.insert(id){continue;}
                        let category=p["classificacao_mercadologica_id"].as_i64().and_then(|id|categories.get(&id)).map(String::as_str).unwrap_or("Supermercado");
                        let parsed=normalize_product(p,b,&c.website,category,observed);
                        let (ok,reason)=match &parsed {Ok(_)=>(true,None),Err(e)=>(false,Some(e.to_string()))};
                        sqlx::query("INSERT INTO source_catalog_items(supermarket_id,branch_id,source_id,raw_json,observed_at,run_id,normalized,reason) VALUES(?,?,?,?,?,?,?,?) ON CONFLICT(supermarket_id,branch_id,source_id) DO UPDATE SET raw_json=excluded.raw_json,observed_at=excluded.observed_at,run_id=excluded.run_id,normalized=excluded.normalized,reason=excluded.reason").bind(&c.supermarket_id).bind(&b.branch_id).bind(format!("vip-{id}")).bind(serde_json::to_string(p)?).bind(observed).bind(run).bind(ok).bind(reason).execute(pool).await?;
                        raw_count+=1;if let Ok(row)=parsed {normalized.push(row);}
                    }
                    if !normalized.is_empty() {
                        if catalog::import(pool,&c.supermarket_id,&normalized,Some(BASIS)).await.is_ok() {normalized_count+=normalized.len() as i64;}
                        else {for row in normalized {match catalog::import(pool,&c.supermarket_id,std::slice::from_ref(&row),Some(BASIS)).await {
                            Ok(_)=>normalized_count+=1,
                            Err(e)=>{sqlx::query("UPDATE source_catalog_items SET normalized=0,reason=? WHERE supermarket_id=? AND branch_id=? AND source_id=?").bind(e.to_string()).bind(&c.supermarket_id).bind(&b.branch_id).bind(&row.source_id).execute(pool).await?;}
                        }}}
                    }
                    tracing::info!(source=%c.supermarket_id,department=dep,page,pages,raw=raw_count,normalized=normalized_count,"public_catalog_page");
                    if page>=pages {break;}
                }
                ensure!(count==expected,"departamento incompleto; dados anteriores preservados");
            }
            // Missing or unnormalizable rows never imply stock=false, but cannot remain current eligible offers.
            sqlx::query("UPDATE offers SET available=NULL WHERE supermarket_id=? AND branch_id=? AND source_product_id IN(SELECT id FROM source_products WHERE supermarket_id=? AND source_kind='retail' AND source_id LIKE 'vip-%' AND NOT EXISTS(SELECT 1 FROM source_catalog_items i WHERE i.supermarket_id=source_products.supermarket_id AND i.branch_id=? AND i.source_id=source_products.source_id AND i.run_id=? AND i.normalized=1))").bind(&c.supermarket_id).bind(&b.branch_id).bind(&c.supermarket_id).bind(&b.branch_id).bind(run).execute(pool).await?;
        }
        ensure!(raw_count>0 && normalized_count>0,"nenhum produto normalizado");
        sqlx::query("UPDATE supermarkets SET integration_status='authorized' WHERE id=?").bind(&c.supermarket_id).execute(pool).await?;
        Ok(())
    }.await;
    let error = result.as_ref().err().map(ToString::to_string);
    sqlx::query("UPDATE collection_runs SET status=?,finished_at=?,imported=?,requests=?,error=? WHERE id=?").bind(if result.is_ok(){"success"}else{"failed"}).bind(now()).bind(normalized_count).bind(http.requests).bind(&error).bind(run).execute(pool).await?;
    sqlx::query("UPDATE catalogue_collectors SET status=?,last_finished_at=?,last_error=?,requests=?,raw_items=?,normalized_items=?,scope_rejected=?,last_success_at=CASE WHEN ? THEN ? ELSE last_success_at END WHERE supermarket_id=?").bind(if result.is_ok(){"success"}else{"failed"}).bind(now()).bind(&error).bind(http.requests).bind(raw_count).bind(normalized_count).bind(rejected_cities).bind(result.is_ok()).bind(now()).bind(&c.supermarket_id).execute(pool).await?;
    result
}
pub async fn run(pool: &SqlitePool, path: &Path, watch: bool) -> Result<()> {
    loop {
        let configs: Vec<Config> = serde_json::from_slice(&tokio::fs::read(path).await?)?;
        let mut failed = false;
        let mut seen = HashSet::new();
        for c in &configs {
            ensure!(seen.insert(&c.supermarket_id), "rede duplicada");
            if !c.enabled {
                continue;
            }
            if let Err(e) = collect(pool, c).await {
                failed = true;
                tracing::error!(source=%c.supermarket_id,error=%e,"public_catalog_failed");
            }
        }
        if !watch {
            ensure!(!failed, "uma ou mais coletas falharam; veja /v1/collectors");
            return Ok(());
        }
        tokio::select! {_=tokio::signal::ctrl_c()=>return Ok(()),_=tokio::time::sleep(Duration::from_secs(15))=>{}}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn branch() -> Branch {
        Branch {
            distribution_id: 1,
            branch_id: "sorrinorte-industrial".into(),
            cnpj: "02756536000167".into(),
            street: "Palmares".into(),
            number: "489".into(),
        }
    }
    fn item() -> Value {
        json!({"produto_id":3,"descricao":"Arroz marca exemplo 5kg","codigo_barras":"7896025800106","preco":"23.90","disponivel":true,"unidade_sigla":"UN","link":"arroz-marca-exemplo-5kg"})
    }
    #[test]
    fn decimal_prices_are_exact_and_positive() {
        assert_eq!(cents(&json!("23.90")).unwrap(), 2390);
        assert_eq!(cents(&json!("17.990")).unwrap(), 1799);
        for v in ["-0.99", "-1.00", "0", "1.001", "NaN", "1.2.3"] {
            assert!(cents(&json!(v)).is_err(), "{v}");
        }
    }
    #[test]
    fn store_session_rejects_customer_operator_and_wrong_org() {
        let token = |claims: Value| {
            format!(
                "header.{}.signature",
                base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .encode(serde_json::to_vec(&claims).unwrap())
            )
        };
        assert!(
            check_visitor(
                &token(
                    json!({"org":365,"client":null,"operator":null,"vipcommerceClienteId":null})
                ),
                365
            )
            .is_ok()
        );
        for claims in [
            json!({"org":324}),
            json!({"org":365,"client":123}),
            json!({"org":365,"operator":1}),
            json!({"org":365,"vipcommerceClienteId":"42"}),
        ] {
            assert!(check_visitor(&token(claims), 365).is_err());
        }
    }
    #[test]
    fn city_cnpj_and_branch_are_checked_together() {
        let mut v = json!({"id":1,"cnpj":"02756536000167","ativo":true,"endereco":{"cidade":"Sorriso","estado":"Mato Grosso","logradouro":"Rua Palmares","numero":"489"}});
        assert!(verify_branch(&v, &branch()).is_ok());
        v["endereco"]["cidade"] = json!("Sinop");
        assert!(verify_branch(&v, &branch()).is_err());
        v["endereco"]["cidade"] = json!("Sorriso");
        v["cnpj"] = json!("00000000000000");
        assert!(verify_branch(&v, &branch()).is_err());
    }
    #[test]
    fn packaging_is_conservative_and_fractional_weight_uses_sale_unit() {
        assert_eq!(package(&item()).unwrap(), (5000, "g".into()));
        let mut p = item();
        p["descricao"] = json!("Carne bovina 500g");
        p["unidade_sigla"] = json!("KG");
        p["possui_unidade_diferente"] = json!(true);
        p["quantidade_unidade_diferente"] = json!(0.5);
        assert_eq!(package(&p).unwrap(), (500, "g".into()));
        for name in ["Leite caixa", "Kit 2x200g", "Produto 500g + brinde 100g"] {
            p = item();
            p["descricao"] = json!(name);
            assert!(package(&p).is_err(), "{name}");
        }
    }
    #[test]
    fn club_price_keeps_conditions_and_regular_price() {
        let mut p = item();
        p["oferta"] = json!({"nome":"Cliente Todo Dia","categoria":"G","tipo_oferta_id":1,"preco_oferta":"19.99","quantidade_minima":1,"quantidade_pagar":1,"quantidade_oferta":1});
        let r = normalize_product(
            &p,
            &branch(),
            "https://www.sorrinorte.com.br",
            "Mercearia",
            now(),
        )
        .unwrap();
        assert_eq!(r.offer_type, "club");
        assert!(r.conditions.is_some());
        assert_eq!(r.price_cents, Some(2390));
        assert_eq!(r.promotional_price_cents, Some(1999));
        p["oferta"]["nome"] = json!("Ofertas");
        let r = normalize_product(
            &p,
            &branch(),
            "https://www.sorrinorte.com.br",
            "Mercearia",
            now(),
        )
        .unwrap();
        assert_eq!(r.offer_type, "promotion");
        assert!(r.conditions.is_none());
    }
    #[tokio::test]
    async fn visitor_header_cannot_be_sent_to_another_origin() {
        let configs: Vec<Config> =
            serde_json::from_str(include_str!("../../config/vipcommerce.json")).unwrap();
        let mut http = Http::new(&configs[0]).unwrap();
        http.token = Some(HeaderValue::from_static("Bearer synthetic-test"));
        assert!(http.bytes("https://example.org", None, true).await.is_err());
        assert_eq!(http.requests, 0);
    }
}
