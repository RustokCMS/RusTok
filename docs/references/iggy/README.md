# Iggy Reference-пакет (RusToK)

Дата последней актуализации: **2026-02-19**.

> Пакет фиксирует рабочий слой интеграции Iggy в RusToK (`rustok-iggy`, `rustok-iggy-connector`, `rustok-outbox`) и защищает от ложных переносов из Kafka/NATS.

## 1) Минимальный рабочий пример: поднять transport

```rust
use rustok_iggy::{IggyConfig, IggyTransport};

let config = IggyConfig::default();
let transport = IggyTransport::new(config).await?;

if transport.is_connected() {
    // transport готов для EventTransport::publish
}

transport.shutdown().await?;
```

## 2) Минимальный рабочий пример: write + event через транзакцию

```rust
let txn = db.begin().await?;

// ... write в доменные таблицы

transactional_bus
    .publish_in_tx(&txn, tenant_id, Some(actor_id), event)
    .await?;

txn.commit().await?;
```

Это каноничный путь RusToK для write-flow с событиями.

## 3) Актуальные сигнатуры API (в репозитории)

### `rustok-iggy`
- `pub async fn new(config: IggyConfig) -> Result<Self>`
- `pub async fn shutdown(&self) -> Result<()>`
- `pub async fn subscribe_as_group(&self, group: &str) -> Result<()>`
- `pub async fn replay(&self) -> Result<()>`
- `pub fn config(&self) -> &IggyConfig`
- `pub fn is_connected(&self) -> bool`

### `rustok-iggy-connector`
- `pub async fn connect(&self, config: &ConnectorConfig) -> Result<(), ConnectorError>`
- `pub async fn publish(&self, request: PublishRequest) -> Result<(), ConnectorError>`
- `pub async fn subscribe(&self, stream: &str, topic: &str, partition: u32) -> Result<Box<dyn MessageSubscriber>, ConnectorError>`
- `pub async fn shutdown(&self) -> Result<(), ConnectorError>`
- `pub async fn recv(&mut self) -> Result<Option<Vec<u8>>, ConnectorError>`

### `rustok-outbox`
- `pub async fn publish_in_tx<C>(&self, txn: &C, tenant_id: Uuid, actor_id: Option<Uuid>, event: DomainEvent) -> Result<()> where C: ConnectionTrait`

## 4) Чего делать нельзя (типичные ложные паттерны из Kafka/NATS)

1. **Нельзя предполагать kafka-only semantics (acks/offset commit API), которых нет в текущем abstraction.**
   - Антипаттерн: добавлять в бизнес-код ручные offset-коммиты или direct SDK вызовы Kafka.
   - Правильно: использовать `EventTransport`/`TransactionalEventBus`.

2. **Нельзя использовать fire-and-forget publish для write-flow, требующего консистентности.**
   - Антипаттерн: `publish(...)` до/вместо транзакционного пути.
   - Правильно: `publish_in_tx(...)` при write + event.

3. **Нельзя переносить NATS subject-модель как есть на stream/topic/partition Iggy.**
   - Антипаттерн: проектировать routing только по строковому `subject` без учёта `stream/topic/partition_key`.

4. **Нельзя выдумывать поля конфигурации и режимы коннектора.**
   - В актуальном коде режимы только `Embedded | Remote`, а конфиг идёт через `IggyConfig -> ConnectorConfig`.

## 5) Синхронизация с кодом (регламент)

- При изменениях в `crates/rustok-iggy/**`, `crates/rustok-iggy-connector/**`, `crates/rustok-outbox/**`:
  1) обновить примеры и сигнатуры в этом reference;
  2) обновить дату в шапке;
  3) проверить, что антипаттерны всё ещё релевантны.
