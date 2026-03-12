//! Redis Cluster support for horizontal scaling
//! 
//! Provides distributed Redis operations across multiple nodes
//! for 1000+ concurrent agent support.

use redis::aio::MultiplexedConnection;
use redis::RedisError;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Configuration for Redis Cluster
#[derive(Debug, Clone)]
pub struct ClusterConfig {
    /// List of Redis node addresses (host:port)
    pub nodes: Vec<String>,
    /// Maximum number of retries for failed operations
    pub max_retries: u32,
    /// Connection timeout in seconds
    pub timeout_secs: u64,
    /// Pool size per node
    pub pool_size: usize,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            nodes: vec!["127.0.0.1:6379".to_string()],
            max_retries: 3,
            timeout_secs: 5,
            pool_size: 10,
        }
    }
}

/// Redis Cluster client with node-aware routing
pub struct RedisCluster {
    /// Node connections indexed by host:port
    nodes: Arc<RwLock<HashMap<String, MultiplexedConnection>>>,
    /// Configuration
    config: ClusterConfig,
    /// Healthy node cache (for quick routing)
    healthy_nodes: Arc<RwLock<Vec<String>>>,
}

impl RedisCluster {
    /// Create a new cluster client
    pub async fn new(config: ClusterConfig) -> Result<Self, RedisError> {
        let mut nodes = HashMap::new();
        let mut healthy = Vec::new();

        for node_addr in &config.nodes {
            match Self::connect_node(node_addr).await {
                Ok(conn) => {
                    nodes.insert(node_addr.clone(), conn);
                    healthy.push(node_addr.clone());
                    info!("[Cluster] Connected to Redis node: {}", node_addr);
                }
                Err(e) => {
                    warn!("[Cluster] Failed to connect to {}: {}", node_addr, e);
                }
            }
        }

        if nodes.is_empty() {
            return Err(RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to connect to any Redis node",
            )));
        }

        info!(
            "[Cluster] Initialized with {} nodes ({} healthy)",
            config.nodes.len(),
            nodes.len()
        );

        Ok(Self {
            nodes: Arc::new(RwLock::new(nodes)),
            config,
            healthy_nodes: Arc::new(RwLock::new(healthy)),
        })
    }

    /// Connect to a single Redis node
    async fn connect_node(addr: &str) -> Result<MultiplexedConnection, RedisError> {
        let client = redis::Client::open(format!("redis://{}/", addr))?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(conn)
    }

    /// Get connection for a specific key (consistent hashing)
    pub async fn get_connection(&self, key: &str) -> Result<(String, MultiplexedConnection), RedisError> {
        let healthy = self.healthy_nodes.read().await;
        
        if healthy.is_empty() {
            return Err(RedisError::from((
                redis::ErrorKind::IoError,
                "No healthy Redis nodes available",
            )));
        }

        // Simple consistent hashing
        let node_idx = Self::hash_key(key) % healthy.len();
        let node_addr = healthy[node_idx].clone();
        
        drop(healthy); // Release read lock

        let nodes = self.nodes.read().await;
        let conn = nodes
            .get(&node_addr)
            .ok_or_else(|| RedisError::from((
                redis::ErrorKind::IoError,
                "Node connection not found",
            )))?;

        Ok((node_addr, conn.clone()))
    }

    /// Hash a key to determine node (FNV-1a)
    fn hash_key(key: &str) -> usize {
        let mut hash: usize = 0x811c9dc5;
        for byte in key.bytes() {
            hash ^= byte as usize;
            hash = hash.wrapping_mul(0x01000193);
        }
        hash
    }

    /// Execute operation with retry logic
    pub async fn execute_with_retry<F, Fut, T>(
        &self,
        key: &str,
        mut operation: F,
    ) -> Result<T, RedisError>
    where
        F: FnMut(MultiplexedConnection) -> Fut,
        Fut: std::future::Future<Output = Result<T, RedisError>>,
    {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match self.get_connection(key).await {
                Ok((node_addr, conn)) => {
                    match operation(conn).await {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            warn!(
                                "[Cluster] Node {} operation failed (attempt {}/{}): {}",
                                node_addr,
                                attempt + 1,
                                self.config.max_retries,
                                e
                            );
                            last_error = Some(e);
                            self.mark_node_unhealthy(&node_addr).await;
                        }
                    }
                }
                Err(e) => {
                    error!("[Cluster] Failed to get connection: {}", e);
                    last_error = Some(e);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100 * ((attempt + 1) as u64))).await;
        }

        Err(last_error.unwrap_or_else(|| RedisError::from((
            redis::ErrorKind::IoError,
            "All retry attempts failed",
        ))))
    }

    /// Mark a node as unhealthy
    async fn mark_node_unhealthy(&self, addr: &str) {
        let mut healthy = self.healthy_nodes.write().await;
        healthy.retain(|n| n != addr);
        warn!("[Cluster] Marked node {} as unhealthy", addr);
    }

    /// Health check and reconnect
    pub async fn health_check(&self) {
        let mut nodes = self.nodes.write().await;
        let mut healthy = self.healthy_nodes.write().await;

        // Check existing connections
        let mut to_remove = Vec::new();
        for (addr, _) in nodes.iter() {
            match Self::connect_node(addr).await {
                Ok(_) => {
                    if !healthy.contains(addr) {
                        healthy.push(addr.clone());
                        info!("[Cluster] Node {} reconnected", addr);
                    }
                }
                Err(_) => {
                    to_remove.push(addr.clone());
                }
            }
        }

        for addr in to_remove {
            nodes.remove(&addr);
            healthy.retain(|h| h != &addr);
        }

        info!(
            "[Cluster] Health check complete: {}/{} nodes healthy",
            healthy.len(),
            nodes.len()
        );
    }

    /// Get cluster statistics
    pub async fn stats(&self) -> ClusterStats {
        let nodes = self.nodes.read().await;
        let healthy = self.healthy_nodes.read().await;
        
        ClusterStats {
            total_nodes: nodes.len(),
            healthy_nodes: healthy.len(),
            unhealthy_nodes: nodes.len() - healthy.len(),
        }
    }
}

/// Cluster statistics
#[derive(Debug)]
pub struct ClusterStats {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub unhealthy_nodes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hash_key_consistency() {
        let key = "test:tenant:namespace";
        let hash1 = RedisCluster::hash_key(key);
        let hash2 = RedisCluster::hash_key(key);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_distribution() {
        let nodes = vec!["node1", "node2", "node3"];
        let mut distribution: HashMap<usize, usize> = HashMap::new();

        for i in 0..1000 {
            let key = format!("key:{}", i);
            let idx = RedisCluster::hash_key(&key) % nodes.len();
            *distribution.entry(idx).or_insert(0) += 1;
        }

        // Should be roughly evenly distributed
        for (_, count) in &distribution {
            assert!(*count > 250, "Distribution too skewed: {:?}", distribution);
        }
    }
}
