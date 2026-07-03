//! In-memory cache adapter.

use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::Duration;
use lru::LruCache;
use tracing::{debug, instrument};

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
    #[instrument(skip(self), fields(key = %key))]
    async fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        if let Some(entry) = cache.get_mut(key) {
            if entry.is_expired() {
                debug!(key = %key, "cache miss — entry expired, evicting");
                cache.pop(key);
                policy.remove(key.as_str());
                return Ok(None);
            }

            entry.touch();
            policy.record_access(key.as_str());
            debug!(key = %key, "cache hit");
            Ok(Some(entry.value.clone()))
        } else {
            debug!(key = %key, "cache miss");
            Ok(None)
        }
    }

    #[instrument(skip(self, value), fields(key = %key))]
    async fn set(&self, key: CacheKey, value: CacheValue) -> Result<(), String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        // Evict if necessary to stay within capacity.
        while cache.len() >= self.max_capacity {
            if let Some(evict_key) = policy.select_eviction() {
                debug!(evict_key = %evict_key, "evicting LRU entry to make room");
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
        debug!(key = %key, "cache set");

        Ok(())
    }

    #[instrument(skip(self), fields(key = %key))]
    async fn remove(&self, key: &CacheKey) -> Result<(), String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        cache.pop(key);
        policy.remove(key.as_str());
        debug!(key = %key, "cache remove");

        Ok(())
    }

    async fn contains(&self, key: &CacheKey) -> Result<bool, String> {
        let cache = self.cache.read().map_err(|e| e.to_string())?;
        Ok(cache.contains(key))
    }

    #[instrument(skip(self))]
    async fn clear(&self) -> Result<(), String> {
        let mut cache = self.cache.write().map_err(|e| e.to_string())?;
        let mut policy = self.policy.write().map_err(|e| e.to_string())?;

        let prev_len = cache.len();
        cache.clear();
        policy.clear();
        debug!(entries_removed = prev_len, "cache cleared");

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
        let cache = Arc::new(InMemoryCache::new(1000));
        let mut handles = Vec::new();

        // Populate cache
        for i in 0..100 {
            let key = CacheKey::from(format!("key{}", i));
            let value = CacheValue::serialize(&i).unwrap();
            cache.set(key, value).await.unwrap();
        }

        // Spawn 10 concurrent readers
        for _ in 0..10 {
            let cache = cache.clone();
            handles.push(tokio::spawn(async move {
                for i in 0..100 {
                    let key = CacheKey::from(format!("key{}", i));
                    let result = cache.get(&key).await.unwrap();
                    assert!(result.is_some(), "key{} should exist", i);
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let cache = Arc::new(InMemoryCache::new(1000));
        let mut handles = Vec::new();

        // Spawn 20 concurrent writers
        for i in 0..20 {
            let cache = cache.clone();
            handles.push(tokio::spawn(async move {
                for j in 0..50 {
                    let key = CacheKey::from(format!("writer{}-{}", i, j));
                    let value = CacheValue::serialize(&j).unwrap();
                    cache.set(key, value).await.unwrap();
                }
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert!(cache.len().await.unwrap() > 0);

        // Verify written values
        for i in 0..20 {
            for j in (0..50).step_by(10) {
                let key = CacheKey::from(format!("writer{}-{}", i, j));
                let result = cache.get(&key).await.unwrap();
                assert!(result.is_some(), "writer{}-{} should exist", i, j);
                let value: i32 = result.unwrap().deserialize().unwrap();
                assert_eq!(value, j);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrent_read_write_mixed() {
        let cache = Arc::new(InMemoryCache::new(500));
        let mut handles = Vec::new();

        // Seed some data
        for i in 0..200 {
            let key = CacheKey::from(format!("seed{}", i));
            let value = CacheValue::serialize(&i).unwrap();
            cache.set(key, value).await.unwrap();
        }

        // Readers
        let cache_r = cache.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..200 {
                let key = CacheKey::from(format!("seed{}", i));
                let result = cache_r.get(&key).await.unwrap();
                assert!(result.is_some());
            }
        }));

        // Writers
        let cache_w = cache.clone();
        handles.push(tokio::spawn(async move {
            for i in 0..100 {
                let key = CacheKey::from(format!("mixed{}", i));
                let value = CacheValue::serialize(&i).unwrap();
                cache_w.set(key, value).await.unwrap();
            }
        }));

        // Remover
        let cache_d = cache.clone();
        handles.push(tokio::spawn(async move {
            // Remove even-numbered seeds
            for i in (0..200).step_by(2) {
                let key = CacheKey::from(format!("seed{}", i));
                let _ = cache_d.remove(&key).await;
            }
        }));

        for h in handles {
            h.await.unwrap();
        }
    }
}
