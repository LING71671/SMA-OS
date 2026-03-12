//! Data Aggregator
//!
//! This module provides functionality to aggregate eBPF data over time windows,
//! calculate statistics (P99 latency, block rate, etc.), and detect anomalies.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use tracing::{debug, info, warn};

use crate::collector::DataPoint;

/// Aggregated statistics over a time window
#[derive(Debug, Clone)]
pub struct AggregatedStats {
    /// Packets processed per second
    pub packets_per_second: f64,
    /// Block rate (percentage of packets blocked)
    pub block_rate: f64,
    /// Average latency in nanoseconds
    pub avg_latency_ns: f64,
    /// P99 latency in nanoseconds
    pub p99_latency_ns: u64,
    /// Total packets processed in the window
    pub total_packets: u64,
    /// Total packets blocked in the window
    pub total_blocked: u64,
    /// Window duration
    pub window_duration: Duration,
    /// Timestamp of aggregation
    pub timestamp: Instant,
}

impl Default for AggregatedStats {
    fn default() -> Self {
        Self {
            packets_per_second: 0.0,
            block_rate: 0.0,
            avg_latency_ns: 0.0,
            p99_latency_ns: 0,
            total_packets: 0,
            total_blocked: 0,
            window_duration: Duration::from_secs(0),
            timestamp: Instant::now(),
        }
    }
}

/// Anomaly detection result
#[derive(Debug, Clone, PartialEq)]
pub enum Anomaly {
    /// High packet rate detected
    HighPacketRate { rate: f64, threshold: f64 },
    /// High block rate detected
    HighBlockRate { rate: f64, threshold: f64 },
    /// High latency detected
    HighLatency { latency_ns: u64, threshold_ns: u64 },
    /// Sudden spike in traffic
    TrafficSpike { current: u64, baseline: f64 },
    /// Sudden drop in traffic
    TrafficDrop { current: u64, baseline: f64 },
}

/// Data aggregator that maintains a sliding window of data points
pub struct DataAggregator {
    /// Size of the aggregation window
    window_size: Duration,
    /// Buffer of data points within the window
    buffer: VecDeque<DataPoint>,
    /// Baseline for anomaly detection (packets per second)
    baseline_pps: Option<f64>,
    /// Thresholds for anomaly detection
    thresholds: AnomalyThresholds,
}

/// Thresholds for anomaly detection
#[derive(Debug, Clone)]
pub struct AnomalyThresholds {
    /// Maximum acceptable packets per second
    pub max_pps: f64,
    /// Maximum acceptable block rate (0.0 - 1.0)
    pub max_block_rate: f64,
    /// Maximum acceptable latency in nanoseconds
    pub max_latency_ns: u64,
    /// Traffic spike threshold (multiplier of baseline)
    pub spike_multiplier: f64,
    /// Traffic drop threshold (multiplier of baseline)
    pub drop_multiplier: f64,
}

impl Default for AnomalyThresholds {
    fn default() -> Self {
        Self {
            max_pps: 100000.0,       // 100k packets per second
            max_block_rate: 0.5,     // 50% block rate
            max_latency_ns: 1000000, // 1ms in nanoseconds
            spike_multiplier: 3.0,   // 3x baseline
            drop_multiplier: 0.3,    // 30% of baseline
        }
    }
}

impl DataAggregator {
    /// Create a new DataAggregator with the specified window size
    pub fn new(window_size: Duration) -> Self {
        info!("Creating DataAggregator with window size {:?}", window_size);
        Self {
            window_size,
            buffer: VecDeque::new(),
            baseline_pps: None,
            thresholds: AnomalyThresholds::default(),
        }
    }

    /// Create a new DataAggregator with custom thresholds
    pub fn with_thresholds(window_size: Duration, thresholds: AnomalyThresholds) -> Self {
        info!(
            "Creating DataAggregator with window size {:?} and custom thresholds",
            window_size
        );
        Self {
            window_size,
            buffer: VecDeque::new(),
            baseline_pps: None,
            thresholds,
        }
    }

    /// Add a new data point to the aggregator
    pub fn add_data_point(&mut self, point: DataPoint) {
        debug!(
            "Adding data point: processed={}, blocked={}",
            point.packets_processed, point.packets_blocked
        );

        // Remove old data points outside the window
        let cutoff = point.timestamp.saturating_sub(self.window_size);
        while let Some(front) = self.buffer.front() {
            if front.timestamp < cutoff {
                self.buffer.pop_front();
            } else {
                break;
            }
        }

        // Add the new point
        self.buffer.push_back(point);
    }

    /// Get aggregated statistics for the current window
    pub fn get_aggregated(&self) -> AggregatedStats {
        if self.buffer.is_empty() {
            return AggregatedStats::default();
        }

        let total_packets: u64 = self.buffer.iter().map(|p| p.packets_processed).sum();
        let total_blocked: u64 = self.buffer.iter().map(|p| p.packets_blocked).sum();

        // Calculate window duration
        let first_ts = self
            .buffer
            .front()
            .map(|p| p.timestamp)
            .unwrap_or_else(Instant::now);
        let last_ts = self
            .buffer
            .back()
            .map(|p| p.timestamp)
            .unwrap_or_else(Instant::now);
        let window_duration = last_ts.duration_since(first_ts);

        // Calculate packets per second
        let packets_per_second = if window_duration.as_secs_f64() > 0.0 {
            total_packets as f64 / window_duration.as_secs_f64()
        } else {
            0.0
        };

        // Calculate block rate
        let block_rate = if total_packets > 0 {
            total_blocked as f64 / total_packets as f64
        } else {
            0.0
        };

        // Calculate average latency
        let total_latency: u64 = self.buffer.iter().map(|p| p.processing_time_ns).sum();
        let avg_latency_ns = if !self.buffer.is_empty() {
            total_latency as f64 / self.buffer.len() as f64
        } else {
            0.0
        };

        // Calculate P99 latency
        let p99_latency_ns = self.calculate_p99_latency();

        AggregatedStats {
            packets_per_second,
            block_rate,
            avg_latency_ns,
            p99_latency_ns,
            total_packets,
            total_blocked,
            window_duration,
            timestamp: Instant::now(),
        }
    }

    /// Detect anomalies based on current aggregated statistics
    pub fn detect_anomaly(&self) -> Option<Anomaly> {
        let stats = self.get_aggregated();

        // Check for high packet rate
        if stats.packets_per_second > self.thresholds.max_pps {
            return Some(Anomaly::HighPacketRate {
                rate: stats.packets_per_second,
                threshold: self.thresholds.max_pps,
            });
        }

        // Check for high block rate
        if stats.block_rate > self.thresholds.max_block_rate {
            return Some(Anomaly::HighBlockRate {
                rate: stats.block_rate,
                threshold: self.thresholds.max_block_rate,
            });
        }

        // Check for high latency
        if stats.p99_latency_ns > self.thresholds.max_latency_ns {
            return Some(Anomaly::HighLatency {
                latency_ns: stats.p99_latency_ns,
                threshold_ns: self.thresholds.max_latency_ns,
            });
        }

        // Check for traffic spikes/drops if we have a baseline
        if let Some(baseline) = self.baseline_pps {
            if stats.packets_per_second > baseline * self.thresholds.spike_multiplier {
                return Some(Anomaly::TrafficSpike {
                    current: stats.total_packets,
                    baseline,
                });
            }

            if stats.packets_per_second < baseline * self.thresholds.drop_multiplier
                && baseline > 0.0
            {
                return Some(Anomaly::TrafficDrop {
                    current: stats.total_packets,
                    baseline,
                });
            }
        }

        None
    }

    /// Set the baseline for anomaly detection
    pub fn set_baseline(&mut self, baseline_pps: f64) {
        info!("Setting baseline PPS to {}", baseline_pps);
        self.baseline_pps = Some(baseline_pps);
    }

    /// Update anomaly thresholds
    pub fn set_thresholds(&mut self, thresholds: AnomalyThresholds) {
        info!("Updating anomaly thresholds: {:?}", thresholds);
        self.thresholds = thresholds;
    }

    /// Get the current thresholds
    pub fn thresholds(&self) -> &AnomalyThresholds {
        &self.thresholds
    }

    /// Get the number of data points in the buffer
    pub fn data_point_count(&self) -> usize {
        self.buffer.len()
    }

    /// Clear all data points
    pub fn clear(&mut self) {
        info!("Clearing all data points from aggregator");
        self.buffer.clear();
    }

    /// Calculate P99 latency from the current buffer
    fn calculate_p99_latency(&self) -> u64 {
        if self.buffer.is_empty() {
            return 0;
        }

        let mut latencies: Vec<u64> = self.buffer.iter().map(|p| p.processing_time_ns).collect();

        if latencies.is_empty() {
            return 0;
        }

        // Sort latencies to find P99
        latencies.sort_unstable();

        // Calculate P99 index (99th percentile)
        let p99_index = ((latencies.len() as f64) * 0.99) as usize;
        let p99_index = p99_index.min(latencies.len() - 1);

        latencies[p99_index]
    }

    /// Get the window size
    pub fn window_size(&self) -> Duration {
        self.window_size
    }
}

impl Default for DataAggregator {
    fn default() -> Self {
        Self::new(Duration::from_secs(60))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_data_point(packets: u64, blocked: u64, latency: u64) -> DataPoint {
        DataPoint {
            timestamp: Instant::now(),
            packets_processed: packets,
            packets_blocked: blocked,
            processing_time_ns: latency,
        }
    }

    #[test]
    fn test_aggregator_new() {
        let window = Duration::from_secs(60);
        let aggregator = DataAggregator::new(window);
        assert_eq!(aggregator.window_size(), window);
        assert_eq!(aggregator.data_point_count(), 0);
    }

    #[test]
    fn test_add_data_point() {
        let mut aggregator = DataAggregator::new(Duration::from_secs(60));
        let point = create_test_data_point(100, 10, 1000);

        aggregator.add_data_point(point);
        assert_eq!(aggregator.data_point_count(), 1);
    }

    #[test]
    fn test_get_aggregated_empty() {
        let aggregator = DataAggregator::new(Duration::from_secs(60));
        let stats = aggregator.get_aggregated();

        assert_eq!(stats.packets_per_second, 0.0);
        assert_eq!(stats.block_rate, 0.0);
        assert_eq!(stats.avg_latency_ns, 0.0);
        assert_eq!(stats.p99_latency_ns, 0);
    }

    #[test]
    fn test_get_aggregated_with_data() {
        let mut aggregator = DataAggregator::new(Duration::from_secs(60));

        // Add multiple data points
        for i in 0..10 {
            let point = create_test_data_point(100 * (i + 1) as u64, 10, 1000);
            aggregator.add_data_point(point);
        }

        let stats = aggregator.get_aggregated();
        assert!(stats.total_packets > 0);
        assert!(stats.packets_per_second >= 0.0);
    }

    #[test]
    fn test_detect_anomaly_high_packet_rate() {
        let thresholds = AnomalyThresholds {
            max_pps: 100.0,
            max_block_rate: 0.5,
            max_latency_ns: 1000000,
            spike_multiplier: 3.0,
            drop_multiplier: 0.3,
        };

        let mut aggregator = DataAggregator::with_thresholds(Duration::from_secs(60), thresholds);

        // Add data points that exceed the threshold
        for _ in 0..5 {
            let point = create_test_data_point(1000, 0, 100);
            aggregator.add_data_point(point);
        }

        let anomaly = aggregator.detect_anomaly();
        assert!(matches!(anomaly, Some(Anomaly::HighPacketRate { .. })));
    }

    #[test]
    fn test_detect_anomaly_high_block_rate() {
        let thresholds = AnomalyThresholds {
            max_pps: 100000.0,
            max_block_rate: 0.1, // 10%
            max_latency_ns: 1000000,
            spike_multiplier: 3.0,
            drop_multiplier: 0.3,
        };

        let mut aggregator = DataAggregator::with_thresholds(Duration::from_secs(60), thresholds);

        // Add data points with high block rate (50%)
        for _ in 0..5 {
            let point = create_test_data_point(100, 50, 100);
            aggregator.add_data_point(point);
        }

        let anomaly = aggregator.detect_anomaly();
        assert!(matches!(anomaly, Some(Anomaly::HighBlockRate { .. })));
    }

    #[test]
    fn test_detect_anomaly_high_latency() {
        let thresholds = AnomalyThresholds {
            max_pps: 100000.0,
            max_block_rate: 0.5,
            max_latency_ns: 500, // Very low threshold
            spike_multiplier: 3.0,
            drop_multiplier: 0.3,
        };

        let mut aggregator = DataAggregator::with_thresholds(Duration::from_secs(60), thresholds);

        // Add data points with high latency
        for _ in 0..5 {
            let point = create_test_data_point(100, 0, 1000);
            aggregator.add_data_point(point);
        }

        let anomaly = aggregator.detect_anomaly();
        assert!(matches!(anomaly, Some(Anomaly::HighLatency { .. })));
    }

    #[test]
    fn test_detect_anomaly_traffic_spike() {
        let thresholds = AnomalyThresholds {
            max_pps: 100000.0,
            max_block_rate: 0.5,
            max_latency_ns: 1000000,
            spike_multiplier: 2.0,
            drop_multiplier: 0.3,
        };

        let mut aggregator = DataAggregator::with_thresholds(Duration::from_secs(60), thresholds);
        aggregator.set_baseline(100.0); // 100 pps baseline

        // Add data points that exceed spike threshold (2x baseline = 200 pps)
        for _ in 0..5 {
            let point = create_test_data_point(1000, 0, 100);
            aggregator.add_data_point(point);
        }

        let anomaly = aggregator.detect_anomaly();
        assert!(matches!(anomaly, Some(Anomaly::TrafficSpike { .. })));
    }

    #[test]
    fn test_clear() {
        let mut aggregator = DataAggregator::new(Duration::from_secs(60));

        for i in 0..10 {
            let point = create_test_data_point(100 * (i + 1) as u64, 10, 1000);
            aggregator.add_data_point(point);
        }

        assert_eq!(aggregator.data_point_count(), 10);
        aggregator.clear();
        assert_eq!(aggregator.data_point_count(), 0);
    }

    #[test]
    fn test_p99_latency_calculation() {
        let mut aggregator = DataAggregator::new(Duration::from_secs(60));

        // Add 100 data points with varying latencies
        for i in 0..100 {
            let point = create_test_data_point(100, 0, (i * 100) as u64);
            aggregator.add_data_point(point);
        }

        let stats = aggregator.get_aggregated();
        // P99 of 0, 100, 200, ..., 9900 should be around 9900
        assert!(stats.p99_latency_ns >= 9800);
    }
}
