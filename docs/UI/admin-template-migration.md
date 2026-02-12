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

### "Атомарность заранее" без догматизма

Структура, которая обычно живет дольше всего:

1. `design-tokens` (цвета, spacing, typography, radii, shadows, motion)
2. `ui/primitives` (Button, Input, Dialog, Table primitives)
3. `ui/composites` (SearchBar, UserMenu, FilterBar, StatCard)
4. `features/<module>` (бизнес-компоненты, routes, data adapters)
5. `app-shell` (layout, navigation, access boundaries)

Это не strict Atomic Design naming, но по сути покрывает Atom→Molecule→Organism и при этом лучше совпадает с module-first delivery.


Приоритеты обновлены под быстрый запуск рабочей админки:

1. **Auth + роли + навигация** (чтобы можно было безопасно ходить по всем разделам)
2. Адаптация готовых экранов из шаблона под наш API/permissions
3. Таблицы и тяжелые data-grid сценарии — **в последнюю очередь**

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
