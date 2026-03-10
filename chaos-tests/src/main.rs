//! Chaos Tests Runner for SMA-OS
//!
//! This is the main entry point for running chaos engineering tests.
//! It executes various failure scenarios to test system resilience.

mod framework;
mod scenarios;
mod reporters;

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(author, version, about = "Chaos engineering test framework for SMA-OS")]
struct Args {
    /// Scenario to run (all, node-failure, network-partition, resource-exhaustion)
    #[arg(short, long, default_value = "all")]
    scenario: String,

    /// Configuration file path
    #[arg(short, long, default_value = "configs/chaos-config.yaml")]
    config: String,

    /// Run in dry-run mode (don't actually inject failures)
    #[arg(long)]
    dry_run: bool,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Output format (text, json)
    #[arg(short, long, default_value = "text")]
    output: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(&args.log_level)),
        )
        .init();

    info!("Starting Chaos Tests");
    info!("Scenario: {}", args.scenario);
    info!("Config: {}", args.config);
    info!("Dry run: {}", args.dry_run);
    info!("Output format: {}", args.output);

    // Load configuration
    let config = framework::load_config(&args.config)?;
    
    // Run the specified scenario
    match args.scenario.as_str() {
        "all" => run_all_scenarios(&config, args.dry_run).await?,
        "node-failure" => scenarios::node_failure::run(&config, args.dry_run).await?,
        "network-partition" => scenarios::network_partition::run(&config, args.dry_run).await?,
        "resource-exhaustion" => scenarios::resource_exhaustion::run(&config, args.dry_run).await?,
        _ => {
            eprintln!("Unknown scenario: {}", args.scenario);
            std::process::exit(1);
        }
    }

    info!("Chaos tests completed successfully");
    Ok(())
}

async fn run_all_scenarios(config: &framework::ChaosConfig, dry_run: bool) -> Result<()> {
    info!("Running all chaos scenarios");
    
    // Scenario 1: Node Failure
    info!("=== Running Node Failure Scenario ===");
    scenarios::node_failure::run(config, dry_run).await?;
    
    // Scenario 2: Network Partition
    info!("=== Running Network Partition Scenario ===");
    scenarios::network_partition::run(config, dry_run).await?;
    
    // Scenario 3: Resource Exhaustion
    info!("=== Running Resource Exhaustion Scenario ===");
    scenarios::resource_exhaustion::run(config, dry_run).await?;
    
    Ok(())
}
