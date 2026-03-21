# rustok-tenant docs

В этой папке хранится документация модуля `crates/rustok-tenant`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Canonical scope

This file is the canonical module-level documentation for `crates/rustok-tenant`.
Module-specific tenancy documentation must stay in this crate, not under `docs/architecture/`.

## Platform invariants

RusToK is multi-tenant by default, and `rustok-tenant` owns the tenant domain contract:

- every tenant-scoped entity must carry `tenant_id`;
- a query without tenant filtering is a critical security bug;
- tenant resolution happens in middleware before request handling reaches business logic;
- module enablement is tenant-scoped through `tenant_modules`.

## Module boundary

`crates/rustok-tenant` owns:

- tenant and tenant-module entities/DTOs/services;
- public CRUD and module-toggle contracts;
- tenant-aware domain logic exposed to the rest of the platform.

`apps/server` owns:

- middleware resolution entry point in `apps/server/src/middleware/tenant.rs`;
- cache infrastructure and cross-instance invalidation wiring;
- health, metrics, and runtime bootstrap around the tenant resolver path.

## Resolution and cache model

Server middleware resolves tenant context by one of three keys:

- `uuid`
- `slug`
- `host`

Current cache model uses `TenantCacheInfrastructure` in `AppContext.shared_store`:

| Layer | Purpose | TTL | Capacity |
|------|---------|-----|----------|
| Positive cache | Resolved tenant context | 5 min | 1000 |
| Negative cache | Not-found protection | 60 sec | 1000 |

Additional runtime properties:

- Redis is used when `RUSTOK_REDIS_URL` or `REDIS_URL` is configured; otherwise server falls back to in-memory cache.
- Cache keys are versioned (`v1:<type>:<value>`) to support schema evolution.
- Cross-instance invalidation goes through Redis pub/sub channel `tenant.cache.invalidate`.
- Concurrent miss storms are coalesced with singleflight-style protection in the server cache layer.

## Data model

Core storage owned by the tenant module:

- `tenants` stores tenant registry data and opaque tenant settings.
- `tenant_modules` stores per-tenant module enablement and tenant-scoped module settings.

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)

## Related docs

- [Server docs](../../../apps/server/docs/README.md)
- [Cache stampede protection](../../../apps/server/docs/CACHE_STAMPEDE_PROTECTION.md)
