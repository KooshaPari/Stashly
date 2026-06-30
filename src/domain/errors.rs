//! Domain errors.

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

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::KeyNotFound("mykey".to_string());
        assert_eq!(err.to_string(), "Key not found: mykey");

        let err = CacheError::SerializationError("bad format".to_string());
        assert_eq!(err.to_string(), "Serialization error: bad format");

        let err = CacheError::DeserializationError("bad bytes".to_string());
        assert_eq!(err.to_string(), "Deserialization error: bad bytes");

        assert_eq!(CacheError::CacheFull.to_string(), "Cache full");

        let err = CacheError::BackendError("connection failed".to_string());
        assert_eq!(err.to_string(), "Backend error: connection failed");

        let err = CacheError::IoError("disk full".to_string());
        assert_eq!(err.to_string(), "IO error: disk full");
    }

    #[test]
    fn test_cache_error_serialize() {
        let err = CacheError::KeyNotFound("k".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("Key not found"));
    }
}
