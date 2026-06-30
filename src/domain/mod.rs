//! Domain layer.

pub mod cache;
pub mod entities;
pub mod errors;
pub mod events;
pub mod policy;
pub mod ports;
pub mod value_objects;

// Re-exports (original Stashly)
pub use cache::{CacheKey, CacheValue, Entry};
pub use errors::CacheError;
pub use policy::{EvictionPolicy, LfuPolicy, LruPolicy, TtlPolicy};
pub use ports::Cache;
