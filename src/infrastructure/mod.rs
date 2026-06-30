//! Infrastructure layer.
//!
//! Provides cross-cutting concerns including structured error handling
//! with correlation IDs, error codes, and recovery hints.

pub mod error;

pub use error::{CacheKitError, ErrorCode, ErrorEnvelope, ErrorSeverity};
