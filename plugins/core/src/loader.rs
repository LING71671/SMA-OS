//! Dynamic plugin loader
//!
//! Supports loading plugins from WASM modules, shared libraries, and Docker containers.

use crate::{Plugin, PluginConfig, PluginError, PluginHealth, PluginManifest, PluginMetadata, PluginCapability};
use async_trait::async_trait;
use std::path::Path;
use tracing::{error, info, warn};

/// Plugin loader for different runtime types
pub struct PluginLoader {
    /// Plugin storage directory
    plugin_dir: std::path::PathBuf,
    /// WASM runtime (if available)
    wasm_runtime: Option<WasmRuntime>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new(plugin_dir: impl AsRef<Path>) -> Self {
        Self {
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
            wasm_runtime: None, // TODO: Initialize WASM runtime
        }
    }
    
    /// Load a plugin from a path
    pub async fn load(&self, path: impl AsRef<Path>) -> Result<Box<dyn Plugin>, PluginError> {
        let path = path.as_ref();
        
        // Detect plugin type from extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match extension {
            "wasm" | "wasm32" => self.load_wasm(path).await,
            "so" | "dll" | "dylib" => self.load_native(path).await,
            _ => {
                // Try to detect from manifest
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    self.load_from_manifest(&manifest_path).await
                } else {
                    Err(PluginError::InvalidManifest(
                        format!("Cannot detect plugin type for: {}", path.display())
                    ))
                }
            }
        }
    }
    
    /// Load WASM plugin
    /// 
    /// NOTE: WASM runtime integration is planned for v1.1.
    /// Currently returns a stub implementation for testing.
    async fn load_wasm(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        info!("[PluginLoader] Loading WASM plugin from {}", path.display());
        
        // Verify file exists and is readable
        if !path.exists() {
            return Err(PluginError::NotFound(format!(
                "WASM file not found: {}",
                path.display()
            )));
        }
        
        // Check file magic number for WASM
        let magic = std::fs::read(path)?;
        if magic.len() < 4 || &magic[0..4] != &[0x00, 0x61, 0x73, 0x6d] {
            return Err(PluginError::InvalidManifest(
                "File is not a valid WASM module".to_string()
            ));
        }
        
        // TODO: Implement WASM loading with wasmtime for v1.1
        // See: https://github.com/LING71671/SMA-OS/issues/5
        warn!("[PluginLoader] WASM runtime not yet implemented, returning stub");
        
        // Return stub implementation for now
        Ok(Box::new(crate::executor::DefaultExecutor::new()))
    }
    
    /// Load native shared library plugin
    async fn load_native(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        info!("[PluginLoader] Loading native plugin from {}", path.display());
        
        // TODO: Implement dynamic library loading with dlopen/libloading
        Err(PluginError::ExecutionFailed("Native plugin loading not yet implemented".to_string()))
    }
    
    /// Load from manifest
    async fn load_from_manifest(&self, path: &Path) -> Result<Box<dyn Plugin>, PluginError> {
        let manifest = PluginManifest::from_file(path)?;
        let plugin_dir = path.parent()
            .ok_or_else(|| PluginError::InvalidManifest("Manifest has no parent directory".to_string()))?;
        
        // Find the plugin binary
        let binary_path = plugin_dir.join("plugin.wasm");
        if binary_path.exists() {
            return self.load_wasm(&binary_path).await;
        }
        
        let binary_path = plugin_dir.join("plugin.so");
        if binary_path.exists() {
            return self.load_native(&binary_path).await;
        }
        
        Err(PluginError::NotFound(format!(
            "No plugin binary found in {}",
            plugin_dir.display()
        )))
    }
    
    /// Validate plugin before loading
    pub fn validate(&self, path: &Path) -> Result<PluginManifest, PluginError> {
        let manifest_path = if path.is_dir() {
            path.join("manifest.json")
        } else {
            path.to_path_buf()
        };
        
        if !manifest_path.exists() {
            return Err(PluginError::InvalidManifest(
                format!("Manifest not found: {}", manifest_path.display())
            ));
        }
        
        PluginManifest::from_file(&manifest_path)
    }
    
    /// Scan directory for available plugins
    pub fn scan(&self) -> Vec<std::path::PathBuf> {
        let mut plugins = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let manifest = path.join("manifest.json");
                    if manifest.exists() {
                        plugins.push(manifest);
                    }
                }
            }
        }
        
        plugins
    }
}

/// WASM runtime wrapper
struct WasmRuntime {
    // TODO: Add wasmtime::Engine
}

/// Stub plugin implementation for testing
pub struct StubPlugin {
    metadata: PluginMetadata,
}

impl StubPlugin {
    pub fn new(metadata: PluginMetadata) -> Self {
        Self { metadata }
    }
}

#[async_trait]
impl Plugin for StubPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }
    
    async fn init(&mut self, _config: PluginConfig) -> Result<(), PluginError> {
        info!("[StubPlugin] Initialized");
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<(), PluginError> {
        info!("[StubPlugin] Shutdown");
        Ok(())
    }
    
    async fn health(&self) -> PluginHealth {
        PluginHealth {
            status: crate::HealthStatus::Healthy,
            last_check: chrono::Utc::now(),
            message: Some("Stub plugin healthy".to_string()),
            metrics: Default::default(),
        }
    }
    
    fn capabilities(&self) -> Vec<PluginCapability> {
        vec![PluginCapability::Executor {
            runtime: crate::ExecutorRuntime::Wasm,
            config: Default::default(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_new() {
        let loader = PluginLoader::new("/tmp/plugins");
        assert!(loader.scan().is_empty()); // Empty directory
    }

    #[test]
    fn test_validate_nonexistent() {
        let loader = PluginLoader::new("/tmp/plugins");
        let result = loader.validate(Path::new("/nonexistent"));
        assert!(result.is_err());
    }
}