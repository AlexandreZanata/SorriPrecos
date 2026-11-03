use anyhow::Result;
use clap::{Parser, Subcommand};
use sorriprecos::{adapters, aiqfome, api, catalog, db, now, vip};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Parser)]
#[command(
    version,
    about = "SorriPreços · API e coleta de catálogos públicos de Sorriso-MT"
)]
struct Cli {
    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "sqlite://data/sorriprecos.db"
    )]
    database_url: String,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Init,
    Serve {
        #[arg(long, env = "BIND", default_value = "127.0.0.1:8080")]
        bind: SocketAddr,
        #[arg(long, env = "RATE_LIMIT", default_value_t = 180)]
        rate_limit: u32,
    },
    Import {
        #[arg(long)]
        source: String,
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        authorization: Option<String>,
    },
    Demo,
    ImportPublicReport {
        #[arg(long)]
        source: String,
        #[arg(long)]
        file: PathBuf,
    },
    Collect {
        #[arg(long, default_value = "config/sources.json")]
        config: PathBuf,
    },
    CollectVip {
        #[arg(long, default_value = "config/vipcommerce.json")]
        config: PathBuf,
        #[arg(long)]
        watch: bool,
    },
    #[command(visible_alias = "collect-local-food")]
    CollectAiqfome {
        #[arg(long, default_value = "config/aiqfome.json")]
        config: PathBuf,
        #[arg(long)]
        watch: bool,
    },
    Maintain,
    BenchSeed {
        #[arg(long)]
        products: usize,
        #[arg(long, default_value_t = 3)]
        offers: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sorriprecos=info,tower_http=warn".into()),
        )
        .init();
    let cli = Cli::parse();
    if cli.database_url == "sqlite://data/sorriprecos.db" {
        tokio::fs::create_dir_all("data").await?;
    }
    let pool = db::open(&cli.database_url).await?;
    db::seed_sources(&pool).await?;
    match cli.command {
        Command::Init => tracing::info!("database_initialized"),
        Command::ImportPublicReport { source, file } => {
            let rows = catalog::read_file(&file).await?;
            for r in &rows {
                anyhow::ensure!(r.source_url.as_deref().is_some_and(|u|u.starts_with("https://www.procon.mt.gov.br/documents/d/procon/tabela-produtos-analisados-pascoa-final#page=")),"relatório público não reconhecido");
                anyhow::ensure!(
                    r.collected_at < now() - 172800,
                    "relatório histórico não pode virar observação atual"
                );
                anyhow::ensure!(
                    r.conditions
                        .as_deref()
                        .is_some_and(|s| s.contains("HISTÓRICO")),
                    "referência histórica precisa de aviso"
                );
            }
            let n=catalog::import(&pool,&source,&rows,Some("PUBLIC_REPORT: Procon-MT Páscoa 2026; dados históricos, sem integração comercial com a loja")).await?;
            tracing::info!(imported = n, "public_historical_report_imported");
        }
        Command::Serve { bind, rate_limit } => {
            let listener = tokio::net::TcpListener::bind(bind).await?;
            tracing::info!(%bind,"api_listening");
            axum::serve(
                listener,
                api::router(api::AppState::new(pool.clone(), rate_limit))
                    .into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await?;
        }
        Command::Import {
            source,
            file,
            authorization,
        } => {
            let rows = catalog::read_file(&file).await?;
            let n = catalog::import(&pool, &source, &rows, authorization.as_deref()).await?;
            tracing::info!(imported = n, "import_complete");
        }
        Command::Demo => {
            db::seed_demo_sources(&pool).await?;
            let rows: Vec<catalog::ImportRow> =
                serde_json::from_str(include_str!("../fixtures/demo.json"))?;
            for source in ["demo-a", "demo-b", "demo-c"] {
                let batch: Vec<_> = rows
                    .iter()
                    .filter(|r| r.branch_id.starts_with(source))
                    .cloned()
                    .map(|mut r| {
                        r.collected_at = now();
                        r
                    })
                    .collect();
                catalog::import(&pool, source, &batch, None).await?;
            }
            tracing::info!("demo_loaded_never_real_prices");
        }
        Command::Collect { config } => adapters::collect_config(&pool, &config).await?,
        Command::CollectVip { config, watch } => vip::run(&pool, &config, watch).await?,
        Command::CollectAiqfome { config, watch } => aiqfome::run(&pool, &config, watch).await?,
        Command::Maintain => {
            sqlx::query("DELETE FROM price_history WHERE collected_at<?")
                .bind(now() - 90 * 86400)
                .execute(&pool)
                .await?;
            sqlx::query("DELETE FROM collection_runs WHERE started_at<?")
                .bind(now() - 30 * 86400)
                .execute(&pool)
                .await?;
            sqlx::query("PRAGMA wal_checkpoint(PASSIVE)")
                .execute(&pool)
                .await?;
            sqlx::query("PRAGMA optimize").execute(&pool).await?;
        }
        Command::BenchSeed { products, offers } => {
            anyhow::ensure!(
                (1..=100_000).contains(&products) && (1..=3).contains(&offers),
                "limites: 100000 produtos, 3 ofertas"
            );
            db::seed_demo_sources(&pool).await?;
            for source in ["demo-a", "demo-b", "demo-c"].into_iter().take(offers) {
                for start in (0..products).step_by(500) {
                    let rows: Vec<_> = (start..products.min(start + 500))
                        .map(|i| catalog::ImportRow {
                            source_id: format!("bench-{i}"),
                            branch_id: format!("{source}-1"),
                            name: format!("Arroz sintético variedade {i:06}"),
                            brand: "DEMO Benchmark".into(),
                            quantity: 5,
                            unit: "kg".into(),
                            presentation: "pacote".into(),
                            category: "Alimentos DEMO".into(),
                            gtin: None,
                            price_cents: Some(
                                2000 + (i % 1000) as i64
                                    + source.as_bytes().last().unwrap().to_owned() as i64,
                            ),
                            promotional_price_cents: None,
                            offer_type: "regular".into(),
                            conditions: None,
                            available: Some(true),
                            source_url: None,
                            collected_at: now(),
                            valid_until: None,
                            source_city: Some("Sorriso".into()),
                            source_state: Some("MT".into()),
                        })
                        .collect();
                    catalog::import(&pool, source, &rows, None).await?;
                }
            }
            tracing::info!(products, offers, "benchmark_seeded");
        }
    }
    pool.close().await;
    Ok(())
}
