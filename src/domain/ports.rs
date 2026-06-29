//! Port definitions.

use async_trait::async_trait;

use super::errors::CacheError;
use super::{CacheKey, CacheValue};

/// Trait for cache implementations.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from the cache.
    async fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, CacheError>;

    /// Set a value in the cache.
    async fn set(&self, key: CacheKey, value: CacheValue) -> Result<(), CacheError>;

    /// Remove a value from the cache.
    async fn remove(&self, key: &CacheKey) -> Result<(), CacheError>;

    /// Check if a key exists.
    async fn contains(&self, key: &CacheKey) -> Result<bool, CacheError>;

    /// Clear all entries.
    async fn clear(&self) -> Result<(), CacheError>;

    /// Get the number of entries.
    async fn len(&self) -> Result<usize, CacheError>;

    /// Check if empty.
    async fn is_empty(&self) -> Result<bool, CacheError>;
}
