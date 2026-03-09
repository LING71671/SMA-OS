pub mod controller;

use controller::{CascadingTeardownController, TeardownTarget};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting SMA-OS Cascading Teardown Controller v2...");

    // The K8s client requires an active cluster or kubeconfig
    // For scaffolding, we will just initialize and wait for signals.
    /*
    let controller = CascadingTeardownController::new().await?;

    let dummy_target = TeardownTarget {
        tenant_id: "tenant-alpha".to_string(),
        namespace: "sma-os-workers-alpha".to_string(),
        task_group_id: Uuid::new_v4(),
        force: true,
    };
    
    controller.execute_teardown(dummy_target).await?;
    */

    tracing::info!("Waiting for incoming teardown commands...");
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down teardown controller.");

    Ok(())
}
