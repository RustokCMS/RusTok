# rustok-telemetry / CRATE_API

## Публичные модули
`metrics`, `otel`.

## Основные публичные типы и сигнатуры
- `pub struct TelemetryConfig`, `pub struct TelemetryHandles`
- `pub enum LogFormat`, `pub enum TelemetryError`
- `pub fn init(config: TelemetryConfig) -> Result<TelemetryHandles, TelemetryError>`
- `pub fn render_metrics() -> Result<String, prometheus::Error>`
- `pub fn current_trace_id() -> Option<String>`

## События
- Публикует: метрики/трейсы observability.
- Потребляет: сигналы и spans из `tracing`/OTel.

## Зависимости от других rustok-крейтов
- нет прямых зависимостей на другие `rustok-*`.

## Частые ошибки ИИ
- Повторно вызывает `init` и получает `SubscriberAlreadySet`.
- Путает application metrics registry и глобальный prometheus registry.

## Минимальный набор контрактов

### Входные DTO/команды
- Входной контракт формируется публичными DTO/командами из crate (см. разделы с `Create*Input`/`Update*Input`/query/filter выше и соответствующие `pub`-экспорты в `src/lib.rs`).
- Все изменения публичных полей DTO считаются breaking-change и требуют синхронного обновления transport-адаптеров `apps/server`.

### Доменные инварианты
- Инварианты модуля фиксируются в сервисах/стейт-машинах и валидации DTO; недопустимые переходы/параметры должны завершаться доменной ошибкой.
- Инварианты multi-tenant boundary (tenant/resource isolation, auth context) считаются обязательной частью контракта.

### События / outbox-побочные эффекты
- Если модуль публикует доменные события, публикация должна идти через транзакционный outbox/transport-контракт без локальных обходов.
- Формат event payload и event-type должен оставаться обратно-совместимым для межмодульных потребителей.

### Ошибки / коды отказов
- Публичные `*Error`/`*Result` типы модуля определяют контракт отказов и не должны терять семантику при маппинге в HTTP/GraphQL/CLI.
- Для validation/auth/conflict/not-found сценариев должен сохраняться устойчивый error-class, используемый тестами и адаптерами.
