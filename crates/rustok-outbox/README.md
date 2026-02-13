# rustok-outbox

`rustok-outbox` — модуль outbox-доставки событий для RusTok.

## Что делает модуль
- сохраняет события в `sys_events` через `OutboxTransport`;
- ретранслирует pending-события через `OutboxRelay`;
- поддерживает claim/dispatch/retry/DLQ-поток обработки;
- предоставляет миграцию схемы `sys_events` и базовые метрики relay.

## Основные компоненты
- `src/transport.rs` — запись событий в outbox и acknowledge.
- `src/relay.rs` — цикл обработки pending-событий, retry/backoff, DLQ.
- `src/entity.rs` — ORM-модель `sys_events`.
- `src/migration.rs` — миграция таблицы и индексов.

## Документация
Дополнительная документация модуля хранится в `docs/`.

## Взаимодействие
- crates/rustok-core (EventTransport/EventEnvelope)
- apps/server (миграции/рантайм relay)
- target transport (например rustok-iggy)

## Паспорт компонента
- **Роль в системе:** Outbox-транспорт и relay: надёжная доставка событий с retry/backoff/DLQ.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - crates/rustok-core (EventTransport/EventEnvelope)
  - apps/server (миграции/relay runtime)
  - target transport (например crates/rustok-iggy)
- **Точки входа:**
  - `crates/rustok-outbox/src/lib.rs`
  - `crates/rustok-outbox/src/relay.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

