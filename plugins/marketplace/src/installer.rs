//! Plugin installer

use crate::{MarketplaceConfig, MarketplaceEntry, MarketplaceError};
use std::path::Path;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Plugin installation options
#[derive(Debug, Clone)]
pub struct InstallOptions {
    pub force: bool,
    pub skip_verify: bool,
    pub skip_deps: bool,
}

impl Default for InstallOptions {
    fn default() -> Self {
        Self {
            force: false,
            skip_verify: false,
            skip_deps: false,
        }
    }
}

/// Plugin installer
pub struct PluginInstaller {
    config: MarketplaceConfig,
}

impl PluginInstaller {
    /// Create new installer
    pub fn new(config: MarketplaceConfig) -> Self {
        Self { config }
    }
    
    /// Install plugin
    pub async fn install(&self, entry: &MarketplaceEntry, opts: InstallOptions) -> Result<(), MarketplaceError> {
        info!("[Installer] Installing {} v{}", entry.name, entry.version);
        
        // Check dependencies
        if !opts.skip_deps {
            self.check_dependencies(entry).await?;
        }
        
        // Download plugin
        let download_path = self.download_path(entry.id);
        self.download(entry, &download_path).await?;
        
        // Verify checksum
        if !opts.skip_verify {
            self.verify_checksum(&download_path, &entry.checksum)?;
        }
        
        // Install to final location
        let install_path = self.install_path(&entry.name, &entry.version);
        self.install_files(&download_path, &install_path).await?;
        
        info!("[Installer] Successfully installed {} v{}", entry.name, entry.version);
        
        Ok(())
    }
    
    /// Check dependencies
    async fn check_dependencies(&self, entry: &MarketplaceEntry) -> Result<(), MarketplaceError> {
        for dep in &entry.dependencies {
            info!("[Installer] Checking dependency: {}", dep);
            // TODO: Check if dependency is installed
        }
        Ok(())
    }
    
    /// Download plugin
    async fn download(&self, entry: &MarketplaceEntry, dest: &Path) -> Result<(), MarketplaceError> {
        use reqwest::Client;
        
        let client = Client::new();
        let url = format!("{}/plugins/{}/download", self.config.registry_url, entry.id);
        
        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| MarketplaceError::Network(e))?;
        
        let bytes = response.bytes().await
            .map_err(|e| MarketplaceError::Network(e))?;
        
        tokio::fs::write(dest, &bytes).await?;
        
        Ok(())
    }
    
    /// Verify checksum
    fn verify_checksum(&self, path: &Path, expected: &str) -> Result<(), MarketplaceError> {
        use sha2::{Digest, Sha256};
        
        let bytes = std::fs::read(path)?;
        let hash = Sha256::digest(&bytes);
        let actual = hex::encode(hash);
        
        if actual != expected {
            return Err(MarketplaceError::ValidationFailed(
                format!("Checksum mismatch: expected {}, got {}", expected, actual)
            ));
        }
        
        Ok(())
    }
    
    /// Install files to destination
    async fn install_files(&self, source: &Path, dest: &Path) -> Result<(), MarketplaceError> {
        tokio::fs::create_dir_all(dest).await?;
        
        // Copy plugin file
        let dest_file = dest.join("plugin.wasm");
        tokio::fs::copy(source, dest_file).await?;
        
        Ok(())
    }
    
    /// Get download path
    fn download_path(&self, id: Uuid) -> std::path::PathBuf {
        let cache_dir = shellexpand::tilde(&self.config.cache_dir).into_owned();
        std::path::PathBuf::from(cache_dir).join(format!("{}.wasm", id))
    }
    
    /// Get install path
    fn install_path(&self, name: &str, version: &str) -> std::path::PathBuf {
        let install_dir = shellexpand::tilde(&self.config.install_dir).into_owned();
        std::path::PathBuf::from(install_dir).join(format!("{}-{}", name, version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}