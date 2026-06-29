use criterion::{black_box, criterion_group, criterion_main, Criterion};

use stashly::adapters::memory::InMemoryCache;
use stashly::domain::cache::{CacheKey, CacheValue};
use stashly::domain::ports::Cache;

fn bench_cache_basic(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("cache_set_1k", |b| {
        b.iter(|| {
            rt.block_on(async {
                let cache = InMemoryCache::new(10_000);
                for i in 0..1000 {
                    let key = CacheKey::from(format!("key_{}", i));
                    let value = CacheValue::serialize(&i).unwrap();
                    cache.set(key, value).await.unwrap();
                }
                black_box(cache.len().await.unwrap());
            });
        });
    });

    c.bench_function("cache_get_hit", |b| {
        let cache = InMemoryCache::new(10_000);
        rt.block_on(async {
            let key = CacheKey::from("bench_key");
            let value = CacheValue::serialize(&42i32).unwrap();
            cache.set(key.clone(), value).await.unwrap();
        });

        b.iter(|| {
            rt.block_on(async {
                let key = CacheKey::from("bench_key");
                let result = cache.get(&key).await.unwrap();
                black_box(result);
            });
        });
    });
}

criterion_group!(benches, bench_cache_basic);
criterion_main!(benches);
