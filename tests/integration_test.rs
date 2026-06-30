//! Integration tests for stashly.
//!
//! These tests exercise the public API of stashly from an external consumer
//! perspective, covering the full cache lifecycle, error handling, service
//! layer typed operations, and structured error envelopes.

use std::sync::Arc;

use stashly::{
    Cache, CacheKey, CacheService, CacheValue, Entry, ErrorCode, ErrorEnvelope, ErrorSeverity,
    InMemoryCache,
};

// ---------------------------------------------------------------------------
// Basic cache operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cache_set_and_get() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("hello");
    let val = CacheValue::serialize(&"world".to_string()).unwrap();

    cache.set(key.clone(), val).await.unwrap();
    let result = cache.get(&key).await.unwrap();
    assert!(result.is_some(), "value should be present after set");

    let decoded: String = result.unwrap().deserialize().unwrap();
    assert_eq!(decoded, "world");
}

#[tokio::test]
async fn test_cache_miss_returns_none() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("nonexistent");
    let result = cache.get(&key).await.unwrap();
    assert!(result.is_none(), "missing key should return None");
}

#[tokio::test]
async fn test_cache_contains() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("exists");
    let val = CacheValue::serialize(&true).unwrap();

    assert!(!cache.contains(&key).await.unwrap());
    cache.set(key.clone(), val).await.unwrap();
    assert!(cache.contains(&key).await.unwrap());
}

#[tokio::test]
async fn test_cache_remove() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("removable");
    let val = CacheValue::serialize(&"data".to_string()).unwrap();

    cache.set(key.clone(), val).await.unwrap();
    assert!(cache.contains(&key).await.unwrap());

    cache.remove(&key).await.unwrap();
    assert!(!cache.contains(&key).await.unwrap());
    assert!(cache.get(&key).await.unwrap().is_none());
}

#[tokio::test]
async fn test_cache_clear() {
    let cache = InMemoryCache::new(100);

    for i in 0..10 {
        let key = CacheKey::from(format!("k-{}", i));
        let val = CacheValue::serialize(&i).unwrap();
        cache.set(key, val).await.unwrap();
    }

    assert_eq!(cache.len().await.unwrap(), 10);
    assert!(!cache.is_empty().await.unwrap());

    cache.clear().await.unwrap();
    assert_eq!(cache.len().await.unwrap(), 0);
    assert!(cache.is_empty().await.unwrap());
}

// ---------------------------------------------------------------------------
// Cache eviction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cache_eviction_when_exceeding_capacity() {
    let cache = InMemoryCache::new(5);

    for i in 0..10 {
        let key = CacheKey::from(format!("key-{}", i));
        let val = CacheValue::serialize(&i).unwrap();
        cache.set(key, val).await.unwrap();
    }

    // Cache should never exceed its capacity
    let len = cache.len().await.unwrap();
    assert!(len <= 5, "cache should not exceed capacity (got {})", len);

    // At least some of the early keys should have been evicted
    let first_key = CacheKey::from("key-0");
    let result = cache.get(&first_key).await.unwrap();
    // key-0 was inserted first and should be long gone under LRU
    assert!(result.is_none(), "first inserted key should be evicted");
}

// ---------------------------------------------------------------------------
// CacheService typed operations
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_cache_service_get_set() {
    let inner = Arc::new(InMemoryCache::new(100));
    let service = CacheService::new(inner);
    let key = CacheKey::from("typed-key");

    service.set(key.clone(), &42_i32).await.unwrap();
    let result: Option<i32> = service.get(&key).await.unwrap();
    assert_eq!(result, Some(42));
}

#[tokio::test]
async fn test_cache_service_miss() {
    let inner = Arc::new(InMemoryCache::new(100));
    let service = CacheService::new(inner);
    let key = CacheKey::from("missing-typed");

    let result: Option<String> = service.get(&key).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_cache_service_complex_type() {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct User {
        id: u64,
        name: String,
        email: String,
    }

    let inner = Arc::new(InMemoryCache::new(100));
    let service = CacheService::new(inner);
    let key = CacheKey::from("user:1");
    let user = User { id: 1, name: "Alice".into(), email: "alice@example.com".into() };

    service.set(key.clone(), &user).await.unwrap();
    let result: Option<User> = service.get(&key).await.unwrap();
    assert_eq!(result, Some(user));
}

#[tokio::test]
async fn test_cache_service_remove() {
    let inner = Arc::new(InMemoryCache::new(100));
    let service = CacheService::new(inner);
    let key = CacheKey::from("to-remove");

    service.set(key.clone(), &"value".to_string()).await.unwrap();
    assert!(service.contains(&key).await.unwrap());

    service.remove(&key).await.unwrap();
    assert!(!service.contains(&key).await.unwrap());
}

#[tokio::test]
async fn test_cache_service_is_empty() {
    let inner = Arc::new(InMemoryCache::new(100));
    let service = CacheService::new(inner);

    assert!(service.is_empty().await.unwrap());

    service.set(CacheKey::from("a"), &1_i32).await.unwrap();
    assert!(!service.is_empty().await.unwrap());
}

// ---------------------------------------------------------------------------
// Error envelope
// ---------------------------------------------------------------------------

#[test]
fn test_error_envelope_creation() {
    let env =
        ErrorEnvelope::new(ErrorCode::CacheMiss, "key not found", ErrorSeverity::Warning, false);
    assert_eq!(env.code, ErrorCode::CacheMiss);
    assert_eq!(env.severity, ErrorSeverity::Warning);
    assert!(!env.retryable);
    assert!(env.correlation_id > 0);
}

#[test]
fn test_error_envelope_with_hint() {
    let env = ErrorEnvelope::new(
        ErrorCode::CacheFull,
        "cache at max capacity",
        ErrorSeverity::Error,
        true,
    )
    .with_hint("Increase cache capacity or evict entries");
    assert!(env.retryable);
    assert_eq!(env.recovery_hint, Some("Increase cache capacity or evict entries"));
}

#[test]
fn test_error_envelope_display_format() {
    let env = ErrorEnvelope::new(
        ErrorCode::SerializationError,
        "serialization failed",
        ErrorSeverity::Error,
        true,
    );
    let display = env.to_string();
    assert!(display.contains("ERROR"));
    assert!(display.contains("corr="));
    assert!(display.contains("serialization failure"));
    assert!(display.contains("serialization failed"));
}

#[test]
fn test_error_envelope_from_cache_error() {
    use stashly::CacheError;
    let cache_err = CacheError::KeyNotFound("my-key".into());
    let env = ErrorEnvelope::from(cache_err);
    assert_eq!(env.code, ErrorCode::CacheMiss);
    assert_eq!(env.severity, ErrorSeverity::Warning);
    assert!(!env.retryable);
    assert!(env.recovery_hint.is_some());
}

// ---------------------------------------------------------------------------
// Entry lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_entry_creation_and_ttl() {
    let key = CacheKey::from("entry-test");
    let value = CacheValue::serialize(&"data".to_string()).unwrap();
    let entry = Entry::new(key, value);

    assert_eq!(entry.access_count, 0);
    assert!(!entry.is_expired());
    assert!(entry.remaining_ttl().is_none()); // no TTL set
}

#[test]
fn test_entry_with_ttl() {
    let key = CacheKey::from("ttl-test");
    let value = CacheValue::serialize(&"ttl-data".to_string()).unwrap();
    let entry = Entry::new(key, value).with_ttl(chrono::Duration::hours(1));

    assert!(!entry.is_expired());
    assert!(entry.remaining_ttl().is_some());
    assert!(entry.remaining_ttl().unwrap() > chrono::Duration::zero());
}

#[test]
fn test_entry_touch_increments_access_count() {
    let key = CacheKey::from("touch-test");
    let value = CacheValue::serialize(&"data".to_string()).unwrap();
    let mut entry = Entry::new(key, value);

    assert_eq!(entry.access_count, 0);
    entry.touch();
    assert_eq!(entry.access_count, 1);
    entry.touch();
    assert_eq!(entry.access_count, 2);
}

#[test]
fn test_expired_entry() {
    let key = CacheKey::from("expired-test");
    let value = CacheValue::serialize(&"stale".to_string()).unwrap();
    let mut entry = Entry::new(key, value).with_ttl(chrono::Duration::hours(1));

    // Manually set expiry in the past
    entry.expires_at = Some(chrono::Utc::now() - chrono::Duration::hours(1));
    assert!(entry.is_expired());
    assert_eq!(entry.remaining_ttl(), Some(chrono::Duration::zero()));
}
