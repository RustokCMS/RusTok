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

### 4.3 Структура UI-пакетов модулей

**Leptos UI** — живёт внутри модульного крейта через feature flags (`/admin`, `/storefront`). Папки `/admin` и `/storefront` сами являются publishable крейтами (crates.io).

```text
crates/rustok-blog/
  Cargo.toml           # rustok-blog (backend)
  src/
  admin/               # rustok-blog-admin → crates.io
    Cargo.toml
    src/
  storefront/         # rustok-blog-storefront → crates.io
    Cargo.toml
    src/
```

`apps/admin/Cargo.toml` зависит от `rustok-blog-admin`, `rustok-commerce-admin` и т.д.
BuildExecutor управляет пересборкой WASM автоматически при install/uninstall.

**Next.js UI** — отдельные npm-пакеты внутри директории `packages/` приложения:

```text
apps/next-admin/
  packages/
    blog/              # @rustok/blog-admin
    commerce/          # @rustok/commerce-admin
  src/                 # само приложение, импортирует из packages/*
  package.json

apps/next-frontend/
  packages/
    blog/              # @rustok/blog-frontend
    commerce/          # @rustok/commerce-frontend
  src/
  package.json
```

Убрать модуль из Next.js:

1. Удалить `apps/next-admin/packages/<name>/`
2. Убрать зависимость из `apps/next-admin/package.json`
3. `npm install && npm run build`

> [!IMPORTANT]
> Авто-установка модулей через marketplace работает только для **Leptos**-стека.
> Next.js приложения требуют ручной пересборки при добавлении/удалении модуля.

### 4.4 UI readiness (non-core)

| Модуль | Leptos (sub-crate) | Next.js (packages/) | Статус |
| --- | --- | --- | --- |
| `content` | `admin/`, `storefront/` | `packages/content/` | ❌ Not ready |
| `commerce` | `admin/`, `storefront/` | `packages/commerce/` | ❌ Not ready |
| `blog` | `admin/`, `storefront/` | `packages/blog/` | ⚠️ Partial |
| `forum` | `admin/`, `storefront/` | `packages/forum/` | ⚠️ Partial |
| `pages` | `admin/` | `packages/pages/` | ❌ Not ready |

> [!NOTE]
> Устаревшие пути:
>
> - `crates/rustok-blog/ui/admin` → мигрирует в `crates/rustok-blog/admin/` (Leptos sub-crate)
> - `apps/next-admin/src/features/blog/` → мигрирует в `apps/next-admin/packages/blog/`


## 5. Приложения

- `apps/server` (`rustok-server`) — API-сервер и orchestration модулей.
- `apps/admin` (`rustok-admin`) — **primary** админ-панель на Leptos (CSR/WASM). Участвует в авто-деплое при install/uninstall модулей.
- `apps/storefront` (`rustok-storefront`) — **primary** storefront на Leptos (SSR). Участвует в авто-деплое при install/uninstall модулей.
- `apps/next-admin` — Next.js Admin (экспериментальная headless-альтернатива). Пересборка **вручную**; не участвует в module install pipeline.
- `apps/next-frontend` — Next.js Storefront (экспериментальная headless-альтернатива). Пересборка **вручную**; не участвует в module install pipeline.
- `crates/rustok-mcp` (bin `rustok-mcp-server`) — MCP сервер/адаптер.

Leptos-стек (`apps/admin`, `apps/storefront`) — основной фокус разработки и авто-деплоя.
Next.js-стек — для headless-режима и JS-разработчиков; управляется независимо.

## 6. Связанные документы

- `docs/modules/registry.md` — реестр приложений и crate'ов.
- `docs/modules/manifest.md` — манифест и правила описания модулей.
- `docs/architecture/principles.md` — архитектурные инварианты и живые runtime contracts.

## 7. Что делать при изменениях модульного состава

При добавлении/удалении модульных crate'ов или их регистрации в сервере:

1. Обновить `apps/server/src/modules/mod.rs` (если меняется runtime-регистрация).
2. Обновить `modules.toml` (required/depends_on/default_enabled).
3. Обновить `docs/modules/overview.md`, `docs/modules/registry.md` и при необходимости `docs/index.md`.
4. Если затронуты Leptos UI-крейты — адд/ремув зависимость `rustok-<module>-admin` в `apps/admin/Cargo.toml` и `apps/storefront/Cargo.toml`; запустить пересборку WASM (BuildExecutor делает это автоматически).
5. Для Next.js: добавить/удалить `apps/next-admin/packages/<module>/` и `apps/next-frontend/packages/<module>/`; обновить `package.json`; `npm install && npm run build`. Авто-деплой **не предусмотрен**.

## 8. Проверка готовности к внедрению Tiptap / Page Builder (blog/forum/pages/content)

Детальный план внедрения вынесен в отдельный документ: [План внедрения Tiptap/Page Builder](./tiptap-page-builder-implementation-plan.md).

Краткий статус:

- backend-контракт (`markdown` + `rt_json_v1` + server-side sanitize/validation) уже готов;
- UI-интеграция в production-маршруты admin-приложений и rollout-процедуры — в работе;
- запуск по умолчанию допускается только после прохождения фаз release-gate из отдельного плана.
