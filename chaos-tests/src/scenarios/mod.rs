//! Chaos Test Scenarios
//!
//! This module contains various chaos test scenarios:
//! - Node failure: Kill/restart containers
//! - Network partition: Isolate services
//! - Resource exhaustion: Consume CPU/memory

pub mod node_failure;
pub mod network_partition;
pub mod resource_exhaustion;
