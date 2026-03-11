# Queue runtime source of truth: `rustok-outbox` + `event_transport_factory`

- Date: 2026-03-11
- Status: Accepted

## Context

Loco queue/jobs подсистема полезна как generic background execution runtime, но она не покрывает в полном объёме требования текущего event-path RusToK для production:

1. **Transactional outbox как write-side инвариант**: событие должно фиксироваться в той же DB-транзакции, что и доменная операция (через `sys_events`), с гарантией атомарности commit/rollback.
2. **Replay-процедуры как first-class capability**: нужен штатный операционный путь для повторной постановки событий из DLQ/backlog без обходных migration-скриптов.
3. **DLQ и наблюдаемость на уровне event lifecycle**: нужны явные статусы, retry-attempts и диагностика в тех же сущностях, где живёт outbox.
4. **Transport switching без изменения write-side**: runtime должен поддерживать независимый выбор relay target (`memory|iggy|...`) при фиксированном `transport = outbox`.

Для этих требований в кодовой базе уже есть специализированный слой `rustok-outbox` и фабрика транспорта в server runtime. Параллельный production-path через Loco jobs создавал бы расхождение контрактов (две модели retry, два path для replay/DLQ, разные точки observability) и повышал бы риск инцидентов при эксплуатации.

## Decision

1. **Source of truth для очередей и event delivery-path в production** фиксируется за:
   - `crates/rustok-outbox` (outbox persistence, relay semantics, DLQ/retry lifecycle);
   - `apps/server/src/services/event_transport_factory.rs` (единая runtime-точка выбора транспорта/relay target).
2. Loco queue/jobs допускается только как вспомогательный non-production механизм (utility/background tasks), который не дублирует event delivery-path.
3. Любые изменения queue/event runtime должны быть совместимы с outbox-first контрактом и проходить через указанные source-of-truth компоненты.

## Consequences

**Плюсы**
- Единый production-path для outbox/retry/DLQ/replay без архитектурного fork.
- Предсказуемая observability-модель и единые runbook-процедуры.
- Безопасное transport switching на relay-уровне без переписывания write-side.

**Ограничения и запреты**
- Запрещено вводить **параллельный production-path** через Loco jobs/queue для доставки доменных событий без отдельного ADR, который явно описывает migration/cutover план и rollback.

**Follow-up**
- Поддерживать ссылки на этот ADR в governance-документах server runtime и в центральной event-архитектуре как policy anchor.
