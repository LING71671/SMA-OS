//! Firecracker REST API Client
//! 
//! This module provides a client for communicating with the Firecracker microVM
//! via its REST API over Unix Domain Sockets.

use std::path::Path;
use serde::{Deserialize, Serialize};
use hyper::{Body, Client, Method, Request, Uri};
use hyper::client::HttpConnector;
use anyhow::{Result, Context};
use tracing::{info, debug, error};

#[cfg(unix)]
use hyperlocal::UnixConnector;

/// Firecracker API error types
#[derive(Debug, thiserror::Error)]
pub enum FirecrackerError {
    #[error("HTTP error: {0}")]
    Http(#[from] hyper::Error),
    
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("VM not found")]
    VmNotFound,
    
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Firecracker REST API Client
pub struct FirecrackerClient {
    #[cfg(unix)]
    client: Client<UnixConnector>,
    #[cfg(not(unix))]
    client: Client<HttpConnector>,
    socket_path: String,
}

impl FirecrackerClient {
    /// Create a new Firecracker client for the given socket path
    pub fn new(socket_path: impl Into<String>) -> Self {
        let socket_path = socket_path.into();
        
        #[cfg(unix)]
        let client = Client::unix();
        #[cfg(not(unix))]
        let client = Client::new();
        
        Self { client, socket_path }
    }
    
    /// Build a URI for the Firecracker API endpoint
    #[cfg(unix)]
    fn build_uri(&self, path: &str) -> Uri {
        format!("unix://{}{}", self.socket_path, path)
            .parse()
            .expect("Valid URI")
    }
    
    #[cfg(not(unix))]
    fn build_uri(&self, path: &str) -> Uri {
        format!("http://localhost{}", path)
            .parse()
            .expect("Valid URI")
    }
    
    /// Send a PUT request with JSON body
    async fn put_json<T: Serialize>(&self, path: &str, body: &T) -> Result<(), FirecrackerError> {
        let uri = self.build_uri(path);
        let json = serde_json::to_string(body)?;
        
        debug!("[Firecracker] PUT {}: {}", path, json);
        
        let req = Request::builder()
            .method(Method::PUT)
            .uri(uri)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(Body::from(json))?;
        
        let resp = self.client.request(req).await?;
        let status = resp.status();
        
        if status.is_success() {
            debug!("[Firecracker] PUT {} succeeded: {}", path, status);
            Ok(())
        } else {
            let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
            let message = String::from_utf8_lossy(&body_bytes).to_string();
            error!("[Firecracker] PUT {} failed: {} - {}", path, status, message);
            Err(FirecrackerError::Api { status: status.as_u16(), message })
        }
    }
    
    /// Send a GET request and parse JSON response
    async fn get_json<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T, FirecrackerError> {
        let uri = self.build_uri(path);
        
        debug!("[Firecracker] GET {}", path);
        
        let req = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .header("Accept", "application/json")
            .body(Body::empty())?;
        
        let resp = self.client.request(req).await?;
        let status = resp.status();
        
        if status.is_success() {
            let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
            let result: T = serde_json::from_slice(&body_bytes)?;
            debug!("[Firecracker] GET {} succeeded", path);
            Ok(result)
        } else {
            let body_bytes = hyper::body::to_bytes(resp.into_body()).await?;
            let message = String::from_utf8_lossy(&body_bytes).to_string();
            error!("[Firecracker] GET {} failed: {} - {}", path, status, message);
            Err(FirecrackerError::Api { status: status.as_u16(), message })
        }
    }
    
    /// Configure the machine
    pub async fn put_machine_config(&self, config: &MachineConfig) -> Result<(), FirecrackerError> {
        self.put_json("/machine-config", config).await
    }
    
    /// Get machine configuration
    pub async fn get_machine_config(&self) -> Result<MachineConfig, FirecrackerError> {
        self.get_json("/machine-config").await
    }
    
    /// Configure the boot source
    pub async fn put_boot_source(&self, source: &BootSource) -> Result<(), FirecrackerError> {
        self.put_json("/boot-source", source).await
    }
    
    /// Configure a drive
    pub async fn put_drive(&self, drive_id: &str, drive: &Drive) -> Result<(), FirecrackerError> {
        let path = format!("/drives/{}", drive_id);
        self.put_json(&path, drive).await
    }
    
    /// Configure a network interface
    pub async fn put_network_interface(&self, iface_id: &str, iface: &NetworkInterface) -> Result<(), FirecrackerError> {
        let path = format!("/network-interfaces/{}", iface_id);
        self.put_json(&path, iface).await
    }
    
    /// Execute an action (Start, Stop, Pause, Resume)
    pub async fn put_action(&self, action: &InstanceAction) -> Result<(), FirecrackerError> {
        self.put_json("/actions", action).await
    }
    
    /// Get metrics
    pub async fn get_metrics(&self) -> Result<Metrics, FirecrackerError> {
        self.get_json("/metrics").await
    }
    
    /// Start the VM instance
    pub async fn start_instance(&self) -> Result<(), FirecrackerError> {
        let action = InstanceAction {
            action_type: ActionType::InstanceStart,
        };
        self.put_action(&action).await
    }
    
    /// Stop the VM instance
    pub async fn stop_instance(&self) -> Result<(), FirecrackerError> {
        let action = InstanceAction {
            action_type: ActionType::SendCtrlAltDel,
        };
        self.put_action(&action).await
    }
    
    /// Pause the VM instance
    pub async fn pause_instance(&self) -> Result<(), FirecrackerError> {
        let action = InstanceAction {
            action_type: ActionType::Pause,
        };
        self.put_action(&action).await
    }
    
    /// Resume the VM instance
    pub async fn resume_instance(&self) -> Result<(), FirecrackerError> {
        let action = InstanceAction {
            action_type: ActionType::Resume,
        };
        self.put_action(&action).await
    }
}

/// Machine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineConfig {
    #[serde(rename = "vcpu_count")]
    pub vcpu_count: u8,
    #[serde(rename = "mem_size_mib")]
    pub memory_size_mib: u32,
    #[serde(rename = "smt", skip_serializing_if = "Option::is_none")]
    pub smt: Option<bool>,
    #[serde(rename = "track_dirty_pages", skip_serializing_if = "Option::is_none")]
    pub track_dirty_pages: Option<bool>,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            vcpu_count: 2,
            memory_size_mib: 512,
            smt: Some(false),
            track_dirty_pages: Some(false),
        }
    }
}

/// Boot source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootSource {
    #[serde(rename = "kernel_image_path")]
    pub kernel_image_path: String,
    #[serde(rename = "initrd_path", skip_serializing_if = "Option::is_none")]
    pub initrd_path: Option<String>,
    #[serde(rename = "boot_args", skip_serializing_if = "Option::is_none")]
    pub boot_args: Option<String>,
}

/// Drive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drive {
    #[serde(rename = "drive_id")]
    pub drive_id: String,
    #[serde(rename = "path_on_host")]
    pub path_on_host: String,
    #[serde(rename = "is_root_device")]
    pub is_root_device: bool,
    #[serde(rename = "is_read_only")]
    pub is_read_only: bool,
    #[serde(rename = "partuuid", skip_serializing_if = "Option::is_none")]
    pub partuuid: Option<String>,
}

/// Network interface configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    #[serde(rename = "iface_id")]
    pub iface_id: String,
    #[serde(rename = "host_dev_name")]
    pub host_dev_name: String,
    #[serde(rename = "guest_mac", skip_serializing_if = "Option::is_none")]
    pub guest_mac: Option<String>,
}

/// Instance action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    #[serde(rename = "InstanceStart")]
    InstanceStart,
    #[serde(rename = "SendCtrlAltDel")]
    SendCtrlAltDel,
    #[serde(rename = "Pause")]
    Pause,
    #[serde(rename = "Resume")]
    Resume,
}

/// Instance action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceAction {
    #[serde(rename = "action_type")]
    pub action_type: ActionType,
}

/// Metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metrics {
    #[serde(rename = "utc_timestamp_ms")]
    pub utc_timestamp_ms: u64,
    #[serde(flatten)]
    pub counters: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_machine_config_serialization() {
        let config = MachineConfig {
            vcpu_count: 4,
            memory_size_mib: 1024,
            smt: Some(true),
            track_dirty_pages: Some(true),
        };
        
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"vcpu_count\":4"));
        assert!(json.contains("\"mem_size_mib\":1024"));
    }
    
    #[test]
    fn test_drive_serialization() {
        let drive = Drive {
            drive_id: "rootfs".to_string(),
            path_on_host: "/path/to/rootfs.ext4".to_string(),
            is_root_device: true,
            is_read_only: false,
            partuuid: None,
        };
        
        let json = serde_json::to_string(&drive).unwrap();
        assert!(json.contains("\"drive_id\":\"rootfs\""));
        assert!(json.contains("\"is_root_device\":true"));
    }
    
    #[test]
    fn test_instance_action_serialization() {
        let action = InstanceAction {
            action_type: ActionType::InstanceStart,
        };
        
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("\"action_type\":\"InstanceStart\""));
    }
}
