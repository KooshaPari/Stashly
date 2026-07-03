//! # Domain Entities
//!
//! Core business objects with identity.
//!
//! ## DDD Entity Principles
//!
//! - Entities have unique identity (key)
//! - Equality based on identity, not attributes
//! - Mutable state managed through domain methods

use std::time::{Duration, Instant};

use super::value_objects::{CacheKey, CacheTier, CacheValue, Ttl};

/// Cache entry entity
///
/// Represents a cached value with metadata.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Cache key (unique identifier)
    pub key: CacheKey,
    /// Cached value
    pub value: CacheValue,
    /// Time-to-live duration
    pub ttl: Ttl,
    /// When the entry was created
    pub created_at: Instant,
    /// When the entry expires
    pub expires_at: Instant,
    /// Which tier(s) the entry is in
    pub tier: CacheTier,
    /// Hit count for LRU tracking
    pub hits: u64,
}

impl CacheEntry {
    /// Create a new cache entry with default TTL.
    pub fn new(key: CacheKey, value: CacheValue) -> Self {
        let ttl = Ttl::default();
        let now = Instant::now();
        Self {
            created_at: now,
            expires_at: now + ttl.as_duration(),
            tier: CacheTier::L1,
            hits: 0,
            key,
            value,
            ttl,
        }
    }

    /// Create a new cache entry with custom TTL.
    pub fn with_ttl(key: CacheKey, value: CacheValue, ttl: Ttl) -> Self {
        let now = Instant::now();
        Self {
            created_at: now,
            expires_at: now + ttl.as_duration(),
            tier: CacheTier::L1,
            hits: 0,
            key,
            value,
            ttl,
        }
    }

    /// Check if the entry has expired.
    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    /// Record a cache hit.
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Get the age of the entry.
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// Get remaining TTL.
    pub fn remaining_ttl(&self) -> Option<Duration> {
        let remaining = self.expires_at.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            None
        } else {
            Some(remaining)
        }
    }

    /// Promote entry to higher tier.
    pub fn promote(&mut self, tier: CacheTier) {
        self.tier = tier;
    }
}

/// Singleflight request entity
///
/// Represents an in-flight request that multiple callers can wait on.
#[derive(Debug, Clone)]
pub struct SingleflightRequest<T> {
    /// Unique key for this request
    pub key: String,
    /// The actual result once complete
    pub result: Option<T>,
    /// Number of waiters
    pub waiters: u32,
    /// When the request was created
    pub created_at: Instant,
    /// Error if the request failed
    pub error: Option<String>,
}

impl<T> SingleflightRequest<T> {
    /// Create a new singleflight request.
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), result: None, waiters: 1, created_at: Instant::now(), error: None }
    }

    /// Add a waiter.
    pub fn add_waiter(&mut self) {
        self.waiters += 1;
    }

    /// Remove a waiter.
    pub fn remove_waiter(&mut self) {
        self.waiters = self.waiters.saturating_sub(1);
    }

    /// Set the result.
    pub fn set_result(&mut self, result: T) {
        self.result = Some(result);
    }

    /// Set an error.
    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    /// Check if request is complete.
    pub fn is_complete(&self) -> bool {
        self.result.is_some() || self.error.is_some()
    }
}

/// Cross-process singleflight request
///
/// Represents a request shared across processes via shared memory.
#[derive(Debug, Clone)]
pub struct CrossProcessRequest {
    /// Unique request ID
    pub request_id: String,
    /// Request key
    pub key: String,
    /// Process ID that started the request
    pub owner_pid: u32,
    /// Number of waiting processes
    pub waiters: u32,
    /// Request start time
    pub started_at: Instant,
    /// Request status
    pub status: CrossProcessStatus,
}

/// Status of a cross-process request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrossProcessStatus {
    /// Request is in progress
    InProgress,
    /// Request completed successfully
    Completed,
    /// Request failed
    Failed,
    /// Request timed out
    TimedOut,
}

impl CrossProcessRequest {
    /// Create a new cross-process request.
    pub fn new(request_id: impl Into<String>, key: impl Into<String>, owner_pid: u32) -> Self {
        Self {
            request_id: request_id.into(),
            key: key.into(),
            owner_pid,
            waiters: 1,
            started_at: Instant::now(),
            status: CrossProcessStatus::InProgress,
        }
    }

    /// Check if request has timed out.
    pub fn is_timed_out(&self, timeout: Duration) -> bool {
        self.started_at.elapsed() > timeout && self.status == CrossProcessStatus::InProgress
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_entry_creation() {
        let key = CacheKey::new("test_key");
        let value = CacheValue::new("test_value");
        let entry = CacheEntry::new(key, value);
        assert!(!entry.is_expired());
        assert_eq!(entry.tier, CacheTier::L1);
        assert_eq!(entry.hits, 0);
    }

    #[test]
    fn test_cache_entry_expiration() {
        let key = CacheKey::new("test_key");
        let value = CacheValue::new("test_value");
        let ttl = Ttl::from_secs(0); // Immediate expiration
        let entry = CacheEntry::with_ttl(key, value, ttl);
        std::thread::sleep(Duration::from_millis(10));
        assert!(entry.is_expired());
    }

    #[test]
    fn test_cache_entry_hit_recording() {
        let key = CacheKey::new("test_key");
        let value = CacheValue::new("test_value");
        let mut entry = CacheEntry::new(key, value);
        entry.record_hit();
        entry.record_hit();
        assert_eq!(entry.hits, 2);
    }

    #[test]
    fn test_singleflight_request() {
        let mut req: SingleflightRequest<String> = SingleflightRequest::new("test_key");
        assert_eq!(req.waiters, 1);
        req.add_waiter();
        assert_eq!(req.waiters, 2);
        assert!(!req.is_complete());
        req.set_result("value".to_string());
        assert!(req.is_complete());
        assert_eq!(req.result, Some("value".to_string()));
    }

    #[test]
    fn test_cross_process_request_timeout() {
        let req = CrossProcessRequest::new("req1", "test_key", 1234);
        assert_eq!(req.status, CrossProcessStatus::InProgress);
        // Simulate timeout check
        std::thread::sleep(Duration::from_millis(100));
        assert!(req.is_timed_out(Duration::from_millis(50)));
        assert!(!req.is_timed_out(Duration::from_secs(1)));
    }
}
