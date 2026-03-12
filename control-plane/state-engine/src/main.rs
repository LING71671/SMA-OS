pub mod engine;
pub mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting SMA-OS Durable State Engine v2...");

    // Prevent immediate exit
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down.");
    Ok(())
}
