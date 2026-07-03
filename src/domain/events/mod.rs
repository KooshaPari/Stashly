//! # Domain Events
//!
//! Immutable events representing state changes.
//!
//! ## Event Sourcing Principles
//!
//! - Events are immutable facts
//! - Append-only log
//! - Reconstruct state by replaying events

use std::time::SystemTime;

use super::value_objects::CacheTier;

/// Domain events for the cache bounded context
#[derive(Debug, Clone)]
pub enum CacheEvent {
    /// Cache hit event
    CacheHit { key: String, tier: CacheTier, timestamp: SystemTime },
    /// Cache miss event
    CacheMiss { key: String, timestamp: SystemTime },
    /// Cache entry created
    CacheEntryCreated { key: String, tier: CacheTier, ttl_secs: u64, timestamp: SystemTime },
    /// Cache entry evicted
    CacheEntryEvicted {
        key: String,
        tier: CacheTier,
        reason: EvictionReason,
        timestamp: SystemTime,
    },
    /// Cache entry expired
    CacheEntryExpired { key: String, tier: CacheTier, timestamp: SystemTime },
    /// Cache cleared
    CacheCleared { tier: Option<CacheTier>, entries_removed: usize, timestamp: SystemTime },
    /// Singleflight request started
    SingleflightStarted { key: String, requester_pid: u32, timestamp: SystemTime },
    /// Singleflight request completed
    SingleflightCompleted {
        key: String,
        result_waiters: u32,
        duration_ms: u64,
        timestamp: SystemTime,
    },
    /// Singleflight request failed
    SingleflightFailed { key: String, error: String, waiters: u32, timestamp: SystemTime },
}

impl CacheEvent {
    /// Get the timestamp of the event.
    pub fn timestamp(&self) -> SystemTime {
        match self {
            CacheEvent::CacheHit { timestamp, .. } => *timestamp,
            CacheEvent::CacheMiss { timestamp, .. } => *timestamp,
            CacheEvent::CacheEntryCreated { timestamp, .. } => *timestamp,
            CacheEvent::CacheEntryEvicted { timestamp, .. } => *timestamp,
            CacheEvent::CacheEntryExpired { timestamp, .. } => *timestamp,
            CacheEvent::CacheCleared { timestamp, .. } => *timestamp,
            CacheEvent::SingleflightStarted { timestamp, .. } => *timestamp,
            CacheEvent::SingleflightCompleted { timestamp, .. } => *timestamp,
            CacheEvent::SingleflightFailed { timestamp, .. } => *timestamp,
        }
    }

    /// Get the key associated with the event.
    pub fn key(&self) -> Option<&str> {
        match self {
            CacheEvent::CacheHit { key, .. } => Some(key),
            CacheEvent::CacheMiss { key, .. } => Some(key),
            CacheEvent::CacheEntryCreated { key, .. } => Some(key),
            CacheEvent::CacheEntryEvicted { key, .. } => Some(key),
            CacheEvent::CacheEntryExpired { key, .. } => Some(key),
            CacheEvent::SingleflightStarted { key, .. } => Some(key),
            CacheEvent::SingleflightCompleted { key, .. } => Some(key),
            CacheEvent::SingleflightFailed { key, .. } => Some(key),
            CacheEvent::CacheCleared { .. } => None,
        }
    }
}

/// Reason for cache eviction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionReason {
    /// Entry was manually removed
    Manual,
    /// Entry expired
    Expired,
    /// LRU eviction (capacity reached)
    Capacity,
    /// Entry was replaced
    Replaced,
}

impl fmt::Display for EvictionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvictionReason::Manual => write!(f, "manual"),
            EvictionReason::Expired => write!(f, "expired"),
            EvictionReason::Capacity => write!(f, "capacity"),
            EvictionReason::Replaced => write!(f, "replaced"),
        }
    }
}

use std::fmt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eviction_reason_display() {
        assert_eq!(EvictionReason::Manual.to_string(), "manual");
        assert_eq!(EvictionReason::Expired.to_string(), "expired");
        assert_eq!(EvictionReason::Capacity.to_string(), "capacity");
        assert_eq!(EvictionReason::Replaced.to_string(), "replaced");
    }

    #[test]
    fn test_cache_event_timestamp() {
        let now = std::time::SystemTime::now();
        let event =
            CacheEvent::CacheHit { key: "k".to_string(), tier: CacheTier::L1, timestamp: now };
        assert_eq!(event.timestamp(), now);
        assert_eq!(event.key(), Some("k"));
    }

    #[test]
    fn test_cache_event_key_method() {
        let now = std::time::SystemTime::now();
        assert_eq!(
            CacheEvent::CacheMiss { key: "miss".to_string(), timestamp: now }.key(),
            Some("miss")
        );
        assert_eq!(
            CacheEvent::CacheCleared { tier: None, entries_removed: 5, timestamp: now }.key(),
            None
        );
    }

    #[test]
    fn test_cache_event_variants() {
        let now = std::time::SystemTime::now();

        let evicted = CacheEvent::CacheEntryEvicted {
            key: "k".to_string(),
            tier: CacheTier::L1,
            reason: EvictionReason::Capacity,
            timestamp: now,
        };
        assert_eq!(evicted.key(), Some("k"));

        let expired = CacheEvent::CacheEntryExpired {
            key: "e".to_string(),
            tier: CacheTier::L2,
            timestamp: now,
        };
        assert_eq!(expired.key(), Some("e"));

        let sf_started = CacheEvent::SingleflightStarted {
            key: "sf".to_string(),
            requester_pid: 100,
            timestamp: now,
        };
        assert_eq!(sf_started.key(), Some("sf"));

        let sf_completed = CacheEvent::SingleflightCompleted {
            key: "sf".to_string(),
            result_waiters: 3,
            duration_ms: 42,
            timestamp: now,
        };
        assert_eq!(sf_completed.key(), Some("sf"));

        let sf_failed = CacheEvent::SingleflightFailed {
            key: "sf".to_string(),
            error: "timeout".to_string(),
            waiters: 2,
            timestamp: now,
        };
        assert_eq!(sf_failed.key(), Some("sf"));

        let created = CacheEvent::CacheEntryCreated {
            key: "new".to_string(),
            tier: CacheTier::L1,
            ttl_secs: 60,
            timestamp: now,
        };
        assert_eq!(created.key(), Some("new"));
    }
}
