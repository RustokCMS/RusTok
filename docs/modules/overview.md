# Документация по модулям RusToK

Этот документ фиксирует текущее состояние модульной архитектуры в репозитории:
- какие **обязательные Core-модули платформы** должны быть включены всегда;
- какие дополнительные доменные модули можно подключать по необходимости;
- какие инфраструктурные crate'ы относятся к **Core-модулям платформы** (обязательный слой ядра).

## 1. Общая картина

RusToK — модульный монолит: модули компилируются в общий бинарник и поднимаются через `ModuleRegistry`.

Ключевой момент: в RusToK core-модули состоят из двух уровней: lifecycle core (`ModuleKind::Core`) и infrastructural core (ядро runtime).

**Где смотреть в коде:**
- Runtime-регистрация модулей: `apps/server/src/modules/mod.rs`
- Синхронизация манифеста и runtime-регистрации: `apps/server/src/modules/manifest.rs`
- Контракт модуля и виды модулей: `crates/rustok-core/src/module.rs`
- Реестр Core/Optional: `crates/rustok-core/src/registry.rs`
- Манифест модулей: `modules.toml`

## 2. Что реально зарегистрировано в сервере

В текущей сборке в `ModuleRegistry` регистрируются:

### Обязательные Core-модули (`ModuleKind::Core`)

| Slug | Crate | Назначение |
| --- | --- | --- |
| `index` | `rustok-index` | **Core (critical)**: CQRS/read-model индексатор |
| `tenant` | `rustok-tenant` | **Core (critical)**: Tenant lifecycle и метаданные |
| `rbac` | `rustok-rbac` | **Core (critical)**: RBAC lifecycle и health |

Эти три модуля считаются **критичными для корректной работы платформы** и являются базовым contract-first минимумом для `apps/server`.

### Дополнительные доменные модули (`ModuleKind::Optional`)

| Slug | Crate | Назначение |
| --- | --- | --- |
| `content` | `rustok-content` | Базовый CMS-контент |
| `commerce` | `rustok-commerce` | e-commerce домен |
| `blog` | `rustok-blog` | Блоговая надстройка (depends_on: `content`) |
| `forum` | `rustok-forum` | Форумный модуль (depends_on: `content`) |
| `pages` | `rustok-pages` | Страницы и меню |

## 3. Core-модули инфраструктурного уровня (вне ModuleRegistry)

Эти crate'ы относятся к обязательным core-модулям платформы на инфраструктурном уровне:

| Crate | Статус | Примечание |
| --- | --- | --- |
| `rustok-core` | **Core (critical)** | Контракты, базовые типы и инфраструктура |
| `rustok-outbox` | **Core (critical)** | Транзакционная доставка событий (required в `modules.toml`) |
| `rustok-telemetry` | **Core (critical)** | Сквозная observability |

Итого обязательные core-модули платформы: `index`, `tenant`, `rbac`, `rustok-core`, `rustok-outbox`, `rustok-telemetry`.

Также есть опциональные/технические инфраструктурные crate'ы (`rustok-iggy`, `rustok-iggy-connector`, `rustok-mcp`, `alloy-scripting`).

## 4. Приложения

- `apps/server` (`rustok-server`) — API-сервер и orchestration модулей.
- `apps/admin` (`rustok-admin`) — админ-панель на Leptos.
- `apps/storefront` (`rustok-storefront`) — storefront на Leptos.
- `crates/rustok-mcp` (bin `rustok-mcp-server`) — MCP сервер/адаптер.

## 5. Связанные документы

- `docs/modules/registry.md` — реестр приложений и crate'ов.
- `docs/modules/manifest.md` — манифест и правила описания модулей.
- `docs/architecture/improvement-recommendations.md` — рекомендации и roadmap архитектуры.

## 6. Что делать при изменениях модульного состава

При добавлении/удалении модульных crate'ов или их регистрации в сервере:
1. Обновить `apps/server/src/modules/mod.rs` (если меняется runtime-регистрация).
2. Обновить `modules.toml` (required/depends_on/default_enabled).
3. Обновить `docs/modules/overview.md`, `docs/modules/registry.md` и при необходимости `docs/index.md`.
