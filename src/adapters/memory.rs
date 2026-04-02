//! In-memory cache adapter.

use std::sync::{Arc, RwLock};
use std::num::NonZeroUsize;
use async_trait::async_trait;
use lru::LruCache;
use crate::domain::{
    Cache, CacheKey, CacheValue, Entry,
    policy::{EvictionPolicy, LruPolicy},
};
use chrono::Duration;

/// In-memory cache implementation.
pub struct InMemoryCache {
    cache: Arc<RwLock<LruCache<CacheKey, Entry>>>,
    policy: Arc<RwLock<LruPolicy>>,
    max_capacity: usize,
}

impl InMemoryCache {
    pub fn new(max_capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(max_capacity)
            .expect("max_capacity must be greater than zero");
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
}
