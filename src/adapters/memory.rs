//! In-memory cache adapter.

use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::Duration;
use lru::LruCache;

use crate::domain::{
    policy::{EvictionPolicy, LruPolicy},
    Cache, CacheKey, CacheValue, Entry,
};

/// In-memory cache implementation.
pub struct InMemoryCache {
    cache: Arc<RwLock<LruCache<CacheKey, Entry>>>,
    policy: Arc<RwLock<LruPolicy>>,
    max_capacity: usize,
}

impl InMemoryCache {
    pub fn new(max_capacity: usize) -> Self {
        let capacity =
            NonZeroUsize::new(max_capacity).expect("max_capacity must be greater than zero");
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            policy: Arc::new(RwLock::new(LruPolicy::new())),
            max_capacity,
        }
    }

    pub fn with_ttl(self, _ttl: Duration) -> Self {
        // TTL support would require additional tracking
        self
    }
}

#[async_trait]
impl Cache for InMemoryCache {
    async fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                cache.pop(key);
                policy.remove(key.as_str());
                return Ok(None);
            }

            entry.touch();
            policy.record_access(key.as_str());
            Ok(Some(entry.value.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: CacheKey, value: CacheValue) -> Result<(), String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        // Evict if necessary
        while cache.len() >= self.max_capacity {
            if let Some(evict_key) = policy.select_eviction() {
                let eviction_key = CacheKey::from(evict_key.clone());
                cache.pop(&eviction_key);
                policy.remove(evict_key.as_str());
            } else {
                break;
            }
        }

        let entry = Entry::new(key.clone(), value);
        cache.push(key.clone(), entry);
        policy.record_access(key.as_str());

        Ok(())
    }

    async fn remove(&self, key: &CacheKey) -> Result<(), String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        cache.pop(key);
        policy.remove(key.as_str());

        Ok(())
    }

    async fn contains(&self, key: &CacheKey) -> Result<bool, String> {
        let cache = self.cache.read().map_err(|e| e.to_string())?;
        Ok(cache.contains(key))
    }

    async fn clear(&self) -> Result<(), String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        cache.clear();
        policy.clear();

        Ok(())
    }

    async fn len(&self) -> Result<usize, String> {
        let cache = self.cache.read().map_err(|e| e.to_string())?;
        Ok(cache.len())
    }

    async fn is_empty(&self) -> Result<bool, String> {
        let cache = self.cache.read().map_err(|e| e.to_string())?;
        Ok(cache.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_operations() {
        let cache = InMemoryCache::new(100);

        let key = CacheKey::from("test");
        let value = CacheValue::serialize(&"hello".to_string()).unwrap();

        cache.set(key.clone(), value).await.unwrap();
        let result = cache.get(&key).await.unwrap();

        assert!(result.is_some());
        let value: String = result.unwrap().deserialize().unwrap();
        assert_eq!(value, "hello");
    }

    #[tokio::test]
    async fn test_eviction() {
        let cache = InMemoryCache::new(2);

        for i in 0..3 {
            let key = CacheKey::from(format!("key{}", i));
            let value = CacheValue::serialize(&i).unwrap();
            cache.set(key, value).await.unwrap();
        }

        // First key should be evicted
        let key0 = CacheKey::from("key0");
        let result = cache.get(&key0).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = InMemoryCache::new(100);

        let key = CacheKey::from("test");
        let value = CacheValue::serialize(&"hello".to_string()).unwrap();

        cache.set(key.clone(), value).await.unwrap();
        cache.remove(&key).await.unwrap();

        let result = cache.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_concurrent_reads() {
        let cache = Arc::new(InMemoryCache::new(100));
        let num_tasks: usize = 20;
        let mut handles = Vec::with_capacity(num_tasks);

        // Populate cache with a shared key
        let setup_key = CacheKey::from("shared");
        let val = CacheValue::serialize(&"concurrent-value".to_string()).unwrap();
        cache.set(setup_key.clone(), val).await.unwrap();

        // Spawn concurrent readers
        for i in 0..num_tasks {
            let c = Arc::clone(&cache);
            let k = setup_key.clone();
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("task-{}", i));
                let v = CacheValue::serialize(&i).unwrap();
                // Every task does a set then a get on both its own key and the shared key
                c.set(key.clone(), v).await.unwrap();
                let result = c.get(&k).await.unwrap();
                assert!(result.is_some(), "shared key should be visible from task {}", i);
                let own = c.get(&key).await.unwrap();
                assert!(own.is_some(), "own key should be visible from task {}", i);
            }));
        }

        for handle in handles {
            handle.await.expect("concurrent task should not panic");
        }

        // Verify all entries are accounted for
        let len = cache.len().await.unwrap();
        assert_eq!(len, num_tasks + 1, "all keys plus shared key should be present");
    }

    #[tokio::test]
    async fn test_concurrent_eviction() {
        // Small cache to force frequent eviction under concurrency
        let cache = Arc::new(InMemoryCache::new(10));
        let num_tasks: usize = 50;
        let mut handles = Vec::with_capacity(num_tasks);

        for i in 0..num_tasks {
            let c = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("key-{}", i));
                let v = CacheValue::serialize(&(i as i32)).unwrap();
                c.set(key.clone(), v).await.unwrap();

                // Verify our write is visible (or was evicted — either is fine)
                let result = c.get(&key).await.unwrap();
                if let Some(val) = result {
                    let decoded: i32 = val.deserialize().unwrap();
                    assert_eq!(decoded, i as i32, "value should match what we wrote");
                }
                // If None, it was evicted — acceptable under capacity pressure
            }));
        }

        for handle in handles {
            handle.await.expect("concurrent eviction task should not panic");
        }

        // Cache should not exceed its capacity
        let len = cache.len().await.unwrap();
        assert!(len <= 10, "cache should not exceed capacity (got {})", len);
    }

    #[tokio::test]
    async fn test_concurrent_clear_and_read() {
        let cache = Arc::new(InMemoryCache::new(100));

        // Pre-populate
        for i in 0..50 {
            let key = CacheKey::from(format!("pre-{}", i));
            let v = CacheValue::serialize(&i).unwrap();
            cache.set(key, v).await.unwrap();
        }

        let mut handles = Vec::new();

        // Concurrent clears and reads
        for _ in 0..10 {
            let c = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                // This may race with another clear — that's intentional
                let _ = c.clear().await;
            }));

            let c = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let k = CacheKey::from("pre-0");
                let _ = c.get(&k).await;
            }));
        }

        for handle in handles {
            handle.await.expect("concurrent clear/read task should not panic");
        }

        // Final state should be valid — either empty or some remaining entries
        let len = cache.len().await.unwrap_or(0);
        assert!(len <= 50, "after clears, cache should not exceed original count");
    }
}
