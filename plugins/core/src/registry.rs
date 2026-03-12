//! Plugin registry for managing plugin lifecycle
//!
//! Provides a centralized registry for loading, unloading, and querying plugins.

use crate::{Plugin, PluginConfig, PluginError, PluginEvent, PluginHealth, PluginManifest, PluginMetadata};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Central plugin registry
pub struct PluginRegistry {
    /// Loaded plugins indexed by ID
    plugins: DashMap<Uuid, Arc<RwLock<Box<dyn Plugin>>>>,
    
    /// Plugin manifests indexed by ID
    manifests: DashMap<Uuid, PluginManifest>,
    
    /// Plugin health status
    health_status: DashMap<Uuid, PluginHealth>,
    
    /// Index by plugin name (name -> IDs)
    name_index: DashMap<String, Vec<Uuid>>,
    
    /// Event broadcaster
    event_tx: broadcast::Sender<PluginEvent>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        
        Self {
            plugins: DashMap::new(),
            manifests: DashMap::new(),
            health_status: DashMap::new(),
            name_index: DashMap::new(),
            event_tx,
        }
    }
    
    /// Subscribe to plugin events
    pub fn subscribe(&self) -> broadcast::Receiver<PluginEvent> {
        self.event_tx.subscribe()
    }
    
    /// Register a new plugin
    pub async fn register(
        &self,
        plugin: Box<dyn Plugin>,
        manifest: PluginManifest,
    ) -> Result<Uuid, PluginError> {
        let metadata = plugin.metadata();
        let plugin_id = Uuid::new_v4();
        
        // Check for name conflicts
        if let Some(ids) = self.name_index.get(&metadata.name) {
            for existing_id in ids.iter() {
                if let Some(existing) = self.manifests.get(existing_id) {
                    if existing.metadata.version == metadata.version {
                        return Err(PluginError::AlreadyLoaded(format!(
                            "Plugin {} version {} already registered",
                            metadata.name, metadata.version
                        )));
                    }
                }
            }
        }
        
        // Initialize plugin with cleanup on failure
        let mut plugin_guard = plugin;
        let config = PluginConfig {
            plugin_id,
            tenant_id: crate::DEFAULT_TENANT_ID.to_string(),
            namespace: crate::DEFAULT_NAMESPACE.to_string(),
            config: Default::default(),
            resource_limits: manifest.resources.clone(),
        };

        // Try to initialize - if it fails, attempt cleanup
        let init_result = plugin_guard.init(config).await;
        if let Err(ref e) = init_result {
            // Attempt graceful cleanup of partially initialized plugin
            warn!("[PluginRegistry] Plugin {} init failed: {}, attempting cleanup", metadata.name, e);
            let _ = plugin_guard.shutdown().await;
            return Err(PluginError::ExecutionFailed(format!(
                "Failed to initialize plugin: {}", e
            )));
        }

        // Store plugin
        self.plugins.insert(plugin_id, Arc::new(RwLock::new(plugin_guard)));
        self.manifests.insert(plugin_id, manifest);
        self.name_index.entry(metadata.name.clone()).or_insert_with(Vec::new).push(plugin_id);
        
        // Set initial health
        let health = PluginHealth {
            status: crate::HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            message: Some("Plugin loaded successfully".to_string()),
            metrics: Default::default(),
        };
        self.health_status.insert(plugin_id, health);
        
        // Broadcast event
        let _ = self.event_tx.send(PluginEvent::Loaded {
            plugin_id,
            name: metadata.name.clone(),
        });
        
        info!("[PluginRegistry] Registered plugin {} ({})", metadata.name, plugin_id);
        
        Ok(plugin_id)
    }
    
    /// Unload a plugin
    pub async fn unregister(&self, plugin_id: Uuid) -> Result<(), PluginError> {
        let plugin = self.plugins.get(&plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;
        
        // Get plugin name before removing
        let name = {
            let guard = plugin.read().await;
            guard.metadata().name.clone()
        };
        
        // Shutdown plugin
        {
            let mut guard = plugin.write().await;
            guard.shutdown().await.map_err(|e| {
                PluginError::ExecutionFailed(format!("Failed to shutdown plugin: {}", e))
            })?;
        }
        
        // Remove from registry
        drop(plugin);
        self.plugins.remove(&plugin_id);
        self.manifests.remove(&plugin_id);
        self.health_status.remove(&plugin_id);
        
        // Update name index
        if let Some(mut ids) = self.name_index.get_mut(&name) {
            ids.retain(|&id| id != plugin_id);
        }
        
        // Broadcast event
        let _ = self.event_tx.send(PluginEvent::Unloaded { plugin_id });
        
        info!("[PluginRegistry] Unregistered plugin {} ({})", name, plugin_id);
        
        Ok(())
    }
    
    /// Get a plugin by ID
    pub fn get(&self, plugin_id: Uuid) -> Option<Arc<RwLock<Box<dyn Plugin>>>> {
        self.plugins.get(&plugin_id).map(|p| p.clone())
    }
    
    /// Get plugin by name and version
    pub fn get_by_name(&self, name: &str, version: Option<&crate::PluginVersion>) -> Option<(Uuid, Arc<RwLock<Box<dyn Plugin>>>)> {
        let ids = self.name_index.get(name)?;
        
        for id in ids.iter() {
            if let Some(manifest) = self.manifests.get(id) {
                if version.is_none() || &manifest.metadata.version == version.unwrap() {
                    if let Some(plugin) = self.plugins.get(id) {
                        return Some((*id, plugin.clone()));
                    }
                }
            }
        }
        
        None
    }
    
    /// Get plugin manifest
    pub fn get_manifest(&self, plugin_id: Uuid) -> Option<PluginManifest> {
        self.manifests.get(&plugin_id).map(|m| m.clone())
    }
    
    /// Update plugin health
    pub async fn update_health(&self, plugin_id: Uuid, health: PluginHealth) -> Result<(), PluginError> {
        let old_health = self.health_status.get(&plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;
        
        // Check for status change
        if old_health.status != health.status {
            let _ = self.event_tx.send(PluginEvent::HealthChanged {
                plugin_id,
                status: health.status.clone(),
            });
        }
        
        self.health_status.insert(plugin_id, health);
        Ok(())
    }
    
    /// Get all loaded plugins
    pub fn list(&self) -> Vec<(Uuid, PluginMetadata)> {
        self.plugins
            .iter()
            .map(|entry| {
                let id = *entry.key();
                // Note: Can't use async in sync context here
                // This is a simplified version
                (id, PluginMetadata {
                    name: format!("plugin-{}", id),
                    version: crate::PluginVersion::new(0, 0, 0),
                    description: String::new(),
                    author: String::new(),
                    license: String::new(),
                    homepage: None,
                    repository: None,
                    keywords: vec![],
                    categories: vec![],
                })
            })
            .collect()
    }
    
    /// Get plugins by capability
    pub fn find_by_capability(&self, capability_type: &str) -> Vec<Uuid> {
        self.manifests
            .iter()
            .filter(|entry| {
                entry.capabilities.iter().any(|cap| {
                    format!("{:?}", cap).to_lowercase().contains(capability_type)
                })
            })
            .map(|entry| *entry.key())
            .collect()
    }
    
    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_plugins: self.plugins.len(),
            healthy_plugins: self.health_status.iter().filter(|h| h.status == crate::HealthStatus::Healthy).count(),
            degraded_plugins: self.health_status.iter().filter(|h| h.status == crate::HealthStatus::Degraded).count(),
            unhealthy_plugins: self.health_status.iter().filter(|h| h.status == crate::HealthStatus::Unhealthy).count(),
        }
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    pub total_plugins: usize,
    pub healthy_plugins: usize,
    pub degraded_plugins: usize,
    pub unhealthy_plugins: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutorPlugin, ExecutionContext, ExecutionResult};

    struct MockPlugin {
        metadata: PluginMetadata,
    }

    #[async_trait::async_trait]
    impl Plugin for MockPlugin {
        fn metadata(&self) -> PluginMetadata {
            self.metadata.clone()
        }

        async fn init(&mut self, _config: PluginConfig) -> Result<(), PluginError> {
            Ok(())
        }

        async fn shutdown(&mut self) -> Result<(), PluginError> {
            Ok(())
        }

        async fn health(&self) -> PluginHealth {
            PluginHealth {
                status: crate::HealthStatus::Healthy,
                last_check: chrono::Utc::now(),
                message: None,
                metrics: Default::default(),
            }
        }

        fn capabilities(&self) -> Vec<crate::PluginCapability> {
            vec![]
        }
    }

    #[tokio::test]
    async fn test_registry_register() {
        let registry = PluginRegistry::new();
        
        let plugin = MockPlugin {
            metadata: PluginMetadata {
                name: "test-plugin".to_string(),
                version: crate::PluginVersion::new(1, 0, 0),
                description: "Test".to_string(),
                author: "Test".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                repository: None,
                keywords: vec![],
                categories: vec![],
            },
        };
        
        let manifest = PluginManifest::default();
        let id = registry.register(Box::new(plugin), manifest).await.unwrap();
        
        assert!(registry.get(id).is_some());
    }

    #[test]
    fn test_registry_stats() {
        let registry = PluginRegistry::new();
        let stats = registry.stats();
        
        assert_eq!(stats.total_plugins, 0);
        assert_eq!(stats.healthy_plugins, 0);
    }
}