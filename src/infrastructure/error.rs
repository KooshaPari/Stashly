//! Infrastructure error handling.
//!
//! Provides structured error types with correlation IDs, severity levels,
//! and recovery hints for observability and debugging.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::domain::errors::CacheError;

/// Thread-safe correlation ID generator.
static CORRELATION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a unique correlation ID for error tracking.
fn next_correlation_id() -> u64 {
    CORRELATION_COUNTER.fetch_add(1, Ordering::Relaxed)
}

/// Typed error codes for structured error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// Configuration issue (e.g. bad capacity, invalid TTL).
    ConfigError,
    /// Initialization failure.
    InitError,
    /// Generic runtime failure.
    RuntimeError,
    /// The requested cache key was not found.
    CacheMiss,
    /// Cache is full and cannot accommodate more entries.
    CacheFull,
    /// Serialization or deserialization of cache values failed.
    SerializationError,
    /// A backend/storage operation failed.
    BackendError,
    /// An internal invariant was violated.
    InternalError,
    /// An I/O operation failed.
    IoError,
}

impl ErrorCode {
    /// Returns a human-readable description of this error code.
    pub fn description(&self) -> &'static str {
        match self {
            ErrorCode::ConfigError => "configuration error",
            ErrorCode::InitError => "initialization error",
            ErrorCode::RuntimeError => "runtime error",
            ErrorCode::CacheMiss => "cache key not found",
            ErrorCode::CacheFull => "cache at capacity",
            ErrorCode::SerializationError => "serialization failure",
            ErrorCode::BackendError => "backend operation failed",
            ErrorCode::InternalError => "internal invariant violation",
            ErrorCode::IoError => "I/O operation failed",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Severity level for error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorSeverity {
    /// Recoverable, non-critical issue (e.g. cache miss).
    Warning,
    /// Operation failed but system remains functional.
    Error,
    /// System-level failure requiring immediate attention.
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "WARNING"),
            ErrorSeverity::Error => write!(f, "ERROR"),
            ErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Structured error envelope with correlation ID, error code, severity,
/// and recovery hints.
///
/// This type is designed for observability: every error carries enough
/// context for debugging without leaking internal state to end users.
#[derive(Debug, Clone)]
pub struct ErrorEnvelope {
    /// Unique correlation ID for tracing this error across logs.
    pub correlation_id: u64,
    /// Typed error code for programmatic handling.
    pub code: ErrorCode,
    /// Human-readable error message (safe for logging).
    pub message: String,
    /// Severity level.
    pub severity: ErrorSeverity,
    /// Whether the operation can be safely retried.
    pub retryable: bool,
    /// Optional hint for recovery (safe for end users).
    pub recovery_hint: Option<&'static str>,
}

impl ErrorEnvelope {
    /// Creates a new error envelope with auto-generated correlation ID.
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        severity: ErrorSeverity,
        retryable: bool,
    ) -> Self {
        Self {
            correlation_id: next_correlation_id(),
            code,
            message: message.into(),
            severity,
            retryable,
            recovery_hint: None,
        }
    }

    /// Attaches a recovery hint to this error envelope.
    pub fn with_hint(mut self, hint: &'static str) -> Self {
        self.recovery_hint = Some(hint);
        self
    }

    /// Converts a `CacheError` into a structured `ErrorEnvelope`.
    pub fn from_cache_error(err: &CacheError) -> Self {
        let (code, severity, retryable, hint) = match err {
            CacheError::KeyNotFound(_) => (
                ErrorCode::CacheMiss,
                ErrorSeverity::Warning,
                false,
                Some("Check that the key exists before reading"),
            ),
            CacheError::SerializationError(_) => (
                ErrorCode::SerializationError,
                ErrorSeverity::Error,
                true,
                Some("Ensure the value type implements Serialize"),
            ),
            CacheError::DeserializationError(_) => (
                ErrorCode::SerializationError,
                ErrorSeverity::Error,
                false,
                Some("Ensure the stored value matches the expected type"),
            ),
            CacheError::CacheFull => (
                ErrorCode::CacheFull,
                ErrorSeverity::Error,
                true,
                Some("Increase cache capacity or evict entries manually"),
            ),
            CacheError::BackendError(_) => (
                ErrorCode::BackendError,
                ErrorSeverity::Error,
                true,
                Some("Check backend connectivity and retry the operation"),
            ),
            CacheError::IoError(_) => (
                ErrorCode::IoError,
                ErrorSeverity::Critical,
                false,
                Some("Check disk/network availability and permissions"),
            ),
        };

        Self {
            correlation_id: next_correlation_id(),
            code,
            message: err.to_string(),
            severity,
            retryable,
            recovery_hint: hint,
        }
    }
}

impl fmt::Display for ErrorEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{} | corr={}] {}: {}",
            self.severity, self.correlation_id, self.code, self.message
        )
    }
}

impl std::error::Error for ErrorEnvelope {}

impl From<CacheError> for ErrorEnvelope {
    fn from(err: CacheError) -> Self {
        ErrorEnvelope::from_cache_error(&err)
    }
}

// --- Existing CacheKitError ---

#[derive(Debug)]
pub enum CacheKitError {
    Config(String),
    Init(String),
    Runtime(String),
}

impl fmt::Display for CacheKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheKitError::Config(msg) => write!(f, "Configuration error: {}", msg),
            CacheKitError::Init(msg) => write!(f, "Initialization error: {}", msg),
            CacheKitError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for CacheKitError {}

impl From<&CacheKitError> for ErrorEnvelope {
    fn from(err: &CacheKitError) -> Self {
        let (code, severity) = match err {
            CacheKitError::Config(_) => (ErrorCode::ConfigError, ErrorSeverity::Error),
            CacheKitError::Init(_) => (ErrorCode::InitError, ErrorSeverity::Critical),
            CacheKitError::Runtime(_) => (ErrorCode::RuntimeError, ErrorSeverity::Error),
        };

        Self {
            correlation_id: next_correlation_id(),
            code,
            message: err.to_string(),
            severity,
            retryable: false,
            recovery_hint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_envelope_creation() {
        let env = ErrorEnvelope::new(
            ErrorCode::CacheMiss,
            "key not found: test-123",
            ErrorSeverity::Warning,
            false,
        );
        assert_eq!(env.code, ErrorCode::CacheMiss);
        assert_eq!(env.severity, ErrorSeverity::Warning);
        assert!(!env.retryable);
        assert!(env.correlation_id > 0);
        assert!(env.recovery_hint.is_none());
    }

    #[test]
    fn test_error_envelope_with_hint() {
        let env = ErrorEnvelope::new(
            ErrorCode::SerializationError,
            "failed to serialize",
            ErrorSeverity::Error,
            true,
        )
        .with_hint("Check that the type implements Serialize");
        assert!(env.retryable);
        assert_eq!(env.recovery_hint, Some("Check that the type implements Serialize"));
    }

    #[test]
    fn test_error_envelope_from_cache_error_key_not_found() {
        let cache_err = CacheError::KeyNotFound("my-key".to_string());
        let env = ErrorEnvelope::from(cache_err);
        assert_eq!(env.code, ErrorCode::CacheMiss);
        assert_eq!(env.severity, ErrorSeverity::Warning);
        assert!(!env.retryable);
        assert!(env.recovery_hint.is_some());
    }

    #[test]
    fn test_error_envelope_from_cache_error_backend() {
        let cache_err = CacheError::BackendError("connection refused".to_string());
        let env = ErrorEnvelope::from(cache_err);
        assert_eq!(env.code, ErrorCode::BackendError);
        assert_eq!(env.severity, ErrorSeverity::Error);
        assert!(env.retryable);
    }

    #[test]
    fn test_error_envelope_from_cache_error_io() {
        let cache_err = CacheError::IoError("permission denied".to_string());
        let env = ErrorEnvelope::from(cache_err);
        assert_eq!(env.code, ErrorCode::IoError);
        assert_eq!(env.severity, ErrorSeverity::Critical);
        assert!(!env.retryable);
    }

    #[test]
    fn test_error_envelope_from_cache_kit_error() {
        let kit_err = CacheKitError::Config("invalid capacity".to_string());
        let env = ErrorEnvelope::from(&kit_err);
        assert_eq!(env.code, ErrorCode::ConfigError);
        assert_eq!(env.severity, ErrorSeverity::Error);
    }

    #[test]
    fn test_error_envelope_display() {
        let env =
            ErrorEnvelope::new(ErrorCode::CacheMiss, "not found", ErrorSeverity::Warning, false);
        let display = env.to_string();
        assert!(display.contains("WARNING"));
        assert!(display.contains("corr="));
        assert!(display.contains("cache key not found"));
        assert!(display.contains("not found"));
    }

    #[test]
    fn test_unique_correlation_ids() {
        let env1 = ErrorEnvelope::new(ErrorCode::CacheMiss, "a", ErrorSeverity::Warning, false);
        let env2 = ErrorEnvelope::new(ErrorCode::CacheFull, "b", ErrorSeverity::Error, false);
        assert_ne!(env1.correlation_id, env2.correlation_id);
    }

    #[test]
    fn test_error_code_descriptions() {
        assert_eq!(ErrorCode::CacheMiss.description(), "cache key not found");
        assert_eq!(ErrorCode::ConfigError.description(), "configuration error");
        assert_eq!(ErrorCode::SerializationError.description(), "serialization failure");
        assert_eq!(ErrorCode::InternalError.description(), "internal invariant violation");
        assert_eq!(ErrorCode::IoError.description(), "I/O operation failed");
    }
}
