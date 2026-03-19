# Flex — Architecture

> Документация модуля: [`docs/modules/flex.md`](../modules/flex.md)
> Implementation plan: [`crates/flex/docs/implementation-plan.md`](../../crates/flex/docs/implementation-plan.md)
> Module docs: [`crates/flex/docs/README.md`](../../crates/flex/docs/README.md)

---

## Статус на 2026-03-19

Flex больше не стоит описывать как эксперимент внутри `rustok-core`. Текущая архитектура уже разделена на два слоя:

- `rustok-core::field_schema` хранит базовые типы, валидаторы и migration helpers для attached-mode custom fields;
- `crates/flex` хранит transport-agnostic orchestration, registry и standalone contracts;
- `apps/server` держит только adapter/wiring слой: SeaORM, GraphQL, cache/RBAC gates, event emission и bootstrap.

Attached mode считается рабочим production contract. Standalone mode начат, но ещё не является live API surface платформы.

---

## Архитектурный смысл

Flex нужен для runtime-определяемых полей там, где:

- стандартный доменный модуль уже существует;
- расширение схемы нужно per-tenant;
- выделять новый отдельный доменный модуль из-за нескольких edge-cases нецелесообразно.

Flex не заменяет нормализованные модели в критичных доменах. Для `orders`, `payments`, `inventory` и других write-critical сущностей приоритет остаётся за явными колонками и доменными таблицами.

---

## Границы модулей

| Слой | Ответственность |
|---|---|
| `rustok-core::field_schema` | `FieldType`, `FieldDefinition`, `ValidationRule`, `CustomFieldsSchema`, `HasCustomFields`, validation helpers, migration helpers |
| `crates/flex` | `FieldDefRegistry`, `FieldDefinitionService`, orchestration helpers, error mapping, standalone contracts/events |
| `apps/server` | SeaORM adapters, GraphQL API, cache adapter, RBAC checks, event bus wiring, migration/bootstrap |
| Доменные модули | Собственные `metadata` колонки, field-definition таблицы, entity-specific services |

Ключевой инвариант:

- generic Flex contracts не знают о `user`, `product`, `topic` и других конкретных сущностях;
- маршрутизация по `entity_type` собирается в composition root через registry;
- данные attached-mode остаются в таблицах доменных модулей, а не в отдельном global Flex storage.

---

## Attached Mode

### Что уже реализовано

- базовые field-schema типы и валидатор живут в `rustok-core`;
- generic registry/orchestration вынесены в `crates/flex`;
- `apps/server` использует `FieldDefRegistry` и прямые импорты из `flex`;
- GraphQL CRUD для field definitions работает через server adapters;
- cache invalidation и event-driven hooks уже проходят через общий Flex contract;
- для attached-mode зарегистрированы event envelopes и transport-agnostic error mapping.

### Подключённые entity types

| Entity type | Статус |
|---|---|
| `user` | Подключён |
| `product` | Подключён |
| `node` | Подключён |
| `topic` | Подключён |
| `order` | Не закрыт: нужен `orders.metadata` как source column |

### Runtime flow

1. GraphQL/adapter слой получает `entity_type`.
2. `FieldDefRegistry` резолвит подходящий `FieldDefinitionService`.
3. Service читает/изменяет field-definition rows в таблице доменного модуля.
4. `CustomFieldsSchema` валидирует metadata payload.
5. Server adapter инвалидирует cache и публикует событие через event bus/outbox.

---

## Metadata Readiness

Attached mode требует `metadata: Json/JSONB` в сущности-потребителе.

Текущее состояние:

| Entity | Статус |
|---|---|
| `users` | Готово |
| `products` | Готово |
| `nodes` | Готово |
| `topics` | Готово через `nodes.metadata` |
| `orders` | Не готово |

Это главный незакрытый structural gap для полного attached-mode parity.

---

## Events, Cache, Observability

Flex использует общий платформенный событийный контракт:

- attached-mode изменения публикуют события field-definition lifecycle;
- server layer слушает их для cache invalidation и audit/secondary reactions;
- cache живёт как adapter concern, а не как часть `crates/flex`.

Для architecture-level contract важно следующее:

- событие является фактом изменения схемы, а не местом хранения бизнес-логики;
- `crates/flex` формирует envelope/contract;
- transport, persistence и replay-политика остаются в общем event runtime платформы.

---

## RBAC

У Flex два разных уровня доступа:

1. Управление определениями полей.
2. Запись значений custom fields в metadata конкретной сущности.

Текущий контракт:

- управление field definitions идёт через server-side admin/superadmin gate;
- изменение значений custom fields наследует права самой сущности;
- Flex не вводит отдельный live RBAC runtime поверх Casbin, а опирается на существующий platform RBAC contract.

---

## Standalone Mode

Standalone mode больше не только идея на будущее. Сейчас уже есть:

- `crates/flex/src/standalone.rs` с transport-agnostic DTO/commands;
- orchestration helpers для schemas/entries;
- event helpers для `flex.schema.*` и `flex.entry.*`;
- server-side миграции и SeaORM entities для `flex_schemas` и `flex_entries`;
- validation service и базовый SeaORM adapter `FlexStandaloneSeaOrmService`.

Но standalone mode пока не является завершённым продуктовым контрактом. Ещё не закрыты:

- public REST API;
- public GraphQL API;
- RBAC surface для `flex.schemas.*` / `flex.entries.*`;
- indexer/read model;
- cascade rules для attached relations;
- полный integration test pass;
- финальная пользовательская документация.

---

## Что считать текущим архитектурным решением

На 2026-03-19 актуальны такие решения:

- attached-mode custom fields остаются модульно-привязанными и не переезжают в единый global storage;
- generic Flex orchestration живёт в `crates/flex`, а не в `apps/server`;
- `rustok-core` продолжает быть домом для field-schema primitives, потому что ими пользуются и attached-mode, и standalone contracts;
- standalone mode развивается поверх `crates/flex`, но не меняет attached-mode contract;
- `apps/server` остаётся composition/adapters слоем для GraphQL, RBAC, cache, event bus и persistence wiring.

---

## Остаточные долги

На сегодня для Flex всё ещё открыты только реальные хвосты:

1. Добавить `orders.metadata`, чтобы закрыть attached-mode parity для orders.
2. Дожать integration scenarios для attached-mode GraphQL CRUD и cache invalidation.
3. Закрыть standalone public API, RBAC и indexer contract.
4. Решить advanced scope отдельно: schema versioning, import/export, computed/conditional fields.

Это уже не migration архитектуры, а нормальный backlog развития.
