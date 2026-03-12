//! SMA-OS Plugin System Core
//!
//! Provides the foundation for dynamically loaded plugins that extend SMA-OS functionality.
//! Supports custom executors, middleware, and extensions.

pub mod registry;
pub mod loader;
pub mod executor;
pub mod manifest;
pub mod sandbox;

pub use registry::PluginRegistry;
pub use loader::PluginLoader;
pub use executor::{ExecutorPlugin, ExecutionContext, ExecutionResult};
pub use manifest::{PluginManifest, PluginMetadata, PluginCapability};
pub use sandbox::PluginSandbox;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    /// Initialize the plugin
    async fn init(&mut self, config: PluginConfig) -> Result<(), PluginError>;
    
    /// Shutdown the plugin gracefully
    async fn shutdown(&mut self) -> Result<(), PluginError>;
    
    /// Get plugin health status
    async fn health(&self) -> PluginHealth;
    
    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_id: Uuid,
    pub tenant_id: String,
    pub namespace: String,
    pub config: HashMap<String, serde_json::Value>,
    pub resource_limits: ResourceLimits,
}

/// Resource limits for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_cores: f64,
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub network_mbps: u64,
    pub timeout_secs: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_cores: 1.0,
            memory_mb: 512,
            disk_mb: 1024,
            network_mbps: 100,
            timeout_secs: 300,
        }
    }
}

/// Plugin health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHealth {
    pub status: HealthStatus,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub message: Option<String>,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Plugin error types
#[derive(Error, Debug)]
pub enum PluginError {
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin already loaded: {0}")]
    AlreadyLoaded(String),
    
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Sandbox error: {0}")]
    SandboxError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Plugin version with semver support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginVersion {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
}

impl PluginVersion {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
        }
    }
    
    pub fn with_prerelease(mut self, pre: String) -> Self {
        self.prerelease = Some(pre);
        self
    }
}

impl std::fmt::Display for PluginVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.prerelease {
            write!(f, "-{}", pre)?;
        }
        Ok(())
    }
}

/// Plugin event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginEvent {
    Loaded { plugin_id: Uuid, name: String },
    Unloaded { plugin_id: Uuid },
    ExecutionStarted { execution_id: Uuid, plugin_id: Uuid },
    ExecutionCompleted { execution_id: Uuid, result: ExecutionResult },
    ExecutionFailed { execution_id: Uuid, error: String },
    HealthChanged { plugin_id: Uuid, status: HealthStatus },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_version_display() {
        let version = PluginVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
        
        let version_pre = PluginVersion::new(1, 2, 3).with_prerelease("alpha.1".to_string());
        assert_eq!(version_pre.to_string(), "1.2.3-alpha.1");
    }

    #[test]
    fn test_plugin_version_ordering() {
        let v1 = PluginVersion::new(1, 0, 0);
        let v2 = PluginVersion::new(1, 1, 0);
        let v3 = PluginVersion::new(1, 1, 1);
        
        assert!(v1 < v2);
        assert!(v2 < v3);
    }
}