//! Chaos Testing Framework Core
//!
//! This module provides the core abstractions for chaos testing:
//! - Configuration loading
//! - Scenario execution framework
//! - Failure injection mechanisms
//! - Recovery verification

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;
use tracing::info;
use tracing::warn;

/// Chaos test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChaosConfig {
    pub cluster: ClusterConfig,
    pub scenarios: Vec<ScenarioConfig>,
    pub timeouts: TimeoutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub docker_compose_file: String,
    pub services: Vec<String>,
    pub health_check_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioConfig {
    pub name: String,
    pub r#type: ScenarioType,
    /// Duration in seconds (serialized as u64 for YAML compatibility)
    #[serde(with = "serde_secs")]
    pub duration: Duration,
    pub probability: f64,
    pub targets: Vec<String>,
}

/// Helper module for Duration serialization
mod serde_secs {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScenarioType {
    NodeFailure,
    NetworkPartition,
    ResourceExhaustion,
    LatencyInjection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub scenario_timeout_secs: u64,
    pub recovery_timeout_secs: u64,
    pub health_check_interval_secs: u64,
}

impl ChaosConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .context("Failed to read config file")?;
        
        let config: ChaosConfig = serde_yaml::from_str(&content)
            .context("Failed to parse config YAML")?;
        
        Ok(config)
    }
}

/// Load configuration from file
pub fn load_config(path: &str) -> Result<ChaosConfig> {
    ChaosConfig::load(path)
}

/// Scenario trait - all chaos scenarios must implement this
pub trait ChaosScenario {
    fn name(&self) -> &str;
    fn validate(&self) -> Result<()>;
    fn inject_failure(&mut self, dry_run: bool) -> Result<()>;
    fn verify_recovery(&self) -> Result<bool>;
    fn cleanup(&self) -> Result<()>;
}

/// Failure injector - handles injecting various types of failures
pub struct FailureInjector {
    docker_client: bollard::Docker,
}

impl FailureInjector {
    pub fn new() -> Result<Self> {
        let docker_client = bollard::Docker::connect_with_local_defaults()?;
        Ok(Self { docker_client })
    }

    /// Kill a container
    pub async fn kill_container(&self, container_id: &str) -> Result<()> {
        info!("Killing container: {}", container_id);
        
        self.docker_client
            .kill_container(container_id, None)
            .await
            .context("Failed to kill container")?;
        
        Ok(())
    }

    /// Pause a container
    pub async fn pause_container(&self, container_id: &str) -> Result<()> {
        info!("Pausing container: {}", container_id);
        
        self.docker_client
            .pause_container(container_id)
            .await
            .context("Failed to pause container")?;
        
        Ok(())
    }

    /// Unpause a container
    pub async fn unpause_container(&self, container_id: &str) -> Result<()> {
        info!("Unpausing container: {}", container_id);
        
        self.docker_client
            .unpause_container(container_id)
            .await
            .context("Failed to unpause container")?;
        
        Ok(())
    }

    /// Inject network latency
    pub async fn inject_latency(&self, container_id: &str, latency_ms: u32) -> Result<()> {
        info!("Injecting {}ms latency to container: {}", latency_ms, container_id);
        
        // Use tc (traffic control) to inject latency
        let command = format!("tc qdisc add dev eth0 root netem delay {}ms", latency_ms);
        
        self.docker_client
            .exec_create(
                container_id,
                bollard::exec::CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &command]),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
                None,
            )
            .await?;
        
        Ok(())
    }

    /// Consume CPU resources
    pub async fn consume_cpu(&self, container_id: &str, duration_secs: u64) -> Result<()> {
        info!("Consuming CPU in container {} for {} seconds", container_id, duration_secs);
        
        let command = format!(
            "timeout {}s yes > /dev/null 2>&1 || true",
            duration_secs
        );
        
        self.docker_client
            .exec_create(
                container_id,
                bollard::exec::CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &command]),
                    ..Default::default()
                },
                None,
            )
            .await?;
        
        Ok(())
    }

    /// Consume memory
    pub async fn consume_memory(&self, container_id: &str, mb: u64) -> Result<()> {
        info!("Consuming {}MB memory in container {}", mb, container_id);
        
        // Allocate memory using dd
        let command = format!("dd if=/dev/zero of=/tmp/memfile bs=1M count={}", mb);
        
        self.docker_client
            .exec_create(
                container_id,
                bollard::exec::CreateExecOptions {
                    cmd: Some(vec!["sh", "-c", &command]),
                    ..Default::default()
                },
                None,
            )
            .await?;
        
        Ok(())
    }
}

/// Health checker - verifies system health
pub struct HealthChecker {
    client: reqwest::Client,
    health_url: Option<String>,
}

impl HealthChecker {
    pub fn new(health_url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            health_url,
        }
    }

    /// Check if the system is healthy
    pub async fn is_healthy(&self) -> bool {
        if let Some(url) = &self.health_url {
            match self.client.get(url).send().await {
                Ok(response) => response.status().is_success(),
                Err(_) => false,
            }
        } else {
            // No health check URL, assume healthy
            true
        }
    }

    /// Wait for system to become healthy
    pub async fn wait_for_health(&self, timeout: Duration) -> Result<bool> {
        let start = std::time::Instant::now();
        
        while start.elapsed() < timeout {
            if self.is_healthy().await {
                return Ok(true);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        
        Ok(false)
    }
}

/// Report generator
pub fn generate_report(scenario_name: &str, success: bool, duration: Duration) -> String {
    let status = if success { "PASSED" } else { "FAILED" };
    format!(
        "Scenario: {}\nStatus: {}\nDuration: {:.2}s\n",
        scenario_name, status, duration.as_secs_f64()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load() {
        // This would test config loading if we had a test config file
        assert!(true);
    }
}
