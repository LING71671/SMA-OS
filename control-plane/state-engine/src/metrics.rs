//! Prometheus metrics for SMA-OS monitoring
//!
//! Phase 3.4: Advanced monitoring and alerting

use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Registry,
    exponential_buckets, register_counter, register_counter_vec, register_gauge,
    register_gauge_vec, register_histogram, register_histogram_vec,
};
use std::sync::Arc;
use tracing::{info, error};

/// Global metrics registry
pub struct MetricsRegistry {
    registry: Registry,
    
    // Event metrics
    pub events_appended_total: CounterVec,
    pub events_queried_total: CounterVec,
    pub event_append_latency: HistogramVec,
    pub event_query_latency: HistogramVec,
    
    // Storage metrics
    pub redis_connections: Gauge,
    pub postgres_connections: Gauge,
    pub cache_hit_ratio: GaugeVec,
    pub storage_size_bytes: GaugeVec,
    
    // Rate limiting metrics
    pub rate_limited_requests: CounterVec,
    pub active_tenants: Gauge,
    
    // Cluster metrics
    pub cluster_nodes: Gauge,
    pub cluster_healthy_nodes: Gauge,
    pub cluster_operations: CounterVec,
    
    // Failover metrics
    pub failover_events: Counter,
    pub recovery_events: Counter,
    pub component_health: GaugeVec,
}

impl MetricsRegistry {
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Registry::new();

        // Event metrics
        let events_appended_total = register_counter_vec!(
            "sma_events_appended_total",
            "Total number of events appended",
            &["tenant_id", "namespace"]
        )?;

        let events_queried_total = register_counter_vec!(
            "sma_events_queried_total",
            "Total number of events queried",
            &["tenant_id", "namespace"]
        )?;

        let event_append_latency = register_histogram_vec!(
            "sma_event_append_latency_seconds",
            "Event append latency in seconds",
            &["tenant_id"],
            exponential_buckets(0.001, 2.0, 15).unwrap()
        )?;

        let event_query_latency = register_histogram_vec!(
            "sma_event_query_latency_seconds",
            "Event query latency in seconds",
            &["tenant_id"],
            exponential_buckets(0.001, 2.0, 15).unwrap()
        )?;

        // Storage metrics
        let redis_connections = register_gauge!(
            "sma_redis_connections",
            "Number of active Redis connections"
        )?;

        let postgres_connections = register_gauge!(
            "sma_postgres_connections",
            "Number of active PostgreSQL connections"
        )?;

        let cache_hit_ratio = register_gauge_vec!(
            "sma_cache_hit_ratio",
            "Cache hit ratio",
            &["tier"]
        )?;

        let storage_size_bytes = register_gauge_vec!(
            "sma_storage_size_bytes",
            "Storage size in bytes",
            &["tier"]
        )?;

        // Rate limiting metrics
        let rate_limited_requests = register_counter_vec!(
            "sma_rate_limited_requests_total",
            "Total number of rate limited requests",
            &["tenant_id"]
        )?;

        let active_tenants = register_gauge!(
            "sma_active_tenants",
            "Number of active tenants"
        )?;

        // Cluster metrics
        let cluster_nodes = register_gauge!(
            "sma_cluster_nodes_total",
            "Total number of cluster nodes"
        )?;

        let cluster_healthy_nodes = register_gauge!(
            "sma_cluster_healthy_nodes",
            "Number of healthy cluster nodes"
        )?;

        let cluster_operations = register_counter_vec!(
            "sma_cluster_operations_total",
            "Total cluster operations",
            &["operation", "status"]
        )?;

        // Failover metrics
        let failover_events = register_counter!(
            "sma_failover_events_total",
            "Total number of failover events"
        )?;

        let recovery_events = register_counter!(
            "sma_recovery_events_total",
            "Total number of recovery events"
        )?;

        let component_health = register_gauge_vec!(
            "sma_component_health",
            "Component health status (0=unhealthy, 1=degraded, 2=healthy)",
            &["component"]
        )?;

        info!("[Metrics] Prometheus metrics registry initialized");

        Ok(Self {
            registry,
            events_appended_total,
            events_queried_total,
            event_append_latency,
            event_query_latency,
            redis_connections,
            postgres_connections,
            cache_hit_ratio,
            storage_size_bytes,
            rate_limited_requests,
            active_tenants,
            cluster_nodes,
            cluster_healthy_nodes,
            cluster_operations,
            failover_events,
            recovery_events,
            component_health,
        })
    }

    /// Record event append
    pub fn record_event_append(&self, tenant_id: &str, namespace: &str, latency: std::time::Duration) {
        self.events_appended_total
            .with_label_values(&[tenant_id, namespace])
            .inc();
        self.event_append_latency
            .with_label_values(&[tenant_id])
            .observe(latency.as_secs_f64());
    }

    /// Record event query
    pub fn record_event_query(&self, tenant_id: &str, namespace: &str, latency: std::time::Duration) {
        self.events_queried_total
            .with_label_values(&[tenant_id, namespace])
            .inc();
        self.event_query_latency
            .with_label_values(&[tenant_id])
            .observe(latency.as_secs_f64());
    }

    /// Update cache hit ratio
    pub fn set_cache_hit_ratio(&self, tier: &str, ratio: f64) {
        self.cache_hit_ratio.with_label_values(&[tier]).set(ratio);
    }

    /// Record rate limited request
    pub fn record_rate_limited(&self, tenant_id: &str) {
        self.rate_limited_requests
            .with_label_values(&[tenant_id])
            .inc();
    }

    /// Update cluster metrics
    pub fn update_cluster_metrics(&self, total: usize, healthy: usize) {
        self.cluster_nodes.set(total as f64);
        self.cluster_healthy_nodes.set(healthy as f64);
    }

    /// Record failover
    pub fn record_failover(&self) {
        self.failover_events.inc();
    }

    /// Record recovery
    pub fn record_recovery(&self) {
        self.recovery_events.inc();
    }

    /// Update component health
    pub fn set_component_health(&self, component: &str, status: i64) {
        self.component_health.with_label_values(&[component]).set(status as f64);
    }

    /// Export metrics as text
    pub fn gather(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }

    /// Get registry reference
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

impl Default for MetricsRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics registry")
    }
}

/// Metrics endpoint for Prometheus scraping
pub async fn serve_metrics(registry: Arc<MetricsRegistry>, port: u16) {
    use hyper::{Body, Response, Server, service::{make_service_fn, service_fn}};
    use prometheus::Encoder;
    
    let make_svc = make_service_fn(move |_conn| {
        let registry = registry.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |_req| {
                let metrics = registry.gather();
                let encoder = prometheus::TextEncoder::new();
                let body = encoder.encode_to_string(&metrics).unwrap();
                
                async move {
                    Ok::<_, hyper::Error>(Response::builder()
                        .header("Content-Type", encoder.format_type())
                        .body(Body::from(body))
                        .unwrap())
                }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::bind(&addr).serve(make_svc);

    info!("[Metrics] Prometheus metrics server listening on port {}", port);

    if let Err(e) = server.await {
        error!("[Metrics] Server error: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry() {
        let registry = MetricsRegistry::new().unwrap();
        
        registry.record_event_append("tenant-1", "default", std::time::Duration::from_millis(10));
        registry.set_cache_hit_ratio("redis", 0.95);
        registry.update_cluster_metrics(5, 4);
        
        let metrics = registry.gather();
        assert!(!metrics.is_empty());
    }
}
