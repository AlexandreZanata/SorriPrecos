//! Only explicitly configured, authorized feeds. No retailer's private API is used.
use crate::{
    catalog::{self, ImportRow, MAX_BYTES},
    now,
};
use anyhow::{Context, Result, bail, ensure};
use async_trait::async_trait;
use reqwest::{Client, StatusCode, header};
use serde::Deserialize;
use sqlx::{Row, SqlitePool};
use std::{collections::HashSet, time::Duration};
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceConfig {
    pub supermarket_id: String,
    pub enabled: bool,
    pub authorization_reference: Option<String>,
    pub url: Option<String>,
    pub format: String,
    pub interval_seconds: u64,
    #[serde(default = "pages")]
    pub max_pages: usize,
}
fn pages() -> usize {
    100
}

#[async_trait]
pub trait SourceAdapter {
    fn source_id(&self) -> &str;
    async fn collect(&self, pool: &SqlitePool) -> Result<usize>;
}

pub struct AuthorizedFeedAdapter {
    pub config: SourceConfig,
    client: Client,
}
impl AuthorizedFeedAdapter {
    pub fn new(config: SourceConfig) -> Result<Self> {
        ensure!(config.enabled, "adaptador pendente/desabilitado");
        ensure!(
            config
                .authorization_reference
                .as_deref()
                .is_some_and(|s| s.len() >= 8),
            "autorização não documentada"
        );
        ensure!(
            config.interval_seconds >= 600,
            "intervalo mínimo: 10 minutos"
        );
        ensure!(
            (1..=100).contains(&config.max_pages),
            "máximo de páginas: 100"
        );
        ensure!(
            ["csv", "json"].contains(&config.format.as_str()),
            "formato inválido"
        );
        let u = Url::parse(config.url.as_deref().context("URL ausente")?)?;
        ensure!(
            u.username().is_empty() && u.password().is_none(),
            "credenciais na URL não permitidas"
        );
        ensure!(
            u.scheme() == "https"
                || (u.scheme() == "http"
                    && matches!(u.host_str(), Some("127.0.0.1" | "localhost"))),
            "feed deve usar HTTPS"
        );
        let client = Client::builder()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("SorriPrecos/0.1 (authorized-feed)")
            .build()?;
        Ok(Self { config, client })
    }
}

pub fn retry_delay(value: Option<&str>, attempt: u32) -> Duration {
    if let Some(v) = value {
        if let Ok(n) = v.parse::<u64>() {
            return Duration::from_secs(n);
        }
        if let Ok(t) = httpdate::parse_http_date(v) {
            return t
                .duration_since(std::time::SystemTime::now())
                .unwrap_or_default();
        }
    }
    Duration::from_secs(2u64.pow(attempt.min(6)))
}

#[derive(Deserialize)]
struct Page {
    items: Vec<ImportRow>,
    #[serde(default)]
    next: Option<String>,
}

#[async_trait]
impl SourceAdapter for AuthorizedFeedAdapter {
    fn source_id(&self) -> &str {
        &self.config.supermarket_id
    }
    async fn collect(&self, pool: &SqlitePool) -> Result<usize> {
        let initial = Url::parse(self.config.url.as_deref().unwrap())?;
        let cursor = sqlx::query(
            "SELECT etag,last_modified,last_success FROM source_cursors WHERE supermarket_id=?",
        )
        .bind(self.source_id())
        .fetch_optional(pool)
        .await?;
        if cursor.as_ref().is_some_and(|r| {
            r.get::<Option<i64>, _>("last_success")
                .is_some_and(|t| now() - t < (self.config.interval_seconds as i64))
        }) {
            return Ok(0);
        }
        let run:i64=sqlx::query_scalar("INSERT INTO collection_runs(supermarket_id,started_at,status) VALUES(?,?,'running') RETURNING id").bind(self.source_id()).bind(now()).fetch_one(pool).await?;
        let mut requests = 0i64;
        let result:Result<usize>=async {
            let mut next=Some(initial.clone());let mut visited=HashSet::new();let mut total=0;
            let mut new_etag=None;let mut new_modified=None;
            for page_no in 0..self.config.max_pages {
                let Some(u)=next.take() else {break;};
                ensure!(u.origin()==initial.origin() && u.username().is_empty() && u.password().is_none(),"paginação fora da origem autorizada");
                ensure!(visited.insert(u.to_string()),"ciclo de paginação");
                let mut response=None;
                for attempt in 0..3u32 {
                    let mut req=self.client.get(u.clone());
                    if page_no==0 && let Some(c)=&cursor {
                        if let Some(v)=c.get::<Option<String>,_>("etag") {req=req.header(header::IF_NONE_MATCH,v);}
                        if let Some(v)=c.get::<Option<String>,_>("last_modified") {req=req.header(header::IF_MODIFIED_SINCE,v);}
                    }
                    requests+=1;
                    match req.send().await {
                        Ok(r) if r.status()==StatusCode::TOO_MANY_REQUESTS || r.status().is_server_error()=> {
                            ensure!(attempt<2,"origem indisponível: {}",r.status());
                            let delay=retry_delay(r.headers().get(header::RETRY_AFTER).and_then(|v|v.to_str().ok()),attempt);
                            ensure!(delay<=Duration::from_secs(120),"Retry-After longo; adiar para próxima execução");
                            tokio::time::sleep(delay).await;
                        },
                        Ok(r)=>{response=Some(r);break;},
                        Err(e)=>{if attempt==2 {return Err(e.into());} tokio::time::sleep(retry_delay(None,attempt)).await;}
                    }
                }
                let mut r=response.context("sem resposta")?;
                if r.status()==StatusCode::NOT_MODIFIED && page_no==0 {
                    sqlx::query("UPDATE source_cursors SET last_success=? WHERE supermarket_id=?").bind(now()).bind(self.source_id()).execute(pool).await?;
                    return Ok(0);
                }
                ensure!(r.status().is_success(),"resposta HTTP {}",r.status());
                if page_no==0 {
                    new_etag=r.headers().get(header::ETAG).and_then(|v|v.to_str().ok()).map(str::to_owned);
                    new_modified=r.headers().get(header::LAST_MODIFIED).and_then(|v|v.to_str().ok()).map(str::to_owned);
                }
                ensure!(r.content_length().is_none_or(|n|n<=MAX_BYTES as u64),"página excede 8 MiB");
                let mut bytes=Vec::new();
                while let Some(chunk)=r.chunk().await? {ensure!(bytes.len()+chunk.len()<=MAX_BYTES,"página excede 8 MiB");bytes.extend_from_slice(&chunk);}
                let rows=if self.config.format=="json" && bytes.iter().find(|b|!b.is_ascii_whitespace())==Some(&b'{') {
                    let p:Page=serde_json::from_slice(&bytes)?;
                    next=p.next.map(|s|u.join(&s)).transpose()?;
                    p.items
                } else {catalog::parse(&bytes,&self.config.format)?};
                ensure!(rows.len()<=catalog::MAX_ROWS,"página excede 10000 registros");
                if !rows.is_empty() {total+=catalog::import(pool,self.source_id(),&rows,self.config.authorization_reference.as_deref()).await?;}
                if next.is_some() {tokio::time::sleep(Duration::from_millis(500)).await;}
            }
            ensure!(next.is_none(),"limite de páginas atingido; cursor não será avançado");
            sqlx::query("INSERT INTO source_cursors(supermarket_id,etag,last_modified,last_success) VALUES(?,?,?,?) ON CONFLICT(supermarket_id) DO UPDATE SET etag=excluded.etag,last_modified=excluded.last_modified,last_success=excluded.last_success")
                .bind(self.source_id()).bind(new_etag).bind(new_modified).bind(now()).execute(pool).await?;
            if total>0 {sqlx::query("UPDATE supermarkets SET integration_status='authorized',audit_class='A' WHERE id=? AND is_demo=0").bind(self.source_id()).execute(pool).await?;}
            Ok(total)
        }.await;
        sqlx::query("UPDATE collection_runs SET finished_at=?,status=?,imported=?,error=?,requests=? WHERE id=?").bind(now()).bind(if result.is_ok(){"success"}else{"failed"}).bind(result.as_ref().copied().unwrap_or(0) as i64).bind(result.as_ref().err().map(ToString::to_string)).bind(requests).bind(run).execute(pool).await?;
        result
    }
}

pub async fn collect_config(pool: &SqlitePool, path: &std::path::Path) -> Result<()> {
    let configs: Vec<SourceConfig> = serde_json::from_slice(&tokio::fs::read(path).await?)?;
    let mut failed = false;
    let mut seen = HashSet::new();
    // Sequential execution caps concurrency to one, including shared-host retailers.
    for config in configs {
        ensure!(
            seen.insert(config.supermarket_id.clone()),
            "fonte duplicada na configuração"
        );
        if !config.enabled {
            tracing::info!(source=%config.supermarket_id,"source_pending");
            continue;
        }
        let source = config.supermarket_id.clone();
        match AuthorizedFeedAdapter::new(config) {
            Ok(adapter) => match adapter.collect(pool).await {
                Ok(n) => tracing::info!(source, imported = n, "collection_complete"),
                Err(e) => {
                    failed = true;
                    tracing::error!(source,error=%e,"collection_failed");
                }
            },
            Err(e) => {
                failed = true;
                tracing::error!(source,error=%e,"configuration_invalid");
            }
        }
    }
    if failed {
        bail!("uma ou mais fontes falharam; dados anteriores preservados");
    }
    Ok(())
}
