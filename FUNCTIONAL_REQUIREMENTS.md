# Functional Requirements — Stashly

## FR-CACHE: Cache Operations

### FR-CACHE-001: Get Operation
The system SHALL provide a `get` method to retrieve values by key.
**Traces to:** E1.1
**Code Location:** `src/domain/cache.rs`

### FR-CACHE-002: Put Operation
The system SHALL provide a `put` method to store key-value pairs with optional TTL.
**Traces to:** E1.1
**Code Location:** `src/domain/cache.rs`

### FR-CACHE-003: Delete Operation
The system SHALL provide a `delete` method to remove entries by key.
**Traces to:** E1.1
**Code Location:** `src/domain/cache.rs`

### FR-CACHE-004: Async Support
The system SHALL provide async versions of all cache operations.
**Traces to:** E1.3
**Code Location:** `src/adapters/`

## FR-BACKEND: Backend Adapters

### FR-BACKEND-001: In-Memory Backend
The system SHALL provide an in-memory cache implementation using DashMap.
**Traces to:** E2.1
**Code Location:** `src/adapters/memory.rs`

### FR-BACKEND-002: Redis Backend
The system SHALL provide a Redis backend adapter.
**Traces to:** E2.2
**Code Location:** `src/adapters/redis.rs`

### FR-BACKEND-003: Backend Trait
The system SHALL define a `Backend` trait that all adapters implement.
**Traces to:** E2.0
**Code Location:** `src/domain/backend.rs`

## FR-TTL: TTL Management

### FR-TTL-001: Entry TTL
The system SHALL support setting TTL per cache entry.
**Traces to:** E3.1
**Code Location:** `src/domain/entry.rs`

### FR-TTL-002: Auto Expiration
The system SHALL automatically expire entries past their TTL.
**Traces to:** E3.2
**Code Location:** `src/application/`

### FR-TTL-003: TTL Options
The system SHALL support both time-to-live and time-to-idle expiration.
**Traces to:** E3.3
**Code Location:** `src/domain/entry.rs`

## FR-EVICT: Eviction Policies

### FR-EVICT-001: LRU Policy
The system SHALL implement Least Recently Used eviction.
**Traces to:** E4.1
**Code Location:** `src/domain/eviction.rs`

### FR-EVICT-002: LFU Policy
The system SHALL implement Least Frequently Used eviction.
**Traces to:** E4.2
**Code Location:** `src/domain/eviction.rs`

### FR-EVICT-003: FIFO Policy
The system SHALL implement First In First Out eviction.
**Traces to:** E4.3
**Code Location:** `src/domain/eviction.rs`
