//! Performance benchmarks for Stashly cache operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use stashly::adapters::memory::InMemoryCache;
use stashly::adapters::tiered::TieredCache;
use stashly::domain::policy::{EvictionPolicy, LfuPolicy, LruPolicy};
use stashly::domain::Cache;
use stashly::domain::CacheKey;
use stashly::ports::driven::{CachePort, CacheWritePort};

fn bench_inmemory_get_set(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("inmemory");

    group.bench_function("set_1000", |b| {
        b.iter_batched(
            || {
                let cache = InMemoryCache::new(10_000);
                let key = CacheKey::from(black_box("bench-key"));
                let value = stashly::domain::CacheValue::serialize(&42u64).unwrap();
                (cache, key, value)
            },
            |(cache, key, value)| {
                rt.block_on(async { cache.set(key, value).await.unwrap() });
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("get_hit", |b| {
        b.iter_batched(
            || {
                let cache = InMemoryCache::new(10_000);
                let key = CacheKey::from("bench-key");
                let value = stashly::domain::CacheValue::serialize(&42u64).unwrap();
                rt.block_on(async { cache.set(key.clone(), value).await.unwrap() });
                cache
            },
            |cache| {
                let key = CacheKey::from(black_box("bench-key"));
                rt.block_on(async { cache.get(&key).await.unwrap() });
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("get_miss", |b| {
        let cache = InMemoryCache::new(10_000);
        b.iter(|| {
            let key = CacheKey::from(black_box("nonexistent-key"));
            rt.block_on(async { cache.get(&key).await.unwrap() });
        })
    });

    group.finish();
}

fn bench_eviction_policy(c: &mut Criterion) {
    let mut group = c.benchmark_group("eviction_policy");

    group.bench_function("lru_record_1000", |b| {
        b.iter_batched(
            || LruPolicy::new(),
            |mut policy| {
                for i in 0..1000 {
                    policy.record_access(black_box(&format!("key{}", i)));
                }
                black_box(policy.select_eviction());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("lfu_record_1000", |b| {
        b.iter_batched(
            || LfuPolicy::new(),
            |mut policy| {
                for i in 0..1000 {
                    policy.record_access(black_box(&format!("key{}", i)));
                }
                black_box(policy.select_eviction());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_tiered_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("tiered");

    group.bench_function("set_get_100", |b| {
        b.iter_batched(
            || TieredCache::default(),
            |mut cache| {
                for i in 0..100 {
                    let key = stashly::domain::value_objects::CacheKey::from(black_box(
                        format!("key{}", i),
                    ));
                    let value =
                        stashly::domain::value_objects::CacheValue::from(black_box("value"));
                    cache.set(key, value).unwrap();
                }
                for i in 0..100 {
                    let key = stashly::domain::value_objects::CacheKey::from(format!("key{}", i));
                    black_box(cache.get(&key));
                }
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("cleanup_1000", |b| {
        b.iter_batched(
            || {
                let mut cache = TieredCache::default();
                for i in 0..1000 {
                    let key = stashly::domain::value_objects::CacheKey::from(format!("key{}", i));
                    let value =
                        stashly::domain::value_objects::CacheValue::from("value");
                    cache.set(key, value).unwrap();
                }
                cache
            },
            |cache| {
                black_box(cache.cleanup());
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

criterion_group!(benches, bench_inmemory_get_set, bench_eviction_policy, bench_tiered_cache);
criterion_main!(benches);
