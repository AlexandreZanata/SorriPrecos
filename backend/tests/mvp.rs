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
        photo_source_url: None,
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
    assert_eq!(v["items"].as_array().unwrap().len(), 2);
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
    assert!(!states.contains(&"unknown"));
    assert!(
        catalog::search(&p, "arroz", 20, 0, true)
            .await
            .unwrap()
            .is_empty()
    );
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
    assert_eq!(v["items"].as_array().unwrap().len(), 1);
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
    r.source_url = Some("https://www.procon.mt.gov.br/documents/d/procon/tabela-produtos-analisados-pascoa-final#page=12".into());
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
    let (_, current) = call(app(&p), "/v1/products/search?q=arroz").await;
    assert!(current["items"].as_array().unwrap().is_empty());
    let (_, v) = call(app(&p), "/v1/products/search?q=arroz&historical=true").await;
    assert!(v["items"][0]["min_price_cents"].is_null());
    assert_eq!(v["items"][0]["reference_price_cents"], 6990);
    assert_eq!(v["items"][0]["historical_branch_count"], 1);
    let id = v["items"][0]["id"].as_i64().unwrap();
    let (_, prices) = call(
        app(&p),
        &format!("/v1/products/{id}/prices?historical=true"),
    )
    .await;
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

#[tokio::test]
async fn barcode_search_uses_canonical_gtin_and_preserves_mode() {
    let (_d, p) = fixture().await;
    let mut r = row("demo-a", 2390);
    r.gtin = Some("7896025800106".into());
    catalog::import(&p, "demo-a", &[r], None).await.unwrap();
    for code in ["7896025800106", "07896025800106"] {
        let (_, v) = call(app(&p), &format!("/v1/products/search?q={code}&demo=true")).await;
        assert_eq!(v["items"].as_array().unwrap().len(), 1);
        assert_eq!(v["items"][0]["min_price_cents"], 2390);
        let (_, v) = call(app(&p), &format!("/v1/products/search?q={code}&demo=false")).await;
        assert!(v["items"].as_array().unwrap().is_empty());
        let (_, v) = call(
            app(&p),
            &format!("/v1/products/search?q={code}&demo=true&historical=true"),
        )
        .await;
        assert!(v["items"].as_array().unwrap().is_empty());
    }
}

#[tokio::test]
async fn search_aliases_respect_exact_packaging_and_keep_product_identity() {
    let (_dir, pool) = fixture().await;
    let items = [
        ("mandioca-1000", "Mandioca congelada", 1000),
        ("macaxeira-1000", "Macaxeira descascada", 1000),
        ("macaxeira-1500", "Macaxeira descascada", 1500),
        ("mandioca-2500", "Mandioca congelada 1 kg · caixa", 2500),
    ]
    .map(|(sku, name, quantity)| ImportRow {
        source_id: sku.into(),
        name: name.into(),
        quantity,
        unit: "g".into(),
        ..row("demo-a", 1000)
    });
    catalog::import(&pool, "demo-a", &items, None)
        .await
        .unwrap();
    let (status, result) = call(app(&pool), "/v1/products/search?q=aipim%201kg&demo=true").await;
    assert_eq!(status, StatusCode::OK);
    let matches = result["items"].as_array().unwrap();
    assert_eq!(matches.len(), 2);
    assert_ne!(matches[0]["id"], matches[1]["id"]);
    assert!(matches.iter().all(|product| product["quantity"] == 1000));

    let (_, result) = call(
        app(&pool),
        "/v1/products/search?q=aipim%201%2C5%20kg&demo=true",
    )
    .await;
    let matches = result["items"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["quantity"], 1500);
}

#[tokio::test]
async fn search_corrects_transposed_letters_with_exact_size() {
    let (_dir, pool) = fixture().await;
    let mut large = row("demo-a", 2300);
    let mut small = large.clone();
    large.source_id = "arroz-5".into();
    small.source_id = "arroz-1".into();
    small.quantity = 1;
    catalog::import(&pool, "demo-a", &[large, small], None)
        .await
        .unwrap();

    let (_, result) = call(
        app(&pool),
        "/v1/products/search?q=arorz%20brnaco%205kg&demo=true",
    )
    .await;
    let matches = result["items"].as_array().unwrap();
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0]["quantity"], 5000);
    assert_eq!(matches[0]["name"], "Arroz branco");
}

#[tokio::test]
async fn product_retains_first_public_photo_across_offers() {
    let (_dir, pool) = fixture().await;
    let mut initial = row("sorriso", 2300);
    initial.branch_id = "sorriso-centro".into();
    let first = "https://files.aiqfome.com/itens/store/first.jpg";
    initial.photo_source_url = Some(first.into());
    catalog::import(
        &pool,
        "sorriso",
        &[initial.clone()],
        Some("fixture authorization"),
    )
    .await
    .unwrap();

    let mut later = initial.clone();
    later.branch_id = "delmoro-rota".into();
    later.photo_source_url = Some("https://files.aiqfome.com/itens/store/second.jpg".into());
    catalog::import(&pool, "delmoro", &[later], Some("fixture authorization"))
        .await
        .unwrap();

    let product = &catalog::search(&pool, "arroz", 10, 0, false).await.unwrap()[0];
    assert_eq!(
        product.photo_path,
        Some(format!("/v1/products/{}/photo", product.id))
    );
    let source: String = sqlx::query_scalar("SELECT photo_source_url FROM products WHERE id=?")
        .bind(product.id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(source, first);

    let id = product.id;
    sqlx::query("INSERT INTO product_photo_cache(product_id,webp) VALUES(?,X'01')")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO product_photo_retry VALUES(?,1,?)")
        .bind(id)
        .bind(now() + 60)
        .execute(&pool)
        .await
        .unwrap();
    initial.photo_source_url = Some("https://files.aiqfome.com/itens/store/replaced.jpg".into());
    initial.collected_at += 1;
    catalog::import(&pool, "sorriso", &[initial], Some("fixture authorization"))
        .await
        .unwrap();

    let updated = catalog::product(&pool, id).await.unwrap().unwrap();
    assert_eq!(updated.photo_revision, product.photo_revision + 1);
    assert_eq!(updated.photo_path, product.photo_path);
    let (cached, retry): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM product_photo_cache WHERE product_id=?),
                (SELECT COUNT(*) FROM product_photo_retry WHERE product_id=?)",
    )
    .bind(id)
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((cached, retry), (0, 0));
}

#[tokio::test]
async fn product_sync_sends_freshness_until_the_price_changes() {
    let (_dir, pool) = fixture().await;
    let first = row("demo-a", 2399);
    catalog::import(&pool, "demo-a", std::slice::from_ref(&first), None)
        .await
        .unwrap();
    let id = catalog::search(&pool, "arroz", 1, 0, true).await.unwrap()[0].id;
    async fn sync(pool: &SqlitePool, id: i64, version: Value) -> Value {
        let response = app(pool)
            .oneshot(
                Request::post("/v1/products/sync")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"products":[{"id":id,"comparison":true,"version":version}]})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body: Value =
            serde_json::from_slice(&response.into_body().collect().await.unwrap().to_bytes())
                .unwrap();
        body["products"][0].clone()
    }
    let initial = sync(&pool, id, Value::Null).await;
    assert_eq!(initial["data"]["product"]["min_price_cents"], 2399);
    let mut next = first;
    next.collected_at += 1;
    catalog::import(&pool, "demo-a", std::slice::from_ref(&next), None)
        .await
        .unwrap();
    let unchanged = sync(&pool, id, initial["version"].clone()).await;
    assert!(unchanged.get("data").is_none());
    assert_eq!(
        unchanged["freshness"]["offers"][0]["collected_at"],
        next.collected_at
    );
    next.price_cents = Some(2499);
    catalog::import(&pool, "demo-a", &[next], None)
        .await
        .unwrap();
    let changed = sync(&pool, id, initial["version"].clone()).await;
    assert_eq!(changed["data"]["product"]["min_price_cents"], 2499);
}

#[tokio::test]
async fn pharmacy_and_market_searches_have_independent_prices() {
    let (_dir, pool) = fixture().await;
    let gtin = "7896025800106";
    let mut market = row("demo-a", 2399);
    market.branch_id = "sorriso-centro".into();
    market.gtin = Some(gtin.into());
    let mut pharmacy = market.clone();
    pharmacy.branch_id = "therapeutica-farmacia-sorriso".into();
    pharmacy.category = "Farmácia".into();
    pharmacy.price_cents = Some(3199);
    catalog::import(&pool, "sorriso", &[market], Some("fixture market"))
        .await
        .unwrap();
    catalog::import(
        &pool,
        "therapeutica-farmacia",
        &[pharmacy],
        Some("fixture pharmacy"),
    )
    .await
    .unwrap();
    let (_, market) = call(app(&pool), &format!("/v1/products/search?q={gtin}")).await;
    let (_, pharmacy) = call(
        app(&pool),
        &format!("/v1/products/search?q={gtin}&sector=pharmacy"),
    )
    .await;
    assert_eq!(market["items"].as_array().unwrap().len(), 1);
    assert_eq!(pharmacy["items"].as_array().unwrap().len(), 1);
    assert_eq!(market["items"][0]["min_price_cents"], 2399);
    assert_eq!(pharmacy["items"][0]["min_price_cents"], 3199);
    assert_ne!(market["items"][0]["id"], pharmacy["items"][0]["id"]);
}

#[tokio::test]
async fn official_flyer_prices_are_searchable_but_not_ranked_as_available() {
    let (_d, pool) = fixture().await;
    let mut flyer = row("demo-a", 1850);
    flyer.branch_id = "atacadao-sorriso".into();
    flyer.source_id = "flyer-test-1-1".into();
    flyer.name = "Arroz exclusivo encarte".into();
    flyer.available = None;
    flyer.valid_until = Some(now() + 3600);
    flyer.offer_type = "club".into();
    flyer.conditions = Some("Oferta no app".into());
    flyer.promotional_price_cents = Some(1750);
    catalog::import(&pool, "atacadao", &[flyer.clone()], Some("encarte público"))
        .await
        .unwrap();
    let (_, search) = call(app(&pool), "/v1/products/search?q=arroz%20exclusivo").await;
    let product = &search["items"][0];
    assert!(product["min_price_cents"].is_null());
    assert_eq!(product["unconfirmed_price_cents"], 1850);
    assert_eq!(product["branch_count"], 0);
    let id = product["id"].as_i64().unwrap();
    let (_, comparison) = call(app(&pool), &format!("/v1/products/{id}/prices")).await;
    let offer = &comparison["items"][0];
    assert_eq!(offer["status"], "unconfirmed_stock");
    assert_eq!(offer["price_cents"], 1850);
    assert_eq!(offer["promotional_price_cents"], 1750);
    assert!(offer["available"].is_null());
    assert!(offer["effective_price_cents"].is_null());
    let res = app(&pool)
        .oneshot(
            Request::post("/v1/products/sync")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({"products":[{"id":id,"historical":false,"comparison":false}]})
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let value: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        value["products"][0]["data"]["product"]["unconfirmed_price_cents"],
        1850
    );
    sqlx::query("UPDATE offers SET valid_until=? WHERE supermarket_id='atacadao'")
        .bind(now() - 1)
        .execute(&pool)
        .await
        .unwrap();
    let (_, expired) = call(app(&pool), "/v1/products/search?q=arroz%20exclusivo").await;
    assert!(expired["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn verified_pharmacy_stock_without_price_is_searchable_until_expiry() {
    let (_d, pool) = fixture().await;
    let mut stock = row("demo-a", 1000);
    stock.branch_id = "drogasil-brescansin".into();
    stock.source_id = "rd-stock-test".into();
    stock.name = "Higiene estoque local".into();
    stock.price_cents = None;
    stock.valid_until = Some(now() + 1800);
    catalog::import(&pool, "drogasil", &[stock], Some("estoque público"))
        .await
        .unwrap();

    let search_url = "/v1/products/search?q=higiene%20estoque&sector=pharmacy";
    let (_, search) = call(app(&pool), search_url).await;
    let product = &search["items"][0];
    let id = product["id"].as_i64().unwrap();
    assert!(product["stock_only_valid_until"].is_i64());
    assert!(product["min_price_cents"].is_null());
    assert_eq!(product["branch_count"], 0);
    let prices_url = format!("/v1/products/{id}/prices");
    let (_, comparison) = call(app(&pool), &prices_url).await;
    let offer = &comparison["items"][0];
    assert_eq!(offer["status"], "stock_confirmed_no_price");
    assert_eq!(offer["available"], true);
    assert!(offer["effective_price_cents"].is_null());

    sqlx::query("UPDATE offers SET valid_until=? WHERE supermarket_id='drogasil'")
        .bind(now() - 1)
        .execute(&pool)
        .await
        .unwrap();
    let (_, expired) = call(app(&pool), search_url).await;
    assert!(expired["items"].as_array().unwrap().is_empty());
    let (_, comparison) = call(app(&pool), &prices_url).await;
    assert_eq!(comparison["items"][0]["status"], "unknown");
}

#[tokio::test]
async fn pharmacy_search_matches_preservatives_by_synonym_and_typo() {
    let (_d, pool) = fixture().await;
    let mut product = row("demo-a", 1029);
    product.branch_id = "drogaria-sao-paulo-brescansin".into();
    product.name = "Preservativo Olla Lubrificado 3 Unidades".into();
    product.brand = "Olla".into();
    product.quantity = 3;
    product.unit = "un".into();
    product.category = "Saúde sexual".into();
    catalog::import(
        &pool,
        "drogaria-sao-paulo",
        &[product],
        Some("fixture pharmacy"),
    )
    .await
    .unwrap();
    for term in [
        "camisinha",
        "preservativo",
        "camisonha",
        "proservativo",
        "camisinha%20olla",
    ] {
        let (_, data) = call(
            app(&pool),
            &format!("/v1/products/search?q={term}&sector=pharmacy"),
        )
        .await;
        assert_eq!(data["items"].as_array().unwrap().len(), 1, "{term}");
        assert_eq!(data["items"][0]["min_price_cents"], 1029);
    }
}
