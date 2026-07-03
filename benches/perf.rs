use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use stashly::domain::value_objects::{CacheKey as VoKey, CacheValue as VoVal};
use stashly::ports::driven::{CachePort, CacheWritePort};
use stashly::TieredCache;
use stashly::{
    domain::{Cache, CacheKey, CacheValue},
    InMemoryCache,
};

// ── InMemoryCache benchmarks ─────────────────────────────────────────────────

fn bench_inmemory_set(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = InMemoryCache::new(10_000);

    c.bench_function("inmemory/set", |b| {
        b.to_async(&rt).iter(|| async {
            let key = CacheKey::from("bench-key");
            let value = CacheValue::serialize(&black_box("bench-value")).unwrap();
            cache.set(key, value).await.unwrap();
        });
    });
}

fn bench_inmemory_get_hit(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = InMemoryCache::new(10_000);

    // Warm up: insert key
    rt.block_on(async {
        let key = CacheKey::from("bench-key");
        let value = CacheValue::serialize(&"bench-value").unwrap();
        cache.set(key, value).await.unwrap();
    });

    c.bench_function("inmemory/get_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let key = CacheKey::from("bench-key");
            black_box(cache.get(&key).await.unwrap());
        });
    });
}

fn bench_inmemory_get_miss(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let cache = InMemoryCache::new(10_000);

    c.bench_function("inmemory/get_miss", |b| {
        b.to_async(&rt).iter(|| async {
            let key = CacheKey::from("nonexistent");
            black_box(cache.get(&key).await.unwrap());
        });
    });
}

fn bench_inmemory_evict(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("inmemory/eviction");
    for cap in [10usize, 100, 1000] {
        group.throughput(Throughput::Elements(cap as u64));
        group.bench_with_input(BenchmarkId::from_parameter(cap), &cap, |b, &cap| {
            b.to_async(&rt).iter(|| async move {
                let cache = InMemoryCache::new(cap);
                // Insert cap+1 entries to trigger eviction on every insert beyond cap.
                for i in 0..=cap {
                    let key = CacheKey::from(format!("key-{i}"));
                    let value = CacheValue::serialize(&i).unwrap();
                    cache.set(key, value).await.unwrap();
                }
                black_box(cache.len().await.unwrap());
            });
        });
    }
    group.finish();
}

// ── TieredCache benchmarks ───────────────────────────────────────────────────

fn bench_tiered_set(c: &mut Criterion) {
    let mut cache = TieredCache::new();
    c.bench_function("tiered/set", |b| {
        b.iter(|| {
            let key = VoKey::new("bench-key");
            let value = VoVal::new(black_box("bench-value"));
            let _ = cache.set(key, value);
        });
    });
}

fn bench_tiered_get_hit(c: &mut Criterion) {
    let mut cache = TieredCache::new();
    let key = VoKey::new("bench-key");
    cache.set(key.clone(), VoVal::new("bench-value")).unwrap();

    c.bench_function("tiered/get_hit", |b| {
        b.iter(|| {
            black_box(cache.get(&key));
        });
    });
}

fn bench_tiered_get_miss(c: &mut Criterion) {
    let cache = TieredCache::new();
    let key = VoKey::new("nonexistent");

    c.bench_function("tiered/get_miss", |b| {
        b.iter(|| {
            black_box(cache.get(&key));
        });
    });
}

criterion_group!(
    inmemory_benches,
    bench_inmemory_set,
    bench_inmemory_get_hit,
    bench_inmemory_get_miss,
    bench_inmemory_evict,
);
criterion_group!(tiered_benches, bench_tiered_set, bench_tiered_get_hit, bench_tiered_get_miss,);
criterion_main!(inmemory_benches, tiered_benches);
