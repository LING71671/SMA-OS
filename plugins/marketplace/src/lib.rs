//! SMA-OS Plugin Marketplace
//!
//! Provides a centralized marketplace for discovering, downloading, and managing plugins.

pub mod store;
pub mod registry_client;
pub mod installer;
pub mod validator;

pub use store::{MarketplaceStore, PluginPackage, PluginDownload};
pub use registry_client::{RegistryClient, RegistryConfig};
pub use installer::{PluginInstaller, InstallOptions};
pub use validator::{PackageValidator, ValidationResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// Primary registry URL
    pub registry_url: String,
    /// Backup registry URLs
    pub backup_registries: Vec<String>,
    /// Plugin installation directory
    pub install_dir: String,
    /// Cache directory
    pub cache_dir: String,
    /// API key for authenticated access
    pub api_key: Option<String>,
    /// Enable telemetry
    pub telemetry_enabled: bool,
}

impl Default for MarketplaceConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://marketplace.sma-os.io/v1".to_string(),
            backup_registries: vec![],
            install_dir: "~/.sma-os/plugins".to_string(),
            cache_dir: "~/.sma-os/cache".to_string(),
            api_key: None,
            telemetry_enabled: true,
        }
    }
}

/// Marketplace entry for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceEntry {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub downloads: u64,
    pub rating: f32,
    pub reviews_count: u32,
    pub tags: Vec<String>,
    pub categories: Vec<String>,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub icon_url: Option<String>,
    pub published_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub supported_versions: Vec<String>,
    pub dependencies: Vec<String>,
    pub size_bytes: u64,
    pub checksum: String,
}

/// Search query for marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub limit: usize,
    pub offset: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            query: None,
            categories: vec![],
            tags: vec![],
            author: None,
            sort_by: SortBy::Downloads,
            sort_order: SortOrder::Desc,
            limit: 20,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    #[serde(rename = "downloads")]
    Downloads,
    #[serde(rename = "rating")]
    Rating,
    #[serde(rename = "updated")]
    Updated,
    #[serde(rename = "name")]
    Name,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    #[serde(rename = "asc")]
    Asc,
    #[serde(rename = "desc")]
    Desc,
}

/// Search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub total: usize,
    pub entries: Vec<MarketplaceEntry>,
    pub has_more: bool,
}

/// Review for a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginReview {
    pub id: Uuid,
    pub plugin_id: Uuid,
    pub author: String,
    pub rating: u8, // 1-5
    pub comment: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Marketplace statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceStats {
    pub total_plugins: u64,
    pub total_downloads: u64,
    pub total_reviews: u64,
    pub top_categories: Vec<(String, u64)>,
}

/// Error types for marketplace
#[derive(thiserror::Error, Debug)]
pub enum MarketplaceError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Version not found: {0}@{1}")]
    VersionNotFound(String, String),
    
    #[error("Installation failed: {0}")]
    InstallationFailed(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Authentication required")]
    AuthenticationRequired,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_config_default() {
        let config = MarketplaceConfig::default();
        assert_eq!(config.registry_url, "https://marketplace.sma-os.io/v1");
        assert!(!config.telemetry_enabled);
    }

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();
        assert_eq!(query.limit, 20);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn test_marketplace_entry_serialization() {
        let entry = MarketplaceEntry {
            id: Uuid::new_v4(),
            name: "test-plugin".to_string(),
            description: "Test plugin".to_string(),
            author: "Test Author".to_string(),
            version: "1.0.0".to_string(),
            downloads: 100,
            rating: 4.5,
            reviews_count: 10,
            tags: vec!["test".to_string()],
            categories: vec!["utility".to_string()],
            license: "MIT".to_string(),
            homepage: None,
            repository: None,
            icon_url: None,
            published_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            supported_versions: vec!["1.0".to_string()],
            dependencies: vec![],
            size_bytes: 1024,
            checksum: "abc123".to_string(),
        };
        
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.is_empty());
    }
}