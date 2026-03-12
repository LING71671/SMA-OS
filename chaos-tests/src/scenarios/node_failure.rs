//! Node Failure Scenario
//!
//! This scenario tests system resilience by killing containers
//! and verifying automatic recovery.

use crate::framework::{ChaosConfig, FailureInjector, HealthChecker, generate_report};
use anyhow::Result;
use std::time::{Duration, Instant};
use tracing::info;
use tracing::warn;

/// Run the node failure scenario
pub async fn run(config: &ChaosConfig, dry_run: bool) -> Result<()> {
    info!("Starting Node Failure scenario");
    info!("Targets: {:?}", config.cluster.services);
    
    let start = Instant::now();
    let mut success = true;

    if !dry_run {
        let mut injector = FailureInjector::new()?;
        let health_checker = HealthChecker::new(config.cluster.health_check_url.clone());

        // For each service, kill and verify recovery
        for service in &config.cluster.services {
            info!("=== Injecting node failure for: {} ===", service);
            
            // Kill the container
            if let Err(e) = inject_node_failure(&mut injector, service).await {
                warn!("Failed to kill container {}: {}", service, e);
                success = false;
                continue;
            }

            // Wait for recovery
            let timeout = Duration::from_secs(config.timeouts.recovery_timeout_secs as u64);
            let recovered = health_checker.wait_for_health(timeout).await?;

            if !recovered {
                warn!("Service {} failed to recover within timeout", service);
                success = false;
            } else {
                info!("Service {} recovered successfully", service);
            }
        }
    } else {
        info!("[DRY RUN] Would kill containers: {:?}", config.cluster.services);
    }

    let duration = start.elapsed();
    let report = generate_report("Node Failure", success, duration);
    println!("{}", report);

    if !success {
        Err(anyhow::anyhow!("Node failure scenario failed"))
    } else {
        Ok(())
    }
}

async fn inject_node_failure(injector: &mut FailureInjector, container_id: &str) -> Result<()> {
    injector.kill_container(container_id).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_failure_dry_run() {
        // This is a placeholder for actual tests
        assert!(true);
    }
}
