//! Registry client for marketplace API

use crate::{MarketplaceConfig, MarketplaceEntry, MarketplaceError, MarketplaceStats, SearchQuery, SearchResults};
use reqwest::{Client, Method};
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

/// HTTP client for marketplace registry
pub struct RegistryClient {
    http: Client,
    config: MarketplaceConfig,
}

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub base_url: String,
    pub timeout_secs: u64,
    pub retry_count: u32,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            base_url: "https://marketplace.sma-os.io/v1".to_string(),
            timeout_secs: 30,
            retry_count: 3,
        }
    }
}

impl RegistryClient {
    /// Create a new registry client
    pub fn new(config: MarketplaceConfig) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { http, config }
    }
    
    /// Search plugins
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResults, MarketplaceError> {
        let url = format!("{}/plugins/search", self.config.registry_url);
        
        let response = self.http
            .get(&url)
            .query(&[
                ("q", query.query.as_deref().unwrap_or("")),
                ("limit", &query.limit.to_string()),
                ("offset", &query.offset.to_string()),
            ])
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(MarketplaceError::Network(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        let results: SearchResults = response.json().await?;
        Ok(results)
    }
    
    /// Get plugin details
    pub async fn get_plugin(&self, id: Uuid) -> Result<MarketplaceEntry, MarketplaceError> {
        let url = format!("{}/plugins/{}", self.config.registry_url, id);
        
        let response = self.http
            .get(&url)
            .send()
            .await?;
        
        if response.status().as_u16() == 404 {
            return Err(MarketplaceError::NotFound(id.to_string()));
        }
        
        let entry: MarketplaceEntry = response.json().await?;
        Ok(entry)
    }
    
    /// Get plugin download URL
    pub async fn get_download_url(&self, id: Uuid) -> Result<String, MarketplaceError> {
        let url = format!("{}/plugins/{}/download", self.config.registry_url, id);
        
        let response = self.http
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;
        
        let result: serde_json::Value = response.json().await?;
        
        result.get("url")
            .and_then(|u| u.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| MarketplaceError::NotFound(
                "Download URL not found".to_string()
            ))
    }
    
    /// Download plugin
    pub async fn download(&self, id: Uuid, dest: &std::path::Path) -> Result<(), MarketplaceError> {
        let url = self.get_download_url(id).await?;
        
        let response = self.http
            .get(&url)
            .send()
            .await?;
        
        let bytes = response.bytes().await?;
        
        tokio::fs::write(dest, &bytes).await?;
        
        info!("[RegistryClient] Downloaded plugin {} to {}", id, dest.display());
        
        Ok(())
    }
    
    /// Get marketplace stats
    pub async fn stats(&self) -> Result<MarketplaceStats, MarketplaceError> {
        let url = format!("{}/stats", self.config.registry_url);
        
        let response = self.http
            .get(&url)
            .send()
            .await?;
        
        let stats: MarketplaceStats = response.json().await?;
        Ok(stats)
    }
    
    /// Publish plugin (requires authentication)
    pub async fn publish(&self, entry: &MarketplaceEntry) -> Result<(), MarketplaceError> {
        if self.config.api_key.is_none() {
            return Err(MarketplaceError::AuthenticationRequired);
        }
        
        let url = format!("{}/plugins", self.config.registry_url);
        
        let response = self.http
            .post(&url)
            .header("Authorization", format!("Bearer {}", 
                self.config.api_key.as_ref().unwrap()))
            .json(entry)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(MarketplaceError::Network(
                reqwest::Error::from(response.error_for_status().unwrap_err())
            ));
        }
        
        info!("[RegistryClient] Published plugin {}", entry.name);
        
        Ok(())
    }
    
    /// Health check
    pub async fn health(&self) -> Result<bool, MarketplaceError> {
        let url = format!("{}/health", self.config.registry_url);
        
        match self.http.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_config_default() {
        let config = RegistryConfig::default();
        assert_eq!(config.base_url, "https://marketplace.sma-os.io/v1");
        assert_eq!(config.timeout_secs, 30);
    }
}