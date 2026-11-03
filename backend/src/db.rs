use anyhow::Result;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};
use std::{str::FromStr, time::Duration};

pub async fn open(url: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(url)?
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(10))
        .pragma("wal_autocheckpoint", "1000");
    let pool = SqlitePoolOptions::new()
        .max_connections(4)
        .min_connections(1)
        .connect_with(opts)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

pub async fn seed_sources(pool: &SqlitePool) -> Result<()> {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("../../config/supermarkets.json"))?;
    let mut tx = pool.begin().await?;
    for m in data.as_array().unwrap() {
        sqlx::query(
            "INSERT OR IGNORE INTO supermarkets(id,name,website,audit_class) VALUES(?,?,?,?)",
        )
        .bind(m["id"].as_str())
        .bind(m["name"].as_str())
        .bind(m["website"].as_str())
        .bind(m["audit_class"].as_str())
        .execute(&mut *tx)
        .await?;
        for b in m["branches"].as_array().unwrap() {
            sqlx::query("INSERT OR IGNORE INTO branches(id,supermarket_id,name,address,city,source_id,active) VALUES(?,?,?,?,'Sorriso',?,?)")
                .bind(b["id"].as_str()).bind(m["id"].as_str()).bind(b["name"].as_str()).bind(b["address"].as_str()).bind(b["source_id"].as_str()).bind(b["active"].as_bool().unwrap_or(true)).execute(&mut *tx).await?;
            if b["active"].as_bool() == Some(false) {
                sqlx::query("UPDATE branches SET active=0 WHERE id=?")
                    .bind(b["id"].as_str())
                    .execute(&mut *tx)
                    .await?;
            }
        }
    }
    tx.commit().await?;
    Ok(())
}

pub async fn seed_demo_sources(pool: &SqlitePool) -> Result<()> {
    for (id, name) in [
        ("demo-a", "Mercado Ipê · DEMO"),
        ("demo-b", "Mercado Cerrado · DEMO"),
        ("demo-c", "Mercado Sol · DEMO"),
    ] {
        sqlx::query("INSERT OR IGNORE INTO supermarkets(id,name,integration_status,is_demo,audit_class) VALUES(?,?,'demo',1,'D')").bind(id).bind(name).execute(pool).await?;
        sqlx::query("INSERT OR IGNORE INTO branches(id,supermarket_id,name,address,city) VALUES(?,?,'Filial de demonstração','Endereço fictício · Sorriso-MT','Sorriso')").bind(format!("{id}-1")).bind(id).execute(pool).await?;
    }
    Ok(())
}
