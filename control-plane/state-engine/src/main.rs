pub mod engine;
pub mod models;

use engine::StateEngine;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting SMA-OS Durable State Engine v2...");

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
    let pg_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://sma:sma@127.0.0.1/sma_state".to_string());

    // NOTE: Requires active databases to run successfully.
    // let engine = StateEngine::new(&redis_url, &pg_url).await?;
    // tracing::info!("State Engine connected to hot tier.");

    // Prevent immediate exit
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down.");
    Ok(())
}
