use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use thiserror::Error;

/// Errors that can occur during MicroVM lifecycle operations
#[derive(Error, Debug)]
pub enum MicroVMError {
    #[error("VM not found: {0}")]
    VmNotFound(String),
    
    #[error("VM already exists: {0}")]
    VmAlreadyExists(String),
    
    #[error("VM is not running: {0}")]
    VmNotRunning(String),
    
    #[error("VM is already running: {0}")]
    VmAlreadyRunning(String),
    
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),
    
    #[error("Snapshot already exists: {0}")]
    SnapshotAlreadyExists(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimit(String),
}

/// Result type for MicroVM operations
pub type Result<T> = std::result::Result<T, MicroVMError>;

/// VM configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    pub vcpu_count: u8,
    pub memory_mb: u32,
    pub kernel_image_path: String,
    pub rootfs_path: String,
    pub network_namespace: Option<String>,
    pub extra_drives: Vec<DriveConfig>,
}

impl Default for VmConfig {
    fn default() -> Self {
        Self {
            vcpu_count: 2,
            memory_mb: 512,
            kernel_image_path: String::from("/opt/firecracker/vmlinux"),
            rootfs_path: String::from("/opt/firecracker/rootfs.ext4"),
            network_namespace: None,
            extra_drives: Vec::new(),
        }
    }
}

/// Drive configuration for additional block devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub drive_id: String,
    pub path: String,
    pub is_read_only: bool,
}

/// VM state enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmState {
    Creating,
    Configured,
    Running,
    Paused,
    Stopped,
    Destroyed,
}

/// Firecracker MicroVM instance
#[derive(Debug, Clone)]
pub struct FirecrackerVM {
    pub vm_id: String,
    pub socket_path: String,
    pub config: VmConfig,
    pub state: VmState,
    pub pid: Option<u32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl FirecrackerVM {
    /// Create a new FirecrackerVM instance
    pub async fn new(vm_id: String, socket_path: String, config: VmConfig) -> Result<Self> {
        // Validate socket path parent directory exists
        let socket_path_obj = Path::new(&socket_path);
        if let Some(parent) = socket_path_obj.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        
        Ok(Self {
            vm_id,
            socket_path,
            config,
            state: VmState::Creating,
            pid: None,
            created_at: chrono::Utc::now(),
        })
    }
    
    /// Configure the VM via Firecracker REST API
    pub async fn configure(&mut self) -> Result<()> {
        info!("[VM {}] Configuring boot source and drives", self.vm_id);
        
        // Validate kernel image exists
        if !Path::new(&self.config.kernel_image_path).exists() {
            return Err(MicroVMError::Config(
                format!("Kernel image not found: {}", self.config.kernel_image_path)
            ));
        }
        
        // Validate rootfs exists
        if !Path::new(&self.config.rootfs_path).exists() {
            return Err(MicroVMError::Config(
                format!("Rootfs not found: {}", self.config.rootfs_path)
            ));
        }
        
        // Simulate Firecracker API configuration
        info!("[VM {}] Boot source configured: {}", self.vm_id, self.config.kernel_image_path);
        info!("[VM {}] Root drive configured: {}", self.vm_id, self.config.rootfs_path);
        
        for drive in &self.config.extra_drives {
            if !Path::new(&drive.path).exists() {
                return Err(MicroVMError::Config(
                    format!("Extra drive not found: {}", drive.path)
                ));
            }
            info!("[VM {}] Extra drive configured: {} -> {}", 
                self.vm_id, drive.drive_id, drive.path);
        }
        
        self.state = VmState::Configured;
        Ok(())
    }
    
    /// Start the VM instance
    pub async fn start(&mut self) -> Result<()> {
        if self.state == VmState::Running {
            return Err(MicroVMError::VmAlreadyRunning(self.vm_id.clone()));
        }
        
        if self.state != VmState::Configured {
            return Err(MicroVMError::Config(
                format!("VM {} must be configured before starting", self.vm_id)
            ));
        }
        
        info!("[VM {}] Starting instance with {} vCPUs, {}MB RAM", 
            self.vm_id, self.config.vcpu_count, self.config.memory_mb);
        
        // Simulate starting the VM process
        self.pid = Some(rand::random::<u32>() % 10000 + 1000);
        self.state = VmState::Running;
        
        info!("[VM {}] Instance started with PID {:?}", self.vm_id, self.pid);
        Ok(())
    }
    
    /// Stop the VM instance gracefully
    pub async fn stop(&mut self) -> Result<()> {
        if self.state != VmState::Running {
            return Err(MicroVMError::VmNotRunning(self.vm_id.clone()));
        }
        
        info!("[VM {}] Stopping instance", self.vm_id);
        
        // Simulate graceful shutdown
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        self.state = VmState::Stopped;
        self.pid = None;
        
        info!("[VM {}] Instance stopped", self.vm_id);
        Ok(())
    }
    
    /// Pause the VM instance
    pub async fn pause(&mut self) -> Result<()> {
        if self.state != VmState::Running {
            return Err(MicroVMError::VmNotRunning(self.vm_id.clone()));
        }
        
        info!("[VM {}] Pausing instance", self.vm_id);
        self.state = VmState::Paused;
        Ok(())
    }
    
    /// Resume a paused VM instance
    pub async fn resume(&mut self) -> Result<()> {
        if self.state != VmState::Paused {
            return Err(MicroVMError::Config(
                format!("VM {} is not paused", self.vm_id)
            ));
        }
        
        info!("[VM {}] Resuming instance", self.vm_id);
        self.state = VmState::Running;
        Ok(())
    }
    
    /// Get VM resource usage statistics
    pub async fn get_stats(&self) -> Result<VmStats> {
        if self.state != VmState::Running {
            return Ok(VmStats::default());
        }
        
        // Simulate resource usage
        Ok(VmStats {
            cpu_usage_percent: 15.5,
            memory_usage_mb: self.config.memory_mb / 4,
            disk_read_bytes: 1024 * 1024 * 50,
            disk_write_bytes: 1024 * 1024 * 10,
            network_rx_bytes: 1024 * 1024 * 5,
            network_tx_bytes: 1024 * 1024 * 2,
        })
    }
}

/// VM statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VmStats {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u32,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub snapshot_id: String,
    pub vm_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub path: String,
    pub size_bytes: u64,
    pub vm_config: VmConfig,
}

/// Manager for MicroVM lifecycle operations
pub struct MicroVMManager {
    vms: Arc<RwLock<HashMap<String, FirecrackerVM>>>,
    snapshots: Arc<RwLock<HashMap<String, Snapshot>>>,
    max_vms: usize,
    snapshot_dir: String,
}

impl MicroVMManager {
    /// Create a new MicroVMManager
    pub fn new(max_vms: usize, snapshot_dir: String) -> Self {
        Self {
            vms: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
            max_vms,
            snapshot_dir,
        }
    }
    
    /// Create a new VM with the given configuration
    pub async fn create(&self, config: VmConfig) -> Result<String> {
        let vm_id = format!("vm-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        
        let vms = self.vms.read().await;
        if vms.len() >= self.max_vms {
            return Err(MicroVMError::ResourceLimit(
                format!("Maximum VM limit reached: {}", self.max_vms)
            ));
        }
        drop(vms);
        
        let socket_path = format!("/tmp/firecracker-{}.socket", vm_id);
        let mut vm = FirecrackerVM::new(vm_id.clone(), socket_path, config).await?;
        vm.configure().await?;
        
        let mut vms = self.vms.write().await;
        vms.insert(vm_id.clone(), vm);
        
        info!("[Manager] Created VM: {}", vm_id);
        Ok(vm_id)
    }
    
    /// Start a VM by ID
    pub async fn start(&self, id: &str) -> Result<()> {
        let mut vms = self.vms.write().await;
        let vm = vms.get_mut(id)
            .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))?;
        
        vm.start().await?;
        Ok(())
    }
    
    /// Stop a VM by ID
    pub async fn stop(&self, id: &str) -> Result<()> {
        let mut vms = self.vms.write().await;
        let vm = vms.get_mut(id)
            .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))?;
        
        vm.stop().await?;
        Ok(())
    }
    
    /// Destroy a VM by ID
    pub async fn destroy(&self, id: &str) -> Result<()> {
        // Check if VM exists and get socket path
        let socket_path = {
            let vms = self.vms.read().await;
            let vm = vms.get(id)
                .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))?;
            
            // Stop if running
            if vm.state == VmState::Running {
                drop(vms); // Release read lock before async call
                self.stop(id).await?;
            }
            
            // Get socket path
            let path = vm.socket_path.clone();
            path
        };

        // Remove VM from map
        let mut vms = self.vms.write().await;
        vms.remove(id);
        drop(vms);

        // Remove socket file
        if Path::new(&socket_path).exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        info!("[Manager] Destroyed VM: {}", id);
        Ok(())
    }
    
    /// Create a snapshot of a VM
    pub async fn snapshot(&self, id: &str) -> Result<String> {
        let vms = self.vms.read().await;
        let vm = vms.get(id)
            .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))?;
        
        if vm.state != VmState::Running {
            return Err(MicroVMError::VmNotRunning(id.to_string()));
        }
        
        let snapshot_id = format!("snap-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        let snapshot_path = format!("{}/{}.snap", self.snapshot_dir, snapshot_id);
        
        // Ensure snapshot directory exists
        let snapshot_dir_path = Path::new(&self.snapshot_dir);
        if !snapshot_dir_path.exists() {
            tokio::fs::create_dir_all(snapshot_dir_path).await?;
        }
        
        let snapshot = Snapshot {
            snapshot_id: snapshot_id.clone(),
            vm_id: id.to_string(),
            created_at: chrono::Utc::now(),
            path: snapshot_path,
            size_bytes: 1024 * 1024 * 100, // Simulate 100MB snapshot
            vm_config: vm.config.clone(),
        };
        
        drop(vms);
        
        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(snapshot_id.clone(), snapshot);
        
        info!("[Manager] Created snapshot {} for VM {}", snapshot_id, id);
        Ok(snapshot_id)
    }
    
    /// Restore a VM from a snapshot
    pub async fn restore(&self, snapshot_id: &str) -> Result<String> {
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots.get(snapshot_id)
            .ok_or_else(|| MicroVMError::SnapshotNotFound(snapshot_id.to_string()))?;
        
        let vm_id = format!("vm-{}", Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        let socket_path = format!("/tmp/firecracker-{}.socket", vm_id);
        
        drop(snapshots);
        
        // Check resource limits
        let vms = self.vms.read().await;
        if vms.len() >= self.max_vms {
            return Err(MicroVMError::ResourceLimit(
                format!("Maximum VM limit reached: {}", self.max_vms)
            ));
        }
        drop(vms);
        
        // Create VM from snapshot config
        let snapshots = self.snapshots.read().await;
        let snapshot = snapshots.get(snapshot_id)
            .ok_or_else(|| MicroVMError::SnapshotNotFound(snapshot_id.to_string()))?;
        let config = snapshot.vm_config.clone();
        drop(snapshots);
        
        let mut vm = FirecrackerVM::new(vm_id.clone(), socket_path, config).await?;
        vm.configure().await?;
        vm.start().await?;
        
        let mut vms = self.vms.write().await;
        vms.insert(vm_id.clone(), vm);
        
        info!("[Manager] Restored VM {} from snapshot {}", vm_id, snapshot_id);
        Ok(vm_id)
    }
    
    /// Get VM information
    pub async fn get_vm(&self, id: &str) -> Result<FirecrackerVM> {
        let vms = self.vms.read().await;
        vms.get(id)
            .cloned()
            .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))
    }
    
    /// List all VMs
    pub async fn list_vms(&self) -> Vec<FirecrackerVM> {
        let vms = self.vms.read().await;
        vms.values().cloned().collect()
    }
    
    /// Get VM state
    pub async fn get_vm_state(&self, id: &str) -> Result<VmState> {
        let vms = self.vms.read().await;
        vms.get(id)
            .map(|vm| vm.state)
            .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))
    }
    
    /// Get VM stats
    pub async fn get_vm_stats(&self, id: &str) -> Result<VmStats> {
        let vms = self.vms.read().await;
        let vm = vms.get(id)
            .ok_or_else(|| MicroVMError::VmNotFound(id.to_string()))?;
        
        vm.get_stats().await
    }
    
    /// List all snapshots
    pub async fn list_snapshots(&self) -> Vec<Snapshot> {
        let snapshots = self.snapshots.read().await;
        snapshots.values().cloned().collect()
    }
    
    /// Delete a snapshot
    pub async fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let mut snapshots = self.snapshots.write().await;
        let snapshot = snapshots.get(snapshot_id)
            .ok_or_else(|| MicroVMError::SnapshotNotFound(snapshot_id.to_string()))?;
        
        // Remove snapshot file
        let path = snapshot.path.clone();
        snapshots.remove(snapshot_id);
        drop(snapshots);
        
        if Path::new(&path).exists() {
            tokio::fs::remove_file(&path).await?;
        }
        
        info!("[Manager] Deleted snapshot: {}", snapshot_id);
        Ok(())
    }
    
    /// Get total resource usage across all VMs
    pub async fn get_total_resources(&self) -> ResourceUsage {
        let vms = self.vms.read().await;
        let mut total_vcpus = 0u64;
        let mut total_memory_mb = 0u64;
        let mut running_count = 0usize;
        
        for vm in vms.values() {
            total_vcpus += vm.config.vcpu_count as u64;
            total_memory_mb += vm.config.memory_mb as u64;
            if vm.state == VmState::Running {
                running_count += 1;
            }
        }
        
        ResourceUsage {
            total_vms: vms.len(),
            running_vms: running_count,
            total_vcpus,
            total_memory_mb,
            max_vms: self.max_vms,
        }
    }
}

/// Resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub total_vms: usize,
    pub running_vms: usize,
    pub total_vcpus: u64,
    pub total_memory_mb: u64,
    pub max_vms: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;
    
    #[test]
    async fn test_microvm_manager_create() {
        let manager = MicroVMManager::new(10, "/tmp/snapshots".to_string());
        let config = VmConfig::default();
        
        let vm_id = manager.create(config).await.unwrap();
        assert!(!vm_id.is_empty());
        
        let vm = manager.get_vm(&vm_id).await.unwrap();
        assert_eq!(vm.vm_id, vm_id);
        assert_eq!(vm.state, VmState::Configured);
    }
    
    #[test]
    async fn test_microvm_manager_lifecycle() {
        let manager = MicroVMManager::new(10, "/tmp/snapshots".to_string());
        let config = VmConfig::default();
        
        // Create VM
        let vm_id = manager.create(config).await.unwrap();
        
        // Start VM
        manager.start(&vm_id).await.unwrap();
        let state = manager.get_vm_state(&vm_id).await.unwrap();
        assert_eq!(state, VmState::Running);
        
        // Stop VM
        manager.stop(&vm_id).await.unwrap();
        let state = manager.get_vm_state(&vm_id).await.unwrap();
        assert_eq!(state, VmState::Stopped);
        
        // Destroy VM
        manager.destroy(&vm_id).await.unwrap();
        let result = manager.get_vm(&vm_id).await;
        assert!(result.is_err());
    }
    
    #[test]
    async fn test_microvm_manager_snapshot() {
        let manager = MicroVMManager::new(10, "/tmp/snapshots".to_string());
        let config = VmConfig::default();
        
        // Create and start VM
        let vm_id = manager.create(config).await.unwrap();
        manager.start(&vm_id).await.unwrap();
        
        // Create snapshot
        let snapshot_id = manager.snapshot(&vm_id).await.unwrap();
        assert!(!snapshot_id.is_empty());
        
        // List snapshots
        let snapshots = manager.list_snapshots().await;
        assert_eq!(snapshots.len(), 1);
        
        // Restore from snapshot
        let restored_vm_id = manager.restore(&snapshot_id).await.unwrap();
        assert!(!restored_vm_id.is_empty());
        
        let state = manager.get_vm_state(&restored_vm_id).await.unwrap();
        assert_eq!(state, VmState::Running);
    }
    
    #[test]
    async fn test_microvm_manager_resource_limits() {
        let manager = MicroVMManager::new(2, "/tmp/snapshots".to_string());
        let config = VmConfig::default();
        
        // Create VMs up to limit
        let _ = manager.create(config.clone()).await.unwrap();
        let _ = manager.create(config.clone()).await.unwrap();
        
        // Should fail at limit
        let result = manager.create(config).await;
        assert!(result.is_err());
        
        match result {
            Err(MicroVMError::ResourceLimit(_)) => (),
            _ => panic!("Expected ResourceLimit error"),
        }
    }
    
    #[test]
    async fn test_vm_not_found() {
        let manager = MicroVMManager::new(10, "/tmp/snapshots".to_string());
        
        let result = manager.get_vm("non-existent").await;
        assert!(result.is_err());
        
        match result {
            Err(MicroVMError::VmNotFound(_)) => (),
            _ => panic!("Expected VmNotFound error"),
        }
    }
}
