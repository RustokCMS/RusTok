# Admin Template Migration Plan

**Template Source:** `vendor/ui/next-shadcn-dashboard-starter`
**Target Apps:**

1. `apps/admin` (Leptos) — **Prioritized**
2. `apps/storefront` (Leptos) — **Follow-up** (reuse shared blocks)

Этот документ описывает процесс переноса UI/UX из готового шаблона в наши админки с учетом **наших библиотек**.

> 🛑 **CRITICAL: DO NOT COPY LOGIC BLINDLY**
> Шаблон содержит моковую логику (faker.js), свои хуки и fetch-запросы.
> **МЫ БЕРЕМ ТОЛЬКО UI (JSX/HTML/CSS).**
> Логику, состояние и API берем из наших `crates/`!
>
> | Feature | ❌ Template Logic | ✅ RusTok Implementation |
> | :--- | :--- | :--- |
> | **Auth** | `next-auth` (in template) | [`leptos-auth`](../../crates/leptos-auth) |
> | **Forms** | `react-hook-form` (local) | [`leptos-hook-form`](../../crates/leptos-hook-form) / Shared Zod |
> | **Tables** | Local `DataTable` implementation | [`leptos-table`](../../crates/leptos-table) / `@tanstack/react-table` |
> | **API** | Mock APIs / Local Fetch | [`leptos-graphql`](../../crates/leptos-graphql) / Generated Clients |

---

## 1. Inventory & Mapping (Инвентаризация)

Список страниц шаблона и их судьба в нашем проекте.

### Core Layout

| Template Component | Function | Action |
| :--- | :--- | :--- |
| `components/layout/app-sidebar.tsx` | Main Sidebar (Collapsible) | **ADOPT** (Critical) |
| `components/layout/header.tsx` | Top Bar (Breadcrumbs, Theme, User) | **ADOPT** |
| `components/layout/user-nav.tsx` | User Dropdown | **ADOPT** (Connect to `leptos-auth`) |

### Pages (Routes)

| Template Route | RusTok Route | Status |
| :--- | :--- | :--- |
| `/dashboard/overview` | `/dashboard` | **ADOPT** (Widgets & Charts) |
| `/dashboard/product` | `/products` (Storefront) | **ADOPT** (Table & Forms) |
| `/dashboard/profile` | `/profile` | **ADOPT** (Forms) |
| `/dashboard/kanban` | `/tasks` (Optional) | *Review later* |
| `/auth/*` | `/auth/*` | **ADOPT** (Login/Register Style) |

---

## 2. Migration Checklist (актуализирован: сначала Auth/RBAC/Navigation)

## 2.1 Целевая архитектура (ядро + loco-слой + подключаемые модули)

Подход фиксируем как **module-first** (по аналогии с WordPress-плагинами):

- `Core Admin` (ядро): auth/session, tenant context, RBAC guards, shell layout, routing, i18n, observability hooks.
- `Loco Integration Layer`: адаптеры к backend-контрактам (`/api/graphql`, auth endpoints, feature flags, capability registry).
- `Module Packages`: каждый модуль добавляет свои route(s), nav entries, permissions, widgets, forms, queries/mutations.

### Контракт модуля (обязательный минимум)

Каждый модуль публикует manifest/registry-описание:

- `id`, `version`, `depends_on`
- `permissions[]` (какие права нужны)
- `nav[]` (какие пункты меню добавить и при каких capability)
- `routes[]` (какие экраны регистрируются)
- `graphql[]` (какие операции/фрагменты использует)
- `ui_slots[]` (куда может встраиваться: dashboard cards, side panels, etc.)

> Важно: модуль не меняет ядро напрямую; он подключается через контракт и capability-checks.

### Наши самописные модули (куда класть новый функционал)

Чтобы не разрасталась случайная обвязка в приложениях, новый функционал кладем в **наши модули/пакеты**, а не в ad-hoc слой внутри `apps/*`.

Базовый приоритет размещения:

- `crates/rustok-core` — auth/tenant/rbac/infra-правила.
- `crates/rustok-content` / `crates/rustok-blog` / `crates/rustok-commerce` / `crates/rustok-forum` — доменная логика.
- `crates/rustok-index` — read models, индексные проекции и быстрые выборки.
- `crates/leptos-auth` / `crates/leptos-graphql` / `crates/leptos-hook-form` / `crates/leptos-table` / `crates/leptos-zod` / `crates/leptos-zustand` — shared frontend runtime-библиотеки.

Что остается в `apps/next-admin` и `apps/admin`:

- только composition/screen layer (layout, route wiring, page composition);
- без дублирования доменной бизнес-логики и без локальных "временных" API-клиентов вне shared-пакетов.

**Правило module-first delivery:**

1. Сначала определить целевой модуль (backend crate и/или shared frontend crate).
2. Реализовать/расширить API-контракт внутри модуля.
3. Подключить модуль в Next и Leptos экранах (параллельно).
4. Зафиксировать в changelog/плане, в какой модуль легла функциональность.

### "Атомарность заранее" без догматизма

Структура, которая обычно живет дольше всего:

1. `design-tokens` (цвета, spacing, typography, radii, shadows, motion)
2. `ui/primitives` (Button, Input, Dialog, Table primitives)
3. `ui/composites` (SearchBar, UserMenu, FilterBar, StatCard)
4. `features/<module>` (бизнес-компоненты, routes, data adapters)
5. `app-shell` (layout, navigation, access boundaries)

Это не strict Atomic Design naming, но по сути покрывает Atom→Molecule→Organism и при этом лучше совпадает с module-first delivery.

### Нужно ли "доворачивать" до атомарной архитектуры

Короткий ответ: **да, но эволюционно и без религиозного рефакторинга**.

Принцип принятия решений:

- Если компонент переиспользуется в 2+ модулях, поднимаем его в `ui/primitives` или `ui/composites`.
- Если логика доменная (RBAC, tenant, billing rules), она остается в `features/<module>` и backend/shared crates.
- `apps/*` не становятся местом хранения бизнес-компонентов "на потом" — только screen composition.

Definition of Done для атомарности в RusTok:

1. Любой новый UI-элемент привязан к одному из уровней: `tokens` / `primitives` / `composites` / `features` / `app-shell`.
2. Нет дубликатов одного и того же компонента в Next и Leptos без причины (фиксируем parity-task).
3. API-компонентов документирован в `IU/docs/api-contracts.md` при выходе из локального модуля в shared слой.
4. Перенос в shared слой делается маленькими шагами в рамках feature delivery, а не отдельным "большим переписыванием".

## 2.2 Zero-config запуск админки для локальной отладки (без ручной настройки)

Чтобы админка "сама понимала", к какому серверу подключаться и какие ключи использовать, фиксируем bootstrap-правила.

### Источники конфигурации (приоритет сверху вниз)

1. Runtime injected config (предпочтительно): `window.__RUSTOK_CONFIG__` / SSR-injected payload.
2. `.env` / `.env.local` (локальная разработка).
3. Safe defaults для dev (localhost-порты и demo tenant).

> UI никогда не содержит захардкоженные production URLs/keys; только runtime/env источники.

### Минимальный runtime config контракт

- `api_base_url` (например, `http://localhost:5150`)
- `graphql_endpoint` (по умолчанию `/api/graphql`)
- `auth_base_url` (по умолчанию `/api/auth`)
- `tenant_slug` (опционально; для single-tenant dev может заполняться автоматически)
- `app_env` (`local`/`staging`/`production`)
- `feature_flags` (опционально)

### Авто-детект в dev

- Если `api_base_url` не передан, использовать origin текущего UI (`window.location.origin`) + прокси путь.
- Если `tenant_slug` пуст, запрашивать `me`/`tenant context` и сохранять выбранный tenant в `leptos-auth` storage.
- Если backend недоступен, показывать diagnostics экран с готовым checklist (endpoint, auth, tenant header).

### Server Registry режим (для multi-server отладки)

Для случаев "админка не знает чей сервер":

- Включаем `server_registry.json` (или endpoint `/api/servers`) для dev/staging.
- Пользователь выбирает target server один раз в UI (селектор в login/start screen).
- Выбор сохраняется локально и подставляется в `api_base_url` + telemetry context.
- Для production этот режим отключается фичефлагом.

### Security и DX ограничения

- Секретные ключи не хранятся во фронте; только public config.
- Токены живут в auth storage/cookie согласно `leptos-auth` контракту.
- Любое изменение runtime config логируется в debug panel (кто/когда/какой target).


Приоритеты обновлены под быстрый запуск рабочей админки:

1. **Auth + роли + навигация** (чтобы можно было безопасно ходить по всем разделам)
2. Адаптация готовых экранов из шаблона под наш API/permissions
3. Таблицы и тяжелые data-grid сценарии — **в последнюю очередь**

### 2.x Параллельная синхронизация Next ↔ Leptos (обязательное правило)

Делаем **параллельно**, а не последовательно:

- если внедрили auth flow в `apps/next-admin`, в том же спринте внедряем эквивалент в `apps/admin` (Leptos);
- если добавили guard/nav-permissions в Next, сразу повторяем контракт и поведение в Leptos;
- если в Next добавили новый экран/feature под RusTok API, фиксируем parity-task для Leptos.

> Next-ветка — это быстрый адаптируемый UI-контур и полигон для UX.
> Leptos-ветка — целевой runtime. Функциональный разрыв между ними не допускается дольше одного спринта.

#### Минимальный parity backlog (идет парно)

1. Auth (sign-in/sign-up/sign-out, refresh, redirect).
2. RBAC (me/permissions + guards + 403 UX).
3. Navigation (permission-aware sidebar/routes).
4. Core pages (overview/profile/workspaces/team).
5. Tables & forms (последний этап, но тоже парно).

### Phase 1: Shell + Auth + RBAC Navigation (P0)

Самая важная часть. Переносим обертку приложения.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **Icons**: Setup `lucide-react` / `lucide-leptos`. | ⬜ | ⬜ | Unified icon set. |
| **Sidebar**: Create `AppSidebar` component. | ⬜ | ⬜ | Поддержка Collapsible state. |
| **Header**: Create `PageHeader` with Breadcrumbs. | ⬜ | ⬜ | Хлебные крошки должны быть динамическими. |
| **Theme**: Dark/Light mode toggle. | ⬜ | ⬜ | У нас уже есть, проверить стили. |
| **UserMenu**: Dropdown with Avatar & Logout. | ⬜ | ⬜ | Подключить `auth.logout()`. |
| **Auth Guard**: Защита приватных роутов. | ⬜ | ⬜ | Redirect + сохранение return URL. |
| **Role Guard**: Проверки ролей/permissions на уровне страниц. | ⬜ | ⬜ | Backend source of truth + UX fallback (403). |
| **Nav RBAC**: Фильтрация пунктов sidebar по permissions. | ⬜ | ⬜ | Не показывать недоступные разделы. |
| **Access UX**: Страница/компонент `Forbidden`. | ⬜ | ⬜ | Единый UX при 403. |

### Phase 2: Адаптация готовых страниц (без heavy tables)

Ниже — страницы, которые уже есть в starter и их нужно адаптировать под RusTok-контракты.

| Template Route | Приоритет | Что адаптируем |
| :--- | :--- | :--- |
| `/auth/sign-in`, `/auth/sign-up` | P0 | Подключение к нашему auth flow/redirect semantics |
| `/dashboard/overview` | P0 | Реальные KPI/виджеты и permission-aware блоки |
| `/dashboard/profile` | P0 | Профиль текущего пользователя + security actions |
| `/dashboard/workspaces` | P0 | Tenant/workspace switching под нашу модель |
| `/dashboard/workspaces/team` | P0 | Управление участниками/ролями в рамках RBAC |
| `/dashboard/billing` | P1 | Статусы тарифа/лимитов (если включено фичефлагом) |
| `/dashboard/exclusive` | P1 | Пример feature-gated страницы под наши фичи |
| `/dashboard/kanban` | P2 | Опционально, после ядра админки |

### Phase 3: Dashboard (Overview polish)

Главная страница с виджетами.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **Stats Cards**: Port `KpiCard` styles. | ⬜ | ⬜ | У нас есть `StatsCard`, обновить дизайн. |
| **Charts**: Add `recharts` / Rust Charts. | ⬜ | ⬜ | `Overview` graph (Sales/Activity). |
| **Recent Sales**: List widget. | ⬜ | ⬜ | Simple table/list. |
| **Layout**: Grid system responsive check. | ⬜ | ⬜ | Mobile check. |

### Phase 4: Tables & Lists (Users/Products) — в последнюю очередь

Самая сложная часть — таблицы с данными.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **DataTable**: Port generic table component. | ⬜ | ⬜ | Shadcn `Table`, `TableHeader`... |
| **Pagination**: Port pagination UI. | ⬜ | ⬜ | Connect to `leptos-shadcn-pagination`. |
| **Filters**: Port Toolbar (Search/Filter). | ⬜ | ⬜ | Connect to URL state. |
| **Columns**: Define User/Product columns. | ⬜ | ⬜ | `Avatar`, `StatusBadge`, `Actions`. |

### Phase 5: Forms (Profile/Auth)

Формы ввода данных.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **Input Fields**: Confirm styles (Input, Select). | ⬜ | ⬜ | Проверить Error states. |
| **Form Layout**: Grid/Stack layout. | ⬜ | ⬜ | `AutoForm` patterns if applicable. |
| **Validation UI**: Error messages styling. | ⬜ | ⬜ | `Zod` error integration. |

---

## 3. Technical Guidelines

Для развертывания и подключения админки к отдельному серверу см. пошаговый quickstart:
- [`docs/UI/admin-server-connection-quickstart.md`](./admin-server-connection-quickstart.md)
- В том числе целевой full-dev сценарий: одной командой поднимаются server + 2 админки + 2 storefront на отдельных портах.
- И нейтральный список методов установки/запуска: Docker Compose, VPS+Docker, Kubernetes (k8s), Railway/Fly/Render, а также ручная установка и one-command install-скрипт (план).
- Сверка паритета библиотек Next Starter ↔ Leptos: [`docs/UI/admin-libraries-parity.md`](./admin-libraries-parity.md) (включая gap-лог и явные замены).
- Snapshot каталога Rust/UI компонентов для внутреннего парсинга/решений: [`docs/UI/rust-ui-component-catalog.md`](./rust-ui-component-catalog.md).
- Инструкция для исполнителя-агента: [`docs/UI/agent-execution-guide.md`](./agent-execution-guide.md).
- PR workflow: если текущий PR был обновлен вне Codex и не поддерживает апдейт из сессии, создаем **новый PR** с отдельным коммитом и ссылкой на предыдущий.

### 3.0 Dependency policy (обязательное правило для агентов и разработчиков)

Запрещено самостоятельно удалять библиотеки/пакеты из проекта только потому, что "не заработало сразу".

Разрешенный порядок действий:

1. Зафиксировать проблему (лог, ошибка, версия, минимальный repro).
2. Проверить официальную документацию и совместимость версий.
3. Предложить исправление интеграции без удаления библиотеки.
4. Если библиотеку действительно нужно заменить — сначала создать RFC/issue с обоснованием, рисками, планом миграции и approval от владельца.

Запрещенный anti-pattern:

- удалить библиотеку, переписать всё на самописный временный слой и не задокументировать решение.

## 3.1 Submodule Status & Integration Assessment (2026-02)

Провели попытку подтянуть шаблонный submodule:

```bash
git submodule update --init --recursive vendor/ui/next-shadcn-dashboard-starter
```

Текущий статус в CI/container: загрузка блокируется сетевым ограничением (`CONNECT tunnel failed, response 403`), поэтому код шаблона в этом окружении не доступен для детального line-by-line аудита.

### Что это означает для "минимальных усилий"

- **Да, можно подключить с минимальными усилиями как отдельное Next-приложение**, если использовать его как `apps/adminka` и проксировать через backend gateway.
- **Нет, нельзя минимально "встроить" в текущий `apps/admin` (Leptos)** без адаптационного слоя, т.к. это другой runtime (React/Next vs Leptos).

### Рекомендованный путь

1. Подтянуть submodule в среде с доступом к GitHub.
2. Запустить starter как отдельный app (без переноса логики).
3. Подключить к нашим API (`/api/graphql` и auth endpoints).
4. Переиспользовать из шаблона только UI-композиции; доменную логику держать в RusTok.

> Для текущего `apps/admin` продолжаем курс на Leptos-first migration по чеклисту выше.

### 🦀 Leptos Implementation

1. Copy component code from `vendor/ui/.../components/...`.
2. Replace `import { ... }` to relative paths.
3. **DELETE** `useFakeData` hooks.
4. **REPLACE** `zod` schemas with shared schemas where possible.
5. Use `constants/nav-items.ts` pattern for Navigation logic (don't hardcode).

### 🦀 Leptos Implementation

1. Look at the `tsx` code to understand structure (Layout -> Grid -> Card).
2. Implement using `view! { ... }` macros.
3. Use `leptos-shadcn-ui` primitives (`Button`, `Card`, `Input`).
4. If a component is missing in `leptos-shadcn-ui`, implement it locally in `apps/admin/src/components/ui`.
