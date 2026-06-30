//! Adapters layer.

pub mod memory;
pub mod tiered;

pub use memory::InMemoryCache;
pub use tiered::TieredCache;
