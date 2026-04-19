use crate::config::AppDbConfig;
use crate::error::Result;
use crate::runtime::DbParam;
use sqlx::Postgres;
use sqlx::postgres::{PgArguments, PgPool, PgPoolOptions};
use sqlx::query::Query;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct PoolKey {
    dsn: String,
    max_connections: u32,
}

static POOLS: OnceLock<RwLock<HashMap<PoolKey, Arc<PgPool>>>> = OnceLock::new();

pub async fn shared_pool(cfg: &AppDbConfig) -> Result<Arc<PgPool>> {
    let key = PoolKey {
        dsn: cfg.dsn.clone(),
        max_connections: cfg.max_connections.max(1),
    };
    let pools = pool_cache();

    if let Some(existing) = pools.read().await.get(&key).cloned() {
        return Ok(existing);
    }

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(key.max_connections)
            .connect(&key.dsn)
            .await?,
    );

    let mut writer = pools.write().await;
    Ok(writer.entry(key).or_insert_with(|| pool.clone()).clone())
}

pub async fn shared_pool_with_defaults(dsn: &str) -> Result<Arc<PgPool>> {
    shared_pool(&AppDbConfig {
        dsn: dsn.to_string(),
        max_connections: 16,
    })
    .await
}

pub fn build_query_with_params<'q>(
    sql: &'q str,
    params: &[&(dyn DbParam + Sync)],
) -> Result<Query<'q, Postgres, PgArguments>> {
    let mut args = PgArguments::default();
    for param in params {
        param.add_to_args(&mut args)?;
    }
    Ok(sqlx::query_with(sql, args))
}

fn pool_cache() -> &'static RwLock<HashMap<PoolKey, Arc<PgPool>>> {
    POOLS.get_or_init(|| RwLock::new(HashMap::new()))
}
