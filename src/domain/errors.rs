//! Domain errors.

use std::fmt;

/// Error codes for structured error categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    /// The requested key was not found.
    KeyNotFound,
    /// Serialization (encode) failure.
    Serialization,
    /// Deserialization (decode) failure.
    Deserialization,
    /// Cache capacity exhausted.
    CacheFull,
    /// Internal/backend error.
    Backend,
    /// I/O operation failed.
    Io,
    /// Internal lock poisoned (a panic occurred while holding a lock).
    LockPoisoned,
    /// Generic internal error.
    Internal,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::KeyNotFound => write!(f, "KEY_NOT_FOUND"),
            ErrorCode::Serialization => write!(f, "SERIALIZATION_ERROR"),
            ErrorCode::Deserialization => write!(f, "DESERIALIZATION_ERROR"),
            ErrorCode::CacheFull => write!(f, "CACHE_FULL"),
            ErrorCode::Backend => write!(f, "BACKEND_ERROR"),
            ErrorCode::Io => write!(f, "IO_ERROR"),
            ErrorCode::LockPoisoned => write!(f, "LOCK_POISONED"),
            ErrorCode::Internal => write!(f, "INTERNAL_ERROR"),
        }
    }
}

/// Recovery hints for callers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecoveryHint {
    /// Retry the operation (e.g., after lock contention).
    Retry,
    /// Check the key exists before operating.
    CheckKey,
    /// Increase cache capacity.
    IncreaseCapacity,
    /// Check data format / serialization.
    CheckFormat,
    /// Check storage / I/O subsystem.
    CheckStorage,
    /// No specific recovery action available.
    None,
}

impl fmt::Display for RecoveryHint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecoveryHint::Retry => write!(f, "retry the operation"),
            RecoveryHint::CheckKey => write!(f, "check the key exists before operating"),
            RecoveryHint::IncreaseCapacity => write!(f, "increase cache capacity"),
            RecoveryHint::CheckFormat => write!(f, "check data format / serialization"),
            RecoveryHint::CheckStorage => write!(f, "check storage / I/O subsystem"),
            RecoveryHint::None => write!(f, "no specific recovery action available"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Key not found: {key}")]
    KeyNotFound { key: String, code: ErrorCode, hint: RecoveryHint },

    #[error("Serialization error: {message}")]
    SerializationError { message: String, code: ErrorCode, hint: RecoveryHint },

    #[error("Deserialization error: {message}")]
    DeserializationError { message: String, code: ErrorCode, hint: RecoveryHint },

    #[error("Cache full")]
    CacheFull { code: ErrorCode, hint: RecoveryHint },

    #[error("Backend error: {message}")]
    BackendError { message: String, code: ErrorCode, hint: RecoveryHint },

    #[error("IO error: {message}")]
    IoError { message: String, code: ErrorCode, hint: RecoveryHint },

    #[error("Lock poisoned: {message}")]
    LockPoisoned { message: String, code: ErrorCode, hint: RecoveryHint },
}

impl CacheError {
    /// Get the error code for this error.
    pub fn code(&self) -> ErrorCode {
        match self {
            CacheError::KeyNotFound { code, .. }
            | CacheError::SerializationError { code, .. }
            | CacheError::DeserializationError { code, .. }
            | CacheError::CacheFull { code, .. }
            | CacheError::BackendError { code, .. }
            | CacheError::IoError { code, .. }
            | CacheError::LockPoisoned { code, .. } => *code,
        }
    }

    /// Get the recovery hint for this error.
    pub fn hint(&self) -> &RecoveryHint {
        match self {
            CacheError::KeyNotFound { hint, .. }
            | CacheError::SerializationError { hint, .. }
            | CacheError::DeserializationError { hint, .. }
            | CacheError::CacheFull { hint, .. }
            | CacheError::BackendError { hint, .. }
            | CacheError::IoError { hint, .. }
            | CacheError::LockPoisoned { hint, .. } => hint,
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
