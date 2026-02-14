# RusToK Custom Libraries Status

**Дата:** 2026-02-13  
**Цель:** Трекинг реализации самописных библиотек для Leptos UI

---

## 📊 Overview

RusToK использует **модульную архитектуру** с самописными библиотеками в `crates/leptos-*` для переиспользования в Leptos-приложениях (`apps/admin`, `apps/storefront`).

**Принцип:** Вся функциональность идёт через модули в `crates/*`, а не ad-hoc код в `apps/*`.

---

## 🔧 Custom Libraries Status

### ✅ Реализовано

| Crate | Назначение | Статус | Используется в |
|-------|-----------|--------|----------------|
| `leptos-graphql` | HTTP transport для GraphQL запросов | ✅ **Готово** | `leptos-auth`, `apps/admin` |
| `leptos-auth` | Auth context, hooks, components, LocalStorage | ✅ **Готово** | `apps/admin`, `apps/storefront` |

---

### 🚧 В разработке

| Crate | Назначение | Статус | Блокирует | Приоритет |
|-------|-----------|--------|-----------|-----------|
| `leptos-table` | Data tables с pagination, sorting, filtering | 🚧 **WIP** | Phase 2: Users list | **P1** |

---

### ✅ Недавно завершено (Phase 1)

| Crate | Назначение | Дата завершения | Используется в |
|-------|-----------|-----------------|----------------|
| `leptos-forms` | Form handling, validation, hooks | 2026-02-14 | Phase 1: Auth forms, Phase 2: CRUD forms |
| `leptos-ui` | DSD-style UI components (Phase 1: Button, Input, Label, Card, Badge, Separator) | 2026-02-14 | All phases |

---

### ⏳ Запланировано

| Crate | Назначение | Блокирует | Приоритет |
|-------|-----------|-----------|-----------|
| `leptos-i18n` | Интернационализация (i18n) | Phase 3: Multi-language | **P2** |
| `leptos-routing` | Extended routing helpers | Phase 2: Complex routes | **P2** |
| `leptos-toast` | Toast notifications | Phase 2: User feedback | **P1** |
| `leptos-modal` | Modal dialogs | Phase 2: Modals | **P1** |
| `leptos-charts` | Charting библиотека | Phase 4: Analytics | **P3** |
| `leptos-file-upload` | File upload с progress | Phase 3: Media management | **P2** |

---

## 📝 Детали по библиотекам

### ✅ `leptos-graphql` (Transport Layer)

**Статус:** ✅ Реализовано  
**Версия:** `0.1.0`  
**Файл:** `crates/leptos-graphql/src/lib.rs`

**Что делает:**
- HTTP client для GraphQL запросов (`POST /api/graphql`)
- Формирует стандартный GraphQL request shape (`query`, `variables`, `extensions`)
- Поддержка persisted queries (`sha256Hash`)
- Автоматическая вставка заголовков `Authorization`, `X-Tenant-Slug`
- Error mapping: `Network`, `Graphql`, `Http`, `Unauthorized`

**API:**
```rust
use leptos_graphql::{execute, GraphqlRequest};

let request = GraphqlRequest::new(query, variables);
let response: MyData = execute(endpoint, request, token, tenant).await?;
```

**Используется в:**
- `leptos-auth` (для sign_in, sign_up, sign_out)
- `apps/admin` (для всех GraphQL запросов)

**Зависимости:**
- `reqwest` — HTTP transport
- `serde`, `serde_json` — serialization
- `thiserror` — error handling

---

### ✅ `leptos-auth` (Authentication)

**Статус:** ✅ Реализовано  
**Версия:** `0.1.0`  
**Файл:** `crates/leptos-auth/src/lib.rs`

**Что делает:**
- `AuthProvider` — context provider для auth state
- `AuthContext` — reactive auth state (user, token, tenant)
- Hooks: `use_auth()`, `use_token()`, `use_current_user()`, `use_is_authenticated()`
- Components: `<ProtectedRoute>`, `<GuestRoute>`, `<RequireAuth>`
- API functions: `api::sign_in()`, `api::sign_up()`, `api::sign_out()`, `api::get_current_user()`
- LocalStorage helpers: `storage::save_session()`, `storage::load_session()`
- Error types: `AuthError` (Unauthorized, InvalidCredentials, Network, Http)

**API:**
```rust
// In app.rs
view! {
    <AuthProvider>
        <Router>
            <Route path="/login" view=Login />
            <ProtectedRoute path="/dashboard" view=Dashboard />
        </Router>
    </AuthProvider>
}

// In components
use leptos_auth::{use_auth, use_is_authenticated};

let auth = use_auth();
let is_authenticated = use_is_authenticated();

// Login
let (user, session) = api::sign_in(email, password, tenant).await?;
```

**Используется в:**
- `apps/admin` (authentication flow)
- `apps/storefront` (customer auth)

**Зависимости:**
- `leptos-graphql` — для GraphQL запросов
- `leptos`, `leptos_router` — для components/hooks
- `gloo-storage` — LocalStorage API
- `serde`, `serde_json` — serialization
- `thiserror` — error handling

**Архитектура:**
```
apps/admin
    ↓ use_auth(), api::sign_in()
leptos-auth
    ↓ leptos_graphql::execute()
leptos-graphql
    ↓ POST /api/graphql
apps/server (GraphQL backend)
```

---

### ✅ `leptos-forms` (Form Handling)

**Статус:** ✅ Реализовано (Core functionality)  
**Версия:** `0.1.0`  
**Дата завершения:** 2026-02-14  
**Файл:** `crates/leptos-forms/src/lib.rs`

**Что должно делать:**
- Form state management (поля, значения, изменения)
- Validation rules (required, email, min_length, custom)
- Error display (per-field, form-level)
- Submit handling (loading, error states)
- Reactive validation (on blur, on change, on submit)

**Desired API:**
```rust
use leptos_forms::{use_form, Field, Validator};

#[component]
fn LoginForm() -> impl IntoView {
    let form = use_form(|| LoginData::default())
        .field("email", Validator::email().required())
        .field("password", Validator::min_length(6).required())
        .on_submit(|data| async move {
            api::sign_in(data.email, data.password, tenant).await
        });

    view! {
        <form on:submit=form.submit>
            <Field form=form name="email" label="Email" />
            <Field form=form name="password" label="Password" type="password" />
            <button disabled=form.is_submitting>
                {move || if form.is_submitting() { "Loading..." } else { "Login" }}
            </button>
        </form>
    }
}
```

**Реализовано:**
- ✅ FormContext — form state management
- ✅ use_form() hook
- ✅ Field component — input with error display
- ✅ Валидаторы (required, email, min_length, max_length, pattern, custom)
- ✅ Error handling (per-field errors, form-level errors)
- ✅ Reactive validation (on blur)
- ✅ Integration с Leptos signals
- ✅ Документация + README

**TODO (Phase 2):**
- [ ] Submit handling integration (async submit)
- [ ] Advanced validation (conditional fields, cross-field validation)
- [ ] File upload support
- [ ] Array fields support
- [ ] Examples в apps/admin/examples/

**Зависимости:**
- `leptos` — reactive primitives
- `serde` — serialization
- `validator` (опционально) — для validation rules
- `thiserror` — error handling

**References:**
- React Hook Form (inspiration)
- Formik (validation patterns)
- `leptos_form` (existing, но устаревший)

---

### 🚧 `leptos-table` (Data Tables)

**Статус:** 🚧 В разработке  
**Блокирует:** Phase 2 (Users list, Posts list)  
**Приоритет:** **P1**

**Что должно делать:**
- Server-side pagination (offset/limit или cursor)
- Server-side sorting (по колонкам)
- Server-side filtering (поиск, фильтры)
- Column configuration (label, width, sortable, render)
- Row selection (single, multiple)
- Actions (edit, delete, bulk actions)
- Loading states, empty states

**Desired API:**
```rust
use leptos_table::{Table, Column, use_table};

#[component]
fn UsersTable() -> impl IntoView {
    let table = use_table::<User>()
        .query(fetch_users)  // async fn
        .pagination(10)
        .sortable(true);

    view! {
        <Table table=table>
            <Column field="email" label="Email" sortable=true />
            <Column field="name" label="Name" sortable=true />
            <Column field="role" label="Role" />
            <Column render=|user| view! {
                <button on:click=move |_| edit_user(user.id)>"Edit"</button>
            } />
        </Table>
    }
}
```

**TODO:**
- [ ] Базовая Table component
- [ ] Pagination logic (offset/limit)
- [ ] Sorting logic (field, direction)
- [ ] Filtering logic (search query)
- [ ] Column configuration
- [ ] Row selection
- [ ] Loading/empty states
- [ ] GraphQL integration
- [ ] Документация + примеры

**Зависимости:**
- `leptos` — reactive primitives
- `leptos-graphql` — для GraphQL запросов
- `serde` — serialization

**References:**
- TanStack Table (v8)
- AG Grid (patterns)
- `leptos-shadcn-pagination` (existing, для pagination UI)

---

### ✅ `leptos-ui` (UI Components)

**Статус:** ✅ Реализовано (Phase 1 components)  
**Версия:** `0.1.0`  
**Дата завершения:** 2026-02-14  
**Файл:** `crates/leptos-ui/src/lib.rs`

**Что должно делать:**
- DSD-style UI components (shadcn подход)
- Variants-based API (размеры, цвета)
- Tailwind-based styling
- Accessibility (ARIA, keyboard navigation)
- Copy-paste friendly (не npm install)

**Structure:**
```
crates/leptos-ui/src/
├── lib.rs            — Re-exports
├── types.rs          — Shared types (Size, Variant)
├── button.rs         — ✅ Button с вариантами
├── input.rs          — ✅ Input
├── label.rs          — ✅ Label
├── card.rs           — ✅ Card, CardHeader, CardContent, CardFooter
├── badge.rs          — ✅ Badge
├── separator.rs      — ✅ Separator
└── (Phase 2+)
    ├── alert.rs          — ⏳ Alert
    ├── dropdown.rs       — ⏳ Dropdown menu
    ├── dialog.rs         — ⏳ Modal dialog
    ├── tabs.rs           — ⏳ Tabs
    ├── table.rs          — ⏳ Table primitives
    ├── textarea.rs       — ⏳ Textarea
    ├── checkbox.rs       — ⏳ Checkbox
    └── select.rs         — ⏳ Select/Combobox
```

**Example API:**
```rust
use leptos_ui::{Button, ButtonVariant, ButtonSize};

view! {
    <Button variant=ButtonVariant::Primary size=ButtonSize::Lg>
        "Click me"
    </Button>
    
    <Button variant=ButtonVariant::Outline on:click=move |_| { /* ... */ }>
        "Outline"
    </Button>
}
```

**TODO:**
- [ ] Button (primary, secondary, outline, ghost, destructive)
- [ ] Input (text, email, password, number)
- [ ] Label
- [ ] Card (Card, CardHeader, CardContent, CardFooter)
- [ ] Badge
- [ ] Separator
- [ ] Alert (info, success, warning, error)
- [ ] Dropdown menu
- [ ] Dialog (modal)
- [ ] Tabs
- [ ] Table primitives (Table, TableHeader, TableRow, TableCell)
- [ ] Skeleton (loading state)
- [ ] Checkbox, Radio
- [ ] Switch
- [ ] Textarea
- [ ] Select/Combobox
- [ ] Документация + Storybook (или leptos-book)

**Зависимости:**
- `leptos` — reactive primitives
- Tailwind CSS — styling (не зависимость, но требуется)

**References:**
- shadcn/ui (React, inspiration)
- `leptos-shadcn-pagination` (existing, pattern reference)
- Radix UI (accessibility patterns)

**Design Principles:**
- **Copy-paste friendly** — не npm install, копируем файлы
- **Variants over composition** — `variant=Primary` вместо множества props
- **Tailwind-first** — классы inline или через `cn()` helper
- **Accessibility** — ARIA labels, keyboard navigation
- **Flat structure** — `leptos_ui::Button` вместо `leptos_ui::button::Button`

---

### ⏳ `leptos-toast` (Notifications)

**Статус:** ⏳ Запланировано  
**Блокирует:** Phase 2 (User feedback)  
**Приоритет:** **P1**

**Что должно делать:**
- Toast notifications (success, error, info, warning)
- Auto-dismiss с таймером
- Manual dismiss
- Queue management (multiple toasts)
- Позиционирование (top-right, top-left, bottom-right, etc.)

**Desired API:**
```rust
use leptos_toast::{use_toast, ToastVariant};

let toast = use_toast();

// Success
toast.success("User created successfully");

// Error
toast.error("Failed to save changes");

// Custom duration
toast.info("Processing...", 5000);
```

**TODO:**
- [ ] Toast component
- [ ] Toast provider/context
- [ ] Queue management
- [ ] Auto-dismiss logic
- [ ] Positioning
- [ ] Animations (slide in/out)
- [ ] Accessibility
- [ ] Документация

**Зависимости:**
- `leptos` — reactive primitives
- `leptos-ui` — Button, Icon

**References:**
- `react-hot-toast`
- `sonner` (Toast library)

---

### ⏳ `leptos-modal` (Modals)

**Статус:** ⏳ Запланировано  
**Блокирует:** Phase 2 (Modals для edit/delete)  
**Приоритий:** **P1**

**Что должно делать:**
- Modal dialogs
- Backdrop с click-outside закрытием
- ESC key для закрытия
- Focus trap
- Scroll lock
- Размеры (sm, md, lg, xl, full)

**Desired API:**
```rust
use leptos_modal::{Modal, use_modal};

let modal = use_modal();

view! {
    <button on:click=move |_| modal.open()>"Open Modal"</button>
    
    <Modal open=modal.is_open on:close=modal.close>
        <h2>"Delete User"</h2>
        <p>"Are you sure?"</p>
        <button on:click=modal.close>"Cancel"</button>
        <button on:click=move |_| { delete_user(); modal.close(); }>"Delete"</button>
    </Modal>
}
```

**TODO:**
- [ ] Modal component
- [ ] Modal context/hooks
- [ ] Backdrop
- [ ] Click-outside handling
- [ ] ESC key handling
- [ ] Focus trap
- [ ] Scroll lock
- [ ] Animations
- [ ] Accessibility (ARIA)
- [ ] Документация

**Зависимости:**
- `leptos` — reactive primitives
- `leptos-ui` — Button, Card
- `web-sys` — DOM APIs (focus, scroll lock)

**References:**
- Radix Dialog
- Headless UI Modal

---

### ⏳ `leptos-i18n` (Internationalization)

**Статус:** ⏳ Запланировано  
**Блокирует:** Phase 3 (Multi-language support)  
**Приоритет:** **P2**

**Что должно делать:**
- Translation files (JSON, YAML)
- Runtime locale switching
- Pluralization
- Date/number formatting
- SSR support (для storefront)

**Desired API:**
```rust
use leptos_i18n::{I18nProvider, use_i18n, t};

view! {
    <I18nProvider locale="en">
        <App />
    </I18nProvider>
}

// In components
let i18n = use_i18n();
let greeting = t!("greeting", name = "John");  // "Hello, John!"

// Or
view! {
    <p>{t!("greeting")}</p>
}
```

**TODO:**
- [ ] Translation file loader
- [ ] Locale context
- [ ] `t!()` macro
- [ ] Pluralization
- [ ] Date/number formatting
- [ ] Locale switching
- [ ] SSR support
- [ ] Документация

**Зависимости:**
- `leptos` — reactive primitives
- `serde`, `serde_json` — для translation files
- `fluent` (опционально) — для complex i18n

**References:**
- `react-i18next`
- `fluent-rs`
- `leptos-fluent` (existing, если есть)

---

### ⏳ `leptos-routing` (Extended Routing)

**Статус:** ⏳ Запланировано  
**Блокирует:** Phase 2 (Complex routes, breadcrumbs)  
**Приоритет:** **P2**

**Что должно делать:**
- Breadcrumbs generation
- Active link detection
- Route guards
- Query params helpers
- Nested routes helpers

**Desired API:**
```rust
use leptos_routing::{use_breadcrumbs, Breadcrumbs};

let breadcrumbs = use_breadcrumbs();

view! {
    <Breadcrumbs>
        {breadcrumbs().iter().map(|crumb| view! {
            <a href=crumb.path>{&crumb.label}</a>
        }).collect_view()}
    </Breadcrumbs>
}
```

**TODO:**
- [ ] Breadcrumbs logic
- [ ] Active link detection
- [ ] Route guards
- [ ] Query params helpers
- [ ] Документация

**Зависимости:**
- `leptos_router` — базовый routing
- `leptos` — reactive primitives

**References:**
- React Router (breadcrumbs)
- Next.js routing patterns

---

### ⏳ `leptos-charts` (Charting)

**Статус:** ⏳ Запланировано  
**Блокирует:** Phase 4 (Analytics dashboard)  
**Приоритет:** **P3**

**Что должно делать:**
- Line charts
- Bar charts
- Pie charts
- Area charts
- Responsive
- Animations

**Desired API:**
```rust
use leptos_charts::{LineChart, ChartData};

let data = ChartData::new()
    .add_series("Sales", vec![10, 20, 30, 40])
    .add_labels(vec!["Jan", "Feb", "Mar", "Apr"]);

view! {
    <LineChart data=data />
}
```

**TODO:**
- [ ] Chart components (Line, Bar, Pie, Area)
- [ ] Data structures
- [ ] SVG rendering
- [ ] Animations
- [ ] Responsive
- [ ] Accessibility
- [ ] Документация

**Зависимости:**
- `leptos` — reactive primitives
- `web-sys` — для SVG rendering
- `plotters` (опционально) — для chart generation

**References:**
- Chart.js
- Recharts
- D3.js patterns

---

### ⏳ `leptos-file-upload` (File Upload)

**Статус:** ⏳ Запланировано  
**Блокирует:** Phase 3 (Media management)  
**Приоритет:** **P2**

**Что должно делать:**
- File picker
- Drag & drop
- Progress bar
- Preview (images)
- Multiple files
- Size/type validation

**Desired API:**
```rust
use leptos_file_upload::{FileUpload, use_file_upload};

let upload = use_file_upload()
    .max_size(5 * 1024 * 1024)  // 5MB
    .accept("image/*")
    .on_complete(|files| { /* ... */ });

view! {
    <FileUpload upload=upload>
        <p>"Drag files here or click to upload"</p>
    </FileUpload>
    
    <Show when=move || upload.is_uploading()>
        <progress value=upload.progress()></progress>
    </Show>
}
```

**TODO:**
- [ ] FileUpload component
- [ ] Drag & drop logic
- [ ] Progress tracking
- [ ] Preview rendering
- [ ] Validation
- [ ] Upload to server
- [ ] Документация

**Зависимости:**
- `leptos` — reactive primitives
- `web-sys` — File API
- `gloo-file` — file handling

**References:**
- `react-dropzone`
- Uppy (file uploader)

---

## 🔀 Интеграция с существующими библиотеками

### `leptos-shadcn-pagination`

**Статус:** ✅ Уже используется  
**Назначение:** Pagination UI component

**Интеграция:**
- Используется в `leptos-table` для pagination UI
- Стилизация совместима с `leptos-ui`

---

## 🎯 Development Priorities

### Phase 0 (Setup) — **Текущая фаза**
- ✅ `leptos-graphql` (готово)
- ✅ `leptos-auth` (готово)

### Phase 1 (Auth + Navigation) — **Следующая**
- 🚧 `leptos-forms` (критично для login/register)
- 🚧 `leptos-ui` (Button, Input, Card, Label)

### Phase 2 (CRUD Operations)
- 🚧 `leptos-table` (для Users, Posts lists)
- 🚧 `leptos-ui` (Table, Badge, Dropdown, Dialog)
- ⏳ `leptos-toast` (user feedback)
- ⏳ `leptos-modal` (edit/delete dialogs)

### Phase 3 (Advanced Features)
- ⏳ `leptos-i18n` (multi-language)
- ⏳ `leptos-file-upload` (media management)
- ⏳ `leptos-routing` (breadcrumbs)

### Phase 4 (Analytics & Polish)
- ⏳ `leptos-charts` (dashboard charts)

---

## 📋 Contribution Guidelines

### Как добавить новую библиотеку

1. **Создать crate:**
   ```bash
   cargo new --lib crates/leptos-<name>
   ```

2. **Добавить в workspace:**
   ```toml
   # Cargo.toml (root)
   [workspace]
   members = [
       "crates/leptos-<name>",
       # ...
   ]
   ```

3. **Создать README:**
   ```markdown
   # leptos-<name>
   
   ## Назначение
   
   ## Взаимодействие
   
   ## API
   
   ## Examples
   ```

4. **Обновить этот файл:**
   - Добавить в раздел "🚧 В разработке" или "⏳ Запланировано"
   - Описать API, TODO, dependencies

5. **Создать tracking issue:**
   ```markdown
   Title: [leptos-<name>] Implementation
   Labels: enhancement, library
   ```

---

## 🔗 Resources

### Документация
- `/docs/UI/README.md` — общая документация UI
- `/docs/UI/GRAPHQL_ARCHITECTURE.md` — GraphQL architecture
- `/crates/leptos-*/README.md` — документация по библиотекам

### Примеры
- `apps/admin` — использование библиотек
- `apps/storefront` — использование библиотек

### References
- [Leptos Book](https://leptos-rs.github.io/leptos/)
- [shadcn/ui](https://ui.shadcn.com/) — design system inspiration
- [TanStack Table](https://tanstack.com/table) — table patterns
- [React Hook Form](https://react-hook-form.com/) — form patterns

---

**Last updated:** 2026-02-13  
**Maintainer:** CTO Agent

> 💡 **Tip:** Держите этот файл актуальным при добавлении/изменении библиотек!
