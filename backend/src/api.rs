use crate::{catalog, now};
use axum::{
    Json, Router,
    body::Body,
    extract::{ConnectInfo, Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{Row, SqlitePool};
use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub rate: Arc<Mutex<HashMap<IpAddr, (Instant, u32)>>>,
    pub requests_per_minute: u32,
}
impl AppState {
    pub fn new(pool: SqlitePool, requests_per_minute: u32) -> Self {
        Self {
            pool,
            rate: Default::default(),
            requests_per_minute,
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/supermarkets", get(supermarkets))
        .route("/v1/branches", get(branches))
        .route("/v1/products/search", get(search))
        .route("/v1/products/sync", post(sync_products))
        .route("/v1/products/{id}", get(product))
        .route("/v1/products/{id}/prices", get(prices))
        .route("/v1/products/{id}/photo", get(photo))
        .route("/v1/catalog/status", get(status))
        .route("/v1/collectors", get(collectors))
        .fallback(|| async {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error":"rota não encontrada"})),
            )
        })
        .layer(middleware::from_fn_with_state(state.clone(), limits))
        .layer(tower::limit::ConcurrencyLimitLayer::new(64))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn limits(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if req.uri().to_string().len() > 2048 {
        return error(StatusCode::URI_TOO_LONG, "requisição muito longa");
    }
    let ip = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|p| p.0.ip())
        .unwrap_or(IpAddr::from([127, 0, 0, 1]));
    let allowed = {
        let mut map = state.rate.lock().unwrap();
        if map.len() >= 4096 {
            map.retain(|_, (t, _)| t.elapsed() < Duration::from_secs(60));
        }
        if map.len() >= 4096 && !map.contains_key(&ip) {
            false
        } else {
            let (time, count) = map.entry(ip).or_insert((Instant::now(), 0));
            if time.elapsed() >= Duration::from_secs(60) {
                *time = Instant::now();
                *count = 0;
            }
            *count = count.saturating_add(1);
            *count <= state.requests_per_minute
        }
    };
    if !allowed {
        let mut res = error(
            StatusCode::TOO_MANY_REQUESTS,
            "limite de consultas atingido",
        );
        res.headers_mut()
            .insert(header::RETRY_AFTER, "60".parse().unwrap());
        return res;
    }
    match tokio::time::timeout(Duration::from_secs(10), next.run(req)).await {
        Ok(mut res) => {
            res.headers_mut()
                .insert("x-content-type-options", "nosniff".parse().unwrap());
            res
        }
        Err(_) => error(StatusCode::GATEWAY_TIMEOUT, "tempo de consulta excedido"),
    }
}

type ApiResult = Result<Response, ApiError>;
pub struct ApiError(anyhow::Error);
impl<E: Into<anyhow::Error>> From<E> for ApiError {
    fn from(e: E) -> Self {
        Self(e.into())
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!(error=%self.0,"database_request_failed");
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "não foi possível consultar o catálogo",
        )
    }
}
fn error(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({"error":msg}))).into_response()
}

fn cached<T: Serialize>(headers: &HeaderMap, value: &T) -> Response {
    let body = serde_json::to_vec(value).unwrap();
    let mut hash = DefaultHasher::new();
    body.hash(&mut hash);
    let etag = format!("W/\"{:x}\"", hash.finish());
    let matches = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|h| h.split(',').any(|e| e.trim() == etag || e.trim() == "*"));
    let mut res = if matches {
        StatusCode::NOT_MODIFIED.into_response()
    } else {
        Response::new(Body::from(body))
    };
    res.headers_mut().insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    res.headers_mut()
        .insert(header::ETAG, etag.parse().unwrap());
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        "public, max-age=30, must-revalidate".parse().unwrap(),
    );
    res
}

async fn health(State(s): State<AppState>) -> ApiResult {
    sqlx::query("SELECT 1").execute(&s.pool).await?;
    Ok(Json(json!({"status":"ok","version":env!("CARGO_PKG_VERSION")})).into_response())
}

#[derive(Default, Deserialize)]
struct Mode {
    #[serde(default)]
    demo: bool,
    #[serde(default)]
    historical: bool,
}
async fn supermarkets(
    State(s): State<AppState>,
    Query(m): Query<Mode>,
    headers: HeaderMap,
) -> ApiResult {
    let rows=sqlx::query("SELECT s.*,MAX(o.collected_at) last_collected_at,COUNT(DISTINCT CASE WHEN b.active=1 THEN b.id END) branch_count,COUNT(DISTINCT o.product_id) product_count FROM supermarkets s LEFT JOIN branches b ON b.supermarket_id=s.id LEFT JOIN offers o ON o.branch_id=b.id AND o.source_product_id IN(SELECT id FROM source_products WHERE source_kind=?) WHERE s.is_demo=? GROUP BY s.id ORDER BY s.name").bind(if m.historical {"public_report"} else {"retail"}).bind(m.demo).fetch_all(&s.pool).await?;
    let items:Vec<Value>=rows.iter().map(|r|json!({"id":r.get::<String,_>("id"),"name":r.get::<String,_>("name"),"website":r.get::<Option<String>,_>("website"),"integration_status":r.get::<String,_>("integration_status"),"audit_class":r.get::<String,_>("audit_class"),"is_demo":r.get::<bool,_>("is_demo"),"last_collected_at":r.get::<Option<i64>,_>("last_collected_at"),"branch_count":r.get::<i64,_>("branch_count"),"product_count":r.get::<i64,_>("product_count")})).collect();
    Ok(cached(&headers, &json!({"items":items})))
}

async fn branches(
    State(s): State<AppState>,
    Query(m): Query<Mode>,
    headers: HeaderMap,
) -> ApiResult {
    let rows=sqlx::query("SELECT b.*,s.name supermarket_name,MAX(o.collected_at) last_collected_at FROM branches b JOIN supermarkets s ON s.id=b.supermarket_id LEFT JOIN offers o ON o.branch_id=b.id AND o.source_product_id IN(SELECT id FROM source_products WHERE source_kind=?) WHERE s.is_demo=? GROUP BY b.id ORDER BY s.name,b.name").bind(if m.historical {"public_report"} else {"retail"}).bind(m.demo).fetch_all(&s.pool).await?;
    let items:Vec<Value>=rows.iter().map(|r|json!({"id":r.get::<String,_>("id"),"supermarket_id":r.get::<String,_>("supermarket_id"),"supermarket_name":r.get::<String,_>("supermarket_name"),"name":r.get::<String,_>("name"),"address":r.get::<String,_>("address"),"city":"Sorriso","state":"MT","last_collected_at":r.get::<Option<i64>,_>("last_collected_at")})).collect();
    Ok(cached(&headers, &json!({"items":items})))
}

#[derive(Deserialize)]
struct Search {
    q: String,
    sector: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    after: i64,
    #[serde(default)]
    demo: bool,
    #[serde(default)]
    historical: bool,
}
fn default_limit() -> i64 {
    20
}
async fn search(
    State(s): State<AppState>,
    Query(q): Query<Search>,
    headers: HeaderMap,
) -> ApiResult {
    if q.q.chars().count() < 2
        || q.q.chars().count() > 100
        || !(1..=50).contains(&q.limit)
        || q.after < 0
    {
        return Ok(error(
            StatusCode::BAD_REQUEST,
            "busca: 2 a 100 caracteres; limit: 1 a 50; after: positivo",
        ));
    }
    let sector = q.sector.as_deref().unwrap_or("market");
    if !matches!(sector, "market" | "pharmacy") {
        return Ok(error(
            StatusCode::BAD_REQUEST,
            "setor deve ser market ou pharmacy",
        ));
    }
    let mut items = catalog::search_sector(
        &s.pool,
        &q.q,
        q.limit + 1,
        q.after,
        q.demo,
        q.historical,
        sector,
    )
    .await?;
    let has_more = items.len() > q.limit as usize;
    items.truncate(q.limit as usize);
    let next_after = if has_more {
        items.last().map(|p| p.id)
    } else {
        None
    };
    let versions: HashMap<String, String> = items
        .iter()
        .map(|product| {
            let (version, _) = product_version(&json!({"product":product}));
            (product.id.to_string(), version)
        })
        .collect();
    Ok(cached(
        &headers,
        &json!({"items":items,"next_after":next_after,"is_demo":q.demo,"is_historical":q.historical,"versions":versions}),
    ))
}

async fn photo(State(s): State<AppState>, Path(id): Path<i64>) -> Response {
    match crate::photos::load(&s.pool, id).await {
        Ok(Some(webp)) => (
            [
                ("content-type", "image/webp"),
                ("cache-control", "public, max-age=600"),
            ],
            webp,
        )
            .into_response(),
        Ok(None) => (StatusCode::NOT_FOUND, [(header::CACHE_CONTROL, "no-store")]).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            [(header::CACHE_CONTROL, "no-store")],
        )
            .into_response(),
    }
}

async fn product(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Query(mode): Query<Mode>,
    headers: HeaderMap,
) -> ApiResult {
    match catalog::product_scope(&s.pool, id, mode.historical).await? {
        Some(p) => Ok(cached(&headers, &p)),
        None => Ok(error(StatusCode::NOT_FOUND, "produto não encontrado")),
    }
}

async fn prices(
    State(s): State<AppState>,
    Path(id): Path<i64>,
    Query(mode): Query<Mode>,
    headers: HeaderMap,
) -> ApiResult {
    let Some(p) = catalog::product_scope(&s.pool, id, mode.historical).await? else {
        return Ok(error(StatusCode::NOT_FOUND, "produto não encontrado"));
    };
    let data = comparison_data(&s.pool, p, mode.historical, now()).await?;
    Ok(cached(&headers, &data))
}

async fn comparison_data(
    pool: &SqlitePool,
    p: catalog::Product,
    historical: bool,
    at: i64,
) -> anyhow::Result<Value> {
    let rows=sqlx::query("SELECT b.id branch_id,b.name branch_name,b.address,s.id supermarket_id,s.name supermarket_name,o.price_cents,o.promotional_price_cents,o.offer_type,o.conditions,o.available,o.source_url,o.collected_at,o.valid_until,(SELECT source_id FROM source_products WHERE id=o.source_product_id) source_sku FROM branches b JOIN supermarkets s ON s.id=b.supermarket_id JOIN offers o ON o.branch_id=b.id AND o.product_id=? AND o.source_product_id IN(SELECT id FROM source_products WHERE source_kind=?) WHERE b.active=1 AND s.is_demo=? ORDER BY s.name,b.id").bind(p.id).bind(if historical {"public_report"} else {"retail"}).bind(p.is_demo).fetch_all(pool).await?;
    let mut items:Vec<Value>=rows.iter().map(|r| {
        let collected=r.get::<Option<i64>,_>("collected_at");let available=r.get::<Option<bool>,_>("available");
        let price=r.get::<Option<i64>,_>("price_cents");let promo=r.get::<Option<i64>,_>("promotional_price_cents");
        let kind=r.get::<Option<String>,_>("offer_type");let conditions=r.get::<Option<String>,_>("conditions");
        let until=r.get::<Option<i64>,_>("valid_until");
        let state=if collected.is_none() {"unknown"} else if available==Some(false) {"unavailable"} else if price.is_none() || until.is_some_and(|t|t<at) {"unknown"} else if collected.is_some_and(|t|t<at-172800) {"stale"} else if available.is_none() && r.get::<String,_>("supermarket_id")=="atacadao" && r.get::<String,_>("source_sku").starts_with("flyer-") && until.is_some() {"unconfirmed_stock"} else if available.is_none() {"unknown"} else {"recent"};
        let effective=if state=="recent" {if matches!(kind.as_deref(),Some("regular"|"promotion")) && conditions.as_deref().unwrap_or("").is_empty() {promo.or(price)} else {price}} else {None};
        json!({"branch_id":r.get::<String,_>("branch_id"),"branch_name":r.get::<String,_>("branch_name"),"address":r.get::<String,_>("address"),"supermarket_id":r.get::<String,_>("supermarket_id"),"supermarket_name":r.get::<String,_>("supermarket_name"),"price_cents":price,"promotional_price_cents":promo,"effective_price_cents":effective,"offer_type":kind,"conditions":conditions,"available":available,"source_url":r.get::<Option<String>,_>("source_url"),"collected_at":collected,"valid_until":until,"status":state,"is_demo":p.is_demo,"city":"Sorriso","state":"MT"})
    }).collect();
    // Multiple source SKUs can map to the same canonical product in one branch.
    // Keep the cheapest eligible observation per branch; otherwise prefer the latest.
    items.sort_by_key(|v| {
        (
            v["effective_price_cents"].as_i64().unwrap_or(i64::MAX),
            std::cmp::Reverse(v["collected_at"].as_i64().unwrap_or(0)),
        )
    });
    let mut seen = std::collections::HashSet::new();
    items.retain(|v| seen.insert(v["branch_id"].as_str().unwrap().to_owned()));
    Ok(
        json!({"product":p,"items":items,"coverage_note":"Menor preço conhecido entre as filiais com dados recentes. Cobertura incompleta."}),
    )
}

#[derive(Deserialize)]
struct SyncProduct {
    id: i64,
    #[serde(default)]
    historical: bool,
    #[serde(default)]
    comparison: bool,
    version: Option<String>,
}

#[derive(Deserialize)]
struct SyncRequest {
    products: Vec<SyncProduct>,
}

async fn sync_products(State(s): State<AppState>, Json(request): Json<SyncRequest>) -> ApiResult {
    if request.products.len() > 40 || request.products.iter().any(|entry| entry.id <= 0) {
        return Ok(error(
            StatusCode::BAD_REQUEST,
            "lote de até 40 produtos com IDs positivos",
        ));
    }
    let at = now();
    let mut updates = Vec::with_capacity(request.products.len());
    for entry in request.products {
        let Some(product) = catalog::product_scope(&s.pool, entry.id, entry.historical).await?
        else {
            updates.push(json!({"id":entry.id,"historical":entry.historical,"removed":true}));
            continue;
        };
        if !entry.historical
            && product.min_price_cents.is_none()
            && product.unconfirmed_price_cents.is_none()
            && !entry.comparison
        {
            updates.push(json!({"id":entry.id,"historical":entry.historical,"removed":true}));
            continue;
        }
        let payload = if entry.comparison {
            comparison_data(&s.pool, product, entry.historical, at).await?
        } else {
            json!({"product":product})
        };
        let (version, freshness) = product_version(&payload);
        let mut response = json!({"id":entry.id,"historical":entry.historical,"version":version});
        if entry.version.as_deref() == Some(version.as_str()) {
            response["freshness"] = freshness;
        } else {
            response["data"] = payload;
        }
        updates.push(response);
    }
    Ok(Json(json!({"products":updates})).into_response())
}

fn product_version(payload: &Value) -> (String, Value) {
    let mut content = payload.clone();
    let last = content["product"]
        .as_object_mut()
        .and_then(|p| p.remove("last_collected_at"));
    let mut observed = Vec::new();
    if let Some(offers) = content.get_mut("items").and_then(Value::as_array_mut) {
        for offer in offers.iter_mut() {
            if let Some(object) = offer.as_object_mut() {
                let collected = object.remove("collected_at");
                let valid = object.remove("valid_until");
                if collected.as_ref().is_some_and(Value::is_number) {
                    observed.push(json!({"branch_id":object["branch_id"],"collected_at":collected,"valid_until":valid}));
                }
            }
        }
        // Re-observing a tied offer may reorder it without changing its content.
        offers.sort_by(|left, right| left["branch_id"].as_str().cmp(&right["branch_id"].as_str()));
    }
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    (
        format!("{:016x}", hasher.finish()),
        json!({"last_collected_at":last,"offers":observed}),
    )
}

async fn status(State(s): State<AppState>, headers: HeaderMap) -> ApiResult {
    let row=sqlx::query("SELECT (SELECT COUNT(*) FROM supermarkets WHERE is_demo=0) discovered_supermarkets,(SELECT COUNT(*) FROM branches b JOIN supermarkets s ON s.id=b.supermarket_id WHERE s.is_demo=0) discovered_branches,(SELECT COUNT(DISTINCT sp.supermarket_id) FROM source_products sp JOIN supermarkets s ON s.id=sp.supermarket_id WHERE s.is_demo=0 AND sp.source_kind='retail') imported_supermarkets,(SELECT COUNT(*) FROM (SELECT supermarket_id FROM catalogue_collectors WHERE last_success_at IS NOT NULL UNION SELECT supermarket_id FROM source_cursors WHERE last_success IS NOT NULL)) automatic_integrations,(SELECT COUNT(DISTINCT sp.product_id) FROM source_products sp JOIN supermarkets s ON s.id=sp.supermarket_id WHERE s.is_demo=0 AND sp.source_kind='retail') real_products,(SELECT COUNT(*) FROM products WHERE is_demo=1) demo_products,(SELECT MAX(o.collected_at) FROM offers o JOIN source_products sp ON sp.id=o.source_product_id JOIN products p ON p.id=o.product_id WHERE p.is_demo=0 AND sp.source_kind='retail') last_collected_at,(SELECT COUNT(DISTINCT product_id) FROM source_products WHERE source_kind='public_report') historical_products,(SELECT COUNT(DISTINCT supermarket_id) FROM source_products WHERE source_kind='public_report') historical_supermarkets,(SELECT MAX(o.collected_at) FROM offers o JOIN source_products sp ON sp.id=o.source_product_id WHERE sp.source_kind='public_report') historical_last_collected_at,(SELECT COUNT(DISTINCT p.id) FROM products p JOIN offers o ON o.product_id=p.id JOIN source_products sp ON sp.id=o.source_product_id WHERE p.is_demo=0 AND sp.source_kind='retail' AND o.price_cents IS NOT NULL AND o.collected_at>=CAST(strftime('%s','now') AS INTEGER)-172800 AND o.available=1 AND (o.valid_until IS NULL OR o.valid_until>=CAST(strftime('%s','now') AS INTEGER))) recent_products").fetch_one(&s.pool).await?;
    let mut v = serde_json::Map::new();
    for key in [
        "discovered_supermarkets",
        "discovered_branches",
        "imported_supermarkets",
        "automatic_integrations",
        "real_products",
        "demo_products",
        "last_collected_at",
        "historical_products",
        "historical_supermarkets",
        "historical_last_collected_at",
        "recent_products",
    ] {
        v.insert(
            key.into(),
            row.get::<Option<i64>, _>(key)
                .map(Value::from)
                .unwrap_or(Value::Null),
        );
    }
    v.insert("freshness_hours".into(), json!(48));
    v.insert(
        "scope".into(),
        json!({"city":"Sorriso","state":"MT","ibge_code":"5107925"}),
    );
    v.insert(
        "coverage_note".into(),
        json!("Levantamento inicial; não representa todos os estabelecimentos ativos da cidade."),
    );
    Ok(cached(&headers, &v))
}

async fn collectors(State(s): State<AppState>, headers: HeaderMap) -> ApiResult {
    let rows=sqlx::query("SELECT c.*,s.name FROM catalogue_collectors c JOIN supermarkets s ON s.id=c.supermarket_id ORDER BY s.name").fetch_all(&s.pool).await?;
    let items:Vec<Value>=rows.iter().map(|r|json!({"supermarket_id":r.get::<String,_>("supermarket_id"),"name":r.get::<String,_>("name"),"adapter":r.get::<String,_>("adapter"),"interval_seconds":r.get::<i64,_>("interval_seconds"),"status":r.get::<String,_>("status"),"last_started_at":r.get::<Option<i64>,_>("last_started_at"),"last_success_at":r.get::<Option<i64>,_>("last_success_at"),"last_finished_at":r.get::<Option<i64>,_>("last_finished_at"),"last_error":r.get::<Option<String>,_>("last_error"),"requests":r.get::<i64,_>("requests"),"raw_items":r.get::<i64,_>("raw_items"),"normalized_items":r.get::<i64,_>("normalized_items"),"scope_rejected":r.get::<i64,_>("scope_rejected")})).collect();
    Ok(cached(
        &headers,
        &json!({"items":items,"scope":{"city":"Sorriso","state":"MT"},"polling_seconds":600}),
    ))
}
