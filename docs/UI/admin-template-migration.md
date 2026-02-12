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

## 2. Migration Checklist

### Phase 1: Shell (Layout & Navigation)

Самая важная часть. Переносим обертку приложения.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **Icons**: Setup `lucide-react` / `lucide-leptos`. | ⬜ | ⬜ | Unified icon set. |
| **Sidebar**: Create `AppSidebar` component. | ⬜ | ⬜ | Поддержка Collapsible state. |
| **Header**: Create `PageHeader` with Breadcrumbs. | ⬜ | ⬜ | Хлебные крошки должны быть динамическими. |
| **Theme**: Dark/Light mode toggle. | ⬜ | ⬜ | У нас уже есть, проверить стили. |
| **UserMenu**: Dropdown with Avatar & Logout. | ⬜ | ⬜ | Подключить `auth.logout()`. |

### Phase 2: Dashboard (Overview)

Главная страница с виджетами.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **Stats Cards**: Port `KpiCard` styles. | ⬜ | ⬜ | У нас есть `StatsCard`, обновить дизайн. |
| **Charts**: Add `recharts` / Rust Charts. | ⬜ | ⬜ | `Overview` graph (Sales/Activity). |
| **Recent Sales**: List widget. | ⬜ | ⬜ | Simple table/list. |
| **Layout**: Grid system responsive check. | ⬜ | ⬜ | Mobile check. |

### Phase 3: Tables & Lists (Users/Products)

Самая сложная часть — таблицы с данными.

| Task | 🧩 Template UI | 🦀 Leptos | Notes |
| :--- | :--- | :--- | :--- |
| **DataTable**: Port generic table component. | ⬜ | ⬜ | Shadcn `Table`, `TableHeader`... |
| **Pagination**: Port pagination UI. | ⬜ | ⬜ | Connect to `leptos-shadcn-pagination`. |
| **Filters**: Port Toolbar (Search/Filter). | ⬜ | ⬜ | Connect to URL state. |
| **Columns**: Define User/Product columns. | ⬜ | ⬜ | `Avatar`, `StatusBadge`, `Actions`. |

### Phase 4: Forms (Profile/Auth)

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
