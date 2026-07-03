//! Infrastructure error handling.

use std::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_kit_error_display() {
        let err = CacheKitError::Config("missing host".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing host");

        let err = CacheKitError::Init("connection failed".to_string());
        assert_eq!(err.to_string(), "Initialization error: connection failed");

        let err = CacheKitError::Runtime("out of memory".to_string());
        assert_eq!(err.to_string(), "Runtime error: out of memory");
    }

    #[test]
    fn test_cache_kit_error_is_std_error() {
        fn assert_implements_error<T: std::error::Error>() {}
        assert_implements_error::<CacheKitError>();
    }
}
