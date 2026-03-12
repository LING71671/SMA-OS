//! Rate limiter for 1000+ concurrent agents
//!
//! Prevents resource exhaustion under high load

use governor::clock::DefaultClock;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{info, warn};

/// Per-tenant rate limiter
pub struct TenantRateLimiter {
    limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    tenant_id: String,
}

impl TenantRateLimiter {
    /// Create a rate limiter for a tenant
    /// Default: 100 events/second per tenant
    pub fn new(tenant_id: String) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(100).unwrap());
        let limiter = Arc::new(RateLimiter::direct(quota));
        
        info!("[RateLimiter] Created for tenant {}: 100 req/s", tenant_id);
        
        Self {
            limiter,
            tenant_id,
        }
    }

    /// Check if request is allowed
    pub async fn check(&self) -> Result<(), String> {
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => {
                warn!("[RateLimiter] Rate limit exceeded for tenant {}", self.tenant_id);
                Err(format!("Rate limit exceeded for tenant {}", self.tenant_id))
            }
        }
    }
}

/// Global rate limiter registry
pub struct RateLimiterRegistry {
    // In production, use DashMap for concurrent access
    limiters: Arc<dashmap::DashMap<String, Arc<TenantRateLimiter>>>,
}

impl RateLimiterRegistry {
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Get or create rate limiter for tenant
    pub fn get_or_create(&self, tenant_id: &str) -> Arc<TenantRateLimiter> {
        let arc_limiter = Arc::new(TenantRateLimiter::new(tenant_id.to_string()));
        self.limiters
            .entry(tenant_id.to_string())
            .or_insert(arc_limiter)
            .clone()
    }
}

impl Default for RateLimiterRegistry {
    fn default() -> Self {
        Self::new()
    }
}
