//! # Domain Events
//!
//! Immutable events representing state changes.
//!
//! ## Event Sourcing Principles
//!
//! - Events are immutable facts
//! - Append-only log
//! - Reconstruct state by replaying events

use super::value_objects::CacheTier;
use std::time::SystemTime;

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
