//! Integration tests for the `TieredCache` adapter via the public `CachePort` / `CacheWritePort` ports.

use std::thread;
use std::time::Duration;

use stashly::{
    domain::value_objects::{CacheTier, Ttl},
    ports::driven::{CachePort, CacheWritePort, StatsPort},
    TieredCache,
};

#[test]
fn set_and_get_roundtrip() {
    let mut cache = TieredCache::new();
    cache.set("key".into(), "value".into()).unwrap();
    assert_eq!(cache.get(&"key".into()), Some("value".into()));
}

#[test]
fn get_returns_none_for_missing_key() {
    let cache = TieredCache::new();
    assert!(cache.get(&"absent".into()).is_none());
}

#[test]
fn remove_deletes_entry_from_both_tiers() {
    let mut cache = TieredCache::new();
    cache.set("k".into(), "v".into()).unwrap();
    cache.remove(&"k".into()).unwrap();
    assert!(cache.get(&"k".into()).is_none());
}

#[test]
fn clear_all_tiers() {
    let mut cache = TieredCache::new();
    for i in 0..3u32 {
        cache.set(format!("k{i}").into(), format!("v{i}").into()).unwrap();
    }
    let removed = cache.clear(None).unwrap();
    // Each entry lives in L1 and L2, so removed == 3 * 2.
    assert_eq!(removed, 6);
    assert!(cache.get(&"k0".into()).is_none());
}

#[test]
fn clear_l1_only_leaves_l2_intact() {
    let mut cache = TieredCache::new();
    cache.set("x".into(), "y".into()).unwrap();

    // Clear only L1; L2 should still hold the entry.
    cache.clear(Some(CacheTier::L1)).unwrap();

    // get_internal checks L1 first then L2 — L2 hit should still return the value.
    assert_eq!(cache.get(&"x".into()), Some("y".into()));
}

#[test]
fn ttl_expiration_returns_none_after_expiry() {
    let mut cache = TieredCache::new();
    cache.set_with_ttl("ttl-key".into(), "ttl-val".into(), Ttl::from_millis(5)).unwrap();

    thread::sleep(Duration::from_millis(20));
    assert!(cache.get(&"ttl-key".into()).is_none());
}

#[test]
fn stats_track_hits_and_misses() {
    let mut cache = TieredCache::new();
    cache.set("s".into(), "v".into()).unwrap();

    cache.get(&"s".into()); // hit
    cache.get(&"absent".into()); // miss

    let stats = cache.get_stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 1);
    assert!((stats.hit_rate() - 0.5).abs() < 0.01);
}

#[test]
fn stats_size_reflects_live_entries() {
    let mut cache = TieredCache::new();
    cache.set("a".into(), "1".into()).unwrap();
    cache.set("b".into(), "2".into()).unwrap();

    // Each entry is mirrored in L1 and L2; size counts both tiers.
    let stats = cache.get_stats();
    assert_eq!(stats.size, 4);
}

#[test]
fn overwrite_same_key_returns_new_value() {
    let mut cache = TieredCache::new();
    cache.set("k".into(), "old".into()).unwrap();
    cache.set("k".into(), "new".into()).unwrap();
    assert_eq!(cache.get(&"k".into()), Some("new".into()));
}
