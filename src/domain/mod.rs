//! Domain layer.

pub mod cache;
pub mod errors;
pub mod policy;
pub mod ports;

// Re-exports
pub use cache::{CacheKey, CacheValue, Entry};
pub use errors::CacheError;
pub use policy::{EvictionPolicy, LfuPolicy, LruPolicy, TtlPolicy};
pub use ports::Cache;
