# rustok-pages / CRATE_API

## Публичные модули
`dto`, `entities`, `error`, `services`.

## Основные публичные типы и сигнатуры
- `pub struct PagesModule`
- `pub struct PageService`, `MenuService`, `BlockService`
- `pub struct Page`, `Menu`, `Block`
- `pub enum PagesError`, `pub type PagesResult<T>`

## События
- Публикует domain events страниц/меню/блоков через `TransactionalEventBus`.
- Потребляет: внешние события напрямую не подписывает.

## Зависимости от других rustok-крейтов
- `rustok-core`
- `rustok-content`
- `rustok-outbox`

## Частые ошибки ИИ
- Путает `Page` (страница) и `Block` (контентный блок) в сигнатурах сервисов.
- Забывает синхронизировать публикацию/снятие с публикации в `PageService`.
- Использует DTO вместо ORM-entity в запросах SeaORM.

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
