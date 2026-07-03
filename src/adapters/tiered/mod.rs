//! # In-Memory Adapters
//!
//! Pure in-memory implementations for testing and development.
//!
//! ## Usage
//!
//! ```rust
//! use stashly::TieredCache;
//! use stashly::ports::driven::{CachePort, CacheWritePort};
//!
//! let mut cache = TieredCache::default();
//! cache.set("key".into(), "value".into()).unwrap();
//! assert_eq!(cache.get(&"key".into()), Some("value".into()));
//! ```

use std::time::Instant;

use dashmap::DashMap;
use lru::LruCache;
use parking_lot::Mutex;

use crate::domain::entities::CacheEntry;
use crate::domain::events::CacheEvent;
use crate::domain::value_objects::{CacheKey, CacheStats, CacheTier, CacheValue, Ttl};
use crate::ports::driven::{CacheError, CachePort, CacheWritePort, EventPort, StatsPort};

/// Two-tier cache with L1 (LRU) for hot data and L2 (DashMap) for warm data.
pub struct TieredCache {
    /// L1: LRU cache for hot data
    l1: Mutex<LruCache<CacheKey, (CacheValue, Instant)>>,
    /// L2: DashMap for warm data
    l2: DashMap<CacheKey, (CacheValue, Instant)>,
    /// Statistics
    stats: Mutex<CacheStats>,
    /// Events
    events: Mutex<Vec<CacheEvent>>,
    /// Default TTL
    default_ttl: Ttl,
}

impl TieredCache {
    /// Create a new cache with default settings.
    pub fn new() -> Self {
        Self::with_config(1000, 10000, Ttl::from_secs(3600))
    }

    /// Create a cache with custom configuration.
    pub fn with_config(l1_max_size: usize, _l2_max_size: usize, default_ttl: Ttl) -> Self {
        Self {
            l1: Mutex::new(LruCache::new(std::num::NonZeroUsize::new(l1_max_size).unwrap())),
            l2: DashMap::new(),
            stats: Mutex::new(CacheStats::new()),
            events: Mutex::new(Vec::new()),
            default_ttl,
        }
    }

    /// Get from both tiers.
    fn get_internal(&self, key: &CacheKey) -> Option<(CacheValue, CacheTier)> {
        let now = Instant::now();

        // Check L1
        {
            let mut l1 = self.l1.lock();
            if let Some(entry) = l1.get(key) {
                if entry.1 > now {
                    return Some((entry.0.clone(), CacheTier::L1));
                }
                l1.pop(key);
            }
        }

        // Check L2
        if let Some(entry) = self.l2.get(key) {
            let (value, expiry) = (entry.0.clone(), entry.1);
            if expiry > now {
                return Some((value, CacheTier::L2));
            }
            drop(entry);
            self.l2.remove(key);
        }

        None
    }

    /// Clean up expired entries.
    pub fn cleanup(&self) -> usize {
        let now = Instant::now();
        let mut removed = 0;

        // Clean L1
        {
            let mut l1 = self.l1.lock();
            let keys: Vec<_> = l1
                .iter()
                .filter(|(_, (_, expiry))| *expiry <= now)
                .map(|(k, _)| k.clone())
                .collect();

            for key in keys {
                l1.pop(&key);
                removed += 1;
            }
        }

        // Clean L2 - dashmap requires iterating and collecting keys
        let expired_keys: Vec<CacheKey> = self
            .l2
            .iter()
            .filter(|entry| {
                let (_, expiry) = entry.value();
                *expiry <= now
            })
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired_keys {
            self.l2.remove(&key);
            removed += 1;
        }

        removed
    }
}

impl Default for TieredCache {
    fn default() -> Self {
        Self::new()
    }
}

impl CachePort for TieredCache {
    fn get(&self, key: &CacheKey) -> Option<CacheValue> {
        let result = self.get_internal(key);
        if let Some((value, tier)) = result {
            self.stats.lock().record_hit(tier);
            Some(value)
        } else {
            self.stats.lock().record_miss();
            None
        }
    }

    fn get_entry(&self, key: &CacheKey) -> Option<CacheEntry> {
        self.get_internal(key).map(|(value, _tier)| CacheEntry::new(key.clone(), value))
    }
}

impl CacheWritePort for TieredCache {
    fn set(&mut self, key: CacheKey, value: CacheValue) -> Result<(), CacheError> {
        self.set_with_ttl(key, value, self.default_ttl)
    }

    fn set_with_ttl(
        &mut self,
        key: CacheKey,
        value: CacheValue,
        ttl: Ttl,
    ) -> Result<(), CacheError> {
        let now = Instant::now();
        let expiry = now + ttl.as_duration();

        // Set in L1
        self.l1.lock().put(key.clone(), (value.clone(), expiry));

        // Set in L2
        self.l2.insert(key, (value, expiry));

        Ok(())
    }

    fn remove(&mut self, key: &CacheKey) -> Result<(), CacheError> {
        self.l1.lock().pop(key);
        self.l2.remove(key);
        Ok(())
    }

    fn clear(&mut self, tier: Option<CacheTier>) -> Result<usize, CacheError> {
        let mut count = 0;
        match tier {
            Some(CacheTier::L1) => {
                count = self.l1.lock().len();
                self.l1.lock().clear();
            }
            Some(CacheTier::L2) => {
                count = self.l2.len();
                self.l2.clear();
            }
            Some(CacheTier::L3) => {
                // No-op for in-memory cache
            }
            None => {
                count = self.l1.lock().len() + self.l2.len();
                self.l1.lock().clear();
                self.l2.clear();
            }
        }
        Ok(count)
    }
}

impl StatsPort for TieredCache {
    fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.lock().clone();
        stats.size = self.l1.lock().len() + self.l2.len();
        stats
    }

    fn record_hit(&mut self, tier: CacheTier) {
        self.stats.lock().record_hit(tier);
    }

    fn record_miss(&mut self) {
        self.stats.lock().record_miss();
    }

    fn record_eviction(&mut self) {
        self.stats.lock().record_eviction();
    }

    fn reset(&mut self) {
        let mut stats = self.stats.lock();
        *stats = CacheStats::new();
    }
}

impl EventPort for TieredCache {
    fn publish(&mut self, event: CacheEvent) -> Result<(), String> {
        self.events.lock().push(event);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;
    use crate::domain::value_objects::Ttl;
    use crate::ports::driven::CacheWritePort;

    #[test]
    fn test_basic_cache_operations() {
        let mut cache = TieredCache::default();
        cache.set("key1".into(), "value1".into()).unwrap();
        assert_eq!(cache.get(&"key1".into()), Some("value1".into()));
    }

    #[test]
    fn test_cache_miss() {
        let cache = TieredCache::default();
        assert_eq!(cache.get(&"nonexistent".into()), None);
    }

    #[test]
    fn test_cache_remove() {
        let mut cache = TieredCache::default();
        cache.set("key1".into(), "value1".into()).unwrap();
        cache.remove(&"key1".into()).unwrap();
        assert_eq!(cache.get(&"key1".into()), None);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = TieredCache::default();
        cache.set("key1".into(), "value1".into()).unwrap();
        cache.set("key2".into(), "value2".into()).unwrap();
        // Entries are stored in both L1 and L2, so clear counts both
        let count = cache.clear(None).unwrap();
        assert_eq!(count, 4); // 2 entries * 2 tiers
        assert_eq!(cache.get(&"key1".into()), None);
    }

    #[test]
    fn test_stats() {
        let mut cache = TieredCache::default();
        cache.set("key1".into(), "value1".into()).unwrap();
        cache.get(&"key1".into());
        cache.get(&"nonexistent".into());

        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_ttl_expiration() {
        let mut cache = TieredCache::default();
        CacheWritePort::set_with_ttl(
            &mut cache,
            "ttl-key".into(),
            "ttl-value".into(),
            Ttl::from_millis(1),
        )
        .unwrap();

        thread::sleep(Duration::from_millis(5));
        assert_eq!(cache.get(&"ttl-key".into()), None);
    }

    #[test]
    fn test_cleanup_removes_expired_entries() {
        let mut cache = TieredCache::default();
        CacheWritePort::set_with_ttl(
            &mut cache,
            "ttl-clean".into(),
            "ttl-value".into(),
            Ttl::from_millis(1),
        )
        .unwrap();

        thread::sleep(Duration::from_millis(5));

        let removed = cache.cleanup();
        assert!(removed >= 1);
        assert_eq!(cache.get(&"ttl-clean".into()), None);
    }
}
