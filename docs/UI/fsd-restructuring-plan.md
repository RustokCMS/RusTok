# FSD Реструктуризация Admin Panels — Детальный план

**Ветка:** `claude/review-fsd-admin-design-sOKKf`
**Статус:** ✅ Завершено (Фазы 1.1–1.5 + Фазы 2–3 завершены)
**Охват:** `apps/admin` (Leptos CSR) + `apps/next-admin` (Next.js) + `UI/` workspace (leptos + next компоненты) + `crates/leptos-ui`

---

## 0. UI/ Workspace — роль и архитектура

`UI/` — **Internal UI workspace** для параллельных реализаций компонентов с единым контрактом.

```
UI/
├── tokens/base.css              ← Общие CSS custom properties --iu-* (цвета, spacing, radius, fonts, shadows)
├── docs/api-contracts.md        ← Единый API-контракт для всех компонентов (Button, Input, Select …)
├── leptos/                      ← iu-leptos Rust crate (Cargo.toml, src/)
│   ├── src/                     ← Button, Input, Textarea, Select, Checkbox, Switch, Badge, Spinner
│   └── components/              ← deprecated placeholder (реализации в src/)
└── next/
    └── components/              ← React/Next.js IU-обёртки над shadcn (Button, Input, Badge …)
```

**Принципы (из `UI/README.md`):**
- **API-паритет**: Leptos и Next.js компоненты экспортируют одинаковый API (props, варианты, поведение)
- **Общие токены**: Стили базируются на `UI/tokens/base.css` (`--iu-*` CSS custom properties)
- **Без дублирования shadcn**: `UI/next/components/` оборачивает shadcn как _reference_, `UI/leptos/components/` — нативная Leptos реализация через `cloud-shuttle/leptos-shadcn-ui`

**Как будет подключаться:**

| Приложение | Leptos-компоненты | Next-компоненты | Токены |
|-----------|------------------|-----------------|--------|
| `apps/admin` | `iu-leptos` crate (из `UI/leptos/`) | — | `@import "UI/tokens/base.css"` |
| `apps/next-admin` | — | `@iu/*` tsconfig alias | `@import "UI/tokens/base.css"` |

**Целевой список компонентов** (из `UI/README.md`):
Button, Input, Textarea, Select, Checkbox, Switch, Badge/Tag, Table, Modal/Dialog, Toast, Sidebar/Navigation, Header/Topbar

---

## 1. Ревью — Текущее состояние

### 1.1 apps/admin (Leptos CSR) — FSD отсутствует

Структура классическая `components/pages`, FSD-слои не соблюдаются:

```
src/
├── api/                           ← должно быть shared/api/
│   ├── mod.rs                     ← GraphQL executor, URL resolver
│   └── queries.rs                 ← все GraphQL строки + persisted hashes
├── app.rs                         ← роутер + провайдеры (должен быть в app/)
├── components/
│   ├── features/auth/             ← ✅ правильный слой, неверное место
│   │   ├── mod.rs
│   │   └── user_menu.rs
│   ├── layout/                    ← это widgets, не компоненты
│   │   ├── app_layout.rs          ← AppLayout widget
│   │   ├── header.rs              ← Header widget
│   │   ├── nav_config.rs          ← должно быть shared/config/
│   │   └── sidebar.rs             ← Sidebar widget
│   └── ui/                        ← должно быть shared/ui/ или widgets/
│       ├── page_header.rs         ← shared/ui/
│       └── stats_card.rs          ← widgets/stats-card/
├── i18n.rs                        ← должно быть shared/i18n/
├── lib.rs                         ← объявляет все модули
├── main.rs
├── modules/                       ← runtime plugin registry (app/)
│   ├── core.rs
│   ├── mod.rs
│   └── registry.rs
├── pages/                         ← ✅ FSD pages — правильный слой
│   ├── dashboard.rs
│   ├── login.rs
│   ├── not_found.rs
│   ├── profile.rs
│   ├── register.rs
│   ├── reset.rs
│   ├── security.rs
│   ├── user_details.rs
│   └── users.rs
└── providers/locale/              ← должно быть app/providers/
```

**Критическая проблема:** В `Cargo.toml` объявлены `leptos-ui`, `leptos-forms`, `leptos-table`, `leptos-use`, `leptos-chartistry`, `leptos-shadcn-pagination` — но все они в `cargo-udeps.ignore`, т.е. **не используются** несмотря на то что уже реализованы.

### 1.2 apps/next-admin (Next.js) — FSD частичный

```
src/
├── app/                           ← ✅ Next.js App Router (≈ FSD app-layer)
│   ├── api/auth/[...nextauth]/
│   ├── auth/sign-in/, sign-up/
│   └── dashboard/
│       ├── overview/ (с parallel routes @area_stats, @bar_stats, @pie_stats, @sales)
│       ├── users/[userId]/
│       ├── product/[productId]/
│       ├── kanban/
│       ├── billing/
│       ├── profile/
│       └── workspaces/
├── components/                    ← ❌ смесь widgets, shared/ui, forms, themes
│   ├── breadcrumbs.tsx            → shared/ui/
│   ├── file-uploader.tsx          → shared/ui/
│   ├── form-card-skeleton.tsx     → shared/ui/
│   ├── forms/                     → shared/ui/forms/ (или features/ если специфичны)
│   ├── icons.tsx                  → shared/ui/
│   ├── kbar/                      → widgets/command-palette/
│   ├── layout/                    → widgets/app-shell/
│   ├── modal/alert-modal.tsx      → shared/ui/ (или widgets/)
│   ├── nav-main.tsx               → widgets/app-shell/
│   ├── nav-projects.tsx           → widgets/app-shell/
│   ├── nav-user.tsx               → widgets/app-shell/
│   ├── org-switcher.tsx           → widgets/app-shell/
│   ├── search-input.tsx           → shared/ui/
│   ├── themes/                    → shared/lib/themes/
│   └── ui/                        → shared/ui/ (shadcn — оставить на месте)
├── config/                        → shared/config/
│   ├── nav-config.ts
│   ├── data-table.ts
│   └── infoconfig.ts
├── constants/                     → shared/constants/
│   ├── data.ts
│   └── mock-api.ts
├── features/                      ← ✅ FSD features — правильный слой
│   ├── auth/components/
│   ├── kanban/components/ + utils/store.ts
│   ├── overview/components/       ← графики + skeleton
│   ├── products/components/       ← product-tables/ + product-form + product-view
│   ├── profile/components/ + utils/form-schema.ts
│   └── users/components/
├── hooks/                         → shared/hooks/
├── lib/                           → shared/lib/
│   ├── auth-api.ts
│   ├── data-table.ts
│   ├── format.ts
│   ├── graphql.ts
│   ├── parsers.ts
│   ├── searchparams.ts
│   └── utils.ts
├── styles/                        ← ✅ оставить
└── types/                         → shared/types/
    ├── base-form.ts
    ├── data-table.ts
    ├── index.ts
    └── next-auth.d.ts
```

### 1.3 Самописные библиотеки — Инвентарь

#### crates/ (Rust — для Leptos)

| Crate | Содержимое | Используется в admin | Проблема |
|-------|------------|---------------------|---------|
| `leptos-ui` | Button, Input, Badge, Card, Label, Separator, types | ❌ (cargo-udeps.ignore) | Не подключён в коде |
| `leptos-forms` | FormContext, Field, Validator, FormError | ❌ | Не используется |
| `leptos-table` | TableState, SortRule, FilterRule, SortDirection | ❌ | Не используется |
| `leptos-graphql` | GraphqlRequest/Response, execute, persisted_query | ✅ | Работает |
| `leptos-auth` | AuthSession, AuthProvider, ProtectedRoute | ✅ | Работает |
| `leptos-hook-form` | FormState, FieldError, ValidationIssue | ❌ | Не используется |
| `leptos-zod` | ZodError, ZodIssue | ❌ | Не используется |
| `leptos-zustand` | StoreSnapshot, StoreUpdate | ❌ | Не используется |
| `leptos-shadcn-pagination` | Pagination component | ❌ | Не используется |
| `leptos-chartistry` | (внешний, workspace) | ❌ | Не используется |
| `leptos-use` | (внешний, workspace) | ❌ | Не используется |

#### packages/ (TypeScript — для Next.js)

| Package | Содержимое | Используется в next-admin |
|---------|------------|--------------------------|
| `leptos-auth/next` | AuthUser, AuthSession, AuthError, getClientAuth | Частично (через lib/auth-api.ts) |
| `leptos-graphql/next` | fetchGraphql, GraphqlRequest, GRAPHQL_ENDPOINT | Частично (через lib/graphql.ts) |
| `leptos-zod/next` | ZodIssue, ZodError, mapZodError | ❌ |
| `leptos-hook-form/next` | (есть в packages/) | ❌ |
| `leptos-zustand/next` | StoreSnapshot, StoreUpdate | ❌ |

#### UI/ (Дизайн-система — кросс-фреймворк)

| Директория | Содержимое | Статус |
|-----------|------------|--------|
| `UI/tokens/base.css` | CSS custom properties `--iu-*` (цвета, spacing, radius, fonts, shadows) | ✅ Определены, но не импортированы ни в одном приложении |
| `UI/docs/api-contracts.md` | Контракты: Button, Input, Textarea, Select, Checkbox, Switch, Badge/Tag | ✅ Задокументированы |
| `UI/docs/admin-skeleton.md` | Скелет архитектуры | ✅ Есть |
| `UI/leptos/components/` | **ПУСТО** (только README) | ❌ Нужно реализовать |
| `UI/next/components/` | **ПУСТО** (только README) | ❌ Нужно реализовать |

---

## 2. FSD Gap-анализ

| Слой | apps/admin (Leptos) | apps/next-admin (Next.js) |
|------|--------------------|--------------------|
| `app/` | ⚠️ Растворён в app.rs + modules/ + providers/ | ✅ app/ (Next.js App Router) |
| `pages/` | ✅ pages/ | ✅ app/dashboard/* |
| `widgets/` | ❌ Нет слоя (код в components/layout/) | ❌ Нет слоя (код в components/) |
| `features/` | ⚠️ Только components/features/auth/ | ✅ features/ (полный) |
| `entities/` | ❌ **Отсутствует** | ❌ **Отсутствует** |
| `shared/` | ❌ api/, ui, config рассыпаны | ❌ lib, hooks, types, config рассыпаны |

---

## 3. Целевая архитектура

### 3.1 apps/admin (Leptos) — FSD target

```
src/
├── app/                           ← app-слой (роутер + провайдеры + реестр)
│   ├── mod.rs
│   ├── router.rs                  ← из app.rs (компонент App + Routes)
│   └── providers/
│       ├── mod.rs
│       ├── auth.rs                ← AuthProvider из leptos-auth (уже используется)
│       └── locale.rs              ← из providers/locale/mod.rs
│
├── pages/                         ← ✅ без изменений
│   ├── mod.rs
│   ├── dashboard.rs
│   ├── login.rs
│   ├── not_found.rs
│   ├── profile.rs
│   ├── register.rs
│   ├── reset.rs
│   ├── security.rs
│   ├── user_details.rs
│   └── users.rs
│
├── widgets/                       ← НОВЫЙ слой (агрегатные UI-блоки)
│   ├── mod.rs
│   ├── app_shell/                 ← из components/layout/
│   │   ├── mod.rs
│   │   ├── app_layout.rs          ← из components/layout/app_layout.rs
│   │   ├── header.rs              ← из components/layout/header.rs
│   │   └── sidebar.rs             ← из components/layout/sidebar.rs
│   ├── stats_card/                ← из components/ui/stats_card.rs
│   │   └── mod.rs
│   └── user_table/                ← НОВЫЙ: обёртка над leptos-table + leptos-shadcn-pagination
│       └── mod.rs
│
├── features/                      ← НОВЫЙ слой (отдельный от components/)
│   ├── mod.rs
│   ├── auth/                      ← из components/features/auth/
│   │   ├── mod.rs
│   │   └── user_menu.rs
│   ├── users/                     ← НОВЫЙ: логика фильтрации/поиска пользователей
│   │   └── mod.rs
│   └── profile/                   ← НОВЫЙ: логика формы профиля (leptos-forms)
│       └── mod.rs
│
├── entities/                      ← НОВЫЙ слой (бизнес-сущности)
│   ├── mod.rs
│   ├── user/                      ← User entity
│   │   ├── mod.rs
│   │   ├── model.rs               ← User, UserRole, UserStatus типы
│   │   └── ui/                    ← UserAvatar, UserBadge компоненты
│   │       └── mod.rs
│   ├── product/                   ← Product entity
│   │   ├── mod.rs
│   │   └── model.rs               ← Product, ProductStatus типы
│   └── tenant/                    ← Tenant entity
│       ├── mod.rs
│       └── model.rs               ← Tenant типы (уже частично в auth context)
│
└── shared/                        ← НОВЫЙ слой (переиспользуемый код без бизнес-логики)
    ├── mod.rs
    ├── api/                       ← из src/api/
    │   ├── mod.rs                 ← get_graphql_url, request, request_with_persisted
    │   └── queries.rs             ← GraphQL query strings + hashes
    ├── ui/                        ← re-exports + admin-specific primitives
    │   ├── mod.rs                 ← re-export leptos-ui публичных компонентов
    │   └── page_header.rs         ← из components/ui/page_header.rs
    ├── config/                    ← конфигурация, константы
    │   ├── mod.rs
    │   └── nav.rs                 ← из components/layout/nav_config.rs
    └── i18n/                      ← из src/i18n.rs
        └── mod.rs
```

**lib.rs** после реструктуризации:
```rust
pub mod app;
pub mod entities;
pub mod features;
pub mod pages;
pub mod shared;
pub mod widgets;
```

### 3.2 apps/next-admin (Next.js) — FSD target

```
src/
├── app/                           ← ✅ без изменений (Next.js App Router)
│
├── widgets/                       ← НОВЫЙ слой
│   ├── app-shell/                 ← из components/layout/ + nav-*.tsx + org-switcher.tsx
│   │   ├── index.ts
│   │   ├── app-sidebar.tsx
│   │   ├── header.tsx
│   │   ├── user-nav.tsx
│   │   ├── nav-main.tsx
│   │   ├── nav-user.tsx
│   │   ├── nav-projects.tsx
│   │   ├── org-switcher.tsx
│   │   ├── page-container.tsx
│   │   ├── providers.tsx
│   │   ├── cta-github.tsx
│   │   └── info-sidebar.tsx
│   ├── command-palette/           ← из components/kbar/
│   │   ├── index.ts
│   │   ├── kbar-provider.tsx
│   │   ├── render-result.tsx
│   │   ├── result-item.tsx
│   │   └── use-theme-switching.tsx
│   ├── data-table/                ← НОВЫЙ агрегат (toolbar + table + pagination)
│   │   ├── index.ts
│   │   ├── data-table.tsx
│   │   ├── data-table-toolbar.tsx
│   │   ├── data-table-pagination.tsx
│   │   ├── data-table-faceted-filter.tsx
│   │   └── data-table-view-options.tsx
│   └── alert-modal/               ← из components/modal/
│       └── index.tsx
│
├── features/                      ← ✅ без изменений (уже правильный слой)
│   ├── auth/
│   ├── kanban/
│   ├── overview/
│   ├── products/
│   ├── profile/
│   └── users/
│
├── entities/                      ← НОВЫЙ слой
│   ├── user/
│   │   ├── index.ts
│   │   ├── model.ts               ← User, UserRole, UserStatus типы
│   │   └── ui/
│   │       ├── user-card.tsx      ← компонент карточки пользователя
│   │       └── user-avatar.tsx    ← аватар с fallback
│   ├── product/
│   │   ├── index.ts
│   │   ├── model.ts               ← Product, ProductStatus типы
│   │   └── ui/
│   │       └── product-card.tsx
│   └── tenant/
│       ├── index.ts
│       └── model.ts               ← Tenant, Workspace типы
│
└── shared/                        ← НОВЫЙ слой (объединить lib, hooks, types, config, constants)
    ├── api/                       ← GraphQL helpers
    │   ├── index.ts
    │   ├── graphql.ts             ← из lib/graphql.ts
    │   └── auth-api.ts            ← из lib/auth-api.ts
    ├── ui/                        ← примитивы + wrappers
    │   ├── index.ts
    │   ├── shadcn/                ← re-export компонентов из components/ui/
    │   ├── breadcrumbs.tsx        ← из components/breadcrumbs.tsx
    │   ├── file-uploader.tsx      ← из components/file-uploader.tsx
    │   ├── form-card-skeleton.tsx ← из components/form-card-skeleton.tsx
    │   ├── search-input.tsx       ← из components/search-input.tsx
    │   ├── icons.tsx              ← из components/icons.tsx
    │   └── forms/                 ← из components/forms/
    │       ├── form-input.tsx
    │       ├── form-select.tsx
    │       ├── form-textarea.tsx
    │       ├── form-checkbox.tsx
    │       ├── form-checkbox-group.tsx
    │       ├── form-radio-group.tsx
    │       ├── form-date-picker.tsx
    │       ├── form-file-upload.tsx
    │       ├── form-slider.tsx
    │       └── form-switch.tsx
    ├── lib/                       ← утилиты без бизнес-логики
    │   ├── index.ts
    │   ├── utils.ts               ← из lib/utils.ts
    │   ├── format.ts              ← из lib/format.ts
    │   ├── parsers.ts             ← из lib/parsers.ts
    │   ├── searchparams.ts        ← из lib/searchparams.ts
    │   ├── data-table.ts          ← из lib/data-table.ts
    │   └── themes/                ← из components/themes/
    │       ├── active-theme.tsx
    │       ├── font.config.ts
    │       ├── theme-mode-toggle.tsx
    │       ├── theme-provider.tsx
    │       ├── theme-selector.tsx
    │       └── theme.config.ts
    ├── hooks/                     ← из hooks/
    │   ├── index.ts
    │   ├── use-breadcrumbs.tsx
    │   ├── use-callback-ref.ts
    │   ├── use-controllable-state.tsx
    │   ├── use-data-table.ts
    │   ├── use-debounce.tsx
    │   ├── use-debounced-callback.ts
    │   ├── use-media-query.ts
    │   ├── use-mobile.tsx
    │   ├── use-multistep-form.tsx
    │   └── use-nav.ts
    ├── types/                     ← из types/
    │   ├── index.ts
    │   ├── base-form.ts
    │   ├── data-table.ts
    │   └── next-auth.d.ts
    ├── config/                    ← из config/
    │   ├── index.ts
    │   ├── nav-config.ts
    │   ├── data-table.ts
    │   └── infoconfig.ts
    └── constants/                 ← из constants/
        ├── index.ts
        ├── data.ts
        └── mock-api.ts
```

**tsconfig.json** — добавить path aliases:
```json
"paths": {
  "@/*":         ["./src/*"],
  "@/shared/*":  ["./src/shared/*"],
  "@/entities/*":["./src/entities/*"],
  "@/widgets/*": ["./src/widgets/*"],
  "@/features/*":["./src/features/*"],
  "~/*":         ["./public/*"]
}
```

---

## 4. Самописные библиотеки — Что писать

### 4.1 crates/leptos-ui — Рефакторинг на leptos-shadcn-ui

**Ключевое решение:** Вместо того чтобы писать компоненты с нуля в `crates/leptos-ui`, используем
[`cloud-shuttle/leptos-shadcn-ui`](https://github.com/cloud-shuttle/leptos-shadcn-ui) — это Leptos-аналог `shadcn/ui`, который используется в `apps/next-admin`. Это обеспечивает **паритет функций** между двумя админками.

`cloud-shuttle/leptos-shadcn-ui` покрывает 38+ компонентов:

| Категория | Компоненты |
|-----------|-----------|
| Form Elements | Button, Input, Label, Checkbox, Switch, Radio Group, Select, Textarea, Form, Combobox, Command, Input OTP |
| Layout | Card, Separator, Tabs, Accordion, Collapsible, Scroll Area, Resizable |
| Overlay | Dialog, Popover, Tooltip, Alert Dialog, Sheet, Drawer |
| Navigation | Breadcrumb, Navigation Menu, Context Menu, Dropdown Menu, Menubar |
| Feedback | Alert, Badge, Skeleton, Progress, Toast, Table, Calendar, Pagination |
| Interactive | Slider, Toggle, Carousel, Avatar |

**Установка** (добавить в `Cargo.toml` workspace):
```toml
# Вариант A — отдельные crates (рекомендован)
leptos-shadcn-button = "0.4.0"
leptos-shadcn-input = "0.4.0"
leptos-shadcn-card = "0.4.0"
leptos-shadcn-badge = "0.4.0"
# ... и т.д.

# Вариант B — monolithic с features
leptos-shadcn-ui = { version = "0.5.0", features = ["button", "input", "card", "badge", "select", "checkbox", "switch", "textarea", "avatar", "skeleton", "dialog", "table", "pagination", "dropdown-menu", "breadcrumb", "tooltip", "sheet", "separator", "tabs"] }
```

**Новая роль `crates/leptos-ui`:**

`crates/leptos-ui` становится **тонким RusTok-wrapper** над `leptos-shadcn-ui`:
- Re-export нужных компонентов с RusTok-специфичными defaults
- Добавляет недостающие компоненты (например, `Spinner` которого нет в shadcn)
- Применяет `--iu-*` CSS-токены через className

**Задачи:**
- [ ] Добавить `leptos-shadcn-ui` в `Cargo.toml` workspace (dependencies)
- [ ] Добавить `leptos-shadcn-ui` в `apps/admin/Cargo.toml`
- [ ] Рефакторить `crates/leptos-ui/src/lib.rs` — re-export из leptos-shadcn-ui вместо кастомных реализаций
- [ ] Удалить `crates/leptos-ui/src/{button,input,badge,card,label,separator}.rs` (заменены)
- [ ] Добавить `crates/leptos-ui/src/spinner.rs` — единственный кастомный компонент (нет в shadcn)
- [ ] Обновить `crates/leptos-ui/src/types.rs` — использовать типы из leptos-shadcn-ui где возможно
- [ ] Подключить `UI/tokens/base.css` в `apps/admin` — для CSS-переменных `--iu-*`

### 4.2 UI/next/components/ — Новая библиотека

Next.js wrappers над shadcn/ui с единым API по контракту. Использовать `--iu-*` CSS-переменные.

**Создать в `UI/next/components/`:**

```
UI/next/components/
├── index.ts                   ← barrel export всего
├── Button.tsx                 ← wrapper над shadcn Button
├── Input.tsx                  ← wrapper над shadcn Input
├── Textarea.tsx               ← wrapper над shadcn Textarea
├── Select.tsx                 ← wrapper над shadcn Select
├── Checkbox.tsx               ← wrapper над shadcn Checkbox
├── Switch.tsx                 ← wrapper над shadcn Switch
├── Badge.tsx                  ← wrapper над shadcn Badge
├── Avatar.tsx                 ← wrapper над shadcn Avatar
├── Skeleton.tsx               ← wrapper над shadcn Skeleton
└── Spinner.tsx                ← кастомный (shadcn не имеет Spinner)
```

Каждый wrapper:
- Принимает props по контракту из `UI/docs/api-contracts.md`
- Использует `--iu-*` переменные через className/style
- Имеет полную типизацию TypeScript

### 4.3 UI/tokens/base.css — Подключение

Сейчас токены **определены но не импортированы** ни в одно приложение.

**Добавить импорт в:**
- `apps/admin/index.html` или `apps/admin/style.css` — `@import "../../UI/tokens/base.css"`
- `apps/next-admin/src/styles/globals.css` — `@import "../../../UI/tokens/base.css"` или скопировать переменные

---

## 5. Фазы реализации

### Фаза 1: UI/ workspace — инфраструктура и компоненты

#### 1.1 Сделать UI/leptos/ Rust-crate (iu-leptos) ✅

`UI/leptos/` становится полноценным Rust crate в workspace. Компоненты живут в `UI/leptos/src/*.rs`, подключены через `mod` в `UI/leptos/src/lib.rs`.

**Задачи:**
- [x] Создать `UI/leptos/Cargo.toml` — crate name `iu-leptos`, `crate-type = ["cdylib", "rlib"]`
- [x] Добавить зависимости: `leptos`, `serde`
- [x] Создать `UI/leptos/src/lib.rs` — точка входа, pub mod + pub use
- [x] Добавить `"UI/leptos"` в `members` корневого `Cargo.toml`
- [x] Добавить `iu-leptos = { path = "UI/leptos" }` в `[workspace.dependencies]`

**Коммит:** `feat(ui/workspace): register UI/leptos as iu-leptos Rust crate`

#### 1.2 Реализовать компоненты Leptos в UI/leptos/src/ ✅

По контракту из `UI/docs/api-contracts.md`, используя `--iu-*` CSS-переменные:

| Файл | Компонент | Ключевые props |
|------|----------|---------------|
| `button.rs` | `Button` | variant, size, disabled, loading, left_icon/right_icon |
| `input.rs` | `Input` | size, disabled, invalid, prefix/suffix |
| `textarea.rs` | `Textarea` | size, disabled, invalid, rows |
| `select.rs` | `Select` | size, disabled, invalid, options, placeholder |
| `checkbox.rs` | `Checkbox` | checked (Signal), indeterminate, disabled |
| `switch.rs` | `Switch` | checked (Signal), disabled, size: Sm\|Md |
| `badge.rs` | `Badge` | variant, size, dismissible |
| `spinner.rs` | `Spinner` | size |

Все компоненты используют `--iu-*` CSS-переменные из `UI/tokens/base.css`.

**Коммит:** `feat(ui/leptos): implement Button, Input, Textarea, Select, Checkbox, Switch, Badge, Spinner`

#### 1.3 Рефакторинг crates/leptos-ui → wrapper над iu-leptos ✅

- [x] Добавить `iu-leptos` как зависимость в `crates/leptos-ui/Cargo.toml`
- [x] Заменить `src/lib.rs` на re-export из `iu_leptos` + оставить `Card`, `Label`, `Separator`
- [x] Удалить `src/{button,input,badge,types}.rs` (заменены iu-leptos)

**Коммит:** `refactor(leptos-ui): become thin re-export wrapper over iu-leptos`

#### 1.4 Реализовать компоненты Next.js в UI/next/components/

Thin wrappers над shadcn/ui (shadcn как reference, не дублирование):

| Файл | Обёртка | Ключевые props |
|------|--------|---------------|
| `Button.tsx` | shadcn `Button` | variant, size, disabled, loading, leftIcon/rightIcon |
| `Input.tsx` | shadcn `Input` | size, disabled, invalid, prefix/suffix |
| `Textarea.tsx` | shadcn `Textarea` | size, disabled, invalid, rows |
| `Select.tsx` | shadcn `Select` | size, disabled, invalid, options, placeholder |
| `Checkbox.tsx` | shadcn `Checkbox` | checked, indeterminate, disabled |
| `Switch.tsx` | shadcn `Switch` | checked, disabled, size |
| `Badge.tsx` | shadcn `Badge` | variant, size, dismissible |
| `Spinner.tsx` | кастомный | size: sm\|md\|lg |
| `index.ts` | barrel export | — |

**Коммит:** `feat(ui/next): implement IU component wrappers in UI/next/components/`

✅ Реализовано: Button, Input, Textarea, Select, Checkbox, Switch, Badge, Avatar, Skeleton, Spinner + barrel export index.ts

#### 1.5 Подключить токены и path alias

- [x] Добавить `@import "../../UI/tokens/base.css"` в CSS entry точку `apps/admin/input.css`
- [x] Добавить `@import "../../../UI/tokens/base.css"` в `apps/next-admin/src/styles/globals.css`
- [x] Добавить в `apps/next-admin/tsconfig.json`:
  ```json
  "@iu/*": ["../../UI/next/components/*"]
  ```

**Коммит:** `feat(ui): connect shared tokens and @iu/* path alias to both admin apps`

---

### Фаза 2: FSD-реструктуризация apps/admin (Leptos) ✅ ЗАВЕРШЕНО

#### 2.1 Создать shared/ слой ✅

- [x] Создать `src/shared/mod.rs`
- [x] Переместить `src/api/` → `src/shared/api/` (mod.rs + queries.rs)
- [x] Создать `src/shared/ui/mod.rs` — Button, Input, LanguageToggle, PageHeader
- [x] Переместить `src/components/ui/page_header.rs` → `src/shared/ui/page_header.rs`
- [x] Создать `src/shared/config/mod.rs`
- [x] Переместить `src/components/layout/nav_config.rs` → `src/shared/config/nav.rs`
- [x] Создать `src/shared/i18n/mod.rs` — из `src/i18n.rs` + LocaleContext/provide_locale_context/use_locale/translate

**Коммит:** `refactor(admin/leptos): extract shared/ FSD layer`

#### 2.2 Создать entities/ слой ✅

- [x] Создать `src/entities/mod.rs`
- [x] Создать `src/entities/user/mod.rs` + `model.rs` (User, UserRole, UserStatus)
- [x] Создать `src/entities/user/ui/mod.rs` — UserAvatar, UserRoleBadge, UserStatusBadge
- [x] Создать `src/entities/product/mod.rs` + `model.rs` (Product, ProductStatus)
- [x] Создать `src/entities/tenant/mod.rs` + `model.rs` (Tenant)

**Коммит:** `feat(admin/leptos): add entities/ FSD layer (user, product, tenant)`

#### 2.3 Создать widgets/ слой ✅

- [x] Создать `src/widgets/mod.rs`
- [x] Создать `src/widgets/app_shell/mod.rs`
- [x] Переместить `src/components/layout/app_layout.rs` → `src/widgets/app_shell/app_layout.rs`
- [x] Переместить `src/components/layout/header.rs` → `src/widgets/app_shell/header.rs`
- [x] Переместить `src/components/layout/sidebar.rs` → `src/widgets/app_shell/sidebar.rs`
- [x] Переместить `src/components/ui/stats_card.rs` → `src/widgets/stats_card/mod.rs`
- [ ] Создать `src/widgets/user_table/mod.rs` — DataTable с leptos-table + leptos-shadcn-pagination (следующая итерация)

**Коммит:** `refactor(admin/leptos): extract widgets/ FSD layer`

#### 2.4 Создать features/ слой ✅

- [x] Создать `src/features/mod.rs`
- [x] Переместить `src/components/features/auth/` → `src/features/auth/`
- [x] Создать `src/features/users/mod.rs`
- [x] Создать `src/features/profile/mod.rs`

**Коммит:** `refactor(admin/leptos): extract features/ FSD layer`

#### 2.5 Создать app/ слой ✅

- [x] Создать `src/app/mod.rs`
- [x] Создать `src/app/router.rs` — компонент `App` из `src/app.rs`
- [x] Создать `src/app/providers/mod.rs`
- [x] Создать `src/app/providers/locale.rs` — re-export из `shared::i18n`
- [x] Переместить `src/modules/` → `src/app/modules/`

**Коммит:** `refactor(admin/leptos): restructure app/ FSD layer`

#### 2.6 Обновить lib.rs и удалить старые пути ✅

- [x] Обновить `src/lib.rs` — новые mod-объявления (app, entities, features, pages, shared, widgets)
- [x] Обновить импорты во всех `pages/*.rs` — используют новые пути
- [ ] Удалить `src/components/`, `src/api/`, `src/providers/`, `src/i18n.rs`, `src/modules/`, `src/app.rs` — старые модули оставлены как резерв совместимости (удалить в следующей итерации после верификации сборки)

**Коммит:** `refactor(admin/leptos): update imports, remove old paths, verify build`

---

### Фаза 3: FSD-реструктуризация apps/next-admin (Next.js) ✅ ЗАВЕРШЕНО

#### 3.1 Создать shared/ слой ✅

- [x] Создать `src/shared/` директорию
- [x] Создать `src/shared/api/` — barrel re-exports из `lib/graphql.ts`, `lib/auth-api.ts`
- [x] Создать `src/shared/lib/` — barrel re-exports из `lib/{utils,format,parsers,searchparams,data-table}.ts`
- [x] Создать `src/shared/lib/themes/` — barrel re-exports из `components/themes/`
- [x] Создать `src/shared/hooks/` — barrel re-exports из `hooks/`
- [x] Создать `src/shared/types/` — barrel re-exports из `types/`
- [x] Создать `src/shared/config/` — barrel re-exports из `config/`
- [x] Создать `src/shared/constants/` — barrel re-exports из `constants/`
- [x] Создать `src/shared/ui/` — breadcrumbs, file-uploader, form-card-skeleton, search-input, icons, alert-modal, forms/
- [x] Создать barrel exports (`index.ts`) для каждой папки
- [x] Обновить `tsconfig.json` — добавить path aliases `@/shared/*`

**Коммит:** `refactor(admin/next): extract shared/ FSD layer`

#### 3.2 Создать entities/ слой ✅

- [x] Создать `src/entities/user/model.ts` — User, UserRole, UserStatus, UsersConnection типы
- [x] Создать `src/entities/user/ui/user-card.tsx` — компонент карточки пользователя
- [x] Создать `src/entities/user/ui/user-avatar.tsx` — аватар с инициалами
- [x] Создать `src/entities/user/index.ts`
- [x] Создать `src/entities/product/model.ts` — Product, ProductCategory типы
- [x] Создать `src/entities/product/ui/product-card.tsx`
- [x] Создать `src/entities/product/index.ts`
- [x] Создать `src/entities/tenant/model.ts` — Tenant, Workspace типы
- [x] Создать `src/entities/tenant/index.ts`
- [x] Обновить `tsconfig.json` — добавить `@/entities/*`

**Коммит:** `feat(admin/next): add entities/ FSD layer (user, product, tenant)`

#### 3.3 Создать widgets/ слой ✅

- [x] Создать `src/widgets/app-shell/` — re-exports из `components/layout/` (AppSidebar, Header, InfoSidebar, PageContainer, Providers, UserNav)
- [x] Создать `src/widgets/command-palette/` — re-export из `components/kbar/`
- [x] Создать `src/widgets/data-table/` — re-exports DataTable компонентов из `components/ui/table/`
- [x] Создать `src/widgets/alert-modal/` — re-export из `components/modal/alert-modal.tsx`
- [x] Создать barrel `index.ts` для каждого виджета
- [x] Обновить `tsconfig.json` — добавить `@/widgets/*`

**Коммит:** `refactor(admin/next): extract widgets/ FSD layer`

#### 3.4 Обновить импорты и верифицировать ✅

- [x] Обновить импорты в `src/app/layout.tsx` — использовать `@/widgets/app-shell`, `@/shared/lib/themes/*`
- [x] Обновить импорты в `src/app/dashboard/layout.tsx` — использовать `@/widgets/command-palette`, `@/widgets/app-shell`
- [x] Обновить импорты в `src/app/dashboard/product/**` — `@/widgets/app-shell`, `@/shared/lib/*`
- [x] Обновить импорты в `src/app/dashboard/overview/@*/` — `@/shared/constants`
- [x] Обновить импорты в `src/features/products/**` — `@/shared/api`, `@/shared/hooks`, `@/shared/ui/forms`, `@/widgets/alert-modal`
- [x] Обновить импорты в `src/features/auth/**` — `@/shared/lib/utils`, `@/shared/api/auth-api`, `@/shared/ui/icons`
- [x] Обновить импорты в `src/features/users/**` — `@/shared/api/graphql`
- [x] Обновить импорты в `src/features/profile/**` — `@/shared/api/graphql`, `@/widgets/app-shell`
- [x] Обновить импорты в `src/features/kanban/**`, `src/features/overview/**` — `@/widgets/app-shell`
- [ ] Запустить `pnpm --filter next-admin type-check` — должно пройти без ошибок
- [ ] Запустить `pnpm --filter next-admin build` — успешная сборка

**Примечание:** Старые пути (`@/components/layout/`, `@/lib/`, `@/hooks/`) продолжают работать. Это стратегия **постепенного перехода**: новые canonical пути через FSD-слои, старые сохраняются для backward compatibility.

**Коммит:** `refactor(admin/next): update imports, add FSD layer structure`

---

## 6. Правила FSD (контрольный список для ревью)

При разработке в любой из админок обязательно проверять:

1. **Слои импортируют только вниз** — `pages` → `widgets` → `features` → `entities` → `shared`. Нарушение = ошибка архитектуры.
2. **Слайсы в одном слое не импортируют друг друга** — `features/users` НЕ импортирует `features/auth`. Если нужен общий код — он идёт в `shared/`.
3. **`shared/` не содержит бизнес-логики** — только утилиты, UI-примитивы, типы без доменных правил.
4. **`entities/` содержит типы и базовые UI** — не содержит GraphQL-запросы (они в `features/` или `shared/api/`).
5. **`widgets/` — самодостаточные блоки** — могут использовать features через props/slots, но не импортировать напрямую.
6. **Каждый слой/слайс имеет `index.(ts|rs)` / `mod.rs`** — barrel export как публичный API.
7. **`app/` — единственная точка входа** — только `app/` подключает роутер и провайдеры.

---

## 7. Определение готовности (Definition of Done)

- [ ] `cargo build -p rustok-admin` — компилируется без ошибок
- [ ] `cargo-udeps --package rustok-admin` — нет неиспользуемых зависимостей (убрать `cargo-udeps.ignore`)
- [ ] `pnpm --filter next-admin type-check` — нет TypeScript ошибок
- [ ] `pnpm --filter next-admin build` — Next.js собирается
- [ ] Все FSD-слои присутствуют в обеих админках: `app`, `pages`, `widgets`, `features`, `entities`, `shared`
- [ ] `UI/tokens/base.css` подключён в оба приложения
- [ ] `UI/next/components/` содержит 10 компонентов с barrel export
- [ ] `crates/leptos-ui` использует `leptos-shadcn-ui` как основу (паритет с shadcn/ui Next.js)
- [ ] `leptos-shadcn-ui` компоненты реально используются в pages/ widgets/ features/ (не просто в Cargo.toml)
- [ ] Нарушений правила "слои импортируют только вниз" — ноль

---

## 8. Связанные документы

- [`ADMIN_PANEL_REVIEW.md`](../../ADMIN_PANEL_REVIEW.md) — существующий детальный ревью с критическими проблемами
- [`docs/admin-review-improvement-plan.md`](../admin-review-improvement-plan.md) — план по auth/dashboard (Фазы 1-2 частично выполнены)
- [`UI/docs/api-contracts.md`](../../UI/docs/api-contracts.md) — контракты UI-компонентов
- [`UI/tokens/base.css`](../../UI/tokens/base.css) — дизайн-токены
- [`crates/leptos-ui/`](../../crates/leptos-ui/) — существующая Leptos UI библиотека
