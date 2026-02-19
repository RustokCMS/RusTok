# rustok-iggy-connector docs

В этой папке хранится документация модуля `crates/rustok-iggy-connector`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Overview

`rustok-iggy-connector` обеспечивает абстракцию для подключения к Iggy в двух режимах:
- Embedded (встроенный сервер)
- Remote (внешний сервер)

## Quick Start

```rust
use rustok_iggy_connector::{ConnectorConfig, IggyConnector, RemoteConnector};

let connector = RemoteConnector::new();
connector.connect(&ConnectorConfig::default()).await?;
connector.publish(PublishRequest::simple("tenant-1", b"data".to_vec(), "event-1")).await?;
connector.shutdown().await?;
```

## Configuration

Основные параметры конфигурации:
- `mode`: Embedded или Remote
- `stream_name`: Имя стрима
- `topic_name`: Имя топика
- `partitions`: Количество партиций
