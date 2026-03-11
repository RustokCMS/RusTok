# Документация по модулям RusToK

Этот документ фиксирует текущее состояние модульной архитектуры в репозитории:
- какие **обязательные Core-модули платформы** должны быть включены всегда;
- какие дополнительные доменные модули можно подключать по необходимости;
- какие остальные обязательные core-модули входят в ядро платформы.

## 1. Общая картина

RusToK — модульный монолит: модули компилируются в общий бинарник и поднимаются через `ModuleRegistry`.

Ключевой момент: в RusToK есть обязательные core-модули платформы и дополнительные optional-модули.

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

## 3. Остальные обязательные core-модули

Эти crate'ы относятся к обязательным core-модулям платформы:

| Crate | Статус | Примечание |
| --- | --- | --- |
| `rustok-core` | **Core (critical)** | Контракты, базовые типы и инфраструктура |
| `rustok-outbox` | **Core (critical)** | Транзакционная доставка событий (required в `modules.toml`) |
| `rustok-telemetry` | **Core (critical)** | Сквозная observability |

Итого обязательные core-модули платформы: `index`, `tenant`, `rbac`, `rustok-core`, `rustok-outbox`, `rustok-telemetry`.

Также есть дополнительные optional crate'ы (`rustok-iggy`, `rustok-iggy-connector`, `rustok-mcp`, `alloy-scripting`).

## 4. UI composition policy для optional-модулей

### 4.1 Базовое правило

Для модулей `ModuleKind::Optional` UI-слой **не должен хардкодиться в приложениях** (`apps/admin`, `apps/next-admin`, `apps/storefront`, `apps/next-frontend`).
Экраны, меню, nav items, guards и редакторы подключаются из модульных UI-пакетов, поставляемых самим модулем.

### 4.2 Исключение для core

Следующие модули и crate'ы считаются платформенным core-слоем и **не обязаны** следовать UI-паттерну модульных пакетов:

- Core-модули: `index`, `tenant`, `rbac`.
- Платформенные core crate'ы: `rustok-core`, `rustok-outbox`, `rustok-telemetry` (и их инфраструктурные зависимости).

### 4.3 Guidance по структуре модульных UI-пакетов

Рекомендуемая структура внутри доменного crate (раздельно для Next и Leptos):

- `crates/rustok-<module>/ui/admin-next`
- `crates/rustok-<module>/ui/admin-leptos`
- `crates/rustok-<module>/ui/frontend-next`
- `crates/rustok-<module>/ui/frontend-leptos`

Допустимый transitional-вариант до разделения: `ui/admin` и `ui/frontend`, но в документации модуля обязательно явно указать, для какого runtime (Next или Leptos) предназначен пакет.

Минимальные entry points:

- для admin: `adminNavItems` (или эквивалентный контракт admin-навигации, slot-компонентов и guard'ов);
- для storefront: `frontendNavItems` (или эквивалентный контракт storefront-навигации/слотов);
- для dual-stack модулей контракт должен быть реализован отдельно для Next и отдельно для Leptos.

Подключение выполняется через единый модульный контракт в приложениях:

- для `apps/next-admin`: через module registry (`registerAdminModule` / `getAdminNavItems`);
- для `apps/admin` (Leptos): через registry-композицию `AdminComponentRegistration`;
- для `apps/next-frontend`: через `registerStorefrontModule`;
- для `apps/storefront` (Leptos): через `StorefrontComponentRegistration`.

Требование: приложение агрегирует модульные контракты (registry/composition), а не содержит доменно-специфичный UI-код optional-модуля в собственном `nav-config`/роутинге.

Важно для сборки пакетов: UI-пакеты optional-модулей должны подключаться как явные зависимости соответствующих host-приложений (workspace/file dependency) с предсказуемыми entry points, чтобы install/rebuild не терял модульный UI на этапе bundle/build.

### 4.4 UI package readiness (non-core)

| Модуль | Admin UI package | Frontend UI package | Статус readiness | TODO (missing entry points) |
| --- | --- | --- | --- | --- |
| `content` | отсутствует | отсутствует | ❌ Not ready | Создать минимум `ui/admin-next` + `ui/admin-leptos` и `ui/frontend-next` + `ui/frontend-leptos`; экспортировать `adminNavItems`/`frontendNavItems`; подключить через единый registry-контракт в `apps/admin` и `apps/next-admin`. |
| `commerce` | отсутствует | отсутствует | ❌ Not ready | Создать Next+Leptos UI-пакеты (`admin-*`/`frontend-*`); определить контракты навигации/guard'ов; убрать хардкод экранов из приложений. |
| `blog` | `crates/rustok-blog/ui/admin` (Next) | `crates/rustok-blog/ui/frontend` (Next) | ⚠️ Partial | Это референс для **Next**-пакетов; добавить Leptos-пакеты (`admin-leptos`, `frontend-leptos`) либо явно зафиксировать single-stack поддержку в контракте модуля. |
| `forum` | частично (через `rustok-blog/ui/admin`, Next) | отсутствует | ⚠️ Partial | Вынести в самостоятельные Next/Leptos entry points; добавить `frontendNavItems` минимум для Next и Leptos storefront-контракт. |
| `pages` | отсутствует | отсутствует | ❌ Not ready | Создать Next+Leptos модульные UI-пакеты и entry points для меню/редакторов/guard'ов. |
| `alloy-scripting` | отсутствует | отсутствует | ❌ Not ready | Определить UI surface (если требуется), добавить `ui/admin-next`/`ui/admin-leptos` entry points и контракт интеграции; для frontend зафиксировать N/A или реализовать Next+Leptos пакеты. |

> Референс-образец модульных UI-пакетов в репозитории сейчас покрывает **Next runtime**: `crates/rustok-blog/ui/admin` и `crates/rustok-blog/ui/frontend`.

## 5. Приложения

- `apps/server` (`rustok-server`) — API-сервер и orchestration модулей.
- `apps/admin` (`rustok-admin`) — админ-панель на Leptos (CSR/WASM).
- `apps/storefront` (`rustok-storefront`) — storefront на Leptos (SSR).
- `apps/next-admin` — Next.js Admin (основной React-based admin dashboard).
- `apps/next-frontend` — Next.js Storefront (React-based storefront).
- `crates/rustok-mcp` (bin `rustok-mcp-server`) — MCP сервер/адаптер.

Принцип развития UI-стеков: пары Leptos и Next.js развиваются консистентно и параллельно —
`apps/admin` ↔ `apps/next-admin`, `apps/storefront` ↔ `apps/next-frontend`. При изменениях UI-контрактов
изменения и планы внедрения должны синхронно отражаться в обеих парах приложений.

## 6. Связанные документы

- `docs/modules/registry.md` — реестр приложений и crate'ов.
- `docs/modules/manifest.md` — манифест и правила описания модулей.
- `docs/architecture/improvement-recommendations.md` — рекомендации и roadmap архитектуры.

## 7. Что делать при изменениях модульного состава

При добавлении/удалении модульных crate'ов или их регистрации в сервере:
1. Обновить `apps/server/src/modules/mod.rs` (если меняется runtime-регистрация).
2. Обновить `modules.toml` (required/depends_on/default_enabled).
3. Обновить `docs/modules/overview.md`, `docs/modules/registry.md` и при необходимости `docs/index.md`.
4. Если затронуты UI-контракты, синхронно обновить implementation-планы и связанные docs для обоих UI-стеков:
   Leptos (`apps/admin`, `apps/storefront`) и Next.js (`apps/next-admin`, `apps/next-frontend`).

## 8. Проверка готовности к внедрению Tiptap / Page Builder (blog/forum/pages/content)

Детальный план внедрения вынесен в отдельный документ: [План внедрения Tiptap/Page Builder](./tiptap-page-builder-implementation-plan.md).

Краткий статус:
- backend-контракт (`markdown` + `rt_json_v1` + server-side sanitize/validation) уже готов;
- UI-интеграция в production-маршруты admin-приложений и rollout-процедуры — в работе;
- запуск по умолчанию допускается только после прохождения фаз release-gate из отдельного плана.

