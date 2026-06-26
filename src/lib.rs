//! Stashly — Universal caching framework with multi-tier and singleflight support.
//!
//! # Architecture
//!
//! Stashly follows hexagonal architecture:
//!
//! - **Domain**: Pure business logic (cache entities, policies, value objects)
//! - **Application**: Use cases and cache service (CQRS commands/queries)
//! - **Adapters**: Backend implementations (memory, tiered, Redis, etc.)
//! - **Ports**: Driven and driving ports for hexagonal architecture
//! - **Infrastructure**: Cross-cutting concerns
//!
//! # Absorbed Projects
//!
//! - **thegent-cache** — Multi-tier caching (L1/L2), singleflight support, CQRS
//!
//! # Features
//!
//! - **Universal Caching**: Memory, Redis, tiered backends
//! - **Multi-Tier**: L1 (LRU) + L2 (concurrent) + L3 (persistent)
//! - **Singleflight**: Deduplicate concurrent requests for same key
//! - **CQRS**: Separate command and query interfaces
//! - **Events**: Domain events for cache operations
//! - **TTL & Eviction**: Configurable expiration policies

pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod ports;

// Re-exports (original Stashly)
pub use adapters::memory::InMemoryCache;
pub use adapters::tiered::TieredCache;
pub use domain::entities::{CacheEntry, SingleflightRequest};
pub use domain::errors::CacheError;
pub use domain::events::CacheEvent;
pub use domain::value_objects::{CacheStats, CacheTier, Ttl};
pub use domain::{Cache, CacheKey, CacheValue, Entry};
pub use infrastructure::error::CacheKitError;
// Re-exports (from thegent-cache)
pub use ports::driven::{CachePort, CacheWritePort, EvictionPort, SingleflightPort, StatsPort};

/// Two-tier cache re-export for convenience
pub mod cache {
    pub use crate::adapters::tiered::TieredCache;
}

/// Framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
