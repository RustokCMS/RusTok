# rustok-forum / CRATE_API

## Публичные модули
`constants`, `controllers`, `dto`, `entities`, `error`, `graphql`, `locale`, `services`.

## Основные публичные типы и сигнатуры
- `pub struct ForumModule`
- `pub struct CategoryService`, `TopicService`, `ReplyService`, `ModerationService`
- `pub mod graphql` -> `ForumQuery`, `ForumMutation`
- `pub mod controllers` -> `routes()`
- Публичные DTO/константы из `dto::*` и `constants::*`
- `pub enum ForumError`, `pub type ForumResult<T>`
- `pub mod locale` — хелперы `resolve_translation`, `resolve_body`, `available_locales`

## DTO изменения (актуально)
### TopicResponse / TopicListItem
- Добавлены: `effective_locale: String`, `available_locales: Vec<String>`, `slug: String`, `author_id: Option<Uuid>`
### ReplyResponse / ReplyListItem
- Добавлены: `effective_locale: String`, `author_id: Option<Uuid>`, `parent_reply_id: Option<Uuid>` (в ListItem)
### CategoryResponse / CategoryListItem
- Добавлены: `effective_locale: String`, `available_locales: Vec<String>`
### CreateTopicInput
- Добавлено: `slug: Option<String>`
### ListRepliesFilter (новый)
- Пагинация ответов: `page`, `per_page`, `locale`
### ModerationService
- Сигнатуры `approve_reply`, `reject_reply`, `hide_reply`, `pin_topic`, `unpin_topic` теперь принимают `tenant_id: Uuid`
- `close_topic`, `archive_topic` теперь принимают `tenant_id: Uuid`

## Locale fallback chain
Порядок поиска перевода: `requested → "en" → первый доступный`.
Поле `effective_locale` сообщает, какой locale реально вернули.

## События
Публикует форумные доменные события через outbox pipeline:
- `ForumTopicCreated` — при создании темы
- `ForumTopicReplied` — при добавлении ответа
- `ForumTopicStatusChanged` — при изменении статуса темы (close/archive)
- `ForumTopicPinned` — при закреплении/откреплении темы
- `ForumReplyStatusChanged` — при модерации ответа (approve/reject/hide)

Все новые форумные события определены в `rustok-core::events::DomainEvent`.

## Зависимости от других rustok-крейтов
- `rustok-content`
- `rustok-core`
- `rustok-outbox`

## Частые ошибки ИИ
- Неправильно использует лимиты/константы модерации из `constants`.
- Путает иерархию category/topic/reply в импортах сущностей.
- Игнорирует tenant-boundary в сервисных фильтрах.
- Путает `locale` (запрошенный) и `effective_locale` (фактически использованный).
- Передаёт `ModerationService` методы без `tenant_id` — теперь он обязателен.

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
