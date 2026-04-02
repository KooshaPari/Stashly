//! Port definitions.

use async_trait::async_trait;
use super::{CacheKey, CacheValue};

/// Trait for cache implementations.
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from the cache.
    async fn get(&self, key: &CacheKey) -> Result<Option<CacheValue>, String>;

    /// Set a value in the cache.
    async fn set(&self, key: CacheKey, value: CacheValue) -> Result<(), String>;

    /// Remove a value from the cache.
    async fn remove(&self, key: &CacheKey) -> Result<(), String>;

    /// Check if a key exists.
    async fn contains(&self, key: &CacheKey) -> Result<bool, String>;

    /// Clear all entries.
    async fn clear(&self) -> Result<(), String>;

    /// Get the number of entries.
    async fn len(&self) -> Result<usize, String>;

    /// Check if empty.
    async fn is_empty(&self) -> Result<bool, String>;
}
