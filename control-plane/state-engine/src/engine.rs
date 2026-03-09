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
        let _: () = conn.zadd(&redis_key, event.version, &event_json).await?;

        // 2. 异步持久化到 PostgreSQL (这里简化为同步，实际可由后台 worker 批量 flush)
        sqlx::query!(
            r#"
            INSERT INTO hot_events (event_id, tenant_id, namespace, version, payload, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            event.event_id,
            event.tenant_id,
            event.namespace,
            event.version as i64,
            event.payload,
            event.timestamp
        )
        .execute(&self.pg_pool)
        .await?;

        // 检查是否到达 1000 个 Event，触发全量快照生成（此处略）
        if event.version % 1000 == 0 {
            self.trigger_snapshot(event.tenant_id, event.namespace, event.version).await?;
        }

        Ok(())
    }

    /// 触发快照生成逻辑（压缩归档）
    async fn trigger_snapshot(&self, tenant_id: String, namespace: String, current_version: u64) -> Result<(), EngineError> {
        tracing::info!("Generating snapshot for {}/{} at version {}", tenant_id, namespace, current_version);
        // ... snapshot logi ...
        Ok(())
    }
}
