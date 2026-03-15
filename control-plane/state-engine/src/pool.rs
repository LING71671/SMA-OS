//! Connection pool for PostgreSQL
//!
//! Provides a simple initialization wrapper for sqlx::PgPool.
//! Previous implementation used deadpool wrapping sqlx::PgPool (double-pooling).
//! Now directly uses sqlx's built-in connection pool.

use std::time::Duration;
use tracing::info;

/// Connection pool configuration
pub struct PoolConfig {
    pub max_connections: u32,
    pub acquire_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            acquire_timeout: Duration::from_secs(5),
        }
    }
}

/// Create a PostgreSQL connection pool with the given configuration.
/// Uses sqlx's built-in pooling — no external pool wrapper needed.
pub async fn create_pg_pool(
    pg_url: &str,
    config: PoolConfig,
) -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .connect(pg_url)
        .await?;

    info!(
        "[Pool] PostgreSQL connection pool initialized (max: {})",
        config.max_connections
    );
    Ok(pool)
}
