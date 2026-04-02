# PRD — Stashly

## Overview

`Stashly` is a universal caching abstraction framework for Rust, following hexagonal architecture principles. It provides TTL support, invalidation, and multi-backend support.

## Goals

- Provide a unified caching interface
- Support multiple cache backends (memory, Redis, Memcached)
- TTL and expiration management
- Configurable eviction policies (LRU, LFU, FIFO)
- Cache invalidation strategies

## Epics

### E1 — Cache Abstraction
- E1.1 Define Cache trait with get/put/delete operations
- E1.2 Support key-value and collection operations
- E1.3 Provide async and sync implementations

### E2 — Backend Adapters
- E2.1 In-memory cache backend
- E2.2 Redis backend adapter
- E2.3 Memcached backend adapter

### E3 — TTL and Expiration
- E3.1 TTL support per-entry
- E3.2 Automatic expiration
- E3.3 Time-to-idle and time-to-live options

### E4 — Eviction Policies
- E4.1 LRU (Least Recently Used)
- E4.2 LFU (Least Frequently Used)
- E4.3 FIFO (First In First Out)

## Acceptance Criteria

- Cache operations work with all backends
- TTL is respected correctly
- Eviction policies work correctly
- Error handling is consistent
