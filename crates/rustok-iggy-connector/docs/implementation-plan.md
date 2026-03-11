# rustok-iggy-connector module implementation plan

## Scope and objective

This document captures the implementation plan for `rustok-iggy-connector` in RusToK and
serves as the source of truth for rollout sequencing.

Primary objective: provide a stable abstraction layer for Iggy connectivity
supporting both embedded and remote modes.

## Target architecture

- `IggyConnector` trait defines the connector contract
- `RemoteConnector` implements external Iggy server connection
- `EmbeddedConnector` implements in-process Iggy server
- `MessageSubscriber` trait for message consumption
- Feature flag `iggy` enables full SDK integration

## Delivery phases

### Phase 0 — Foundation ✅ DONE

- [x] Baseline crate/module structure
- [x] Base docs and registry presence
- [x] Core compile-time integration with workspace

### Phase 1 — Contract Implementation ✅ DONE

- [x] `IggyConnector` trait with `connect`, `publish`, `subscribe`, `shutdown`
- [x] `RemoteConnector` implementation
- [x] `EmbeddedConnector` implementation
- [x] `ConnectorConfig` with embedded/remote settings
- [x] `PublishRequest` for message publishing
- [x] `MessageSubscriber` trait for consumption
- [x] `ConnectorError` with proper error variants
- [x] Partition calculation utilities
- [x] Unit tests for all components
- [x] Optional Iggy SDK support via feature flag

### Phase 2 — Integration (in progress)

- [ ] Full Iggy SDK integration when `iggy` feature enabled
- [ ] Consumer group offset management
- [ ] Message batching for high-throughput
- [ ] Connection pooling and reconnection
- [ ] TLS support verification

### Phase 3 — Productionization (planned)

- [ ] Performance benchmarks
- [ ] Health checks and metrics
- [ ] Security audit (TLS, auth)
- [ ] Runbooks and operational docs

## Component Status

| Component | Status | Notes |
|-----------|--------|-------|
| `lib.rs` | ✅ Complete | All public exports |
| `ConnectorMode` | ✅ Complete | Embedded/Remote enum |
| `ConnectorConfig` | ✅ Complete | Full configuration |
| `PublishRequest` | ✅ Complete | With builder methods |
| `IggyConnector` | ✅ Complete | Full trait |
| `RemoteConnector` | ✅ Complete | With Iggy SDK support |
| `EmbeddedConnector` | ✅ Complete | With lifecycle management |
| `MessageSubscriber` | ✅ Complete | Trait + implementations |
| `ConnectorError` | ✅ Complete | All error variants |
| `calculate_partition` | ✅ Complete | Deterministic hashing |

## Usage

See [README](../README.md) for usage examples.

## Testing

Unit tests cover:
- Configuration parsing
- Partition calculation
- Mode serialization
- Request building
- Error handling

Integration tests require:
- Running Iggy server (for remote mode)
- Or `iggy` feature enabled (for embedded mode)

## Checklist

- [x] контрактные тесты покрывают все публичные use-case.

