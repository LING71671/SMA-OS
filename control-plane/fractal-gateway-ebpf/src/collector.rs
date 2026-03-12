//! eBPF Data Collector
//!
//! This module provides functionality to collect data from eBPF maps,
//! including packet statistics, processing times, and blocked packets.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::FractalGatewayEbpf;

/// Statistics collected from eBPF maps
#[derive(Debug, Clone, Default)]
pub struct CollectorStats {
    /// Total number of packets processed
    pub packets_processed: u64,
    /// Total number of packets blocked
    pub packets_blocked: u64,
    /// Total processing time in nanoseconds
    pub processing_time_ns: u64,
    /// Number of active blocked IPs
    pub blocked_ip_count: usize,
    /// Timestamp of last collection
    pub last_collection: Option<Instant>,
}

/// A single data point collected from eBPF
#[derive(Debug, Clone)]
pub struct DataPoint {
    /// Timestamp when the data was collected
    pub timestamp: Instant,
    /// Number of packets processed since last collection
    pub packets_processed: u64,
    /// Number of packets blocked since last collection
    pub packets_blocked: u64,
    /// Processing time in nanoseconds
    pub processing_time_ns: u64,
}

/// eBPF data collector that reads statistics from eBPF maps
pub struct EbpfCollector {
    /// Current statistics protected by RwLock for thread-safe access
    stats: Arc<RwLock<CollectorStats>>,
    /// Flag to control the collection loop
    running: AtomicBool,
    /// Collection interval
    interval: Duration,
    /// Previous stats for delta calculation
    prev_stats: Arc<RwLock<CollectorStats>>,
}

impl EbpfCollector {
    /// Create a new EbpfCollector with default settings
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(CollectorStats::default())),
            running: AtomicBool::new(false),
            interval: Duration::from_secs(1),
            prev_stats: Arc::new(RwLock::new(CollectorStats::default())),
        }
    }

    /// Create a new EbpfCollector with custom collection interval
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            stats: Arc::new(RwLock::new(CollectorStats::default())),
            running: AtomicBool::new(false),
            interval,
            prev_stats: Arc::new(RwLock::new(CollectorStats::default())),
        }
    }

    /// Start collecting data from eBPF maps
    ///
    /// This method spawns an async task that periodically reads from
    /// eBPF maps and updates the statistics.
    pub async fn start_collecting(&self, ebpf: &mut FractalGatewayEbpf) -> Result<()> {
        if self.running.load(Ordering::SeqCst) {
            warn!("Collector is already running");
            return Ok(());
        }

        info!("Starting eBPF data collection with interval {:?}", self.interval);
        self.running.store(true, Ordering::SeqCst);

        let stats = Arc::clone(&self.stats);
        let prev_stats = Arc::clone(&self.prev_stats);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = Arc::clone(&running);
        let interval_duration = self.interval;

        // Spawn the collection task
        tokio::spawn(async move {
            let mut ticker = interval(interval_duration);

            while running_clone.load(Ordering::SeqCst) {
                ticker.tick().await;

                // Collect data from eBPF maps
                match Self::collect_from_ebpf(ebpf).await {
                    Ok(current_stats) => {
                        // Calculate deltas
                        let prev = prev_stats.read().map_err(|e| {
                            anyhow::anyhow!("Failed to acquire read lock: {}", e)
                        });

                        if let Ok(prev) = prev {
                            let data_point = DataPoint {
                                timestamp: Instant::now(),
                                packets_processed: current_stats
                                    .packets_processed
                                    .saturating_sub(prev.packets_processed),
                                packets_blocked: current_stats
                                    .packets_blocked
                                    .saturating_sub(prev.packets_blocked),
                                processing_time_ns: current_stats.processing_time_ns,
                            };

                            debug!(
                                "Collected data point: processed={}, blocked={}",
                                data_point.packets_processed, data_point.packets_blocked
                            );

                            // Update previous stats
                            if let Ok(mut prev) = prev_stats.write() {
                                *prev = current_stats.clone();
                            }

                            // Update current stats
                            if let Ok(mut stats_guard) = stats.write() {
                                stats_guard.packets_processed = current_stats.packets_processed;
                                stats_guard.packets_blocked = current_stats.packets_blocked;
                                stats_guard.processing_time_ns = current_stats.processing_time_ns;
                                stats_guard.blocked_ip_count = current_stats.blocked_ip_count;
                                stats_guard.last_collection = Some(Instant::now());
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to collect eBPF data: {}", e);
                    }
                }
            }

            info!("eBPF data collection stopped");
        });

        Ok(())
    }

    /// Stop the data collection
    pub async fn stop_collecting(&self) -> Result<()> {
        if !self.running.load(Ordering::SeqCst) {
            warn!("Collector is not running");
            return Ok(());
        }

        info!("Stopping eBPF data collection");
        self.running.store(false, Ordering::SeqCst);

        // Give the collection task time to stop gracefully
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Get a copy of the current statistics
    pub fn get_stats(&self) -> Option<CollectorStats> {
        self.stats.read().ok().map(|guard| guard.clone())
    }

    /// Get the collection interval
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Check if the collector is currently running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Collect data from eBPF maps
    ///
    /// This is a placeholder implementation that would read from actual eBPF maps.
    /// In a real implementation, this would read from maps like:
    /// - PACKET_STATS: HashMap tracking packet counts
    /// - BLOCKED_IPS: HashMap of blocked IPs
    /// - PROCESSING_TIME: Array or HashMap of processing times
    async fn collect_from_ebpf(ebpf: &mut FractalGatewayEbpf) -> Result<CollectorStats> {
        // In a real implementation, we would read from eBPF maps here
        // For now, we return placeholder data based on the FractalGatewayEbpf interface

        let blocked_count = ebpf.get_blocked_count().context("Failed to get blocked IP count")?;

        // Placeholder: In real implementation, read from eBPF maps
        // These would be actual counters maintained by the eBPF program
        let stats = CollectorStats {
            packets_processed: 0, // Would read from eBPF map
            packets_blocked: blocked_count as u64,
            processing_time_ns: 0, // Would read from eBPF map
            blocked_ip_count: blocked_count,
            last_collection: Some(Instant::now()),
        };

        Ok(stats)
    }
}

impl Default for EbpfCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_new() {
        let collector = EbpfCollector::new();
        assert!(!collector.is_running());
        assert_eq!(collector.interval(), Duration::from_secs(1));
        assert!(collector.get_stats().is_some());
    }

    #[test]
    fn test_collector_with_interval() {
        let interval = Duration::from_millis(500);
        let collector = EbpfCollector::with_interval(interval);
        assert_eq!(collector.interval(), interval);
    }

    #[test]
    fn test_collector_stats_default() {
        let stats = CollectorStats::default();
        assert_eq!(stats.packets_processed, 0);
        assert_eq!(stats.packets_blocked, 0);
        assert_eq!(stats.processing_time_ns, 0);
        assert_eq!(stats.blocked_ip_count, 0);
        assert!(stats.last_collection.is_none());
    }

    #[test]
    fn test_data_point_creation() {
        let point = DataPoint {
            timestamp: Instant::now(),
            packets_processed: 100,
            packets_blocked: 10,
            processing_time_ns: 1000,
        };

        assert_eq!(point.packets_processed, 100);
        assert_eq!(point.packets_blocked, 10);
        assert_eq!(point.processing_time_ns, 1000);
    }

    #[tokio::test]
    async fn test_collector_start_stop() {
        // Note: This test requires a valid FractalGatewayEbpf instance
        // which may not be available in test environment without proper setup
        // For now, we just test the basic state management

        let collector = EbpfCollector::new();
        assert!(!collector.is_running());

        // Since we can't easily create a FractalGatewayEbpf in tests,
        // we just verify the initial state
        let stats = collector.get_stats();
        assert!(stats.is_some());
    }
}
