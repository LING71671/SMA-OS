use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{info, warn, error, debug};

use crate::microvm::{MicroVMManager, VmState};

/// Health status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// System resource information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_usage_percent: f64,
    pub memory_total_mb: u64,
    pub memory_used_mb: u64,
    pub memory_free_mb: u64,
    pub memory_usage_percent: f64,
    pub disk_total_gb: u64,
    pub disk_used_gb: u64,
    pub disk_free_gb: u64,
    pub disk_usage_percent: f64,
}

impl Default for SystemResources {
    fn default() -> Self {
        Self {
            cpu_usage_percent: 0.0,
            memory_total_mb: 0,
            memory_used_mb: 0,
            memory_free_mb: 0,
            memory_usage_percent: 0.0,
            disk_total_gb: 0,
            disk_used_gb: 0,
            disk_free_gb: 0,
            disk_usage_percent: 0.0,
        }
    }
}

/// VM health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmHealth {
    pub vm_id: String,
    pub state: String,
    pub is_responsive: bool,
    pub last_heartbeat: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u32,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: String,
    pub uptime_seconds: u64,
    pub version: String,
    pub system_resources: SystemResources,
    pub vm_summary: VmHealthSummary,
}

/// VM health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmHealthSummary {
    pub total_vms: usize,
    pub running_vms: usize,
    pub stopped_vms: usize,
    pub error_vms: usize,
    pub healthy_percentage: f64,
}

/// Heartbeat information
#[derive(Debug, Clone)]
pub struct Heartbeat {
    pub vm_id: String,
    pub timestamp: Instant,
    pub is_alive: bool,
}

/// Health monitor configuration
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    pub check_interval_secs: u64,
    pub heartbeat_timeout_secs: u64,
    pub cpu_warning_threshold: f64,
    pub cpu_critical_threshold: f64,
    pub memory_warning_threshold: f64,
    pub memory_critical_threshold: f64,
    pub disk_warning_threshold: f64,
    pub disk_critical_threshold: f64,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 30,
            heartbeat_timeout_secs: 60,
            cpu_warning_threshold: 70.0,
            cpu_critical_threshold: 90.0,
            memory_warning_threshold: 80.0,
            memory_critical_threshold: 95.0,
            disk_warning_threshold: 85.0,
            disk_critical_threshold: 95.0,
        }
    }
}

/// Health monitor for tracking system and VM health
pub struct HealthMonitor {
    manager: Arc<MicroVMManager>,
    config: HealthMonitorConfig,
    start_time: Instant,
    heartbeats: Arc<RwLock<Vec<Heartbeat>>>,
    last_system_resources: Arc<RwLock<SystemResources>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(manager: Arc<MicroVMManager>, config: HealthMonitorConfig) -> Self {
        Self {
            manager,
            config,
            start_time: Instant::now(),
            heartbeats: Arc::new(RwLock::new(Vec::new())),
            last_system_resources: Arc::new(RwLock::new(SystemResources::default())),
        }
    }
    
    /// Start the health monitoring loop
    pub async fn start_monitoring(&self) {
        info!("[Health] Starting health monitoring with {}s interval", 
            self.config.check_interval_secs);
        
        let mut interval = interval(Duration::from_secs(self.config.check_interval_secs));
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.check_system_health().await {
                error!("[Health] System health check failed: {}", e);
            }
            
            if let Err(e) = self.check_vm_health().await {
                error!("[Health] VM health check failed: {}", e);
            }
        }
    }
    
    /// Check system health and update resource metrics
    async fn check_system_health(&self) -> anyhow::Result<()> {
        debug!("[Health] Checking system health");
        
        let resources = self.collect_system_resources().await?;
        
        // Check thresholds and log warnings
        if resources.cpu_usage_percent > self.config.cpu_critical_threshold {
            warn!("[Health] CPU usage critical: {:.1}%", resources.cpu_usage_percent);
        } else if resources.cpu_usage_percent > self.config.cpu_warning_threshold {
            warn!("[Health] CPU usage high: {:.1}%", resources.cpu_usage_percent);
        }
        
        if resources.memory_usage_percent > self.config.memory_critical_threshold {
            warn!("[Health] Memory usage critical: {:.1}%", resources.memory_usage_percent);
        } else if resources.memory_usage_percent > self.config.memory_warning_threshold {
            warn!("[Health] Memory usage high: {:.1}%", resources.memory_usage_percent);
        }
        
        if resources.disk_usage_percent > self.config.disk_critical_threshold {
            warn!("[Health] Disk usage critical: {:.1}%", resources.disk_usage_percent);
        } else if resources.disk_usage_percent > self.config.disk_warning_threshold {
            warn!("[Health] Disk usage high: {:.1}%", resources.disk_usage_percent);
        }
        
        let mut last_resources = self.last_system_resources.write().await;
        *last_resources = resources;
        
        Ok(())
    }
    
    /// Collect system resource information
    async fn collect_system_resources(&self) -> anyhow::Result<SystemResources> {
        // Simulate resource collection
        // In production, this would read from /proc/stat, /proc/meminfo, etc.
        
        let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
        let memory_info = self.get_memory_info().await.unwrap_or((0, 0, 0));
        let disk_info = self.get_disk_info().await.unwrap_or((0, 0, 0));
        
        let memory_usage_percent = if memory_info.0 > 0 {
            (memory_info.1 as f64 / memory_info.0 as f64) * 100.0
        } else {
            0.0
        };
        
        let disk_usage_percent = if disk_info.0 > 0 {
            (disk_info.1 as f64 / disk_info.0 as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(SystemResources {
            cpu_usage_percent: cpu_usage,
            memory_total_mb: memory_info.0,
            memory_used_mb: memory_info.1,
            memory_free_mb: memory_info.2,
            memory_usage_percent,
            disk_total_gb: disk_info.0,
            disk_used_gb: disk_info.1,
            disk_free_gb: disk_info.2,
            disk_usage_percent,
        })
    }
    
    /// Get CPU usage percentage (simulated)
    async fn get_cpu_usage(&self) -> anyhow::Result<f64> {
        // In production: read /proc/stat
        // Simulated value between 10% and 60%
        Ok(10.0 + (rand::random::<f64>() * 50.0))
    }
    
    /// Get memory information in MB (simulated)
    async fn get_memory_info(&self) -> anyhow::Result<(u64, u64, u64)> {
        // In production: read /proc/meminfo
        // Returns (total, used, free)
        let total = 16384; // 16GB
        let used = 8192 + (rand::random::<u64>() % 4096); // 8-12GB used
        let free = total - used;
        Ok((total, used, free))
    }
    
    /// Get disk information in GB (simulated)
    async fn get_disk_info(&self) -> anyhow::Result<(u64, u64, u64)> {
        // In production: use statvfs
        // Returns (total, used, free)
        let total = 500; // 500GB
        let used = 200 + (rand::random::<u64>() % 150); // 200-350GB used
        let free = total - used;
        Ok((total, used, free))
    }
    
    /// Check VM health status
    async fn check_vm_health(&self) -> anyhow::Result<()> {
        debug!("[Health] Checking VM health");
        
        let vms = self.manager.list_vms().await;
        let mut heartbeats = self.heartbeats.write().await;
        
        for vm in vms {
            let is_responsive = vm.state == VmState::Running;
            
            // Update or create heartbeat
            if let Some(heartbeat) = heartbeats.iter_mut().find(|h| h.vm_id == vm.vm_id) {
                if is_responsive {
                    heartbeat.timestamp = Instant::now();
                    heartbeat.is_alive = true;
                } else {
                    heartbeat.is_alive = false;
                }
            } else {
                heartbeats.push(Heartbeat {
                    vm_id: vm.vm_id.clone(),
                    timestamp: Instant::now(),
                    is_alive: is_responsive,
                });
            }
            
            // Check for stale heartbeats
            if is_responsive {
                if let Some(heartbeat) = heartbeats.iter().find(|h| h.vm_id == vm.vm_id) {
                    let elapsed = heartbeat.timestamp.elapsed().as_secs();
                    if elapsed > self.config.heartbeat_timeout_secs {
                        warn!("[Health] VM {} heartbeat timeout ({}s)", vm.vm_id, elapsed);
                    }
                }
            }
        }
        
        // Clean up heartbeats for destroyed VMs
        let vm_ids: std::collections::HashSet<String> = vms.iter().map(|v| v.vm_id.clone()).collect();
        heartbeats.retain(|h| vm_ids.contains(&h.vm_id));
        
        Ok(())
    }
    
    /// Get current health status
    pub async fn get_health_status(&self) -> HealthCheckResponse {
        let uptime = self.start_time.elapsed().as_secs();
        let resources = self.last_system_resources.read().await.clone();
        let vm_summary = self.get_vm_health_summary().await;
        
        // Determine overall status
        let status = if resources.cpu_usage_percent > self.config.cpu_critical_threshold
            || resources.memory_usage_percent > self.config.memory_critical_threshold
            || resources.disk_usage_percent > self.config.disk_critical_threshold
            || vm_summary.healthy_percentage < 50.0
        {
            HealthStatus::Unhealthy
        } else if resources.cpu_usage_percent > self.config.cpu_warning_threshold
            || resources.memory_usage_percent > self.config.memory_warning_threshold
            || resources.disk_usage_percent > self.config.disk_warning_threshold
            || vm_summary.healthy_percentage < 80.0
        {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        
        HealthCheckResponse {
            status: status.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: uptime,
            version: env!("CARGO_PKG_VERSION").to_string(),
            system_resources: resources,
            vm_summary,
        }
    }
    
    /// Get VM health summary
    async fn get_vm_health_summary(&self) -> VmHealthSummary {
        let vms = self.manager.list_vms().await;
        let heartbeats = self.heartbeats.read().await;
        
        let total = vms.len();
        let running = vms.iter().filter(|v| v.state == VmState::Running).count();
        let stopped = vms.iter().filter(|v| v.state == VmState::Stopped).count();
        let error = vms.iter().filter(|v| {
            matches!(v.state, VmState::Creating) || 
            matches!(v.state, VmState::Destroyed)
        }).count();
        
        let healthy_count = vms.iter().filter(|v| {
            v.state == VmState::Running && 
            heartbeats.iter().any(|h| h.vm_id == v.vm_id && h.is_alive)
        }).count();
        
        let healthy_percentage = if total > 0 {
            (healthy_count as f64 / total as f64) * 100.0
        } else {
            100.0
        };
        
        VmHealthSummary {
            total_vms: total,
            running_vms: running,
            stopped_vms: stopped,
            error_vms: error,
            healthy_percentage,
        }
    }
    
    /// Get detailed health information for all VMs
    pub async fn get_vm_health_details(&self) -> Vec<VmHealth> {
        let vms = self.manager.list_vms().await;
        let heartbeats = self.heartbeats.read().await;
        let mut vm_health_list = Vec::new();
        
        for vm in vms {
            let is_responsive = vm.state == VmState::Running;
            let last_heartbeat = heartbeats
                .iter()
                .find(|h| h.vm_id == vm.vm_id)
                .map(|h| {
                    let elapsed = h.timestamp.elapsed().as_secs();
                    format!("{}s ago", elapsed)
                })
                .unwrap_or_else(|| "never".to_string());
            
            let stats = vm.get_stats().await.unwrap_or_default();
            
            vm_health_list.push(VmHealth {
                vm_id: vm.vm_id,
                state: format!("{:?}", vm.state),
                is_responsive,
                last_heartbeat,
                cpu_usage_percent: stats.cpu_usage_percent,
                memory_usage_mb: stats.memory_usage_mb,
            });
        }
        
        vm_health_list
    }
    
    /// Record a heartbeat for a VM
    pub async fn record_heartbeat(&self, vm_id: &str) {
        let mut heartbeats = self.heartbeats.write().await;
        
        if let Some(heartbeat) = heartbeats.iter_mut().find(|h| h.vm_id == vm_id) {
            heartbeat.timestamp = Instant::now();
            heartbeat.is_alive = true;
        } else {
            heartbeats.push(Heartbeat {
                vm_id: vm_id.to_string(),
                timestamp: Instant::now(),
                is_alive: true,
            });
        }
        
        debug!("[Health] Recorded heartbeat for VM: {}", vm_id);
    }
    
    /// Get the health monitor configuration
    pub fn get_config(&self) -> &HealthMonitorConfig {
        &self.config
    }
}

/// Simple heartbeat sender for VMs
pub struct HeartbeatSender {
    monitor: Arc<HealthMonitor>,
    vm_id: String,
    interval_secs: u64,
}

impl HeartbeatSender {
    /// Create a new heartbeat sender
    pub fn new(monitor: Arc<HealthMonitor>, vm_id: String, interval_secs: u64) -> Self {
        Self {
            monitor,
            vm_id,
            interval_secs,
        }
    }
    
    /// Start sending heartbeats
    pub async fn start(&self) {
        let mut interval = interval(Duration::from_secs(self.interval_secs));
        
        loop {
            interval.tick().await;
            self.monitor.record_heartbeat(&self.vm_id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_monitor() -> HealthMonitor {
        let manager = Arc::new(MicroVMManager::new(10, "/tmp/test_snapshots".to_string()));
        let config = HealthMonitorConfig::default();
        HealthMonitor::new(manager, config)
    }
    
    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = create_test_monitor();
        assert_eq!(monitor.get_config().check_interval_secs, 30);
    }
    
    #[tokio::test]
    async fn test_health_status_response() {
        let monitor = create_test_monitor();
        let status = monitor.get_health_status().await;
        
        assert!(!status.status.is_empty());
        assert!(!status.timestamp.is_empty());
        assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    }
    
    #[tokio::test]
    async fn test_system_resources_collection() {
        let monitor = create_test_monitor();
        let resources = monitor.collect_system_resources().await.unwrap();
        
        assert!(resources.cpu_usage_percent >= 0.0 && resources.cpu_usage_percent <= 100.0);
        assert!(resources.memory_total_mb > 0);
        assert!(resources.disk_total_gb > 0);
    }
    
    #[tokio::test]
    async fn test_vm_health_summary() {
        let monitor = create_test_monitor();
        let summary = monitor.get_vm_health_summary().await;
        
        assert_eq!(summary.total_vms, 0);
        assert_eq!(summary.running_vms, 0);
        assert_eq!(summary.healthy_percentage, 100.0);
    }
    
    #[tokio::test]
    async fn test_heartbeat_recording() {
        let monitor = create_test_monitor();
        let monitor_arc = Arc::new(monitor);
        
        monitor_arc.record_heartbeat("test-vm-1").await;
        
        let heartbeats = monitor_arc.heartbeats.read().await;
        assert_eq!(heartbeats.len(), 1);
        assert_eq!(heartbeats[0].vm_id, "test-vm-1");
        assert!(heartbeats[0].is_alive);
    }
}
