//! Plugin manifest definition and validation
//!
//! Each plugin must provide a manifest.json file describing its metadata,
//! capabilities, dependencies, and resource requirements.

use crate::{PluginError, PluginVersion, ResourceLimits};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Plugin manifest - the contract between plugin and host
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub manifest_version: String,
    pub metadata: PluginMetadata,
    pub capabilities: Vec<PluginCapability>,
    pub dependencies: Vec<PluginDependency>,
    pub permissions: Vec<PluginPermission>,
    pub resources: ResourceLimits,
    pub hooks: Vec<HookDefinition>,
}

impl PluginManifest {
    /// Load manifest from JSON string
    pub fn from_json(json: &str) -> Result<Self, PluginError> {
        let manifest: Self = serde_json::from_str(json)
            .map_err(|e| PluginError::InvalidManifest(format!("JSON parse error: {}", e)))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Load manifest from file
    pub fn from_file(path: &std::path::Path) -> Result<Self, PluginError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
    }

    /// Validate manifest completeness
    pub fn validate(&self) -> Result<(), PluginError> {
        // Check required fields
        if self.metadata.name.is_empty() {
            return Err(PluginError::InvalidManifest(
                "Plugin name is required".to_string(),
            ));
        }

        // Validate name length and characters
        if self.metadata.name.len() > crate::MAX_PLUGIN_NAME_LENGTH {
            return Err(PluginError::InvalidManifest(format!(
                "Plugin name too long: {} > {} characters",
                self.metadata.name.len(),
                crate::MAX_PLUGIN_NAME_LENGTH
            )));
        }

        // Validate name contains only alphanumeric characters, hyphens, and underscores
        if !self
            .metadata
            .name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(PluginError::InvalidManifest(format!(
                "Plugin name '{}' contains invalid characters. Use only alphanumeric, hyphens, and underscores",
                self.metadata.name
            )));
        }

        // Validate name doesn't start or end with hyphen/underscore
        if self.metadata.name.starts_with('-') || self.metadata.name.starts_with('_') {
            return Err(PluginError::InvalidManifest(
                "Plugin name cannot start with hyphen or underscore".to_string(),
            ));
        }
        if self.metadata.name.ends_with('-') || self.metadata.name.ends_with('_') {
            return Err(PluginError::InvalidManifest(
                "Plugin name cannot end with hyphen or underscore".to_string(),
            ));
        }

        if self.metadata.version.major == 0
            && self.metadata.version.minor == 0
            && self.metadata.version.patch == 0
        {
            return Err(PluginError::InvalidManifest("Invalid version".to_string()));
        }

        // Validate capabilities
        if self.capabilities.is_empty() {
            return Err(PluginError::InvalidManifest(
                "At least one capability is required".to_string(),
            ));
        }

        // Check for duplicate capabilities
        let mut seen = std::collections::HashSet::new();
        for cap in &self.capabilities {
            if !seen.insert(format!("{:?}", cap)) {
                return Err(PluginError::InvalidManifest(format!(
                    "Duplicate capability: {:?}",
                    cap
                )));
            }
        }

        // Validate resource limits
        if self.resources.memory_mb == 0 {
            return Err(PluginError::InvalidManifest(
                "Memory limit must be greater than 0".to_string(),
            ));
        }
        if self.resources.timeout_secs == 0 {
            return Err(PluginError::InvalidManifest(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: PluginVersion,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
}

/// Plugin capability - what the plugin can do
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", content = "config")]
pub enum PluginCapability {
    /// Custom executor for specific workloads
    #[serde(rename = "executor")]
    Executor {
        runtime: ExecutorRuntime,
        config: HashMap<String, serde_json::Value>,
    },

    /// Middleware for request/response processing
    #[serde(rename = "middleware")]
    Middleware {
        stage: MiddlewareStage,
        priority: i32,
    },

    /// Storage backend extension
    #[serde(rename = "storage")]
    Storage { backend_type: String },

    /// Authentication provider
    #[serde(rename = "auth")]
    Auth { provider_type: String },

    /// Metrics exporter
    #[serde(rename = "metrics")]
    Metrics { format: String, endpoint: String },

    /// Custom webhook handler
    #[serde(rename = "webhook")]
    Webhook {
        events: Vec<String>,
        endpoint: String,
    },
}

/// Executor runtime types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExecutorRuntime {
    #[serde(rename = "wasm")]
    Wasm,
    #[serde(rename = "docker")]
    Docker,
    #[serde(rename = "firecracker")]
    Firecracker,
    #[serde(rename = "native")]
    Native,
}

/// Middleware processing stages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MiddlewareStage {
    #[serde(rename = "pre_auth")]
    PreAuth,
    #[serde(rename = "post_auth")]
    PostAuth,
    #[serde(rename = "pre_execution")]
    PreExecution,
    #[serde(rename = "post_execution")]
    PostExecution,
}

/// Plugin dependency on other plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_constraint: String,
    pub optional: bool,
}

/// Plugin permission requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub resource: String,
    pub actions: Vec<String>,
}

/// Hook definition for lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub event: HookEvent,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookEvent {
    #[serde(rename = "on_init")]
    OnInit,
    #[serde(rename = "on_shutdown")]
    OnShutdown,
    #[serde(rename = "on_health_check")]
    OnHealthCheck,
    #[serde(rename = "on_config_change")]
    OnConfigChange,
}

/// Default manifest for new plugins
impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            manifest_version: "1.0.0".to_string(),
            metadata: PluginMetadata {
                name: "example-plugin".to_string(),
                version: PluginVersion::new(0, 1, 0),
                description: "Example SMA-OS plugin".to_string(),
                author: "Anonymous".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                repository: None,
                keywords: vec![],
                categories: vec![],
            },
            capabilities: vec![],
            dependencies: vec![],
            permissions: vec![],
            resources: ResourceLimits::default(),
            hooks: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let manifest = PluginManifest::default();
        assert!(manifest.validate().is_err()); // Missing capabilities

        let mut manifest = PluginManifest::default();
        manifest.capabilities.push(PluginCapability::Executor {
            runtime: ExecutorRuntime::Wasm,
            config: HashMap::new(),
        });
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_from_json() {
        let json = r#"{
            "manifest_version": "1.0.0",
            "metadata": {
                "name": "test-plugin",
                "version": {"major": 1, "minor": 0, "patch": 0},
                "description": "Test plugin",
                "author": "Test Author",
                "license": "MIT"
            },
            "capabilities": [
                {"type": "executor", "config": {"runtime": "wasm", "config": {}}}
            ],
            "dependencies": [],
            "permissions": [],
            "resources": {
                "cpu_cores": 1.0,
                "memory_mb": 512,
                "disk_mb": 1024,
                "network_mbps": 100,
                "timeout_secs": 300
            },
            "hooks": []
        }"#;

        let manifest = PluginManifest::from_json(json).unwrap();
        assert_eq!(manifest.metadata.name, "test-plugin");
    }
}
