# rustok-iggy

Event streaming transport for the RusToK platform using [Iggy](https://iggy.rs).

## Overview

`rustok-iggy` provides L2 streaming transport (streaming + replay) for domain events. It implements the `EventTransport` trait from `rustok-core` and delegates connection management to `rustok-iggy-connector`.

**Key distinction:**
- `rustok-iggy-connector` — handles Embedded/Remote mode switching, connection lifecycle, and low-level message I/O
- `rustok-iggy` — provides EventTransport implementation, serialization, topology management, and higher-level abstractions (DLQ, replay, consumer groups)

## Features

- **EventTransport Implementation**: Seamless integration with RusToK event system
- **Automatic Topology**: Streams and topics are created automatically
- **Tenant Partitioning**: Events are partitioned by tenant ID for ordering guarantees
- **Multiple Serialization Formats**: JSON (default) and Bincode for high-throughput scenarios
- **Consumer Groups**: Support for distributed consumers via consumer groups
- **Dead Letter Queue**: DLQ support for failed message handling
- **Event Replay**: Replay events for recovery or reprocessing

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    rustok-iggy                      │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │IggyTransport│  │TopologyMgr   │  │DlqManager │  │
│  │(EventTrans- │  │ConsumerGroup │  │ReplayMgr  │  │
│  │  port)      │  │Serialization │  │Health     │  │
│  └──────┬──────┘  └──────────────┘  └───────────┘  │
└─────────┼───────────────────────────────────────────┘
          │ uses
          ▼
┌─────────────────────────────────────────────────────┐
│               rustok-iggy-connector                 │
│  ┌────────────────┐  ┌────────────────┐            │
│  │EmbeddedConnector│  │RemoteConnector │            │
│  │(in-process)    │  │(external server)│           │
│  └────────┬───────┘  └───────┬────────┘            │
└───────────┼──────────────────┼─────────────────────┘
            │                  │
     ┌──────▼──────┐    ┌──────▼──────┐
     │Embedded Iggy│    │Iggy Cluster │
     │  (server)   │    │  (remote)   │
     └─────────────┘    └─────────────┘
```

## Usage

### Basic Setup

```rust
use rustok_iggy::{IggyConfig, IggyTransport};
use rustok_core::events::EventTransport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = IggyConfig::default();
    let transport = IggyTransport::new(config).await?;
    
    // Transport implements EventTransport
    transport.shutdown().await?;
    Ok(())
}
```

### Configuration

```yaml
events:
  transport: iggy
  iggy:
    mode: embedded  # or "remote" — handled by rustok-iggy-connector
    serialization: json  # or "bincode"
    topology:
      stream_name: rustok
      domain_partitions: 8
      replication_factor: 1
    embedded:
      data_dir: ./data/iggy
      tcp_port: 8090
      http_port: 3000
    remote:
      addresses:
        - "127.0.0.1:8090"
      protocol: tcp
      username: rustok
      password: ${IGGY_PASSWORD}
      tls_enabled: false
    retention:
      domain_max_age_days: 30
      domain_max_size_gb: 10
      system_max_age_days: 7
      dlq_max_age_days: 365
```

### Mode Selection

Mode switching is handled by `rustok-iggy-connector`:

- **Embedded Mode**: `rustok-iggy-connector::EmbeddedConnector` runs Iggy server in-process
- **Remote Mode**: `rustok-iggy-connector::RemoteConnector` connects to external Iggy cluster

```rust
// rustok-iggy just passes the mode config to the connector
let config = IggyConfig {
    mode: IggyMode::Remote,  // connector handles the actual connection
    ..Default::default()
};
```

## Components

| Component | Description |
|-----------|-------------|
| `IggyTransport` | Main transport implementing `EventTransport` |
| `TopologyManager` | Manages stream/topic creation tracking |
| `ConsumerGroupManager` | Consumer group coordination |
| `DlqManager` | Dead letter queue handling |
| `ReplayManager` | Event replay orchestration |
| `EventSerializer` | JSON/Bincode serialization |

## Serialization

### JSON (Default)

Human-readable, debugging-friendly:

```rust
let config = IggyConfig {
    serialization: SerializationFormat::Json,
    ..Default::default()
};
```

### Bincode

High-throughput binary format:

```rust
let config = IggyConfig {
    serialization: SerializationFormat::Bincode,
    ..Default::default()
};
```

## Health Check

```rust
use rustok_iggy::health::{health_check, HealthStatus};

let result = health_check(connector.as_ref()).await?;
match result.status {
    HealthStatus::Healthy => println!("All good"),
    HealthStatus::Degraded => println!("Partial issues"),
    HealthStatus::Unhealthy => println!("Critical failure"),
}
```

## Dependencies

- `rustok-core`: Core traits and types (`EventTransport`, `EventEnvelope`)
- `rustok-iggy-connector`: Connection abstraction (Embedded/Remote mode switching, IggyConnector trait)

## Feature Flags

- `iggy`: Enable full Iggy SDK support in connector (optional, for production use)

## Status

> **Experimental**: This module is under active development. API may change.

## Documentation

- [Implementation Plan](./docs/implementation-plan.md)
- [rustok-iggy-connector](../rustok-iggy-connector/README.md) — connector layer documentation
- [Architecture Overview](../../docs/architecture/events.md)
