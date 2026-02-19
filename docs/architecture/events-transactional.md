# Transactional Event Publishing

Полная документация находится в [`docs/architecture/events.md`](./events.md).

## Краткое резюме

`TransactionalEventBus` обеспечивает атомарную публикацию событий: событие сохраняется в БД тогда и только тогда, когда успешно коммитится окружающая транзакция.

**Crate:** `crates/rustok-outbox`  
**Паттерн:** [Transactional Outbox](https://microservices.io/patterns/data/transactional-outbox.html)

```
Service → TransactionalEventBus → OutboxTransport → sys_events таблица
                                                           ↓
                                              Relay Worker → EventBus → Handlers
```

## Использование

```rust
use rustok_outbox::TransactionalEventBus;

let txn = db.begin().await?;
// ... domain operations ...
event_bus.publish_in_tx(&txn, tenant_id, Some(user_id), DomainEvent::NodeCreated { .. }).await?;
txn.commit().await?; // событие сохраняется только здесь
```

## Важно

`rustok-outbox` не является `RusToKModule` и не регистрируется через `ModuleRegistry`.
Это core-инфраструктура, инициализируемая через `build_event_runtime()` в `apps/server/src/app.rs`.
Остановка outbox = потеря гарантий доставки событий для всей платформы.

## Полная документация

→ [`docs/architecture/events.md`](./events.md)
