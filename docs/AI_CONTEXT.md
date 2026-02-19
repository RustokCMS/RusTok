# AI Context для RusToK

Обязательный стартовый контекст для AI-сессий.

## Порядок чтения

1. `docs/index.md`.
2. `docs/AI_CONTEXT.md`.
3. `CRATE_API.md` целевого крейта (если существует).
4. `README.md` целевого крейта.
5. При event-изменениях: `docs/architecture/events.md` и `docs/architecture/events-transactional.md`.

## Общие инварианты

- Доменные модули реализуют `RusToKModule` и подключаются через `ModuleRegistry`.
- `rustok-outbox` не модуль реестра, а инфраструктура для гарантированной доставки событий.
- Для write-flow + events используется transactional outbox.
- Tenant isolation и RBAC обязательны в сервисном слое.
- События и обработчики должны оставаться совместимыми по `DomainEvent`/`EventEnvelope`.

## Постоянные guardrails для AI-генерации

Эти правила считаются долгоживущими инструкциями к действию (а не milestone-заметками):

- Для write + event использовать консистентный путь через transactional публикацию (`publish_in_tx(...)`).
- Любой data-access в domain-коде должен быть tenant-scoped (tenant_id в контексте/фильтрах).
- Модульные границы удерживать через `Cargo.toml`: нет зависимости — нет импорта.
- В production-коде не использовать `unwrap/expect/panic`; применять типизированные ошибки.
- Перед генерацией нового слоя сверяться с локальными reference-пакетами и существующими примерами проекта.

### Приоритет внедрения

- **Сейчас:** lint/CI-проверки (`unwrap/expect/panic`), tenant/event integration checks, поддержка локальных эталонов.
- **Поэтапно:** усиление compile-time паттернов (например, service outcome с событиями, scoped repositories) без массового breaking-рефакторинга.

### Что берём из практических рекомендаций прямо сейчас

1. **Да:** усилить Clippy/CI против `unwrap/expect/panic` и потерянных `Result` (`unused_must_use`).
2. **Да:** для write + event в сервисах использовать только transactional путь (`publish_in_tx(...)` / outbox в той же транзакции).
3. **Да:** добавить/поддерживать тест-контракт для write-сценариев: проверка и записи в БД, и события в outbox.
4. **Да:** сохранять модульные границы через `Cargo.toml` (без прямых cross-domain зависимостей).

### Что не берём «как есть» без адаптации

- **Не берём дословно:** общий `TenantFilter` с жёстким `Column::TenantId` для всех сущностей — слишком хрупко для разнородных моделей.
- **Берём поэтапно:** tenant-scoped abstractions и service outcome-контракты внедряем пилотно по модулям, а не одномоментным breaking-change.

---

## Важные крейты

### 1) `crates/rustok-core`

**Назначение:** общий платформенный фундамент (module contracts, errors, events, permissions, health, metrics).

**Ключевые публичные типы/трейты (10):**
- `RusToKModule`
- `ModuleRegistry`
- `ModuleContext`
- `MigrationSource`
- `DomainEvent`
- `EventEnvelope`
- `EventHandler`
- `EventBus`
- `EventTransport`
- `AppContext`

**Критичные ограничения:**
- Не ломать core-контракты без архитектурного решения.
- Не переносить доменную логику конкретных модулей в core.

### 2) `crates/rustok-events`

**Назначение:** стабильный слой event-контрактов (реэкспорт из core).

**Ключевые публичные типы/трейты (5):**
- `rustok_events::DomainEvent`
- `rustok_events::EventEnvelope`
- `RootDomainEvent`
- `RootEventEnvelope`
- Совместимость с `rustok_core::events::*`

**Критичные ограничения:**
- Не вводить несовместимые параллельные envelope-контракты.
- Сохранять backward compatibility потребителей.

### 3) `crates/rustok-outbox`

**Назначение:** transactional outbox (`sys_events` + relay).

**Ключевые публичные типы/трейты (9):**
- `TransactionalEventBus`
- `OutboxTransport`
- `OutboxRelay`
- `RelayConfig`
- `RelayMetricsSnapshot`
- `SysEvent`
- `SysEvents`
- `SysEventsMigration`
- `SysEventStatus`

**Критичные ограничения:**
- Для write-flow использовать `publish_in_tx`.
- Не отключать relay в production.

### 4) `crates/rustok-iggy`

**Назначение:** event transport поверх Iggy (serialization/topology/groups/DLQ/replay).

**Ключевые публичные типы/трейты (10):**
- `IggyTransport`
- `IggyConfig`
- `IggyMode`
- `TopologyConfig`
- `SerializationFormat`
- `EventSerializer`
- `JsonSerializer`
- `BincodeSerializer`
- `ConsumerGroupManager`
- `ReplayManager`

**Критичные ограничения:**
- Не выдумывать API Iggy.
- Сохранять совместимость сериализации producer/consumer.

### 5) `crates/rustok-iggy-connector`

**Назначение:** low-level коннектор к Iggy (embedded/remote + pub/sub).

**Ключевые публичные типы/трейты (10):**
- `IggyConnector`
- `MessageSubscriber`
- `ConnectorConfig`
- `ConnectorMode`
- `EmbeddedConnectorConfig`
- `RemoteConnectorConfig`
- `PublishRequest`
- `ConnectorError`
- `RemoteConnector`
- `EmbeddedConnector`

**Критичные ограничения:**
- Не придумывать методы/поля коннектора.
- Поддерживать оба режима (`Embedded`/`Remote`).

### 6) `crates/rustok-content`

**Назначение:** базовый контентный домен (nodes/translations/body/state-machine).

**Ключевые публичные типы/трейты (10):**
- `NodeService`
- `ContentNode<S>`
- `Draft`
- `Published`
- `Archived`
- `ToContentStatus`
- `Node`
- `NodeTranslation`
- `Body`
- `ContentError`

**Критичные ограничения:**
- Не ломать переходы state machine.
- События публиковать транзакционно.

### 7) `crates/rustok-commerce`

**Назначение:** commerce-домен (catalog/inventory/pricing/order lifecycle).

**Ключевые публичные типы/трейты (10):**
- `CatalogService`
- `InventoryService`
- `PricingService`
- `Order<S>`
- `Pending`
- `Confirmed`
- `Paid`
- `Shipped`
- `Delivered`
- `Cancelled`

**Критичные ограничения:**
- Не ломать порядок статусов заказа.
- Держать event-flow внутри транзакций.

### 8) `crates/rustok-blog`

**Назначение:** blog-домен поверх content (posts/comments/publishing states).

**Ключевые публичные типы/трейты (10):**
- `PostService`
- `BlogPost<S>`
- `BlogPostStatus`
- `ToBlogPostStatus`
- `CommentStatus`
- `CreatePostInput`
- `UpdatePostInput`
- `PostResponse`
- `BlogError`
- `BlogResult<T>`

**Критичные ограничения:**
- Это wrapper над content-таблицами.
- Публикация постов должна соответствовать state machine.

### 9) `crates/rustok-forum`

**Назначение:** forum-домен (categories/topics/replies/moderation).

**Ключевые публичные типы/трейты (10):**
- `CategoryService`
- `TopicService`
- `ReplyService`
- `ModerationService`
- `CreateCategoryInput`
- `CreateTopicInput`
- `CreateReplyInput`
- `ForumError`
- `ForumResult<T>`
- `ForumModule`

**Критичные ограничения:**
- Соблюдать tenant boundaries.
- Не выносить модерационную логику в инфраструктурный слой.

### 10) `crates/rustok-pages`

**Назначение:** pages-домен (pages/blocks/menus) как wrapper над content.

**Ключевые публичные типы/трейты (10):**
- `PageService`
- `BlockService`
- `MenuService`
- `Page`
- `Block`
- `Menu`
- `CreatePageInput`
- `UpdatePageInput`
- `PagesError`
- `PagesResult<T>`

**Критичные ограничения:**
- Не дублировать schema-подход content без ADR.
- Write + event должны быть консистентны в транзакции.

### 11) `crates/rustok-index`

**Назначение:** CQRS read-модель и поисковая индексация.

**Ключевые публичные типы/трейты (10):**
- `Indexer`
- `LocaleIndexer`
- `IndexerContext`
- `SearchQuery`
- `SearchResult`
- `SearchEngine`
- `IndexDocument`
- `IndexError`
- `IndexResult<T>`
- `IndexModule`

**Критичные ограничения:**
- Индекс не является source of truth.
- Проверять связку event → index handler.

### 12) `crates/rustok-tenant`

**Назначение:** core-модуль multi-tenancy каркаса.

**Ключевые публичные типы/трейты (5):**
- `TenantModule`
- `TenantError`
- `RusToKModule` (реализуемый контракт)
- `MigrationSource` (реализуемый контракт)
- `ModuleKind` (core-классификация)

**Критичные ограничения:**
- Tenant isolation — обязательный инвариант.

### 13) `crates/rustok-rbac`

**Назначение:** core-модуль RBAC интеграции.

**Ключевые публичные типы/трейты (5):**
- `RbacModule`
- `RbacError`
- `RusToKModule` (реализуемый контракт)
- `MigrationSource` (реализуемый контракт)
- `ModuleKind` (core-классификация)

**Критичные ограничения:**
- Не обходить централизованные permission checks.

---

## Do / Don’t

### Do

- Используй только реально существующие API из кода и docs.
- Для доменных write-flow с событиями применяй `publish_in_tx`.
- Проверяй соответствие `EventEnvelope` и обработчиков (тип/версия/payload).
- Сохраняй tenant + RBAC проверки в сервисном слое.

### Don’t

- Не выдумывай API Loco/Iggy.
- Не используй `publish` там, где нужна transactional публикация (`publish_in_tx`).
- Не добавляй handler’ы, несовместимые с текущим `EventEnvelope`.
- Не обходи outbox relay в production event-flow.

## Чек-лист перед коммитом

- Проверены существование и сигнатуры всех использованных публичных типов/методов.
- Проверен корректный путь публикации событий (`publish_in_tx` где требуется).
- Проверена совместимость `EventEnvelope` и event handlers.
- Обновлена релевантная документация (`docs/` + локальные docs крейта).
