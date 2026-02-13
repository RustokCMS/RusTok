# Документация по модулям RusToK

Этот документ описывает текущее состояние модульной архитектуры в репозитории:
- какие модульные crate'ы существуют;
- какие из них реально зарегистрированы в `rustok-server`;
- какие crate'ы относятся к инфраструктуре и приложениями.

## 1. Общая картина

RusToK — модульный монолит: модули компилируются в общий бинарник, но имеют
изолированную ответственность и общий контракт `RusToKModule`.

Ключевой момент: **наличие module crate не равно runtime-регистрации**. Модуль
должен быть явно добавлен в `build_registry()` сервера.

**Где смотреть в коде:**
- Runtime-регистрация: `apps/server/src/modules/mod.rs`
- Контракт модуля/реестр: `crates/rustok-core/src/module.rs`, `crates/rustok-core/src/registry.rs`
- Конфигурация workspace: `Cargo.toml`

## 2. Что реально зарегистрировано в сервере

В текущей сборке в `ModuleRegistry` регистрируются:

| Slug | Crate | Назначение |
| --- | --- | --- |
| `content` | `rustok-content` | Базовый CMS-контент |
| `commerce` | `rustok-commerce` | e-commerce домен |
| `blog` | `rustok-blog` | Блоговая надстройка |
| `forum` | `rustok-forum` | Форумный модуль |
| `pages` | `rustok-pages` | Страницы и меню |

## 3. Module crates в репозитории (с `impl RusToKModule`)

Помимо зарегистрированных модулей, в workspace есть ещё module crate'ы:

| Slug | Crate | Статус в `apps/server` |
| --- | --- | --- |
| `tenant` | `rustok-tenant` | Не регистрируется в текущем `build_registry()` |
| `rbac` | `rustok-rbac` | Не регистрируется в текущем `build_registry()` |
| `index` | `rustok-index` | Не регистрируется в текущем `build_registry()` |

Это важно учитывать при чтении документации и планировании rollout по tenant-модулям.

## 4. Доменные модули и ответственность

### `rustok-content`
- Роль: базовый контентный модуль.
- Основные части: `entities/`, `services/`, `dto/`.

### `rustok-commerce`
- Роль: commerce-домен (каталог, заказы, цены, склад).
- Основные части: `entities/`, `services/`, `dto/`.

### `rustok-blog`
- Роль: блоговая надстройка поверх контента.

### `rustok-forum`
- Роль: форум (категории, темы, ответы, модерация).

### `rustok-pages`
- Роль: страницы и меню.

### `rustok-index`
- Роль: read-model / индексный модуль (CQRS).
- Примечание: в кодовой базе есть, но в текущей серверной регистрации отсутствует.

### `rustok-tenant`
- Роль: tenant metadata/helpers.
- Примечание: есть как module crate, но не зарегистрирован в `build_registry()`.

### `rustok-rbac`
- Роль: role-based access control helpers.
- Примечание: есть как module crate, но не зарегистрирован в `build_registry()`.

## 5. Инфраструктурные crates

- `rustok-core` — контракты модулей, registry, события, базовые типы.
- `rustok-outbox` — outbox-публикация событий.
- `rustok-iggy` — L2 transport/replay.
- `rustok-iggy-connector` — connector-слой для Iggy.
- `rustok-telemetry` — tracing/metrics.
- `rustok-mcp` — MCP toolkit/integration crate.
- `alloy-scripting` — скриптовый движок и orchestration.

## 6. Приложения

- `apps/server` (`rustok-server`) — API-сервер, поднимает `ModuleRegistry`.
- `apps/admin` (`rustok-admin`) — админ-панель.
- `apps/storefront` (`rustok-storefront`) — storefront на Leptos.
- `apps/mcp` (`rustok-mcp-server`) — MCP stdio сервер на базе `rustok-mcp`.

## 7. Связанные документы

- `docs/modules/MODULE_MATRIX.md` — сводная матрица модулей.
- `docs/modules/module-registry.md` — lifecycle/toggle/guards.
- `docs/modules/module-manifest.md` — manifest/rebuild-подход.
- `docs/modules/module-rebuild-plan.md` — roadmap по install/uninstall через rebuild.

## 8. Что делать при изменениях модульного состава

При добавлении/удалении модульных crate'ов или их регистрации в сервере:
1. Обновить `apps/server/src/modules/mod.rs` (если меняется runtime-регистрация).
2. Обновить `docs/modules/modules.md` и `docs/modules/MODULE_MATRIX.md`.
3. Проверить consistency с `docs/modules/module-registry.md`.
