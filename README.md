> **Work-state:** docs polish — `[########--] 80%`

# Stashly

Stashly is a Rust caching framework for building fast, composable cache layers with TTL, singleflight deduplication, CQRS-style read/write separation, and multiple storage backends across memory, Redis, Memcached, and disk. It is structured as a hexagonal architecture so the cache core stays portable while adapters handle backend-specific behavior, metrics, and persistence concerns.

## Usage / Quickstart

Add Stashly to your project:

```toml
[dependencies]
stashly = { git = "https://github.com/KooshaPari/Stashly" }
```

Create a cache and use it:

```rust
use stashly::InMemoryCache;

let cache = InMemoryCache::new();
cache.set("key", "value").await?;
let value = cache.get("key").await?;
```

For multi-tier caching, use the tiered adapter with your preferred capacity and TTL settings.

## Features

- Multiple backends: Memory, Redis, Memcached, Disk
- Multi-tier caching: L1, L2, L3
- Singleflight request deduplication
- TTL expiration
- CQRS-oriented interfaces
- Domain events and metrics

## Documentation

- [API Documentation](https://docs.rs/cachekit)
- [User Guide](https://cachekit.dev/guide)
- [Standards](STANDARDS.md)

## License

MIT OR Apache-2.0
