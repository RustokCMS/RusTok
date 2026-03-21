# rustok-commerce / CRATE_API

## Публичные модули
`controllers`, `dto`, `entities`, `error`, `graphql`, `services`, `state_machine`.

## Основные публичные типы и сигнатуры
- `pub struct CommerceModule`
- `pub struct CatalogService`, `pub struct InventoryService`, `pub struct PricingService`
- `pub struct CommerceQuery`, `pub struct CommerceMutation`
- `pub fn controllers::routes() -> Routes`
- `pub struct Order<S>` + состояния `Pending`, `Confirmed`, `Paid`, `Shipped`, `Delivered`, `Cancelled`
- `pub enum CommerceError`, `pub type CommerceResult<T>`

## События
- Публикует: `DomainEvent::ProductCreated|Updated|Published|Deleted`, `DomainEvent::PriceUpdated`, события остатков/склада из `services/*`.
- Потребляет: внешние события не подписывает напрямую (сервисный вызов).

## Зависимости от других rustok-крейтов
- `rustok-core`
- `rustok-api`
- `rustok-events`
- `rustok-outbox`
- (dev) `rustok-test-utils`

## Частые ошибки ИИ
- Путает доменные ошибки валидации заказа и инфраструктурные `rustok_core::Error`.
- Меняет статус заказа мимо state-machine.
- Забивает на `ValidateEvent` перед публикацией событий.
- Добавляет transport-адаптеры commerce обратно в `apps/server` вместо расширения
  `crates/rustok-commerce/src/graphql/*` и `crates/rustok-commerce/src/controllers/*`.

## Минимальный набор контрактов

### Входные DTO/команды
- Входной контракт формируется публичными DTO/командами из crate (см. разделы с `Create*Input`/`Update*Input`/query/filter выше и соответствующие `pub`-экспорты в `src/lib.rs`).
- Все изменения публичных полей DTO считаются breaking-change и требуют синхронного обновления transport-адаптеров `apps/server`.
- GraphQL/HTTP transport entry points (`graphql::CommerceQuery`, `graphql::CommerceMutation`,
  `controllers::routes`, публичные controller DTO для OpenAPI) тоже входят в публичный
  контракт crate и должны развиваться внутри `rustok-commerce`.

### Доменные инварианты
- Инварианты модуля фиксируются в сервисах/стейт-машинах и валидации DTO; недопустимые переходы/параметры должны завершаться доменной ошибкой.
- Инварианты multi-tenant boundary (tenant/resource isolation, auth context) считаются обязательной частью контракта.
- Transport-адаптеры обязаны выполнять permission-check через `AuthContext.permissions`
  и tenant-scoped запросы/сервисы, не обходя границы арендатора.

### События / outbox-побочные эффекты
- Если модуль публикует доменные события, публикация должна идти через транзакционный outbox/transport-контракт без локальных обходов.
- Формат event payload и event-type должен оставаться обратно-совместимым для межмодульных потребителей.

### Ошибки / коды отказов
- Публичные `*Error`/`*Result` типы модуля определяют контракт отказов и не должны терять семантику при маппинге в HTTP/GraphQL/CLI.
- Для validation/auth/conflict/not-found сценариев должен сохраняться устойчивый error-class, используемый тестами и адаптерами.
