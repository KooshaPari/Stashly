# CLAUDE.md — Stashly

## Project Identity

- **Name**: Stashly
- **Description**: Universal caching abstraction with TTL, invalidation, and multi-backend support
- **Location**: `remote-clones/Stashly/`
- **Language**: Rust
- **License**: MIT OR Apache-2.0

## Architecture

Hexagonal (Ports & Adapters):
- Cache trait is the port (interface)
- Backend implementations are adapters (in-memory, Redis, Memcached, etc.)
- TTL management is built-in
- Eviction policies are pluggable

## Quick Commands

```bash
# Build
cargo build

# Test
cargo test

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --check

# Documentation
cargo doc --open
```

## Key Files

| Path | Purpose |
|------|---------|
| `src/lib.rs` | Main library entry |
| `src/cache.rs` | Cache trait definition |
| `src/backend/` | Backend implementations |
| `src/eviction.rs` | Eviction policies |

## Testing Requirements

- Unit tests for all public APIs
- Property-based tests for cache behavior
- Integration tests for backend adapters
- Minimum 80% code coverage
