//! Connection pool for PostgreSQL and Redis
//!
//! Provides efficient connection management for 1000+ concurrent agents

use deadpool::managed::{Manager, Object, Pool, Metrics};
use std::time::Duration;
use tracing::{info, warn};

/// PostgreSQL connection manager for deadpool
pub struct PgManager {
    pg_url: String,
}

impl PgManager {
    pub fn new(pg_url: String) -> Self {
        Self { pg_url }
    }
}

#[async_trait::async_trait]
impl Manager for PgManager {
    type Type = sqlx::PgPool;
    type Error = sqlx::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&self.pg_url)
            .await
    }

    async fn recycle(
        &self,
        conn: &mut Self::Type,
        _metrics: &Metrics,
    ) -> deadpool::managed::RecycleResult<Self::Error> {
        if conn.is_closed() {
            warn!("[Pool] Connection pool is closed");
            return Err(deadpool::managed::RecycleError::Backend(
                sqlx::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::ConnectionAborted,
                    "Pool closed",
                )),
            ));
        }
        Ok(())
    }
}

/// Connection pool wrapper
pub struct StateEnginePool {
    pg_pool: Pool<PgManager>,
}

impl StateEnginePool {
    pub async fn new(pg_url: &str) -> Result<Self, deadpool::managed::BuildError> {
        let manager = PgManager::new(pg_url.to_string());
        let pool = Pool::builder(manager)
            .max_size(20)
            .build()?;

        info!("[Pool] PostgreSQL connection pool initialized (max: 20)");
        Ok(Self { pg_pool: pool })
    }

    pub async fn get(&self) -> Result<Object<PgManager>, deadpool::managed::PoolError<sqlx::Error>> {
        self.pg_pool.get().await
    }
}
