> **Work-state:** beta — `[########--] 80%`
>
> Multi-tier caching framework (L1/L2/L3), singleflight, TTL, CQRS unified. Backends (Memory, Redis, Memcached, Disk) ported. Tests + metrics wired. Pending: load testing, persistence layer verification, edge-case stress tests.

# Stashly

**Universal caching abstraction with TTL, multi-tier, singleflight, and multi-backend support.**

A hexagonal architecture-based caching framework that absorbs thegent-cache.

## Features

- **Multiple Backends**: Memory, Redis, Memcached, Disk
- **Multi-Tier Caching**: L1 (LRU) + L2 (concurrent) + L3 (persistent)
- **Singleflight**: Deduplicate concurrent requests for the same key
- **TTL Support**: Automatic expiration with configurable duration
- **CQRS**: Separate command and query interfaces
- **Domain Events**: Cache operation events for observability
- **Invalidation Strategies**: LRU, LFU, TTL-based
- **Async/Await**: Full async support with tokio
- **Serialization**: Automatic serialization of cached values
- **Metrics**: Hit/miss rates, latency tracking, per-tier statistics

## Absorbed Projects

- **thegent-cache** — Multi-tier caching (L1/L2), singleflight support, CQRS, domain events

## Architecture

```
Stashly/
├── src/
│   ├── domain/          # Core domain logic (pure)
│   │   ├── cache/       # Cache entities (original)
│   │   ├── entities/    # Domain entities (from thegent-cache)
│   │   ├── events/      # Domain events (from thegent-cache)
│   │   ├── value_objects/ # Value objects (from thegent-cache)
│   │   ├── policy/      # Eviction policies
│   │   ├── ports/       # Interface definitions
│   │   └── errors/      # Domain errors
│   ├── application/     # Application services (CQRS)
│   ├── adapters/        # Backend adapters
│   │   ├── memory/     # In-memory cache (original)
│   │   ├── tiered/     # L1/L2 tiered cache (from thegent-cache)
│   │   └── redis/      # Redis backend
│   ├── ports/           # Hexagonal ports (from thegent-cache)
│   └── infrastructure/  # Cross-cutting concerns
├── tests/               # Integration tests
├── examples/            # Usage examples
└── benches/             # Benchmarks
```

## Quick Start

```rust
use stashly::{InMemoryCache, TieredCache, CachePort, CacheWritePort};

// Simple in-memory cache
let cache = InMemoryCache::new();

// Tiered cache (L1 + L2)
let tiered = TieredCache::with_config(1000, 10000, std::time::Duration::from_secs(3600));
```

## Installation

```toml
[dependencies]
stashly = { git = "https://github.com/KooshaPari/Stashly" }
```
[dependencies]
cachekit = "0.1"
```

## Quick Start

```rust
use cachekit::{Cache, InMemoryCache};

let cache = InMemoryCache::new(1000);
cache.set("key", "value", Duration::from_secs(60)).await?;
let value = cache.get("key").await?;
```

## Documentation

- [API Documentation](https://docs.rs/cachekit)
- [User Guide](https://cachekit.dev/guide)
- [xDD Methodologies](STANDARDS.md)

## License

MIT OR Apache-2.0

/// @trace STASH-001

/// @trace STASH-001
