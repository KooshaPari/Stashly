//! Domain layer.

pub mod cache;
pub mod policy;
pub mod ports;
pub mod errors;

// Re-exports
pub use cache::{CacheKey, CacheValue, Entry};
pub use policy::{EvictionPolicy, LruPolicy, LfuPolicy, TtlPolicy};
pub use errors::CacheError;
pub use ports::Cache;
