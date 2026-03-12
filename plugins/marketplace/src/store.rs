//! Marketplace store for plugin packages

use crate::{MarketplaceConfig, MarketplaceEntry, MarketplaceError, SearchQuery, SearchResults};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Plugin package in the store
#[derive(Debug, Clone)]
pub struct PluginPackage {
    pub entry: MarketplaceEntry,
    pub local_path: Option<std::path::PathBuf>,
    pub verified: bool,
}

/// Plugin download info
#[derive(Debug, Clone)]
pub struct PluginDownload {
    pub entry: MarketplaceEntry,
    pub url: String,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Local marketplace store
pub struct MarketplaceStore {
    config: MarketplaceConfig,
    cache_dir: std::path::PathBuf,
    install_dir: std::path::PathBuf,
}

impl MarketplaceStore {
    /// Create a new marketplace store
    pub fn new(config: MarketplaceConfig) -> Result<Self, MarketplaceError> {
        let cache_dir = shellexpand::tilde(&config.cache_dir).into_owned();
        let install_dir = shellexpand::tilde(&config.install_dir).into_owned();

        let cache_path = std::path::PathBuf::from(cache_dir);
        let install_path = std::path::PathBuf::from(install_dir);

        // Create directories
        std::fs::create_dir_all(&cache_path)?;
        std::fs::create_dir_all(&install_path)?;

        Ok(Self {
            config,
            cache_dir: cache_path,
            install_dir: install_path,
        })
    }

    /// Get cache path for a plugin
    fn cache_path(&self, id: Uuid) -> std::path::PathBuf {
        self.cache_dir.join(format!("{}.wasm", id))
    }

    /// Get install path for a plugin
    fn install_path(&self, name: &str, version: &str) -> std::path::PathBuf {
        self.install_dir.join(format!("{}-{}", name, version))
    }

    /// Check if plugin is cached
    pub fn is_cached(&self, id: Uuid) -> bool {
        self.cache_path(id).exists()
    }

    /// Verify plugin checksum
    pub fn verify_checksum(&self, path: &Path, expected: &str) -> Result<bool, MarketplaceError> {
        let bytes = std::fs::read(path)?;
        let hash = Sha256::digest(&bytes);
        let actual = hex::encode(hash);

        Ok(actual == expected)
    }

    /// Install plugin from cache
    pub fn install(
        &self,
        entry: &MarketplaceEntry,
    ) -> Result<std::path::PathBuf, MarketplaceError> {
        let cache_path = self.cache_path(entry.id);
        let install_path = self.install_path(&entry.name, &entry.version);

        if !cache_path.exists() {
            return Err(MarketplaceError::NotFound(format!(
                "Plugin {} not cached",
                entry.name
            )));
        }

        // Verify checksum
        if !self.verify_checksum(&cache_path, &entry.checksum)? {
            return Err(MarketplaceError::ValidationFailed(
                "Checksum mismatch".to_string(),
            ));
        }

        // Create install directory (async)
        tokio::fs::create_dir_all(&install_path).await?;

        // Copy plugin binary (async)
        let dest = install_path.join("plugin.wasm");
        tokio::fs::copy(&cache_path, &dest).await?;

        // Write manifest (async)
        let manifest = install_path.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(entry)?;
        tokio::fs::write(&manifest, manifest_json).await?;

        info!(
            "[Store] Installed {} v{} to {}",
            entry.name,
            entry.version,
            install_path.display()
        );

        Ok(install_path)
    }

    /// Uninstall plugin
    pub fn uninstall(&self, name: &str, version: &str) -> Result<(), MarketplaceError> {
        let install_path = self.install_path(name, version);

        if !install_path.exists() {
            return Err(MarketplaceError::NotFound(format!(
                "Plugin {}@{} not installed",
                name, version
            )));
        }

        std::fs::remove_dir_all(&install_path)?;

        info!("[Store] Uninstalled {}@{}", name, version);

        Ok(())
    }

    /// List installed plugins
    pub fn list_installed(&self) -> Result<Vec<MarketplaceEntry>, MarketplaceError> {
        let mut entries = Vec::new();

        for entry in std::fs::read_dir(&self.install_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                let manifest = path.join("manifest.json");
                if manifest.exists() {
                    let content = std::fs::read_to_string(&manifest)?;
                    let entry: MarketplaceEntry = serde_json::from_str(&content)?;
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    /// Get installed plugin
    pub fn get_installed(
        &self,
        name: &str,
        version: &str,
    ) -> Result<Option<MarketplaceEntry>, MarketplaceError> {
        let install_path = self.install_path(name, version);
        let manifest = install_path.join("manifest.json");

        if manifest.exists() {
            let content = std::fs::read_to_string(&manifest)?;
            let entry: MarketplaceEntry = serde_json::from_str(&content)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Clean cache
    pub fn clean_cache(&self) -> Result<u64, MarketplaceError> {
        let mut cleaned = 0u64;

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;
            cleaned += metadata.len();
            std::fs::remove_file(entry.path())?;
        }

        info!("[Store] Cleaned {} bytes from cache", cleaned);

        Ok(cleaned)
    }

    /// Get store stats
    pub fn stats(&self) -> Result<StoreStats, MarketplaceError> {
        let mut cache_size = 0u64;
        let mut cache_count = 0u64;

        for entry in std::fs::read_dir(&self.cache_dir)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    cache_size += metadata.len();
                    cache_count += 1;
                }
            }
        }

        let installed = self.list_installed()?.len() as u64;

        Ok(StoreStats {
            cache_size,
            cache_count,
            installed_count: installed,
        })
    }
}

/// Store statistics
#[derive(Debug, Clone)]
pub struct StoreStats {
    pub cache_size: u64,
    pub cache_count: u64,
    pub installed_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_store_paths() {
        let config = MarketplaceConfig::default();
        let store = MarketplaceStore::new(config).unwrap();

        let id = Uuid::new_v4();
        let cache_path = store.cache_path(id);
        assert!(cache_path.to_string_lossy().contains(&id.to_string()));
    }
}
