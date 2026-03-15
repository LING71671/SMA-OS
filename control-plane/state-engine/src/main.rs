//! SMA-OS State Engine — gRPC Server Entry Point
//!
//! Starts the State Engine as a gRPC service exposing AppendEvent, GetEvents,
//! and HealthCheck RPCs. Integrates FailoverManager for component health
//! monitoring and RateLimiterRegistry for tenant-level rate limiting.

use std::sync::Arc;
use tracing::info;

mod engine;
mod models;
mod cluster;
mod pool;
mod limiter;
mod failover;
mod metrics;
mod cache;
mod grpc_service;

use engine::StateEngine;
use failover::{FailoverManager, FailoverConfig};
use limiter::RateLimiterRegistry;
use grpc_service::StateEngineService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("Starting SMA-OS State Engine gRPC Server v2.0...");

    // 从环境变量读取配置
    let pg_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://sma:sma@127.0.0.1/sma_state".to_string());
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string());

    // 初始化 State Engine 核心
    let engine = Arc::new(StateEngine::new(&pg_url, &redis_url).await?);

    // 初始化 FailoverManager —— 监控 Redis、PostgreSQL 等组件健康状态
    let failover_config = FailoverConfig::default();
    let failover = FailoverManager::new(failover_config);
    let failover = Arc::new(failover);

    // 初始化租户级限流器
    let rate_limiter = Arc::new(RateLimiterRegistry::new());

    // 注册所有组件到 FailoverManager
    failover.register_component("redis").await;
    failover.register_component("postgres").await;
    failover.register_component("state_engine").await;

    // 创建 gRPC 服务
    let service = StateEngineService::new(
        engine.clone(),
        failover.clone(),
        rate_limiter.clone(),
    );

    let addr = "[::]:50051".parse()?;
    info!("[gRPC] State Engine listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(service.into_server())
        .serve(addr)
        .await?;

    Ok(())
}
