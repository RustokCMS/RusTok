# rustok-iggy module implementation plan

## Scope and objective

This document captures the implementation plan for `rustok-iggy` in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-iggy`.

Primary objective: provide a production-ready streaming event transport using Iggy.rs
while preserving compatibility with platform-level contracts.

## Target architecture

```
┌─────────────────────────────────────────────────────┐
│                    rustok-iggy                      │
│  Responsibilities:                                  │
│  - EventTransport implementation                    │
│  - Event serialization (JSON/Bincode)              │
│  - Topology management, consumer groups            │
│  - DLQ, replay, health checks                      │
└──────────────────────┬──────────────────────────────┘
                       │ delegates connection to
                       ▼
┌─────────────────────────────────────────────────────┐
│               rustok-iggy-connector                 │
│  Responsibilities:                                  │
│  - Embedded/Remote mode switching                  │
│  - Connection lifecycle (connect, shutdown)        │
│  - Message I/O (publish, subscribe)                │
└─────────────────────────────────────────────────────┘
```

- `rustok-iggy` implements `EventTransport` trait from `rustok-core`
- Delegates to `rustok-iggy-connector` for Embedded/Remote mode abstraction
- Supports both JSON and Bincode serialization formats
- Provides topology management, consumer groups, DLQ, and replay capabilities
- Observability via tracing and health checks

## Delivery phases

### Phase 0 — Foundation ✅ DONE

- [x] Baseline crate/module structure
- [x] Base docs and registry presence
- [x] Core compile-time integration with the workspace
- [x] IggyTransport implementing EventTransport

### Phase 1 — Core Implementation ✅ DONE

- [x] Fix BincodeSerializer (was returning errors)
- [x] TopologyManager with actual state management
- [x] ConsumerGroupManager with group tracking
- [x] DlqManager with entry structure
- [x] ReplayManager with configuration
- [x] Health check functionality
- [x] Unit tests for all components
- [x] Partition calculation utilities
- [x] Remove unused backend/embedded stubs

### Phase 2 — Integration (in progress)

- [ ] Full Iggy SDK integration with `iggy` feature flag
- [ ] Real message consumption via MessageSubscriber
- [ ] Consumer group offset management
- [ ] Actual DLQ message movement
- [ ] Event replay implementation

### Phase 3 — Productionization (planned)

- [ ] Performance benchmarks
- [ ] Connection pooling and reconnection logic
- [ ] Metrics integration (prometheus)
- [ ] Error recovery strategies
- [ ] Security hardening (TLS, auth)
- [ ] Runbooks and operational documentation

## Component status

| Component | Status | Notes |
|-----------|--------|-------|
| `config.rs` | ✅ Complete | Full configuration with serialization |
| `transport.rs` | ✅ Complete | EventTransport implementation |
| `serialization.rs` | ✅ Complete | JSON + Bincode with tests |
| `topology.rs` | ✅ Complete | State management with tests |
| `consumer.rs` | ✅ Complete | Group management with tests |
| `producer.rs` | ✅ Complete | Request building with tests |
| `partitioning.rs` | ✅ Complete | Deterministic partitioning |
| `dlq.rs` | ✅ Complete | DLQ manager with tests |
| `health.rs` | ✅ Complete | Health check with status |
| `replay.rs` | ✅ Complete | Replay configuration |
| `backend/` | ❌ Removed | Unused, functionality in connector |

## Tracking and updates

When updating `rustok-iggy` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-iggy/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.

## Testing strategy

### Unit tests

Each module has comprehensive unit tests:
- Config parsing and defaults
- Serialization roundtrips
- Partition calculation determinism
- Component lifecycle management

### Integration tests

Located in `tests/integration.rs`:
- Require running Iggy backend (marked `#[ignore]`)
- Test full transport lifecycle
- Test both modes (embedded/remote)

### Manual testing

```bash
# Start Iggy server (via Docker)
docker run -p 8090:8090 -p 3000:3000 iggyrs/iggy:latest

# Run integration tests
cargo test -p rustok-iggy --ignored
```
