# AGENTS.md — Stashly

Extends shelf-level AGENTS.md rules for Stashly.

## Project Identity

- **Name**: Stashly
- **Description**: Universal caching abstraction with TTL, invalidation, and multi-backend support
- **Language**: Rust

## Project-Specific Rules

### Test-First Mandate

- **For NEW modules**: test file MUST exist before implementation file
- **For BUG FIXES**: failing test MUST be written before the fix
- **For REFACTORS**: existing tests must pass before AND after

### Quality Gates

All PRs must pass:
- `cargo test --workspace`
- `cargo clippy --workspace -- -D warnings`
- `cargo fmt --check`

### Commit Messages

Format: `<type>(<scope>): <description>`

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `ci`

### File Organization

```
src/
├── lib.rs           # Main library entry
├── cache/           # Cache traits and implementations
├── backend/         # Backend adapters
├── eviction/        # Eviction strategies
└── ttl/            # TTL management
```

## Testing Requirements

- Unit tests for all public APIs
- Property-based tests for cache behavior
- Integration tests for backend adapters
- Minimum 80% code coverage
