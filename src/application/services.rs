//! Cache application service.

use std::sync::Arc;

use crate::domain::errors::CacheError;
use crate::domain::{Cache, CacheKey, CacheValue};

/// Cache service with typed operations.
pub struct CacheService {
    cache: Arc<dyn Cache>,
}

impl CacheService {
    pub fn new(cache: Arc<dyn Cache>) -> Self {
        Self { cache }
    }

    /// Get a typed value from the cache.
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        key: &CacheKey,
    ) -> Result<Option<T>, CacheError> {
        match self.cache.get(key).await {
            Ok(Some(value)) => {
                let result = value.deserialize()?;
                Ok(Some(result))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(CacheError::BackendError(e)),
        }
    }

    /// Set a typed value in the cache.
    pub async fn set<T: serde::Serialize>(
        &self,
        key: CacheKey,
        value: &T,
    ) -> Result<(), CacheError> {
        let cache_value = CacheValue::serialize(value)?;
        self.cache.set(key, cache_value).await.map_err(CacheError::BackendError)
    }

    /// Remove a key from the cache.
    pub async fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        self.cache.remove(key).await.map_err(CacheError::BackendError)
    }

    /// Check if a key exists.
    pub async fn contains(&self, key: &CacheKey) -> Result<bool, CacheError> {
        self.cache.contains(key).await.map_err(CacheError::BackendError)
    }

    /// Get cache size.
    pub async fn len(&self) -> Result<usize, CacheError> {
        self.cache.len().await.map_err(CacheError::BackendError)
    }

    /// Check whether the cache is empty.
    pub async fn is_empty(&self) -> Result<bool, CacheError> {
        self.len().await.map(|len| len == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::memory::InMemoryCache;

    #[tokio::test]
    async fn test_cache_service_get_set() {
        let cache = Arc::new(InMemoryCache::new(100)) as Arc<dyn Cache>;
        let service = CacheService::new(cache);

        let key = CacheKey::from("hello");
        service.set(key.clone(), &"world".to_string()).await.unwrap();

        let result: Option<String> = service.get(&key).await.unwrap();
        assert_eq!(result, Some("world".to_string()));
    }

    #[tokio::test]
    async fn test_cache_service_miss() {
        let cache = Arc::new(InMemoryCache::new(100)) as Arc<dyn Cache>;
        let service = CacheService::new(cache);

        let key = CacheKey::from("nonexistent");
        let result: Option<String> = service.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_service_remove() {
        let cache = Arc::new(InMemoryCache::new(100)) as Arc<dyn Cache>;
        let service = CacheService::new(cache);

        let key = CacheKey::from("remove-me");
        service.set(key.clone(), &"value".to_string()).await.unwrap();
        service.remove(&key).await.unwrap();

        let result: Option<String> = service.get(&key).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_service_contains() {
        let cache = Arc::new(InMemoryCache::new(100)) as Arc<dyn Cache>;
        let service = CacheService::new(cache);

        let key = CacheKey::from("exists");
        assert!(!service.contains(&key).await.unwrap());

        service.set(key.clone(), &"present".to_string()).await.unwrap();
        assert!(service.contains(&key).await.unwrap());
    }

    #[tokio::test]
    async fn test_cache_service_len_and_empty() {
        let cache = Arc::new(InMemoryCache::new(100)) as Arc<dyn Cache>;
        let service = CacheService::new(cache);

        assert!(service.is_empty().await.unwrap());
        assert_eq!(service.len().await.unwrap(), 0);

        service.set(CacheKey::from("a"), &1).await.unwrap();
        assert!(!service.is_empty().await.unwrap());
        assert_eq!(service.len().await.unwrap(), 1);
    }
}
