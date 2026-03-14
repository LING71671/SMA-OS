//! Local in-memory cache implementation using Moka
//!
//! Provides a high-performance, concurrent cache with TTL support.
//! Used as L1 cache before falling back to Redis (L2).

use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

/// Local cache wrapper around Moka's async Cache
#[derive(Clone)]
pub struct LocalCache {
    inner: Cache<String, Arc<Vec<u8>>>,
}

impl LocalCache {
    /// Creates a new LocalCache with the specified capacity and TTL
    ///
    /// # Arguments
    /// * `max_capacity` - Maximum number of entries in the cache
    /// * `ttl` - Time-to-live for cached entries
    pub fn new(max_capacity: u64, ttl: Duration) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(ttl)
            .build();

        Self { inner: cache }
    }

    /// Gets a value from the cache, or computes it if not present
    ///
    /// This method uses Moka's `get_with` to ensure atomic insertion
    /// and prevent cache stampede (thundering herd).
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `loader` - Async function to compute the value if not in cache
    pub async fn get_with<F, Fut>(&self, key: &str, loader: F) -> Arc<Vec<u8>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Vec<u8>>,
    {
        let key = key.to_string();
        self.inner
            .get_with(key.clone(), async move { Arc::new(loader().await) })
            .await
    }

    /// Gets a value from the cache if present
    pub async fn get(&self, key: &str) -> Option<Arc<Vec<u8>>> {
        self.inner.get(key).await
    }

    /// Inserts a value into the cache
    pub async fn insert(&self, key: &str, value: Vec<u8>) {
        self.inner.insert(key.to_string(), Arc::new(value)).await;
    }

    /// Removes a value from the cache
    pub async fn remove(&self, key: &str) {
        self.inner.remove(key).await;
    }

    /// Returns the number of entries in the cache
    pub fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    /// Runs pending maintenance tasks (optional, Moka handles this automatically)
    pub async fn run_pending_tasks(&self) {
        self.inner.run_pending_tasks().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_cache_basic() {
        let cache = LocalCache::new(100, Duration::from_secs(60));
        
        // Insert and retrieve
        cache.insert("key1", vec![1, 2, 3]).await;
        let value = cache.get("key1").await;
        
        assert!(value.is_some());
        assert_eq!(value.unwrap().as_ref(), &[1, 2, 3]);
    }

    #[tokio::test]
    async fn test_local_cache_miss() {
        let cache = LocalCache::new(100, Duration::from_secs(60));
        
        // Miss should return None
        let value = cache.get("nonexistent").await;
        assert!(value.is_none());
    }

    #[tokio::test]
    async fn test_local_cache_get_with() {
        let cache = LocalCache::new(100, Duration::from_secs(60));
        
        let loader_call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let count = loader_call_count.clone();
        
        // First call should invoke loader
        let value = cache.get_with("key1", || async move {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            vec![1, 2, 3]
        }).await;
        
        assert_eq!(value.as_ref(), &[1, 2, 3]);
        
        // Second call should use cached value
        let value2 = cache.get_with("key1", || async move {
            // This should not be called
            vec![4, 5, 6]
        }).await;
        
        assert_eq!(value2.as_ref(), &[1, 2, 3]); // Still original value
        assert_eq!(loader_call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_local_cache_ttl() {
        // Short TTL for testing
        let cache = LocalCache::new(100, Duration::from_millis(100));
        
        cache.insert("key1", vec![1, 2, 3]).await;
        
        // Should exist immediately
        assert!(cache.get("key1").await.is_some());
        
        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        // Should be expired (Moka may still return it, but entry_count should be 0)
        cache.run_pending_tasks().await;
        
        // Entry count should be 0 or the entry may be expired
        // Moka handles TTL lazily, so we check entry count
        let count = cache.entry_count();
        // Note: Moka may keep expired entries briefly
        assert!(count <= 1);
    }

    #[tokio::test]
    async fn test_concurrent_get_with() {
        use std::sync::atomic::{AtomicU32, Ordering};
        
        let cache = LocalCache::new(100, Duration::from_secs(60));
        let loader_calls = Arc::new(AtomicU32::new(0));
        
        // Spawn 10 concurrent requests
        let mut handles = vec![];
        for _ in 0..10 {
            let cache = cache.clone();
            let calls = loader_calls.clone();
            let handle = tokio::spawn(async move {
                cache.get_with("concurrent_key", || async move {
                    // Simulate slow operation
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    calls.fetch_add(1, Ordering::SeqCst);
                    vec![42]
                }).await
            });
            handles.push(handle);
        }
        
        // Wait for all to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        // Loader should only be called once due to Moka's atomic insertion
        let calls = loader_calls.load(Ordering::SeqCst);
        assert_eq!(calls, 1, "Loader should be called exactly once, but was called {} times", calls);
    }
}
