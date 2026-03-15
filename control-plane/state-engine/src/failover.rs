//! Automated failover and recovery for SMA-OS
//!
//! Monitors health and triggers automatic failover when needed

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tracing::{error, info, warn};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub latency_ms: u64,
    pub message: String,
    pub timestamp: Instant,
}

/// Failover configuration
#[derive(Debug, Clone)]
pub struct FailoverConfig {
    /// Health check interval
    pub check_interval: Duration,
    /// Failure threshold before triggering failover
    pub failure_threshold: u32,
    /// Recovery time after failover
    pub recovery_time: Duration,
    /// Enable automatic failover
    pub auto_failover: bool,
}

impl Default for FailoverConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_time: Duration::from_secs(30),
            auto_failover: true,
        }
    }
}

/// Failover manager
pub struct FailoverManager {
    config: FailoverConfig,
    /// Component health status
    pub health_status: Arc<RwLock<HashMap<String, HealthStatus>>>,
    /// Failure counters per component
    failure_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// Last health check time
    last_check: Arc<RwLock<HashMap<String, Instant>>>,
    /// Event broadcaster for failover events
    event_tx: broadcast::Sender<FailoverEvent>,
}

/// Failover events
#[derive(Debug, Clone)]
pub enum FailoverEvent {
    ComponentFailed { component: String, reason: String },
    FailoverStarted { from: String, to: String },
    FailoverCompleted { component: String },
    RecoveryStarted { component: String },
    RecoveryCompleted { component: String },
}

impl FailoverManager {
    pub fn new(config: FailoverConfig) -> Self {
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            config,
            health_status: Arc::new(RwLock::new(HashMap::new())),
            failure_counts: Arc::new(RwLock::new(HashMap::new())),
            last_check: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    /// Get event receiver
    pub fn subscribe(&self) -> broadcast::Receiver<FailoverEvent> {
        self.event_tx.subscribe()
    }

    /// Register a component for health monitoring
    pub async fn register_component(&self, name: &str) {
        let mut status = self.health_status.write().await;
        let mut counts = self.failure_counts.write().await;
        let mut last = self.last_check.write().await;

        status.insert(name.to_string(), HealthStatus::Healthy);
        counts.insert(name.to_string(), 0);
        last.insert(name.to_string(), Instant::now());

        info!("[Failover] Registered component: {}", name);
    }

    /// Update health status
    pub async fn update_health(&self, check: HealthCheck) {
        // 先在锁内收集决策信息，然后释放锁再执行 failover
        let should_failover: Option<(String, String)>;

        {
            let mut status = self.health_status.write().await;
            let mut counts = self.failure_counts.write().await;
            let mut last = self.last_check.write().await;

            let component = check.component.clone();
            let old_status = status.get(&component).copied().unwrap_or(HealthStatus::Healthy);

            // Update last check time
            last.insert(component.clone(), check.timestamp);

            // Update status and failure count
            match check.status {
                HealthStatus::Healthy => {
                    if old_status != HealthStatus::Healthy {
                        info!("[Failover] Component {} recovered to healthy", component);
                        let _ = self.event_tx.send(FailoverEvent::RecoveryCompleted {
                            component: component.clone(),
                        });
                    }
                    counts.insert(component.clone(), 0);
                    should_failover = None;
                }
                HealthStatus::Degraded | HealthStatus::Unhealthy => {
                    let count = counts.entry(component.clone()).or_insert(0);
                    *count += 1;

                    if *count >= self.config.failure_threshold
                        && self.config.auto_failover
                        && old_status.is_healthy()
                    {
                        warn!(
                            "[Failover] Component {} failed threshold {}/{}",
                            component, count, self.config.failure_threshold
                        );
                        should_failover = Some((component.clone(), check.message.clone()));
                    } else {
                        should_failover = None;
                    }
                }
            }

            status.insert(component, check.status);
            // 所有 RwLock 写锁在此作用域结束时自动释放
        }

        // 在锁外执行 failover（其中包含 async sleep，不能持锁）
        if let Some((component, message)) = should_failover {
            self.trigger_failover(&component, &message).await;
        }
    }

    /// Trigger automatic failover
    async fn trigger_failover(&self, component: &str, reason: &str) {
        error!(
            "[Failover] Triggering failover for component {}: {}",
            component, reason
        );

        let _ = self.event_tx.send(FailoverEvent::ComponentFailed {
            component: component.to_string(),
            reason: reason.to_string(),
        });

        // In production, this would:
        // 1. Update DNS/load balancer to route away from failed component
        // 2. Promote replica to primary
        // 3. Notify orchestration layer
        // 4. Update service mesh

        let _ = self.event_tx.send(FailoverEvent::FailoverStarted {
            from: component.to_string(),
            to: format!("{}-replica", component),
        });

        // Simulate failover delay
        tokio::time::sleep(Duration::from_secs(2)).await;

        let _ = self.event_tx.send(FailoverEvent::FailoverCompleted {
            component: component.to_string(),
        });

        info!("[Failover] Failover completed for component {}", component);
    }

    /// Get current health summary
    pub async fn health_summary(&self) -> HealthSummary {
        let status = self.health_status.read().await;
        let mut healthy = 0;
        let mut degraded = 0;
        let mut unhealthy = 0;

        for s in status.values() {
            match s {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Degraded => degraded += 1,
                HealthStatus::Unhealthy => unhealthy += 1,
            }
        }

        HealthSummary {
            total: status.len(),
            healthy,
            degraded,
            unhealthy,
        }
    }

    /// Run health check loop
    pub async fn run_health_checks<F>(&self, checker: F)
    where
        F: Fn() -> Vec<HealthCheck>,
    {
        let mut interval = tokio::time::interval(self.config.check_interval);

        loop {
            interval.tick().await;
            
            let checks = checker();
            for check in checks {
                self.update_health(check).await;
            }

            let summary = self.health_summary().await;
            info!("[Failover] Health check complete: {:?}", summary);
        }
    }
}

/// Health summary for monitoring
#[derive(Debug)]
pub struct HealthSummary {
    pub total: usize,
    pub healthy: usize,
    pub degraded: usize,
    pub unhealthy: usize,
}

impl HealthSummary {
    pub fn overall_status(&self) -> HealthStatus {
        if self.unhealthy > 0 {
            HealthStatus::Unhealthy
        } else if self.degraded > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_failover_trigger() {
        let config = FailoverConfig {
            failure_threshold: 2,
            auto_failover: true,
            ..Default::default()
        };

        let manager = FailoverManager::new(config);
        let mut rx = manager.subscribe();

        manager.register_component("redis-1").await;

        // First failure
        manager.update_health(HealthCheck {
            component: "redis-1".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: 100,
            message: "Connection timeout".to_string(),
            timestamp: Instant::now(),
        }).await;

        // Second failure should trigger failover
        manager.update_health(HealthCheck {
            component: "redis-1".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: 100,
            message: "Connection timeout".to_string(),
            timestamp: Instant::now(),
        }).await;

        // Should receive failover event
        let event = rx.try_recv();
        assert!(event.is_ok());
    }

    #[test]
    fn test_health_summary() {
        let summary = HealthSummary {
            total: 5,
            healthy: 3,
            degraded: 1,
            unhealthy: 1,
        };

        assert_eq!(summary.overall_status(), HealthStatus::Unhealthy);
    }
}
