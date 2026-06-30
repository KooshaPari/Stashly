//! Integration tests for the `InMemoryCache` adapter via the public `Cache` port.

use stashly::{
    domain::{Cache, CacheKey, CacheValue},
    InMemoryCache,
};

#[tokio::test]
async fn set_and_get_roundtrip() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("hello");
    let value = CacheValue::serialize(&"world".to_string()).unwrap();

    cache.set(key.clone(), value).await.unwrap();
    let result = cache.get(&key).await.unwrap().expect("key should be present");
    let decoded: String = result.deserialize().unwrap();
    assert_eq!(decoded, "world");
}

#[tokio::test]
async fn get_returns_none_for_missing_key() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("absent");
    assert!(cache.get(&key).await.unwrap().is_none());
}

#[tokio::test]
async fn remove_deletes_entry() {
    let cache = InMemoryCache::new(100);
    let key = CacheKey::from("to-remove");
    let value = CacheValue::serialize(&42u32).unwrap();

    cache.set(key.clone(), value).await.unwrap();
    assert!(cache.contains(&key).await.unwrap());

    cache.remove(&key).await.unwrap();
    assert!(!cache.contains(&key).await.unwrap());
    assert!(cache.get(&key).await.unwrap().is_none());
}

#[tokio::test]
async fn clear_empties_the_cache() {
    let cache = InMemoryCache::new(100);

    for i in 0u32..5 {
        let key = CacheKey::from(format!("k{i}"));
        let value = CacheValue::serialize(&i).unwrap();
        cache.set(key, value).await.unwrap();
    }

    assert_eq!(cache.len().await.unwrap(), 5);
    cache.clear().await.unwrap();
    assert!(cache.is_empty().await.unwrap());
}

#[tokio::test]
async fn lru_eviction_respects_capacity() {
    // capacity = 2; inserting entries beyond capacity triggers LRU eviction.
    let cache = InMemoryCache::new(2);

    cache.set(CacheKey::from("a"), CacheValue::serialize(&1u32).unwrap()).await.unwrap();
    cache.set(CacheKey::from("b"), CacheValue::serialize(&2u32).unwrap()).await.unwrap();
    // Insert "c" to overflow; one of the first two must be evicted.
    cache.set(CacheKey::from("c"), CacheValue::serialize(&3u32).unwrap()).await.unwrap();

    // After eviction the cache must honour its capacity.
    assert_eq!(cache.len().await.unwrap(), 2);
    // "c" (most recently inserted) must always survive.
    assert!(cache.contains(&CacheKey::from("c")).await.unwrap());
}

#[tokio::test]
async fn overwrite_same_key() {
    let cache = InMemoryCache::new(10);
    let key = CacheKey::from("dup");

    cache.set(key.clone(), CacheValue::serialize(&1u32).unwrap()).await.unwrap();
    cache.set(key.clone(), CacheValue::serialize(&2u32).unwrap()).await.unwrap();

    let val: u32 = cache.get(&key).await.unwrap().unwrap().deserialize().unwrap();
    assert_eq!(val, 2);
    // Only one entry per unique key.
    assert_eq!(cache.len().await.unwrap(), 1);
}

#[tokio::test]
async fn serialization_roundtrip_for_complex_type() {
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
    struct Record {
        id: u64,
        name: String,
    }

    let cache = InMemoryCache::new(10);
    let key = CacheKey::from("record");
    let record = Record { id: 42, name: "stashly".to_string() };
    let value = CacheValue::serialize(&record).unwrap();

    cache.set(key.clone(), value).await.unwrap();
    let decoded: Record = cache.get(&key).await.unwrap().unwrap().deserialize().unwrap();
    assert_eq!(decoded, record);
}
