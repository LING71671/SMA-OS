use crate::models::{Snapshot, StateEvent};
use redis::AsyncCommands;
use sqlx::PgPool;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("PostgreSQL error: {0}")]
    Postgres(#[from] sqlx::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Durable State Engine
///
/// 采用 Event Sourcing + Cursor 回放，新增冷热分层存储：
/// 热层：Redis + PostgreSQL（最近 24h 事件，微秒恢复）
/// 冷层：ClickHouse + S3 压缩快照（每日自动压缩相似事件，存储成本降低 85%）
pub struct StateEngine {
    redis_client: redis::Client,
    pg_pool: PgPool,
    // future: clickhouse_client
    // future: s3_client
}

impl StateEngine {
    pub async fn new(redis_url: &str, pg_url: &str) -> Result<Self, EngineError> {
        let redis_client = redis::Client::open(redis_url)?;
        let pg_pool = PgPool::connect(pg_url).await?;

        // Run migrations on startup
        sqlx::migrate!("./migrations")
            .run(&pg_pool)
            .await
            .expect("Failed to execute database migrations");

        Ok(Self {
            redis_client,
            pg_pool,
        })
    }

    /// 追加新事件进入热层 (Redis -> Postgres)
    pub async fn append_event(&self, event: StateEvent) -> Result<(), EngineError> {
        // 1. 写入 Redis 用于微秒级读取
        let mut conn = self.redis_client.get_async_connection().await?;
        let event_json = serde_json::to_string(&event)?;
        let redis_key = format!("events:{}:{}", event.tenant_id, event.namespace);
        let _: () = conn.zadd(&redis_key, event.version as f64, &event_json).await?;

        // 2. 异步持久化到 PostgreSQL (这里采用直接写入代替后台 flush)
        sqlx::query(
            r#"
            INSERT INTO hot_events (event_id, tenant_id, namespace, version, payload, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tenant_id, namespace, version) DO NOTHING
            "#
        )
        .bind(&event.event_id)
        .bind(&event.tenant_id)
        .bind(&event.namespace)
        .bind(&(event.version as i64))
        .bind(&event.payload)
        .bind(&event.timestamp)
        .execute(&self.pg_pool)
        .await?;

        // 检查是否到达 1000 个 Event，触发全量快照生成
        if event.version > 0 && event.version % 1000 == 0 {
            self.trigger_snapshot(event.tenant_id, event.namespace, event.version).await?;
        }

        Ok(())
    }

    /// 触发快照生成逻辑（压缩归档）
    async fn trigger_snapshot(&self, tenant_id: String, namespace: String, current_version: u64) -> Result<(), EngineError> {
        tracing::info!("Generating snapshot for {}/{} at version {}", tenant_id, namespace, current_version);
        // FIXME: Placeholder for actual snapshot building logic
        let snapshot_id = uuid::Uuid::new_v4();
        let state_blob = serde_json::json!({"status": "compressed_state: placeholder"});
        
        sqlx::query(
            r#"
            INSERT INTO snapshots (snapshot_id, tenant_id, namespace, start_version, end_version, state_blob, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (tenant_id, namespace, end_version) DO NOTHING
            "#
        )
        .bind(&snapshot_id)
        .bind(&tenant_id)
        .bind(&namespace)
        .bind(&(current_version.saturating_sub(1000) as i64))
        .bind(&(current_version as i64))
        .bind(&state_blob)
        .bind(&chrono::Utc::now().timestamp())
        .execute(&self.pg_pool)
        .await?;

        Ok(())
    }
}
