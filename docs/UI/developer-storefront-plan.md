# Unified Development Plan (Admin + Storefront)

Этот документ объединяет планы разработки **Admin Panel** и **Storefront** с упором на Leptos и переиспользование общих элементов между админкой и фронтендом.

> 🛑 **CRITICAL: USE INTERNAL LIBRARIES FIRST**
>
> | **Functional Area** | **🦀 Leptos (Rust)** | **⚛️ Next.js (React, storefront)** |
> | :--- | :--- | :--- |
> | **Forms** | [`leptos-hook-form`](../../crates/leptos-hook-form) | `react-hook-form` |
> | **Validation** | [`leptos-zod`](../../crates/leptos-zod) | `zod` |
> | **Tables** | `leptos-struct-table` | `@tanstack/react-table` |
> | **State** | [`leptos-zustand`](../../crates/leptos-zustand) | `zustand` |
> | **Auth** | [`leptos-auth`](../../crates/leptos-auth) | `next-auth` / custom |
> | **GraphQL** | [`leptos-graphql`](../../crates/leptos-graphql) | `graphql-request` / `urql` |
> | **Pagination** | [`leptos-shadcn-pagination`](../../crates/leptos-shadcn-pagination) | `shadcn/ui` pagination |
> | **Reactive/browser utils** | `leptos-use` | `usehooks-ts` / custom |
> | **I18n** | `leptos_i18n` | `next-intl` |
> | **Metadata/SEO** | `leptos-next-metadata` | `next/metadata` |
> | **Async data/query** | `leptos-query` | `@tanstack/react-query` |
> | **Styling** | `tailwind-rs` | TailwindCSS |

## Принципы

- Мы **не клонируем** библиотеки целиком. Вместо этого делаем **минимальные адаптеры/обёртки** и закрываем пробелы **по мере работы** с админками/витриной.
- Приоритет — **готовые библиотеки и интеграции**; самопис — только если нет адекватного аналога.
- Любые отклонения фиксируем в UI‑документах и матрицах паритета.
- Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

См. базовые источники:

- [UI parity (admin + storefront)](./ui-parity.md)
- [Frontend libraries parity](./admin-libraries-parity.md) (Tech stack overlap)
- [Admin reuse matrix](./admin-reuse-matrix.md) (Leptos ecosystem references)
- [Tech parity tracker](./tech-parity.md)
- [Storefront overview](./storefront.md)

---

## Phase 1 — чек‑лист (восстановлено по коду)

## Phase 1 — Foundation (Completed)

| Работа | Leptos (`apps/admin`) | — |
| --- | --- | --- |
| Базовый layout и навигационный shell. | ✅ | ✅ |
| Dashboard/главная админки. | ✅ | ✅ |
| i18n (RU/EN). | ✅ | ✅ |
| Auth-guard (защита приватных маршрутов). | ✅ | ✅ |
| UI Primitives (shadcn-style). | ✅ | ✅ |

---

## Phase 2 — Users Vertical Slice (Current Status)

### Data Wiring

| Работа | Leptos | Next |
| --- | --- | --- |
| Auth wiring: `POST /api/auth/login`. | ✅ | ✅ |
| Auth wiring: `GET /api/auth/me` (bootstrap). | ✅ | ✅ |
| Users list: GraphQL `users` (pagination). | ✅ | ✅ |
| Users list: фильтры и поиск (URL sync). | ✅ | ✅ |
| Users detail view: GraphQL `user(id)`. | ✅ | ⬜ |

### CRUD Operations

| Работа | Leptos | Next |
| --- | --- | --- |
| Users CRUD: `createUser` mutation. | ✅ | ✅ |
| Users CRUD: `updateUser` mutation. | ✅ | ✅ |
| Users CRUD: `disableUser` mutation. | ✅ | ✅ |

### UI Components

| Работа | Leptos | Next |
| --- | --- | --- |
| PageHeader component. | ✅ | ✅ |
| Breadcrumbs. | ⬜ | ✅ |
| Stats cards (Dashboard). | ✅ | ✅ |
| Toasts (notifications). | ✅ | ✅ |

---

## Phase 3 — Auth & Security (Current Status)

### Authentication

| Работа | Leptos | Next |
| --- | --- | --- |
| Login page: tenant slug + email + password. | ✅ | ✅ |
| Registration: sign-up (email + password + tenant). | ✅ | ✅ |
| Password reset: request email. | ✅ | ✅ |
| Password reset: token + new password. | ✅ | ✅ |

### Profile & Security

| Работа | Leptos | Next |
| --- | --- | --- |
| Profile page: name/avatar/timezone/language. | ✅ | ✅ |
| Change password. | ✅ | ✅ |
| Active sessions list + “sign out all”. | ✅ | ✅ |
| Login history (success/failed). | ✅ | ✅ |

---

## Phase 4 — Content Management (New)

Поддержка модулей `rustok-blog`, `rustok-pages`, `rustok-forum`.

### CMS Core

| Работа | Leptos | Next |
| --- | --- | --- |
| **Pages**: List (Tree view?). | ⬜ | ⬜ |
| **Pages**: Editor (Markdown/WYSIWYG). | ⬜ | ⬜ |
| **File Manager**: Upload & Gallery. | ⬜ | ⬜ |

### Blog & Marketing

| Работа | Leptos | Next |
| --- | --- | --- |
| **Posts**: List & Status workflow (Draft/Pub). | ⬜ | ⬜ |
| **Categories**: Taxonomy management. | ⬜ | ⬜ |
| **SEO**: Meta tags editor per page. | ⬜ | ⬜ |

### Community

| Работа | Leptos | Next |
| --- | --- | --- |
| **Forum**: Topics moderation. | ⬜ | ⬜ |
| **Comments**: Moderation queue. | ⬜ | ⬜ |

---

## Phase 5 — Интеграция UI‑шаблона (Future)

> Подробный план инвентаризации и переноса описан в отдельном документе:
> **[Admin Template Migration Plan](./admin-template-migration.md)**

### Подготовка

| Работа | Leptos | Next |
| --- | --- | --- |
| Инвентаризация шаблона. | ⬜ | ⬜ |
| Карта соответствий: Template → RusTok. | ⬜ | ⬜ |

### Интеграция

| Работа | Leptos | Next |
| --- | --- | --- |
| Перенести страницы (Login/Register/Reset/Profile). | ⬜ | ⬜ |
| Перенести Users list/details + Dashboard. | ⬜ | ⬜ |
| Проверка визуального паритета. | ⬜ | ⬜ |

---

# 🛒 Storefront (Leptos SSR + Next.js)

## Phase 1 — Foundation (Completed)

| Работа | Leptos (`apps/storefront`) | Next.js (`apps/next-frontend`) |
| --- | --- | --- |
| Landing‑shell (hero + CTA + основной layout). | ✅ | ✅ |
| Блоки контента (карточки/фичи/коллекции). | ✅ | ✅ |
| i18n / локализация витрины (RU/EN). | ✅ | ✅ |
| Tailwind‑стили и базовая тема. | ✅ | ✅ |

---

## Phase 2 — Catalog & Discovery (Current Focus)

### Data Wiring

| Работа | Leptos | Next |
| --- | --- | --- |
| Product List: GraphQL `products` (pagination). | ⬜ | ⬜ |
| Product Details: GraphQL `product(slug)`. | ⬜ | ⬜ |
| Categories navigation (`/category/:slug`). | ⬜ | ⬜ |
| Search functionality (Input + Results page). | ⬜ | ⬜ |

### UI Components

| Работа | Leptos | Next |
| --- | --- | --- |
| Product Card component. | ⬜ | ⬜ |
| Price formatting (Currency support). | ⬜ | ⬜ |
| Gallery / Image slider. | ⬜ | ⬜ |

---

## Phase 3 — Content & Marketing (New)

Отображение контента из `rustok-blog` и `rustok-pages`.

| Работа | Leptos | Next |
| --- | --- | --- |
| **Blog**: Index page (List of posts). | ⬜ | ⬜ |
| **Blog**: Post details page (Markdown render). | ⬜ | ⬜ |
| **Pages**: Static pages (About, Terms). | ⬜ | ⬜ |
| **SEO**: Dynamic metadata from backend. | ⬜ | ⬜ |

---

## Phase 4 — Cart & Checkout (Future)

### Logic & State

| Работа | Leptos | Next |
| --- | --- | --- |
| Cart state management (Client-side / LocalStorage). | ⬜ | ⬜ |
| Add to Cart action. | ⬜ | ⬜ |
| Checkout Flow (Guest). | ⬜ | ⬜ |

### Integration

| Работа | Leptos | Next |
| --- | --- | --- |
| Order placement mutation (`createOrder`). | ⬜ | ⬜ |
| Payment Gateway integration stubs. | ⬜ | ⬜ |

---

## Phase 5 — Customer Account (Future)

### Auth & Profile

| Работа | Leptos | Next |
| --- | --- | --- |
| Customer Login/Register forms. | ⬜ | ⬜ |
| Order History list. | ⬜ | ⬜ |
| Address Book management. | ⬜ | ⬜ |

---

## Technical Implementation Notes

### Design System

- **Next.js**: Use `shadcn/ui` components (Admin & Storefront).
- **Leptos**: Use `leptos-shadcn-ui` (Admin & Storefront).
- **Tokens**: Shared design tokens via `docs/ui-parity.md`.

### State Management

- **Auth**: `HttpOnly` cookies + JWT.
- **Storefront Cart**: LocalStorage + Sync.
- **Admin State**: `leptos-query` / `@tanstack/react-query`.
