# Контракт потока доменных событий

Документ фиксирует канонический путь `DomainEvent` в RusToK: от доменной операции до
обновления read-model, индекса и route/canonical state.

Канонические определения `DomainEvent` и `EventEnvelope` поддерживаются в
`rustok-events`; `rustok-core::events` сохраняет совместимый re-export для
переходного периода.

## Канонический путь события

1. Доменный сервис выполняет бизнес-операцию.
2. Изменения данных и запись в outbox (`sys_events`) происходят в одной транзакции.
3. `OutboxRelay` доставляет событие в transport и помечает dispatch state.
4. `EventDispatcher` передаёт событие зарегистрированным consumer-ам.
5. Index/read-model/routing consumers пересчитывают свои проекции идемпотентно.

## Текущие владельцы событий

- `rustok-blog`, `rustok-forum`, `rustok-pages`, `rustok-comments`, `rustok-commerce`
  публикуют доменные события из своих storage-owner моделей.
- `rustok-content` публикует orchestration и canonical-routing события, а также
  поддерживает shared-node helper surface для оставшихся совместимых сценариев.
- `rustok-index` остаётся основным consumer-ом для content/product reindex flow.
- `rustok-outbox` и `rustok-core` обеспечивают transactional delivery/runtime contract.

## Важное уточнение по `Node*` событиям

События `NodeCreated`, `NodeUpdated`, `NodeTranslationUpdated`, `NodePublished`,
`NodeUnpublished`, `NodeDeleted` и `BodyUpdated` не являются больше основной
публичной моделью хранения для `blog`, `forum`, `pages` и `comments`.

Текущая роль этого набора:

- shared-node helper surface;
- helper-path для оставшихся node-backed сценариев;
- source для части reindex/replay tooling.

Новые и развиваемые доменные сценарии должны опираться на typed storage-owner события
своего модуля либо на orchestration/canonical события `rustok-content`, а не расширять
central `NodeService` contract.

---

## Контентные события

### Shared-node helper surface

| DomainEvent | Кто публикует | Кто обрабатывает | Обязательные поля | Статус |
|---|---|---|---|---|
| `NodeCreated` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `kind` | Shared-node helper |
| `NodeUpdated` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `kind` | Shared-node helper |
| `NodeTranslationUpdated` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `locale` | Shared-node helper |
| `NodePublished` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `kind` | Shared-node helper |
| `NodeUnpublished` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `kind` | Shared-node helper |
| `NodeDeleted` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `kind` | Shared-node helper |
| `BodyUpdated` | `rustok-content::NodeService` | `rustok-index::ContentIndexer` | `node_id`, `locale` | Shared-node helper |

Требования:

- consumer должен оставаться идемпотентным;
- replay допустим для recovery и reindex;
- новые продуктовые фичи не должны добавлять новые `kind`/ветки в этот слой без отдельного ADR.

### Orchestration и canonical-routing surface

| DomainEvent | Кто публикует | Кто обрабатывает | Обязательные поля | Назначение |
|---|---|---|---|---|
| `CanonicalUrlChanged` | `rustok-content::ContentOrchestrationService` | `rustok-index::ContentIndexer`, routing/cache consumers | `target_id`, `target_kind`, `locale`, `new_canonical_url` | Переназначение canonical URL |
| `UrlAliasPurged` | `rustok-content::ContentOrchestrationService` | `rustok-index::ContentIndexer`, routing/cache consumers | `target_id`, `target_kind`, `locale`, `urls[]` | Удаление устаревших alias URL |

Эти события являются текущим каноническим контрактом для `topic ↔ post` conversion,
slug migration и redirect/canonical policy.

## Коммерческие события

| DomainEvent | Кто публикует | Кто обрабатывает | Обязательные поля |
|---|---|---|---|
| `ProductCreated` | `rustok-commerce::CatalogService` | `rustok-index::ProductIndexer` | `product_id` |
| `ProductUpdated` | `rustok-commerce::CatalogService` | `rustok-index::ProductIndexer` | `product_id` |
| `ProductPublished` | `rustok-commerce::CatalogService` | `rustok-index::ProductIndexer` | `product_id` |
| `ProductDeleted` | `rustok-commerce::CatalogService` | `rustok-index::ProductIndexer` | `product_id` |
| `VariantCreated` | `server` commerce services | `rustok-index::ProductIndexer` | `variant_id`, `product_id` |
| `VariantUpdated` | `server` commerce services | `rustok-index::ProductIndexer` | `variant_id`, `product_id` |
| `VariantDeleted` | `server` commerce services | `rustok-index::ProductIndexer` | `variant_id`, `product_id` |
| `InventoryUpdated` | `rustok-commerce::InventoryService` | `rustok-index::ProductIndexer` | `variant_id`, `product_id`, `location_id`, `old_quantity`, `new_quantity` |
| `PriceUpdated` | `rustok-commerce::PricingService` | `rustok-index::ProductIndexer` | `variant_id`, `product_id`, `currency`, `new_amount` |

## Системные события индексации

| DomainEvent | Кто публикует | Кто обрабатывает | Обязательные поля |
|---|---|---|---|
| `ReindexRequested` (`target_type = "content"`) | admin/services | `rustok-index::ContentIndexer` | `target_type` |
| `ReindexRequested` (`target_type = "product"`) | admin/services | `rustok-index::ProductIndexer` | `target_type` |
| `IndexUpdated` | indexers/system services | observers/telemetry/audit | `index_name`, `target_id` |

## Retry и отказоустойчивость

- Outbox relay обязан поддерживать backoff и переводить событие в `Failed` только
  после исчерпания лимита попыток.
- Dispatcher retry должен быть конечным и наблюдаемым.
- Consumer-ы обязаны использовать идемпотентные операции:
  `upsert`, `delete-if-exists`, пересчёт по source-of-truth.
- URL-policy сценарии обязаны сопровождаться парой событий
  `CanonicalUrlChanged` + `UrlAliasPurged`.

## Какие модули обязаны ссылаться на этот контракт

- publishers: `rustok-content`, `rustok-commerce`, `rustok-pages`, `rustok-forum`,
  `rustok-blog`, `rustok-comments`, `apps/server`;
- consumers/read-model: `rustok-index`;
- transport/runtime: `rustok-outbox`, `rustok-core`.

Если новый модуль публикует или потребляет `DomainEvent`, в его `docs/README.md`
обязательна секция `Event contracts` со ссылкой на этот документ.

## PR-чеклист для изменений в событиях

- [ ] Добавлен consumer-path для нового события.
- [ ] Добавлен integration test цепочки `publish -> outbox -> delivery -> consumer`.
- [ ] Для нового события есть минимум happy-path и repeat/idempotency test.
- [ ] Обновлены telemetry/runbook артефакты, если меняется runtime consumer loop.
- [ ] Обновлены central docs и docs модуля-публикатора/consumer-а.
