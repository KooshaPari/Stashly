//! Domain errors.

use std::fmt;
use std::time::{Duration, SystemTime};

// ---------------------------------------------------------------------------
// Error severity levels
// ---------------------------------------------------------------------------

/// Severity level for cache errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ErrorSeverity {
    /// Informational — non-fatal, caller can ignore.
    Info,
    /// Warning — operation degraded but recoverable.
    Warning,
    /// Error — operation failed, may require intervention.
    #[default]
    Error,
    /// Critical — system may be in an inconsistent state.
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Info => write!(f, "info"),
            ErrorSeverity::Warning => write!(f, "warning"),
            ErrorSeverity::Error => write!(f, "error"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
}

// ---------------------------------------------------------------------------
// Structured error envelope
// ---------------------------------------------------------------------------

/// Structured error metadata providing context, recovery hints, and correlation.
///
/// Addresses audit finding L14: errors lacked correlation IDs, recovery hints,
/// and retry metadata.
#[derive(Debug, Clone)]
pub struct ErrorEnvelope {
    /// Unique correlation ID for tracing this error across system boundaries.
    pub correlation_id: String,

    /// Timestamp when the error was created.
    pub timestamp: SystemTime,

    /// Severity level.
    pub severity: ErrorSeverity,

    /// Human-readable hint about how to recover from this error.
    pub recovery_hint: Option<String>,

    /// Whether the operation can be retried.
    pub retryable: bool,

    /// Recommended delay before retrying (if retryable).
    pub retry_after: Option<Duration>,

    /// Source component that produced the error.
    pub source: Option<String>,
}

impl ErrorEnvelope {
    /// Create a new minimal error envelope with an auto-generated correlation ID.
    pub fn new() -> Self {
        Self {
            correlation_id: generate_correlation_id(),
            timestamp: SystemTime::now(),
            severity: ErrorSeverity::Error,
            recovery_hint: None,
            retryable: false,
            retry_after: None,
            source: None,
        }
    }

    /// Builder: set severity.
    pub fn severity(mut self, level: ErrorSeverity) -> Self {
        self.severity = level;
        self
    }

    /// Builder: set recovery hint.
    pub fn with_recovery(mut self, hint: impl Into<String>) -> Self {
        self.recovery_hint = Some(hint.into());
        self
    }

    /// Builder: mark as retryable with optional delay.
    pub fn retryable(mut self, delay: Option<Duration>) -> Self {
        self.retryable = true;
        self.retry_after = delay;
        self
    }

    /// Builder: set source component.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Builder: override correlation ID.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = id.into();
        self
    }

    /// Wrap this envelope around an error message.
    pub fn into_error(self, message: impl Into<String>) -> CacheError {
        CacheError::Enriched {
            message: message.into(),
            envelope: self,
        }
    }
}

impl Default for ErrorEnvelope {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Generate a correlation ID (timestamp + random hex)
// ---------------------------------------------------------------------------

fn generate_correlation_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    // Append a few random-ish bytes using the low bits of the timestamp
    let entropy = (nanos & 0xFFFF) as u16;
    format!("stashly-{:016x}-{:04x}", nanos, entropy)
}

// ---------------------------------------------------------------------------
// Cache error (original variants + enriched variant)
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Cache full")]
    CacheFull,

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("IO error: {0}")]
    IoError(String),

    /// Enriched error with structured metadata.
    #[error("{message}")]
    Enriched {
        /// Human-readable error message.
        message: String,
        /// Structured error metadata.
        envelope: ErrorEnvelope,
    },
}

impl CacheError {
    /// Get the error severity (defaults to Error for legacy variants).
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CacheError::Enriched { envelope, .. } => envelope.severity,
            CacheError::CacheFull => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }

    /// Return `true` if the error is recoverable / retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            CacheError::Enriched { envelope, .. } => envelope.retryable,
            CacheError::BackendError(_) | CacheError::IoError(_) => true,
            _ => false,
        }
    }

    /// Return a human-readable recovery hint, if available.
    pub fn recovery_hint(&self) -> Option<&str> {
        match self {
            CacheError::Enriched { envelope, .. } => envelope.recovery_hint.as_deref(),
            CacheError::CacheFull => Some("free up entries by removing stale keys or increasing capacity"),
            CacheError::SerializationError(_) => Some("check that the value implements Serialize"),
            CacheError::DeserializationError(_) => Some("check that the stored type matches the expected Deserialize implementation"),
            _ => None,
        }
    }

    /// Return the correlation ID if this is an enriched error.
    pub fn correlation_id(&self) -> Option<&str> {
        match self {
            CacheError::Enriched { envelope, .. } => Some(&envelope.correlation_id),
            _ => None,
        }
    }

    /// Wrap this error in an enriched envelope.
    pub fn enrich(self) -> Self {
        let message = self.to_string();
        CacheError::Enriched {
            message,
            envelope: ErrorEnvelope::new(),
        }
    }

    /// Wrap this error with a recovery hint.
    pub fn with_recovery(self, hint: impl Into<String>) -> Self {
        let message = self.to_string();
        CacheError::Enriched {
            message,
            envelope: ErrorEnvelope::new()
                .with_recovery(hint)
                .retryable(Some(Duration::from_secs(1))),
        }
    }

    /// Attach a severity to this error.
    pub fn with_severity(self, level: ErrorSeverity) -> Self {
        let message = self.to_string();
        CacheError::Enriched {
            message,
            envelope: ErrorEnvelope::new().severity(level),
        }
    }
}

impl serde::Serialize for CacheError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // FR: L14-ERR-ENVELOPE-CORRELATION — ErrorEnvelope generates unique correlation IDs
    #[test]
    fn test_error_envelope_creation() {
        let envelope = ErrorEnvelope::new();
        assert!(envelope.correlation_id.starts_with("stashly-"));
        assert_eq!(envelope.severity, ErrorSeverity::Error);
        assert!(!envelope.retryable);
        assert!(envelope.recovery_hint.is_none());
    }

    // FR: L14-ERR-ENVELOPE-BUILDER — ErrorEnvelope builder methods work correctly
    #[test]
    fn test_error_envelope_builder() {
        let envelope = ErrorEnvelope::new()
            .severity(ErrorSeverity::Warning)
            .with_recovery("retry the operation")
            .retryable(Some(Duration::from_secs(2)))
            .with_source("memory-cache");

        assert_eq!(envelope.severity, ErrorSeverity::Warning);
        assert_eq!(envelope.recovery_hint, Some("retry the operation".into()));
        assert!(envelope.retryable);
        assert_eq!(envelope.retry_after, Some(Duration::from_secs(2)));
        assert_eq!(envelope.source, Some("memory-cache".into()));
    }

    // FR: L14-ERR-CORRELATION-ID — ErrorEnvelope correlation IDs are unique per instance
    #[test]
    fn test_correlation_ids_are_unique() {
        let e1 = ErrorEnvelope::new();
        let e2 = ErrorEnvelope::new();
        assert_ne!(e1.correlation_id, e2.correlation_id);
    }

    // FR: L14-ERR-ENRICH-SEVERITY — CacheError enrichment preserves severity
    #[test]
    fn test_cache_error_severity() {
        let err = CacheError::CacheFull;
        assert_eq!(err.severity(), ErrorSeverity::Warning);
        assert!(err.recovery_hint().is_some());

        let err = CacheError::KeyNotFound("missing".into());
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.recovery_hint().is_none());
    }

    // FR: L14-ERR-RETRYABLE — Backend/Io errors are retryable by default
    #[test]
    fn test_cache_error_retryable() {
        let backend_err = CacheError::BackendError("timeout".into());
        assert!(backend_err.is_retryable());

        let io_err = CacheError::IoError("disk full".into());
        assert!(io_err.is_retryable());

        let not_found = CacheError::KeyNotFound("missing".into());
        assert!(!not_found.is_retryable());
    }

    // FR: L14-ERR-ENRICH — enrich() wraps an error with envelope metadata
    #[test]
    fn test_cache_error_enrich() {
        let err = CacheError::KeyNotFound("test".into()).enrich();
        assert!(err.correlation_id().is_some());
        assert!(err.correlation_id().unwrap().starts_with("stashly-"));
    }

    // FR: L14-ERR-WITH-RECOVERY — with_recovery adds recovery hint and marks retryable
    #[test]
    fn test_cache_error_with_recovery() {
        let err = CacheError::BackendError("connection refused".into())
            .with_recovery("check network connectivity");
        assert!(err.is_retryable());
        assert_eq!(
            err.recovery_hint(),
            Some("check network connectivity")
        );
    }

    // FR: L14-ERR-ENVELOPE-TO-ERROR — envelope.into_error() creates enriched CacheError
    #[test]
    fn test_envelope_into_error() {
        let envelope = ErrorEnvelope::new()
            .severity(ErrorSeverity::Critical)
            .with_recovery("restart the service");
        let err = envelope.into_error("critical failure in cache backend");
        assert_eq!(err.severity(), ErrorSeverity::Critical);
        assert_eq!(err.recovery_hint(), Some("restart the service"));
    }
}
