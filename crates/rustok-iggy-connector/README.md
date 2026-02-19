# rustok-iggy-connector

## Назначение

`rustok-iggy-connector` — это уровень абстракции для подключения к Iggy (high-performance message streaming platform).

Поддерживает два режима:
- **Embedded**: запуск Iggy сервера внутри приложения
- **Remote**: подключение к внешнему Iggy серверу через TCP/HTTP

## Режимы работы

### Embedded Mode

Встроенный Iggy сервер работает внутри приложения:
- Данные хранятся в локальной директории
- TCP порт для коммуникации
- HTTP порт для дашборда (опционально)

### Remote Mode

Подключение к внешнему Iggy серверу:
- TCP или HTTP протокол
- Аутентификация по username/password
- Поддержка TLS

## Использование

```rust
use rustok_iggy_connector::{
    ConnectorConfig, ConnectorMode, EmbeddedConnector, IggyConnector, PublishRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Embedded mode
    let connector = EmbeddedConnector::new();
    let config = ConnectorConfig::default();
    connector.connect(&config).await?;

    // Publish message
    let request = PublishRequest::simple("tenant-123", b"Hello, Iggy!".to_vec(), "event-1");
    connector.publish(request).await?;

    connector.shutdown().await?;
    Ok(())
}
```

## Конфигурация

### Embedded

```rust
EmbeddedConnectorConfig {
    data_dir: "./data/iggy".to_string(),
    tcp_port: 8090,
    http_port: 3000,
    persistent: true,
}
```

### Remote

```rust
RemoteConnectorConfig {
    addresses: vec!["127.0.0.1:8090".to_string()],
    protocol: "tcp".to_string(),
    username: "rustok".to_string(),
    password: "rustok".to_string(),
    tls_enabled: false,
}
```

## Features

- `iggy` — включает полную поддержку Iggy SDK (опционально)

## Взаимодействие

- `crates/rustok-iggy` — использует этот коннектор для EventTransport
- `apps/server` — инициализирует Iggy транспорт
- Внешний Iggy сервер (для remote mode)

## Паспорт компонента

- **Роль в системе:** Коннектор к Iggy runtime (embedded/remote) и lifecycle-обвязка.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-iggy
  - apps/server
  - внешний Iggy cluster/runtime
- **Точки входа:**
  - `crates/rustok-iggy-connector/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`
