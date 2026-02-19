# rustok-iggy docs

Documentation for the `crates/rustok-iggy` module.

## Documents

- [Implementation Plan](./implementation-plan.md) - Delivery phases and component status

## Module Overview

`rustok-iggy` provides event streaming transport using Iggy.rs. It implements the
`EventTransport` trait and delegates connection management to `rustok-iggy-connector`.

**Responsibility split:**
- `rustok-iggy-connector` — Embedded/Remote mode switching, connection lifecycle, IggyConnector trait
- `rustok-iggy` — EventTransport implementation, serialization, topology, DLQ, replay, consumer groups

## Key Types

| Type | Description |
|------|-------------|
| `IggyTransport` | Main transport implementing EventTransport |
| `IggyConfig` | Configuration for transport setup |
| `TopologyManager` | Stream/topic tracking |
| `ConsumerGroupManager` | Consumer group coordination |
| `DlqManager` | Dead letter queue handling |
| `ReplayManager` | Event replay orchestration |
| `EventSerializer` | JSON/Bincode serialization |

## Quick Reference

```rust
use rustok_iggy::{IggyConfig, IggyTransport, SerializationFormat};
use rustok_core::events::EventTransport;

// Create transport (connector handles mode switching internally)
let config = IggyConfig::default();
let transport = IggyTransport::new(config).await?;

// Use as EventTransport
transport.publish(envelope).await?;

// Cleanup
transport.shutdown().await?;
```

## Related Crates

- [rustok-iggy-connector](../../rustok-iggy-connector/README.md) — Connection layer (Embedded/Remote)
- [rustok-core](../../rustok-core/README.md) — EventTransport trait, EventEnvelope

## Configuration

See the main [README](../README.md) for configuration options.
