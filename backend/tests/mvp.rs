use axum::{
    Json, Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sorriprecos::{
    adapters::{self, SourceAdapter},
    api::{self, AppState},
    catalog::{self, ImportRow},
    db, normalize, now,
};
use sqlx::{Row, SqlitePool};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tower::ServiceExt;

async fn fixture() -> (tempfile::TempDir, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let pool = db::open(&format!("sqlite://{}", path.display()))
        .await
        .unwrap();
    db::seed_sources(&pool).await.unwrap();
    db::seed_demo_sources(&pool).await.unwrap();
    (dir, pool)
}
fn row(source: &str, price: i64) -> ImportRow {
    ImportRow {
        source_id: "arroz".into(),
        branch_id: format!("{source}-1"),
        name: "Arroz branco".into(),
        brand: "DEMO".into(),
        quantity: 5,
        unit: "kg".into(),
        presentation: "pacote".into(),
        category: "Mercearia".into(),
        gtin: None,
        price_cents: Some(price),
        promotional_price_cents: None,
        offer_type: "regular".into(),
        conditions: None,
        available: Some(true),
        source_url: None,
        collected_at: now(),
        valid_until: None,
        source_city: Some("Sorriso".into()),
        source_state: Some("MT".into()),
    }
}
async fn call(app: Router, path: &str) -> (StatusCode, Value) {
    let res = app
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}
fn app(pool: &SqlitePool) -> Router {
    api::router(AppState::new(pool.clone(), 10000))
}

#[test]
fn names_and_accents() {
    assert_eq!(normalize::text("  CAFÉ — AÇÚCAR!  "), "cafe acucar");
}
#[test]
fn gtin_checksum() {
    assert!(normalize::valid_gtin("7891000100103"));
    assert!(!normalize::valid_gtin("7891000100104"));
    assert!(!normalize::valid_gtin("00000000"));
    assert!(!normalize::valid_gtin("abc"));
}
#[test]
fn packaging_units() {
    assert_eq!(normalize::package(5, "kg").unwrap(), (5000, "g".into()));
    assert_ne!(
        normalize::package(1, "kg").unwrap(),
        normalize::package(5, "kg").unwrap()
    );
    assert!(normalize::package(-1, "g").is_err());
    assert!(normalize::package(i64::MAX, "kg").is_err());
}
#[test]
fn cents_are_integers() {
    let mut r = row("demo-a", 0);
    assert!(catalog::validate(&r).is_err());
    r.price_cents = Some(2399);
    assert!(catalog::validate(&r).is_ok());
    r.promotional_price_cents = Some(2500);
    assert!(catalog::validate(&r).is_err());
    let mut value = serde_json::to_value(&r).unwrap();
    value["price_cents"] = json!(23.99);
    assert!(serde_json::from_value::<ImportRow>(value).is_err());
}
#[test]
fn query_sanitized() {
    assert_eq!(
        normalize::fts_query("arroz OR * \" 5kg"),
        "\"arroz\"* AND \"or\"* AND \"5kg\"*"
    );
}
#[test]
fn medicine_rejected() {
    let mut r = row("demo-a", 500);
    r.category = "Medicamentos".into();
    assert!(catalog::validate(&r).is_err());
}
#[test]
fn retry_after_respected() {
    assert_eq!(
        adapters::retry_delay(Some("180"), 0),
        Duration::from_secs(180)
    );
    assert_eq!(adapters::retry_delay(None, 2), Duration::from_secs(4));
}

#[tokio::test]
async fn migrations_wal_and_foreign_keys() {
    let (_d, p) = fixture().await;
    assert_eq!(
        sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
            .fetch_one(&p)
            .await
            .unwrap(),
        "wal"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("PRAGMA foreign_keys")
            .fetch_one(&p)
            .await
            .unwrap(),
        1
    );
    assert!(sqlx::query("INSERT INTO branches(id,supermarket_id,name,address,city) VALUES('bad','missing','x','x','Sorriso')").execute(&p).await.is_err());
}
#[tokio::test]
async fn import_is_idempotent_and_history_changes_only() {
    let (_d, p) = fixture().await;
    let r = row("demo-a", 2399);
    for _ in 0..2 {
        catalog::import(&p, "demo-a", std::slice::from_ref(&r), None)
            .await
            .unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM offers")
            .fetch_one(&p)
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM price_history")
            .fetch_one(&p)
            .await
            .unwrap(),
        1
    );
    let mut r = r;
    r.price_cents = Some(2299);
    catalog::import(&p, "demo-a", &[r], None).await.unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM price_history")
            .fetch_one(&p)
            .await
            .unwrap(),
        2
    );
}
#[tokio::test]
async fn matching_gtin_and_package_conflict() {
    let (_d, p) = fixture().await;
    let mut a = row("demo-a", 2399);
    a.gtin = Some("7891000100103".into());
    catalog::import(&p, "demo-a", &[a], None).await.unwrap();
    let mut b = row("demo-b", 2199);
    b.gtin = Some("07891000100103".into());
    b.name = "Outro título da loja".into();
    catalog::import(&p, "demo-b", &[b.clone()], None)
        .await
        .unwrap();
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM products")
            .fetch_one(&p)
            .await
            .unwrap(),
        1
    );
    b.quantity = 1;
    assert!(catalog::import(&p, "demo-b", &[b], None).await.is_err());
}
#[tokio::test]
async fn distinct_packaging_never_merged() {
    let (_d, p) = fixture().await;
    let a = row("demo-a", 2399);
    let mut b = row("demo-b", 599);
    b.quantity = 1;
    catalog::import(&p, "demo-a", &[a], None).await.unwrap();
    catalog::import(&p, "demo-b", &[b], None).await.unwrap();
    assert_eq!(
        catalog::search(&p, "arroz", 20, 0, true)
            .await
            .unwrap()
            .len(),
        2
    );
}
#[tokio::test]
async fn wrong_branch_rolls_back_entire_batch() {
    let (_d, p) = fixture().await;
    catalog::import(&p, "demo-a", &[row("demo-a", 2399)], None)
        .await
        .unwrap();
    let valid = row("demo-a", 1999);
    let mut invalid = row("demo-b", 999);
    invalid.source_id = "new".into();
    assert!(
        catalog::import(&p, "demo-a", &[valid, invalid], None)
            .await
            .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT price_cents FROM offers")
            .fetch_one(&p)
            .await
            .unwrap(),
        2399
    );
    assert_eq!(
        sqlx::query_scalar::<_, String>(
            "SELECT status FROM collection_runs ORDER BY id DESC LIMIT 1"
        )
        .fetch_one(&p)
        .await
        .unwrap(),
        "failed"
    );
}
#[tokio::test]
async fn old_snapshot_does_not_overwrite() {
    let (_d, p) = fixture().await;
    catalog::import(&p, "demo-a", &[row("demo-a", 2399)], None)
        .await
        .unwrap();
    let mut r = row("demo-a", 999);
    r.collected_at -= 86400;
    assert_eq!(catalog::import(&p, "demo-a", &[r], None).await.unwrap(), 0);
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT price_cents FROM offers")
            .fetch_one(&p)
            .await
            .unwrap(),
        2399
    );
}
#[tokio::test]
async fn text_search_accents_and_package() {
    let (_d, p) = fixture().await;
    let mut r = row("demo-a", 2399);
    r.name = "Café torrado".into();
    catalog::import(&p, "demo-a", &[r], None).await.unwrap();
    assert_eq!(
        catalog::search(&p, "cafe 5kg", 20, 0, true)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        catalog::search(&p, "café 1kg", 20, 0, true)
            .await
            .unwrap()
            .len(),
        0
    );
}
#[tokio::test]
async fn ordering_and_branch_links() {
    let (_d, p) = fixture().await;
    for (s, price) in [("demo-a", 2399), ("demo-b", 2199)] {
        catalog::import(&p, s, &[row(s, price)], None)
            .await
            .unwrap();
    }
    let (_, v) = call(app(&p), "/v1/products/1/prices").await;
    assert_eq!(v["items"][0]["branch_id"], "demo-b-1");
    assert_eq!(v["items"][0]["effective_price_cents"], 2199);
    assert_eq!(v["items"][2]["status"], "unknown");
}
#[tokio::test]
async fn conditional_promo_does_not_mislead() {
    let (_d, p) = fixture().await;
    let mut r = row("demo-a", 2399);
    r.offer_type = "club".into();
    r.promotional_price_cents = Some(999);
    r.conditions = Some("Exige clube".into());
    catalog::import(&p, "demo-a", &[r], None).await.unwrap();
    let (_, v) = call(app(&p), "/v1/products/1/prices").await;
    assert_eq!(v["items"][0]["effective_price_cents"], 2399);
    assert_eq!(v["product"]["min_price_cents"], 2399);
}
#[tokio::test]
async fn stale_unavailable_unknown_and_expired() {
    let (_d, p) = fixture().await;
    let mut a = row("demo-a", 2399);
    a.collected_at -= 172801;
    catalog::import(&p, "demo-a", &[a], None).await.unwrap();
    let mut b = row("demo-b", 2199);
    b.available = Some(false);
    catalog::import(&p, "demo-b", &[b], None).await.unwrap();
    let (_, v) = call(app(&p), "/v1/products/1/prices").await;
    let states: Vec<_> = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["status"].as_str().unwrap())
        .collect();
    assert!(states.contains(&"stale"));
    assert!(states.contains(&"unavailable"));
    assert!(states.contains(&"unknown"));
    assert!(v["product"]["min_price_cents"].is_null());
    let mut c = row("demo-c", 1999);
    c.collected_at = now() - 60;
    c.valid_until = Some(now() - 1);
    catalog::import(&p, "demo-c", &[c], None).await.unwrap();
    let (_, v) = call(app(&p), "/v1/products/1/prices").await;
    assert!(
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|x| x["effective_price_cents"].is_null())
    );
}
#[tokio::test]
async fn real_and_demo_are_isolated_and_authorization_required() {
    let (_d, p) = fixture().await;
    catalog::import(&p, "demo-a", &[row("demo-a", 2399)], None)
        .await
        .unwrap();
    let mut r = row("demo-a", 2599);
    r.branch_id = "sorriso-centro".into();
    assert!(
        catalog::import(&p, "sorriso", &[r.clone()], None)
            .await
            .is_err()
    );
    catalog::import(&p, "sorriso", &[r], Some("fixture de autorização de teste"))
        .await
        .unwrap();
    let real = catalog::search(&p, "arroz", 20, 0, false).await.unwrap();
    assert_eq!(real.len(), 1);
    assert!(!real[0].is_demo);
    let (_, v) = call(app(&p), "/v1/catalog/status").await;
    assert_eq!(v["automatic_integrations"], 0);
    assert_eq!(v["imported_supermarkets"], 1);
}
#[tokio::test]
async fn csv_and_json_roundtrip() {
    let (_d, p) = fixture().await;
    for format in ["json", "csv"] {
        let data = std::fs::read(format!("fixtures/demo.{format}")).unwrap();
        let rows = catalog::parse(&data, format).unwrap();
        let rows: Vec<_> = rows
            .into_iter()
            .filter(|r| r.branch_id == "demo-a-1")
            .collect();
        catalog::import(&p, "demo-a", &rows, None).await.unwrap();
    }
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM offers")
            .fetch_one(&p)
            .await
            .unwrap(),
        12
    );
}
#[tokio::test]
async fn api_limits_and_missing_product() {
    let (_d, p) = fixture().await;
    for path in [
        "/v1/products/search?q=a",
        "/v1/products/search?q=arroz&limit=51",
        "/v1/products/search?q=arroz&after=-1",
    ] {
        assert_eq!(call(app(&p), path).await.0, StatusCode::BAD_REQUEST);
    }
    assert_eq!(
        call(app(&p), "/v1/products/999").await.0,
        StatusCode::NOT_FOUND
    );
    let limited = api::router(AppState::new(p.clone(), 1));
    assert_eq!(call(limited.clone(), "/health").await.0, StatusCode::OK);
    assert_eq!(
        call(limited, "/health").await.0,
        StatusCode::TOO_MANY_REQUESTS
    );
}
#[tokio::test]
async fn etag_conditional_get() {
    let (_d, p) = fixture().await;
    let a = app(&p);
    let response = a
        .clone()
        .oneshot(
            Request::builder()
                .uri("/v1/catalog/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let etag = response.headers().get("etag").unwrap().clone();
    let res = a
        .oneshot(
            Request::builder()
                .uri("/v1/catalog/status")
                .header("if-none-match", etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_MODIFIED);
}
#[tokio::test]
async fn simultaneous_read_and_write() {
    let (_d, p) = fixture().await;
    catalog::import(&p, "demo-a", &[row("demo-a", 2399)], None)
        .await
        .unwrap();
    let writer = p.clone();
    let task = tokio::spawn(async move {
        for i in 0..30 {
            catalog::import(&writer, "demo-a", &[row("demo-a", 2000 + i)], None)
                .await
                .unwrap();
        }
    });
    for _ in 0..30 {
        assert_eq!(
            catalog::search(&p, "arroz", 20, 0, true)
                .await
                .unwrap()
                .len(),
            1
        );
    }
    task.await.unwrap();
}
#[tokio::test]
async fn pagination_no_duplicate_results() {
    let (_d, p) = fixture().await;
    let mut second = row("demo-a", 599);
    second.source_id = "small".into();
    second.quantity = 1;
    catalog::import(&p, "demo-a", &[row("demo-a", 2399), second], None)
        .await
        .unwrap();
    let (_, v) = call(app(&p), "/v1/products/search?q=arroz&demo=true&limit=1").await;
    let after = v["next_after"].as_i64().unwrap();
    let (_, w) = call(
        app(&p),
        &format!("/v1/products/search?q=arroz&demo=true&limit=1&after={after}"),
    )
    .await;
    assert_ne!(v["items"][0]["id"], w["items"][0]["id"]);
}

fn config(url: String) -> adapters::SourceConfig {
    adapters::SourceConfig {
        supermarket_id: "demo-a".into(),
        enabled: true,
        authorization_reference: Some("feed sintético autorizado para teste".into()),
        url: Some(url),
        format: "json".into(),
        interval_seconds: 3600,
        max_pages: 3,
    }
}

#[tokio::test]
async fn authorized_feed_paginates_and_search_never_collects() {
    let (_d, p) = fixture().await;
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut second = row("demo-a", 599);
    second.quantity = 1;
    second.source_id = "small".into();
    let feed = Router::new()
        .route(
            "/feed",
            get(move || {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Json(json!({"items":[row("demo-a",2399)],"next":"/page2"}))
                }
            }),
        )
        .route(
            "/page2",
            get(move || {
                let r = second.clone();
                async move { Json(json!({"items":[r]})) }
            }),
        );
    let server = tokio::spawn(async move { axum::serve(listener, feed).await.unwrap() });
    let adapter =
        adapters::AuthorizedFeedAdapter::new(config(format!("http://{addr}/feed"))).unwrap();
    assert_eq!(adapter.collect(&p).await.unwrap(), 2);
    let before = count.load(Ordering::SeqCst);
    for _ in 0..3 {
        assert_eq!(
            call(app(&p), "/v1/products/search?q=arroz&demo=true")
                .await
                .0,
            StatusCode::OK
        );
    }
    assert_eq!(count.load(Ordering::SeqCst), before);
    let request_count: i64 = sqlx::query_scalar("SELECT MAX(requests) FROM collection_runs")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(request_count, 2);
    assert_eq!(adapter.collect(&p).await.unwrap(), 0);
    server.abort();
}
#[tokio::test]
async fn source_failure_preserves_prices() {
    let (_d, p) = fixture().await;
    catalog::import(&p, "demo-a", &[row("demo-a", 2399)], None)
        .await
        .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route(
                "/feed",
                get(|| async { (StatusCode::TOO_MANY_REQUESTS, [("retry-after", "180")]) }),
            ),
        )
        .await
        .unwrap()
    });
    let adapter =
        adapters::AuthorizedFeedAdapter::new(config(format!("http://{addr}/feed"))).unwrap();
    assert!(adapter.collect(&p).await.is_err());
    let r = sqlx::query("SELECT price_cents FROM offers")
        .fetch_one(&p)
        .await
        .unwrap();
    assert_eq!(r.get::<i64, _>("price_cents"), 2399);
    server.abort();
}

#[tokio::test]
async fn duplicate_canonical_skus_have_one_offer_per_branch() {
    let (_d, p) = fixture().await;
    let a = row("demo-a", 2399);
    let mut b = a.clone();
    b.source_id = "alternative-sku".into();
    b.price_cents = Some(2299);
    catalog::import(&p, "demo-a", &[a, b], None).await.unwrap();
    let (_, v) = call(app(&p), "/v1/products/1/prices").await;
    assert_eq!(v["items"].as_array().unwrap().len(), 3);
    assert_eq!(v["items"][0]["effective_price_cents"], 2299);
    assert_eq!(v["product"]["branch_count"], 1);
}

#[tokio::test]
async fn real_import_rejects_foreign_or_missing_municipality() {
    let (_d, p) = fixture().await;
    let mut r = row("demo-a", 6990);
    r.branch_id = "sorriso-centro".into();
    for city in [Some("Sinop".into()), None] {
        r.source_city = city;
        assert!(
            catalog::import(
                &p,
                "sorriso",
                std::slice::from_ref(&r),
                Some("autorização de teste")
            )
            .await
            .is_err()
        );
    }
    r.source_city = Some("Sorriso".into());
    r.source_state = Some("MS".into());
    assert!(
        catalog::import(
            &p,
            "sorriso",
            std::slice::from_ref(&r),
            Some("autorização de teste")
        )
        .await
        .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM offers")
            .fetch_one(&p)
            .await
            .unwrap(),
        0
    );
}
#[tokio::test]
async fn historical_reference_never_becomes_a_current_minimum() {
    let (_d, p) = fixture().await;
    let mut r = row("demo-a", 6990);
    r.branch_id = "sorriso-centro".into();
    r.collected_at = 1774152000;
    r.available = None;
    r.conditions = Some("HISTÓRICO · Procon-MT / março de 2026".into());
    catalog::import(
        &p,
        "sorriso",
        &[r],
        Some("PUBLIC_REPORT: relatório público de teste"),
    )
    .await
    .unwrap();
    // A newer unconfirmed price must not replace the documented historical reference.
    let mut unknown = row("demo-a", 1000);
    unknown.branch_id = "delmoro-rota".into();
    unknown.available = None;
    catalog::import(&p, "delmoro", &[unknown], Some("referência de teste"))
        .await
        .unwrap();
    let (_, v) = call(app(&p), "/v1/products/search?q=arroz").await;
    assert!(v["items"][0]["min_price_cents"].is_null());
    assert_eq!(v["items"][0]["reference_price_cents"], 6990);
    assert_eq!(v["items"][0]["historical_branch_count"], 1);
    let id = v["items"][0]["id"].as_i64().unwrap();
    let (_, prices) = call(app(&p), &format!("/v1/products/{id}/prices")).await;
    let historic = prices["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["price_cents"] == 6990)
        .unwrap();
    assert_eq!(historic["status"], "stale");
    assert!(historic["available"].is_null());
    assert!(historic["effective_price_cents"].is_null());
    let (_, status) = call(app(&p), "/v1/catalog/status").await;
    assert_eq!(status["scope"]["city"], "Sorriso");
    assert_eq!(status["scope"]["state"], "MT");
    assert_eq!(status["recent_products"], 0);
    assert_eq!(status["historical_products"], 1);
    assert_eq!(status["automatic_integrations"], 0);
}
