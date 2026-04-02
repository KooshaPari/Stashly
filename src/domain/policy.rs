//! Eviction policies.

/// Trait for eviction policies.
pub trait EvictionPolicy: Send + Sync {
    /// Record an access to an entry.
    fn record_access(&mut self, key: &str);

    /// Select an entry for eviction.
    fn select_eviction(&self) -> Option<String>;

    /// Remove an entry from tracking.
    fn remove(&mut self, key: &str);

    /// Clear all tracking.
    fn clear(&mut self);
}

/// LRU (Least Recently Used) eviction policy.
pub struct LruPolicy {
    order: Vec<String>,
    access_order: std::collections::HashMap<String, usize>,
}

impl LruPolicy {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            access_order: std::collections::HashMap::new(),
        }
    }
}

impl Default for LruPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl EvictionPolicy for LruPolicy {
    fn record_access(&mut self, key: &str) {
        self.access_order.remove(key);
        self.access_order.insert(key.to_string(), self.order.len());
        self.order.push(key.to_string());
    }

    fn select_eviction(&self) -> Option<String> {
        self.order.first().cloned()
    }

    fn remove(&mut self, key: &str) {
        self.order.retain(|k| k != key);
        self.access_order.remove(key);
    }

    fn clear(&mut self) {
        self.order.clear();
        self.access_order.clear();
    }
}

/// LFU (Least Frequently Used) eviction policy.
pub struct LfuPolicy {
    access_counts: std::collections::HashMap<String, u64>,
}

impl LfuPolicy {
    pub fn new() -> Self {
        Self {
            access_counts: std::collections::HashMap::new(),
        }
    }
}

impl Default for LfuPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl EvictionPolicy for LfuPolicy {
    fn record_access(&mut self, key: &str) {
        *self.access_counts.entry(key.to_string()).or_insert(0) += 1;
    }

    fn select_eviction(&self) -> Option<String> {
        self.access_counts
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(k, _)| k.clone())
    }

    fn remove(&mut self, key: &str) {
        self.access_counts.remove(key);
    }

    fn clear(&mut self) {
        self.access_counts.clear();
    }
}

/// TTL-based eviction policy.
pub struct TtlPolicy {
    expiration_order: Vec<(String, chrono::DateTime<chrono::Utc>)>,
}

impl TtlPolicy {
    pub fn new() -> Self {
        Self {
            expiration_order: Vec::new(),
        }
    }
}

impl Default for TtlPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl EvictionPolicy for TtlPolicy {
    fn record_access(&mut self, _key: &str) {
        // TTL policy doesn't track access
    }

    fn select_eviction(&self) -> Option<String> {
        let now = chrono::Utc::now();
        self.expiration_order
            .iter()
            .find(|(_, expires)| *expires < now)
            .map(|(k, _)| k.clone())
    }

    fn remove(&mut self, key: &str) {
        self.expiration_order.retain(|(k, _)| k != key);
    }

    fn clear(&mut self) {
        self.expiration_order.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lru_policy() {
        let mut policy = LruPolicy::new();

        policy.record_access("a");
        policy.record_access("b");
        policy.record_access("c");

        assert_eq!(policy.select_eviction(), Some("a".to_string()));

        policy.remove("a");
        assert_eq!(policy.select_eviction(), Some("b".to_string()));
    }

    #[test]
    fn test_lfu_policy() {
        let mut policy = LfuPolicy::new();

        for _ in 0..3 { policy.record_access("a"); }
        for _ in 0..1 { policy.record_access("b"); }
        for _ in 0..2 { policy.record_access("c"); }

        assert_eq!(policy.select_eviction(), Some("b".to_string()));
    }
}
