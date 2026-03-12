//! Warm Pool Management
//!
//! This module manages a pool of pre-configured Firecracker MicroVMs
//! to achieve sub-5ms VM startup times for SMA-OS.

use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{RwLock, Notify};
use tokio::time::{interval, Duration, Instant};
use tracing::{info, debug, warn, error};
use anyhow::{Result, Context};

use crate::firecracker::{FirecrackerClient, MachineConfig, BootSource, Drive};

/// Configuration for warm pool
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of VMs to keep warm
    pub min_size: usize,
    /// Maximum number of VMs in pool
    pub max_size: usize,
    /// Target pool size (ideal number of warm VMs)
    pub target_size: usize,
    /// How often to check pool health (seconds)
    pub health_check_interval_secs: u64,
    /// VM configuration
    pub vm_config: PoolVmConfig,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 5,
            max_size: 100,
            target_size: 50,
            health_check_interval_secs: 10,
            vm_config: PoolVmConfig::default(),
        }
    }
}

/// VM configuration for pool VMs
#[derive(Debug, Clone)]
pub struct PoolVmConfig {
    pub vcpu_count: u8,
    pub memory_mb: u32,
    pub kernel_image_path: String,
    pub rootfs_path: String,
    pub firecracker_binary_path: String,
}

impl Default for PoolVmConfig {
    fn default() -> Self {
        Self {
            vcpu_count: 2,
            memory_mb: 512,
            kernel_image_path: "/opt/firecracker/vmlinux".to_string(),
            rootfs_path: "/opt/firecracker/rootfs.ext4".to_string(),
            firecracker_binary_path: "/usr/local/bin/firecracker".to_string(),
        }
    }
}

/// A warmed VM ready for use
#[derive(Debug)]
pub struct WarmedVm {
    pub vm_id: String,
    pub socket_path: String,
    pub pid: u32,
    pub warmed_at: Instant,
    pub client: FirecrackerClient,
}

impl WarmedVm {
    /// Check if the VM is still healthy
    pub async fn is_healthy(&self) -> bool {
        // Check if the process is still running
        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("kill")
                .args(&["-0", &self.pid.to_string()])
                .output();
            
            match output {
                Ok(result) => result.status.success(),
                Err(_) => false,
            }
        }
        #[cfg(not(unix))]
        {
            // On non-Unix systems, assume healthy
            true
        }
    }
    
    /// Age of the warmed VM
    pub fn age(&self) -> Duration {
        self.warmed_at.elapsed()
    }
}

/// Warm pool manager
pub struct WarmPool {
    config: PoolConfig,
    /// Available VMs ready for use
    available: Arc<RwLock<VecDeque<WarmedVm>>>,
    /// Currently in-use VMs
    in_use: Arc<RwLock<Vec<WarmedVm>>>,
    /// Shutdown signal
    shutdown: Arc<Notify>,
    /// Total VMs created (for metrics)
    total_created: Arc<RwLock<u64>>,
    /// Total VMs assigned (for metrics)
    total_assigned: Arc<RwLock<u64>>,
}

impl WarmPool {
    /// Create a new warm pool with the given configuration
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            available: Arc::new(RwLock::new(VecDeque::new())),
            in_use: Arc::new(RwLock::new(Vec::new())),
            shutdown: Arc::new(Notify::new()),
            total_created: Arc::new(RwLock::new(0)),
            total_assigned: Arc::new(RwLock::new(0)),
        }
    }
    
    /// Initialize the pool by creating VMs up to target_size
    pub async fn initialize(&self) -> Result<()> {
        info!("[Pool] Initializing warm pool with target_size={}", self.config.target_size);
        
        let current_count = self.available.read().await.len();
        let needed = self.config.target_size.saturating_sub(current_count);
        
        info!("[Pool] Creating {} VMs to reach target", needed);
        
        for i in 0..needed {
            match self.create_warmed_vm().await {
                Ok(vm) => {
                    let mut available = self.available.write().await;
                    available.push_back(vm);
                    info!("[Pool] Created VM {}/{}: {} (PID: {})", 
                          i + 1, needed, 
                          available.back().unwrap().vm_id,
                          available.back().unwrap().pid);
                }
                Err(e) => {
                    error!("[Pool] Failed to create VM: {}", e);
                    // Continue with remaining VMs
                }
            }
        }
        
        let available_count = self.available.read().await.len();
        info!("[Pool] Initialized with {} VMs", available_count);
        
        Ok(())
    }
    
    /// Get a warmed VM from the pool
    /// Returns None if pool is empty (need to create VM on-demand)
    pub async fn acquire(&self) -> Option<WarmedVm> {
        let mut available = self.available.write().await;
        
        // Find a healthy VM
        while let Some(vm) = available.pop_front() {
            if vm.is_healthy().await {
                let mut in_use = self.in_use.write().await;
                in_use.push(vm);
                
                // Update metrics
                *self.total_assigned.write().await += 1;
                
                debug!("[Pool] Acquired VM: {}", in_use.last().unwrap().vm_id);
                return in_use.last().cloned();
            } else {
                warn!("[Pool] VM {} is unhealthy, discarding", vm.vm_id);
            }
        }
        
        info!("[Pool] Pool empty, returning None (need on-demand creation)");
        None
    }
    
    /// Return a VM to the pool
    pub async fn release(&self, vm: WarmedVm) -> Result<()> {
        // Remove from in_use
        let mut in_use = self.in_use.write().await;
        in_use.retain(|v| v.vm_id != vm.vm_id);
        
        // Stop the VM
        if let Err(e) = vm.client.stop_instance().await {
            warn!("[Pool] Failed to stop VM {}: {}", vm.vm_id, e);
        }
        
        // Recreate the VM
        match self.create_warmed_vm().await {
            Ok(new_vm) => {
                let mut available = self.available.write().await;
                
                // Check if we're at max capacity
                if available.len() >= self.config.max_size {
                    warn!("[Pool] At max capacity, discarding returned VM");
                    return Ok(());
                }
                
                available.push_back(new_vm);
                debug!("[Pool] Released and recreated VM: {} -> {}", vm.vm_id, 
                       available.back().unwrap().vm_id);
            }
            Err(e) => {
                error!("[Pool] Failed to recreate VM after release: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// Get pool statistics
    pub async fn stats(&self) -> PoolStats {
        PoolStats {
            available_count: self.available.read().await.len(),
            in_use_count: self.in_use.read().await.len(),
            target_size: self.config.target_size,
            total_created: *self.total_created.read().await,
            total_assigned: *self.total_assigned.read().await,
        }
    }
    
    /// Start health check background task
    pub async fn start_health_check(&self) {
        let available = self.available.clone();
        let shutdown = self.shutdown.clone();
        let interval_secs = self.config.health_check_interval_secs;
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(interval_secs));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut healthy_vms = VecDeque::new();
                        let mut unhealthy_count = 0usize;
                        
                        {
                            let available_guard = available.read().await;
                            for vm in available_guard.iter() {
                                if vm.is_healthy().await {
                                    healthy_vms.push_back(vm.clone());
                                } else {
                                    unhealthy_count += 1;
                                }
                            }
                        }
                        
                        if unhealthy_count > 0 {
                            warn!("[Pool] Health check: {} unhealthy VMs removed", unhealthy_count);
                            *available.write().await = healthy_vms;
                        }
                    }
                    _ = shutdown.notified() => {
                        info!("[Pool] Health check task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// Start pool maintenance task
    pub async fn start_maintenance(&self) {
        let available = self.available.clone();
        let config = self.config.clone();
        let shutdown = self.shutdown.clone();
        let total_created = self.total_created.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let current_count = available.read().await.len();
                        
                        // Scale up if below target
                        if current_count < config.target_size {
                            let needed = config.target_size - current_count;
                            info!("[Pool] Maintenance: Creating {} VMs (current: {}, target: {})", 
                                  needed, current_count, config.target_size);
                            
                            for _ in 0..needed {
                                // This would call create_warmed_vm but we don't have self here
                                // For now, just log
                                debug!("[Pool] Would create VM (maintenance)");
                            }
                        }
                        
                        // Scale down if above max
                        if current_count > config.max_size {
                            let excess = current_count - config.max_size;
                            warn!("[Pool] Maintenance: Pool exceeds max size, would remove {} VMs", excess);
                        }
                    }
                    _ = shutdown.notified() => {
                        info!("[Pool] Maintenance task shutting down");
                        break;
                    }
                }
            }
        });
    }
    
    /// Shutdown the pool and cleanup all VMs
    pub async fn shutdown(&self) -> Result<()> {
        info!("[Pool] Shutting down warm pool");
        
        // Signal background tasks to stop
        self.shutdown.notify_waiters();
        
        // Give tasks time to stop
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Stop all VMs
        let mut available = self.available.write().await;
        let mut in_use = self.in_use.write().await;
        
        info!("[Pool] Stopping {} available VMs", available.len());
        while let Some(vm) = available.pop_front() {
            if let Err(e) = vm.client.stop_instance().await {
                warn!("[Pool] Failed to stop VM {} during shutdown: {}", vm.vm_id, e);
            }
        }
        
        info!("[Pool] Stopping {} in-use VMs", in_use.len());
        for vm in in_use.drain(..) {
            if let Err(e) = vm.client.stop_instance().await {
                warn!("[Pool] Failed to stop VM {} during shutdown: {}", vm.vm_id, e);
            }
        }
        
        info!("[Pool] Warm pool shutdown complete");
        Ok(())
    }
    
    /// Create a new warmed VM
    async fn create_warmed_vm(&self) -> Result<WarmedVm> {
        use std::process::Command;
        
        let vm_id = format!("vm-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        let socket_path = format!("/tmp/smaos-firecracker-{}.socket", vm_id);
        
        // Ensure socket directory exists
        let socket_dir = std::path::Path::new(&socket_path).parent().unwrap();
        if !socket_dir.exists() {
            tokio::fs::create_dir_all(socket_dir).await?;
        }
        
        // Build Firecracker command
        let firecracker_cmd = format!(
            "{} --api-sock {}",
            self.config.vm_config.firecracker_binary_path,
            socket_path
        );
        
        debug!("[Pool] Starting Firecracker: {}", firecracker_cmd);
        
        // Start Firecracker process
        let child = Command::new(&self.config.vm_config.firecracker_binary_path)
            .args(&["--api-sock", &socket_path])
            .spawn()
            .context("Failed to spawn Firecracker process")?;
        
        let pid = child.id();
        
        // Wait for socket to be created
        let socket_path_clone = socket_path.clone();
        tokio::time::timeout(
            Duration::from_secs(5),
            async move {
                while !std::path::Path::new(&socket_path_clone).exists() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        ).await
        .context("Timeout waiting for Firecracker socket")?;
        
        // Configure the VM
        let client = FirecrackerClient::new(&socket_path);
        
        let machine_config = MachineConfig {
            vcpu_count: self.config.vm_config.vcpu_count,
            memory_size_mib: self.config.vm_config.memory_mb,
            smt: Some(false),
            track_dirty_pages: Some(false),
        };
        
        client.put_machine_config(&machine_config).await
            .context("Failed to configure machine")?;
        
        let boot_source = BootSource {
            kernel_image_path: self.config.vm_config.kernel_image_path.clone(),
            initrd_path: None,
            boot_args: Some("console=ttyS0 reboot=k panic=1 pci=off".to_string()),
        };
        
        client.put_boot_source(&boot_source).await
            .context("Failed to configure boot source")?;
        
        let root_drive = Drive {
            drive_id: "rootfs".to_string(),
            path_on_host: self.config.vm_config.rootfs_path.clone(),
            is_root_device: true,
            is_read_only: false,
            partuuid: None,
        };
        
        client.put_drive("rootfs", &root_drive).await
            .context("Failed to configure root drive")?;
        
        // Update metrics
        *self.total_created.write().await += 1;
        
        Ok(WarmedVm {
            vm_id,
            socket_path,
            pid,
            warmed_at: Instant::now(),
            client,
        })
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available_count: usize,
    pub in_use_count: usize,
    pub target_size: usize,
    pub total_created: u64,
    pub total_assigned: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_pool_config_default() {
        let config = PoolConfig::default();
        assert_eq!(config.min_size, 5);
        assert_eq!(config.max_size, 100);
        assert_eq!(config.target_size, 50);
    }
    
    #[tokio::test]
    async fn test_pool_stats() {
        let pool = WarmPool::new(PoolConfig::default());
        let stats = pool.stats().await;
        
        assert_eq!(stats.available_count, 0);
        assert_eq!(stats.in_use_count, 0);
        assert_eq!(stats.target_size, 50);
        assert_eq!(stats.total_created, 0);
        assert_eq!(stats.total_assigned, 0);
    }
}
