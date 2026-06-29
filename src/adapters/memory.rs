//! In-memory cache adapter.

use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::Duration;
use lru::LruCache;

use crate::domain::{
    errors::{CacheError, ErrorCode, RecoveryHint},
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
    async fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, CacheError> {
        let mut cache = self.cache.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        let mut policy = self.policy.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;

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

    async fn set(&self, key: CacheKey, value: CacheValue) -> Result<(), CacheError> {
        let mut cache = self.cache.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        let mut policy = self.policy.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;

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

    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        let mut cache = self.cache.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        let mut policy = self.policy.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;

        cache.pop(key);
        policy.remove(key.as_str());

        Ok(())
    }

    async fn contains(&self, key: &CacheKey) -> Result<bool, CacheError> {
        let cache = self.cache.read().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        Ok(cache.contains(key))
    }

    async fn clear(&self) -> Result<(), CacheError> {
        let mut cache = self.cache.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        let mut policy = self.policy.write().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;

        cache.clear();
        policy.clear();

        Ok(())
    }

    async fn len(&self) -> Result<usize, CacheError> {
        let cache = self.cache.read().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        Ok(cache.len())
    }

    async fn is_empty(&self) -> Result<bool, CacheError> {
        let cache = self.cache.read().map_err(|e| CacheError::LockPoisoned {
            message: e.to_string(),
            code: ErrorCode::LockPoisoned,
            hint: RecoveryHint::Retry,
        })?;
        Ok(cache.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
        let key = CacheKey::from("concurrent_key");
        let value = CacheValue::serialize(&42i32).unwrap();
        cache.set(key.clone(), value).await.unwrap();

        let mut handles = Vec::new();
        for _ in 0..10 {
            let cache = Arc::clone(&cache);
            let key = key.clone();
            handles.push(tokio::spawn(async move {
                let result = cache.get(&key).await.unwrap();
                assert!(result.is_some());
                let val: i32 = result.unwrap().deserialize().unwrap();
                assert_eq!(val, 42);
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let cache = Arc::new(InMemoryCache::new(100));
        let mut handles = Vec::new();

        for i in 0..20 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("key_{}", i));
                let value = CacheValue::serialize(&i).unwrap();
                cache.set(key, value).await.unwrap();
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all keys were written
        let cache_ref = &*cache;
        for i in 0..20 {
            let key = CacheKey::from(format!("key_{}", i));
            let result = cache_ref.get(&key).await.unwrap();
            assert!(result.is_some(), "key_{} should exist", i);
            let val: i32 = result.unwrap().deserialize().unwrap();
            assert_eq!(val, i);
        }
    }

    #[tokio::test]
    async fn test_concurrent_read_write() {
        let cache = Arc::new(InMemoryCache::new(100));

        // Pre-populate half the keys
        for i in 0..10 {
            let key = CacheKey::from(format!("rw_{}", i));
            let value = CacheValue::serialize(&i).unwrap();
            cache.set(key, value).await.unwrap();
        }

        let mut handles = Vec::new();

        // Writers
        for i in 10..20 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("rw_{}", i));
                let value = CacheValue::serialize(&i).unwrap();
                cache.set(key, value).await.unwrap();
            }));
        }

        // Readers
        for i in 0..10 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("rw_{}", i));
                let result = cache.get(&key).await.unwrap();
                if let Some(val) = result {
                    let decoded: i32 = val.deserialize().unwrap();
                    assert_eq!(decoded, i);
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all keys exist
        for i in 0..20 {
            let key = CacheKey::from(format!("rw_{}", i));
            let result = cache.get(&key).await.unwrap();
            assert!(result.is_some(), "key rw_{} should exist after concurrent ops", i);
        }
    }

    #[tokio::test]
    async fn test_concurrent_remove_and_read() {
        let cache = Arc::new(InMemoryCache::new(100));

        // Pre-populate
        for i in 0..10 {
            let key = CacheKey::from(format!("cr_{}", i));
            let value = CacheValue::serialize(&i).unwrap();
            cache.set(key, value).await.unwrap();
        }

        let mut handles = Vec::new();

        // Removers
        for i in 0..5 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("cr_{}", i));
                cache.remove(&key).await.unwrap();
            }));
        }

        // Readers
        for i in 0..10 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("cr_{}", i));
                // It's OK if the key was removed or not — just don't panic
                let _ = cache.get(&key).await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Removed keys should be gone
        for i in 0..5 {
            let key = CacheKey::from(format!("cr_{}", i));
            assert!(cache.get(&key).await.unwrap().is_none());
        }
    }

    #[tokio::test]
    async fn test_eviction_under_concurrency() {
        let cache = Arc::new(InMemoryCache::new(5));
        let mut handles = Vec::new();

        for batch in 0..10 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                let start = batch * 100;
                for i in start..start + 100 {
                    let key = CacheKey::from(format!("ek_{}", i));
                    let value = CacheValue::serialize(&i).unwrap();
                    cache.set(key, value).await.unwrap();
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Cache should be at capacity
        let size = cache.len().await.unwrap();
        assert!(size <= 5, "cache should not exceed capacity, got {}", size);
    }

    #[tokio::test]
    async fn test_cache_size_consistency() {
        let cache = Arc::new(InMemoryCache::new(1000));
        let mut handles = Vec::new();

        // 50 concurrent set + remove operations
        for i in 0..50 {
            let cache = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                for j in 0..10 {
                    let key = CacheKey::from(format!("sc_{}_{}", i, j));
                    let value = CacheValue::serialize(&j).unwrap();
                    cache.set(key.clone(), value).await.unwrap();
                    if j % 2 == 0 {
                        cache.remove(&key).await.unwrap();
                    }
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Verify size is consistent with what we expect
        // 50 tasks * 10 ops each, with half removed = 250 remaining
        let size = cache.len().await.unwrap();
        assert_eq!(size, 250, "expected 250 entries, got {}", size);
    }

    #[tokio::test]
    async fn test_concurrent_clear() {
        let cache = Arc::new(InMemoryCache::new(100));

        // Pre-populate
        for i in 0..50 {
            let key = CacheKey::from(format!("cc_{}", i));
            let value = CacheValue::serialize(&i).unwrap();
            cache.set(key, value).await.unwrap();
        }

        // Concurrent clear + operations
        let cache_clone = Arc::clone(&cache);
        let clear_handle = tokio::spawn(async move {
            cache_clone.clear().await.unwrap();
        });

        let mut op_handles = Vec::new();
        for i in 50..60 {
            let cache = Arc::clone(&cache);
            op_handles.push(tokio::spawn(async move {
                let key = CacheKey::from(format!("cc_{}", i));
                let value = CacheValue::serialize(&i).unwrap();
                let _ = cache.set(key, value).await;
            }));
        }

        clear_handle.await.unwrap();
        for handle in op_handles {
            handle.await.unwrap();
        }

        // After clear + concurrent sets, cache should be consistent
        let size = cache.len().await.unwrap();
        assert!(size <= 10, "expected at most 10 entries after clear, got {}", size);
    }
}
