use anyhow::Result;
use tracing::info;

#[cfg(unix)]
use hyperlocal::UnixClientExt;

mod microvm;
mod api;
mod health;

use microvm::{MicroVMManager, VmConfig};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting SMA-OS Firecracker Sandbox Daemon v2.0...");

    // Create VM manager with max 10 VMs and snapshot directory
    let manager = MicroVMManager::new(10, "/tmp/smaos-snapshots".to_string());

    // Example: Create a VM with default config
    let config = VmConfig::default();
    let vm_id = manager.create(config).await?;
    info!("Created VM: {}", vm_id);

    // Start the VM
    manager.start(&vm_id).await?;
    info!("Started VM: {}", vm_id);

    // Get VM stats
    let stats = manager.get_vm_stats(&vm_id).await?;
    info!("VM Stats: CPU: {}%, Memory: {}MB", 
          stats.cpu_usage_percent, stats.memory_usage_mb);

    // List all VMs
    let vms = manager.list_vms().await;
    info!("Total VMs: {}", vms.len());

    // Create a snapshot
    let snapshot_id = manager.snapshot(&vm_id).await?;
    info!("Created snapshot: {}", snapshot_id);

    info!("Daemon running and waiting to assign sandboxes to Orchestration commands...");
    tokio::signal::ctrl_c().await?;
    info!("Shutting down daemon...");

    // Cleanup
    manager.destroy(&vm_id).await?;

    Ok(())
}
