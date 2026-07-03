//! # Value Objects
//!
//! Immutable objects defined by their attributes.
//!
//! ## Value Object Principles
//!
//! - Immutable (no setters, create new instances)
//! - No identity (two VOs with same values are equal)
//! - Self-validating

use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::Duration;

/// Cache key value object
#[derive(Debug, Clone)]
pub struct CacheKey(String);

impl CacheKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for CacheKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CacheKey {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CacheKey {}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// Cache value value object
#[derive(Debug, Clone)]
pub struct CacheValue(String);

impl CacheValue {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<str> for CacheValue {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for CacheValue {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for CacheValue {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl fmt::Display for CacheValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for CacheValue {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CacheValue {}

/// TTL (Time-to-Live) value object
#[derive(Debug, Clone, Copy)]
pub struct Ttl(Duration);

impl Ttl {
    /// Create TTL from seconds.
    pub fn from_secs(seconds: u64) -> Self {
        Self(Duration::from_secs(seconds))
    }

    /// Create TTL from milliseconds.
    pub fn from_millis(millis: u64) -> Self {
        Self(Duration::from_millis(millis))
    }

    /// Create TTL from duration.
    pub fn from_duration(duration: Duration) -> Self {
        Self(duration)
    }

    /// Create a zero TTL (immediate expiration).
    pub fn zero() -> Self {
        Self(Duration::ZERO)
    }

    /// Create a TTL that never expires.
    pub fn infinite() -> Self {
        Self(Duration::MAX)
    }

    /// Get as Duration.
    pub fn as_duration(&self) -> Duration {
        self.0
    }

    /// Get as seconds.
    pub fn as_secs(&self) -> u64 {
        self.0.as_secs()
    }

    /// Check if TTL is zero.
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Check if TTL is effectively infinite.
    pub fn is_infinite(&self) -> bool {
        self.0 == Duration::MAX
    }
}

impl Default for Ttl {
    fn default() -> Self {
        Self::from_secs(3600) // 1 hour default
    }
}

impl fmt::Display for Ttl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            write!(f, "0s")
        } else if self.is_infinite() {
            write!(f, "infinite")
        } else if self.0.as_secs() > 0 {
            write!(f, "{}s", self.0.as_secs())
        } else {
            write!(f, "{}ms", self.0.as_millis())
        }
    }
}

impl PartialEq for Ttl {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Ttl {}

/// Cache tier enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CacheTier {
    /// L1: LRU cache for hot data
    #[default]
    L1,
    /// L2: HashMap for warm data
    L2,
    /// L3: Persistent storage
    L3,
}

impl fmt::Display for CacheTier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheTier::L1 => write!(f, "L1"),
            CacheTier::L2 => write!(f, "L2"),
            CacheTier::L3 => write!(f, "L3"),
        }
    }
}

/// Cache statistics value object
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total hits
    pub hits: u64,
    /// Total misses
    pub misses: u64,
    /// Evictions
    pub evictions: u64,
    /// Current size
    pub size: usize,
    /// Hits per tier
    pub tier_hits: [u64; 3],
}

impl CacheStats {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a hit.
    pub fn record_hit(&mut self, tier: CacheTier) {
        self.hits += 1;
        match tier {
            CacheTier::L1 => self.tier_hits[0] += 1,
            CacheTier::L2 => self.tier_hits[1] += 1,
            CacheTier::L3 => self.tier_hits[2] += 1,
        }
    }

    /// Record a miss.
    pub fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Record an eviction.
    pub fn record_eviction(&mut self) {
        self.evictions += 1;
    }

    /// Get hit rate.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get miss rate.
    pub fn miss_rate(&self) -> f64 {
        1.0 - self.hit_rate()
    }
}

/// Cache configuration value object
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum L1 size
    pub l1_max_size: usize,
    /// Maximum L2 size
    pub l2_max_size: usize,
    /// Default TTL
    pub default_ttl: Ttl,
    /// Cleanup interval
    pub cleanup_interval: Duration,
}

impl CacheConfig {
    pub fn new(l1_max_size: usize, l2_max_size: usize, default_ttl: Ttl) -> Self {
        Self { l1_max_size, l2_max_size, default_ttl, cleanup_interval: Duration::from_secs(60) }
    }

    pub fn with_cleanup_interval(mut self, interval: Duration) -> Self {
        self.cleanup_interval = interval;
        self
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_max_size: 1000,
            l2_max_size: 10000,
            default_ttl: Ttl::default(),
            cleanup_interval: Duration::from_secs(60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        let key = CacheKey::new("test");
        assert_eq!(key.as_str(), "test");
        assert_eq!(key.len(), 4);
        assert!(!key.is_empty());
    }

    #[test]
    fn test_cache_key_equality() {
        let key1 = CacheKey::new("test");
        let key2 = CacheKey::new("test");
        let key3 = CacheKey::new("other");
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_ttl() {
        let ttl = Ttl::from_secs(60);
        assert_eq!(ttl.as_secs(), 60);
        assert!(!ttl.is_zero());
        assert!(!ttl.is_infinite());
    }

    #[test]
    fn test_ttl_display() {
        assert_eq!(Ttl::zero().to_string(), "0s");
        assert_eq!(Ttl::from_secs(120).to_string(), "120s");
        assert_eq!(Ttl::infinite().to_string(), "infinite");
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::new();
        stats.record_hit(CacheTier::L1);
        stats.record_hit(CacheTier::L2);
        stats.record_miss();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 0.666).abs() < 0.01);
    }
}
