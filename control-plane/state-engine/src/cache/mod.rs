//! Local cache module using Moka for high-performance in-memory caching
//!
//! This module provides a local L1 cache that sits in front of Redis (L2)
//! to reduce latency for frequently accessed data.

pub mod local;

pub use local::LocalCache;
