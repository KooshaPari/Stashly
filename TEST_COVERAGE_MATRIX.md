# Test Coverage Matrix - Stashly

**Project**: Stashly
**Document Version**: 1.2
**Last Updated**: 2026-06-29

---

## Coverage Summary

| Metric | Value |
|--------|-------|
| Functional Requirements | 12 (see FR-CACHE, FR-BACKEND, FR-TTL, FR-EVICT) |
| Test Files | 7 inline `#[cfg(test)]` modules |
| Test Functions | 38+ |
| Lines of Code | ~850 |
| Coverage Target | 80% |
| Current Coverage | ~30% (estimated, see coverage gaps) |

---

## Architecture

Hexagonal (Ports & Adapters):
- **Domain**: `src/domain/` - Pure business logic
- **Application**: `src/application/` - Use cases
- **Adapters**: `src/adapters/` - Implementations
- **Infrastructure**: `src/infrastructure/` - External concerns

---

## Test Categories

### Unit Tests
- **Location**: `src/**/` (inline `#[cfg(test)] mod tests` blocks)
- **Purpose**: Test individual components in isolation
- **Coverage Target**: 90%
- **Status**: PARTIALLY IMPLEMENTED

### Integration Tests
- **Location**: Tests exist inline in adapter modules
- **Purpose**: Test component interactions through adapter boundaries
- **Coverage Target**: 75%
- **Status**: PARTIALLY IMPLEMENTED (via tiered + memory adapter tests)

### Property-Based Tests
- **Location**: N/A
- **Purpose**: Randomized testing with shrinking
- **Coverage Target**: Key invariants
- **Status**: NOT IMPLEMENTED

---

## FR to Test Coverage Mapping

| FR ID | Description | Module | Test Location | Coverage Status |
|-------|-------------|--------|---------------|-----------------|
| FR-CACHE-001 | get() method | domain/cache.rs | `domain::cache::tests` | COVERED |
| FR-CACHE-002 | set() method | domain/cache.rs | `domain::cache::tests` | COVERED |
| FR-CACHE-003 | delete() method | domain/cache.rs | `adapters::memory::tests` | COVERED |
| FR-CACHE-004 | Async support | adapters/ | `adapters::memory::tests` | COVERED |
| FR-BACKEND-001 | In-memory backend | adapters/memory.rs | `adapters::memory::tests` | COVERED |
| FR-BACKEND-002 | Redis backend | N/A | N/A | NOT APPLICABLE |
| FR-BACKEND-003 | Backend trait | domain/ports.rs | `adapters::memory::tests` | COVERED |
| FR-TTL-001 | Entry TTL | domain/cache.rs | `domain::cache::tests` | COVERED |
| FR-TTL-002 | Auto expiration | adapters/tiered/ | `adapters::tiered::tests` | COVERED |
| FR-TTL-003 | TTL options | domain/cache.rs | `domain::cache::tests` | COVERED |
| FR-EVICT-001 | LRU policy | domain/policy.rs | `domain::policy::tests` | COVERED |
| FR-EVICT-002 | LFU policy | domain/policy.rs | `domain::policy::tests` | COVERED |
| FR-EVICT-003 | FIFO policy | N/A | N/A | NOT IMPLEMENTED |

---

## Test File Index

| Test Module | Location | Purpose | Tests |
|-------------|----------|---------|-------|
| domain::cache::tests | src/domain/cache.rs | CacheKey, CacheValue serde, Entry expiry | 3 |
| domain::policy::tests | src/domain/policy.rs | LRU/LFU eviction policies | 2 |
| domain::errors::tests | src/domain/errors.rs | CacheError display + serialize | 2 |
| domain::entities::tests | src/domain/entities/mod.rs | CacheEntry, SingleflightRequest, CrossProcessRequest | 5 |
| domain::events::tests | src/domain/events/mod.rs | CacheEvent variants, EvictionReason display | 3 |
| domain::value_objects::tests | src/domain/value_objects/mod.rs | CacheKey, Ttl, CacheStats | 4 |
| adapters::memory::tests | src/adapters/memory.rs | Basic operations, eviction, remove, concurrent access | 7 |
| adapters::tiered::tests | src/adapters/tiered/mod.rs | Basic operations, TTL, cleanup | 7 |
| application::services::tests | src/application/services.rs | CacheService operations | 5 |
| infrastructure::error::tests | src/infrastructure/error.rs | CacheKitError display | 2 |

---

## Coverage Gaps

### Critical Gaps
1. **Redis backend** - Not implemented as a backend yet
2. **FIFO eviction policy** - Not implemented

### Partial Coverage
1. Most domain types are tested inline
2. Concurrency testing added for InMemoryCache (reads + writes + mixed)
3. Tiered cache has basic operation + TTL tests

---

## Recommendations

### Immediate Actions
1. Benchmarks now implemented with Criterion (real workload targets)
2. Concurrency tests added for InMemoryCache
3. All error types and events now have unit tests

### Short-term Actions (This Sprint)
1. Add integration tests for cross-module interactions
2. Target: 50% coverage

### Medium-term Actions (This Month)
1. Add Redis integration tests (when backend is implemented)
2. Add property-based tests for serialization
3. Target: 80% coverage

---

**Total Functional Requirements**: 12
**Covered**: 11 (1 N/A, 1 not implemented)
**Coverage Percentage**: ~30% (estimated)
**Last Updated**: 2026-06-29
