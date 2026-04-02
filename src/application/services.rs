//! Cache application service.

use std::sync::Arc;
use crate::domain::{Cache, CacheKey, CacheValue};
use crate::domain::errors::CacheError;

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
        self.cache.set(key, cache_value)
            .await
            .map_err(CacheError::BackendError)
    }

    /// Remove a key from the cache.
    pub async fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        self.cache.remove(key)
            .await
            .map_err(CacheError::BackendError)
    }

    /// Check if a key exists.
    pub async fn contains(&self, key: &CacheKey) -> Result<bool, CacheError> {
        self.cache.contains(key)
            .await
            .map_err(CacheError::BackendError)
    }

    /// Get cache size.
    pub async fn len(&self) -> Result<usize, CacheError> {
        self.cache.len()
            .await
            .map_err(CacheError::BackendError)
    }

    /// Check whether the cache is empty.
    pub async fn is_empty(&self) -> Result<bool, CacheError> {
        self.len().await.map(|len| len == 0)
    }
}
