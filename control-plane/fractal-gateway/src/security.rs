//! Security Policy Enforcement Module
//!
//! Implements eBPF-based security controls:
//! - seccomp: System call filtering
//! - AppArmor: Mandatory access control profiles  
//! - cgroup: Resource quotas and limits
//! - O(1) quota enforcement for reward/punishment system

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use tracing::{info, debug, warn, error};

/// Security policy configuration for an agent
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub agent_id: String,
    pub seccomp_profile: SeccompProfile,
    pub apparmor_profile: String,
    pub cgroup_limits: CgroupLimits,
    pub network_policy: NetworkPolicy,
    pub dynamic_quotas: DynamicQuotas,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            agent_id: String::new(),
            seccomp_profile: SeccompProfile::Restricted,
            apparmor_profile: "smaos-default".to_string(),
            cgroup_limits: CgroupLimits::default(),
            network_policy: NetworkPolicy::default(),
            dynamic_quotas: DynamicQuotas::default(),
        }
    }
}

/// seccomp profile types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeccompProfile {
    Unrestricted,
    Basic,
    Restricted,
    Strict,
}

impl SeccompProfile {
    pub fn allowed_syscalls(&self) -> Vec<&'static str> {
        match self {
            SeccompProfile::Unrestricted => vec![],
            SeccompProfile::Basic => vec![
                "read", "write", "open", "close", "fstat", "mmap",
                "mprotect", "munmap", "brk", "access", "getpid",
                "exit", "exit_group", "clone", "fork", "vfork",
            ],
            SeccompProfile::Restricted => vec![
                "read", "write", "openat", "close", "fstat", "mmap",
                "mprotect", "munmap", "brk", "access", "getpid",
                "exit", "exit_group", "clone3",
                "socket", "connect", "sendto", "recvfrom",
            ],
            SeccompProfile::Strict => vec![
                "read", "write", "exit", "exit_group",
            ],
        }
    }
}

/// cgroup resource limits
#[derive(Debug, Clone)]
pub struct CgroupLimits {
    pub cpu_quota_us: i64,
    pub cpu_period_us: i64,
    pub memory_limit_bytes: i64,
    pub memory_swap_limit_bytes: i64,
    pub pids_max: i64,
    pub blkio_weight: u16,
}

impl Default for CgroupLimits {
    fn default() -> Self {
        Self {
            cpu_quota_us: 100_000,
            cpu_period_us: 100_000,
            memory_limit_bytes: 512 * 1024 * 1024,
            memory_swap_limit_bytes: 512 * 1024 * 1024,
            pids_max: 100,
            blkio_weight: 500,
        }
    }
}

/// Network policy
#[derive(Debug, Clone)]
pub struct NetworkPolicy {
    pub allowed_egress: Vec<String>,
    pub allowed_ingress: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub default_deny: bool,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self {
            allowed_egress: vec!["10.0.0.0/8".to_string(), "172.16.0.0/12".to_string()],
            allowed_ingress: vec![],
            allowed_ports: vec![80, 443, 8080],
            default_deny: true,
        }
    }
}

/// Dynamic quotas for reward/punishment
#[derive(Debug, Clone)]
pub struct DynamicQuotas {
    pub cpu_multiplier: f64,
    pub memory_multiplier: f64,
    pub bandwidth_multiplier: f64,
    pub priority_level: u8,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for DynamicQuotas {
    fn default() -> Self {
        Self {
            cpu_multiplier: 1.0,
            memory_multiplier: 1.0,
            bandwidth_multiplier: 1.0,
            priority_level: 50,
            last_updated: chrono::Utc::now(),
        }
    }
}

/// Security policy manager
pub struct SecurityPolicyManager {
    policies: Arc<RwLock<HashMap<String, SecurityPolicy>>>,
}

impl SecurityPolicyManager {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a security policy
    pub async fn register_policy(&self, policy: SecurityPolicy) -> Result<()> {
        let mut policies = self.policies.write().await;
        info!("[Security] Registering policy for agent: {}", policy.agent_id);
        policies.insert(policy.agent_id.clone(), policy);
        Ok(())
    }
    
    /// Get security policy
    pub async fn get_policy(&self, agent_id: &str) -> Option<SecurityPolicy> {
        let policies = self.policies.read().await;
        policies.get(agent_id).cloned()
    }
    
    /// Update dynamic quotas - O(1) operation
    pub async fn update_quotas(
        &self,
        agent_id: &str,
        cpu_multiplier: f64,
        memory_multiplier: f64,
        bandwidth_multiplier: f64,
        priority_level: u8,
    ) -> Result<()> {
        let mut policies = self.policies.write().await;
        
        if let Some(policy) = policies.get_mut(agent_id) {
            policy.dynamic_quotas.cpu_multiplier = cpu_multiplier.clamp(0.1, 10.0);
            policy.dynamic_quotas.memory_multiplier = memory_multiplier.clamp(0.1, 10.0);
            policy.dynamic_quotas.bandwidth_multiplier = bandwidth_multiplier.clamp(0.1, 10.0);
            policy.dynamic_quotas.priority_level = priority_level.clamp(0, 100);
            policy.dynamic_quotas.last_updated = chrono::Utc::now();
            
            info!(
                "[Security] Updated quotas for agent {}: CPU={:.2}x, MEM={:.2}x, PRIO={}",
                agent_id, cpu_multiplier, memory_multiplier, priority_level
            );
            
            // Apply quotas via eBPF (O(1))
            self.apply_ebpf_quota(agent_id, &policy.dynamic_quotas).await?;
        }
        
        Ok(())
    }
    
    /// Apply quotas using eBPF (O(1))
    async fn apply_ebpf_quota(&self, agent_id: &str, quotas: &DynamicQuotas) -> Result<()> {
        debug!(
            "[Security] eBPF O(1) quota update for {}: cpu_mult={:.2}, prio={}",
            agent_id, quotas.cpu_multiplier, quotas.priority_level
        );
        
        // In production: update eBPF maps via aya
        // eBPF program checks map on each syscall
        
        Ok(())
    }
    
    /// Generate AppArmor profile
    pub async fn generate_apparmor(&self, agent_id: &str) -> Result<String> {
        let profile = format!(
            r#"# SMA-OS AppArmor Profile for {agent_id}
#include <tunables/global>

profile sma-{agent_id} {{
    #include <abstractions/base>
    #include <abstractions/network>
    
    capability net_bind_service,
    
    /opt/sma-os/** r,
    /tmp/smaos-*/** rw,
    
    deny /etc/shadow r,
    deny capability sys_admin,
    deny capability sys_ptrace,
}}
"#,
            agent_id = agent_id
        );
        
        Ok(profile)
    }
    
    /// Remove security policy
    pub async fn remove_policy(&self, agent_id: &str) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.remove(agent_id);
        info!("[Security] Removed policy for agent {}", agent_id);
        Ok(())
    }
    
    /// Get security statistics
    pub async fn get_stats(&self) -> SecurityStats {
        let policies = self.policies.read().await;
        SecurityStats {
            total_policies: policies.len(),
            restricted_count: policies.values()
                .filter(|p| p.seccomp_profile == SeccompProfile::Restricted)
                .count(),
            strict_count: policies.values()
                .filter(|p| p.seccomp_profile == SeccompProfile::Strict)
                .count(),
        }
    }
}

/// Security statistics
#[derive(Debug)]
pub struct SecurityStats {
    pub total_policies: usize,
    pub restricted_count: usize,
    pub strict_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_seccomp_profile() {
        let profile = SeccompProfile::Strict;
        let syscalls = profile.allowed_syscalls();
        assert_eq!(syscalls.len(), 4);
    }
    
    #[tokio::test]
    async fn test_security_manager() {
        let manager = SecurityPolicyManager::new();
        
        let policy = SecurityPolicy {
            agent_id: "test-agent".to_string(),
            seccomp_profile: SeccompProfile::Restricted,
            apparmor_profile: "test-profile".to_string(),
            cgroup_limits: CgroupLimits::default(),
            network_policy: NetworkPolicy::default(),
            dynamic_quotas: DynamicQuotas::default(),
        };
        
        manager.register_policy(policy).await.unwrap();
        
        let retrieved = manager.get_policy("test-agent").await;
        assert!(retrieved.is_some());
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.total_policies, 1);
    }
}
