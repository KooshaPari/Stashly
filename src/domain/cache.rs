//! Cache entities.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::hash::{Hash, Hasher};
use std::borrow::Cow;
use super::errors::CacheError;

/// Cache key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey(Cow<'static, str>);

impl CacheKey {
    pub fn new(s: impl Into<Cow<'static, str>>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for CacheKey {}

impl From<String> for CacheKey {
    fn from(s: String) -> Self {
        Self(Cow::Owned(s))
    }
}

impl From<&'static str> for CacheKey {
    fn from(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Cache value wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheValue {
    /// Serialized value.
    pub data: Vec<u8>,
    /// Content type.
    pub content_type: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
}

impl CacheValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            content_type: None,
            created_at: Utc::now(),
        }
    }

    pub fn with_content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub fn deserialize<T: serde::de::DeserializeOwned>(&self) -> Result<T, CacheError> {
        serde_json::from_slice(&self.data)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))
    }

    pub fn serialize<T: serde::Serialize>(value: &T) -> Result<Self, CacheError> {
        let data = serde_json::to_vec(value)
            .map_err(|e| CacheError::SerializationError(e.to_string()))?;
        Ok(Self::new(data))
    }
}

/// Cache entry with metadata.
#[derive(Debug, Clone)]
pub struct Entry {
    pub key: CacheKey,
    pub value: CacheValue,
    pub ttl: Option<chrono::Duration>,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
}

impl Entry {
    pub fn new(key: CacheKey, value: CacheValue) -> Self {
        let now = Utc::now();
        Self {
            key,
            value,
            ttl: None,
            expires_at: None,
            access_count: 0,
            last_accessed: now,
        }
    }

    pub fn with_ttl(mut self, ttl: chrono::Duration) -> Self {
        self.ttl = Some(ttl);
        self.expires_at = Some(Utc::now() + ttl);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_accessed = Utc::now();
    }

    pub fn remaining_ttl(&self) -> Option<chrono::Duration> {
        self.expires_at.map(|exp| {
            let remaining = exp - Utc::now();
            if remaining < chrono::Duration::zero() {
                chrono::Duration::zero()
            } else {
                remaining
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key() {
        let key1 = CacheKey::from("test");
        let key2 = CacheKey::from("test");
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_cache_value_serde() {
        let value: String = "hello".to_string();
        let cache_value = CacheValue::serialize(&value).unwrap();
        let decoded: String = cache_value.deserialize().unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_entry_expiry() {
        let mut entry = Entry::new(
            CacheKey::from("test"),
            CacheValue::new(vec![]),
        ).with_ttl(chrono::Duration::hours(1));

        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl().is_some());

        entry.expires_at = Some(Utc::now() - chrono::Duration::hours(1));
        assert!(entry.is_expired());
    }
}
