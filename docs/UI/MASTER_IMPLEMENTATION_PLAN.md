# RusToK UI Master Implementation Plan

**Дата:** 2026-02-13  
**Статус:** 🚀 В работе (Phase 0 → Phase 1)  
**Ответственный:** CTO Agent

> 🚨 **ПЕРЕД НАЧАЛОМ РАБОТЫ ОБЯЗАТЕЛЬНО ПРОЧИТАЙТЕ:**
> 
> - [`docs/UI/CRITICAL_WARNINGS.md`](./CRITICAL_WARNINGS.md) — критичные предупреждения
> - [`docs/UI/CUSTOM_LIBRARIES_STATUS.md`](./CUSTOM_LIBRARIES_STATUS.md) — статус самописных библиотек
> - [`docs/UI/DESIGN_SYSTEM_DECISION.md`](./DESIGN_SYSTEM_DECISION.md) — выбор DSD подхода

---

## 📋 Обзор

**Цель:** Реализовать параллельную разработку двух админок с единым backend API

### Архитектура

```
┌─────────────────────────────────────────────────┐
│         apps/next-admin (Next.js)               │
│  ┌───────────────────────────────────────────┐  │
│  │ React components + Next.js App Router     │  │
│  │ GraphQL client (urql/Apollo)              │  │
│  │ TailwindCSS + shadcn/ui                   │  │
│  └───────────────────────────────────────────┘  │
└─────────────────┬───────────────────────────────┘
                  │
                  │ HTTP + GraphQL
                  ▼
┌─────────────────────────────────────────────────┐
│         apps/server (Loco Backend)               │
│  ┌───────────────────────────────────────────┐  │
│  │ GraphQL API (/api/graphql)                │  │
│  │ REST endpoints (/api/auth/*)              │  │
│  │ PostgreSQL + SeaORM                       │  │
│  │ JWT Auth + RBAC + Multi-tenant            │  │
│  └───────────────────────────────────────────┘  │
└─────────────────┬───────────────────────────────┘
                  │
                  │ HTTP + GraphQL
                  ▼
┌─────────────────────────────────────────────────┐
│         apps/admin (Leptos)                      │
│  ┌───────────────────────────────────────────┐  │
│  │ Leptos components (CSR/WASM)              │  │
│  │ Custom libs (leptos-auth, leptos-ui)     │  │
│  │ TailwindCSS + DSD components              │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

---

## 🎯 Ключевые принципы

1. **Параллельная разработка** — Next и Leptos развиваются одновременно (разрыв макс 1 sprint)
2. **Module-first** — вся функциональность через `crates/*`, не ad-hoc в `apps/*`
3. **DSD Design System** — shadcn подход (copy-paste, variants, Tailwind)
4. **GraphQL-first** — все данные через GraphQL API, минимум REST
5. **Zero-config запуск** — `make dev-start` поднимает все сервисы
6. **НЕ УДАЛЯЕМ БИБЛИОТЕКИ** — чиним, не удаляем (см. CRITICAL_WARNINGS.md)
7. **Самописные библиотеки** — используем `leptos-*` из `crates/` (см. CUSTOM_LIBRARIES_STATUS.md)

---

## 📦 Фазы реализации

### ✅ Фаза 0: Подготовка окружения (ЗАВЕРШЕНА)

**Статус:** ✅ **Завершено**  
**Длительность:** 1-2 дня  
**Дата завершения:** 2026-02-13

#### Выполнено:

- ✅ **0.1. Docker Compose full-dev профиль**
  - Создан `docker-compose.full-dev.yml`
  - Порты: server (5150), next-admin (3000), leptos-admin (3001), storefronts (3100, 3101)
  - Healthchecks для всех сервисов
  - Зависимости настроены

- ✅ **0.2. Environment configuration**
  - `.env.dev.example` в корне
  - Seed данные: `admin@local` / `admin12345`, tenant `demo`
  - Runtime config для обоих UI

- ✅ **0.3. Makefile targets**
  - `make dev-start` — запуск всех сервисов
  - `make dev-stop` — остановка
  - `make dev-logs` — логи
  - `make help` — справка

- ✅ **0.4. Dockerfiles**
  - `apps/server/Dockerfile` (dev + prod stages)
  - `apps/admin/Dockerfile` (Leptos WASM)
  - `apps/next-admin/Dockerfile` (Next.js)

- ✅ **0.5. Custom Libraries (базовые)**
  - ✅ `leptos-graphql` — HTTP transport для GraphQL
  - ✅ `leptos-auth` — Auth context, hooks, components, API

- ✅ **0.6. Documentation**
  - ✅ `QUICKSTART.md` — быстрый старт
  - ✅ `docs/UI/CUSTOM_LIBRARIES_STATUS.md` — статус библиотек
  - ✅ `docs/UI/GRAPHQL_ARCHITECTURE.md` — GraphQL архитектура
  - ✅ `docs/UI/DESIGN_SYSTEM_DECISION.md` — выбор DSD
  - ✅ `scripts/dev-start.sh` — скрипт запуска

**Критерии завершения:** ✅ Все выполнено
- Все 5 сервисов стартуют одной командой
- Server отвечает на `/api/health`, `/api/graphql`
- Обе админки открываются в браузере
- Документация создана

---

### 🚧 Фаза 1: Auth + RBAC + Navigation (ТЕКУЩАЯ)

**Статус:** 🚧 **В работе**  
**Приоритет:** **P0** (критично)  
**Длительность:** 3-5 дней  
**Начало:** 2026-02-13

**Цель:** Базовая оболочка приложения с авторизацией, ролями и навигацией

#### Задачи:

##### 1.1. Backend GraphQL Schema

**Статус:** ⏳ TODO

- [ ] **1.1.1. Auth mutations** (`apps/server/src/graphql/schema.rs`)
  ```graphql
  type Mutation {
    signIn(email: String!, password: String!): SignInPayload!
    signUp(email: String!, password: String!, name: String): SignUpPayload!
    signOut: Boolean!
    refreshToken: RefreshTokenPayload!
    forgotPassword(email: String!): Boolean!
    resetPassword(token: String!, newPassword: String!): Boolean!
  }
  
  type SignInPayload {
    token: String!
    user: User!
  }
  ```

- [ ] **1.1.2. Auth queries**
  ```graphql
  type Query {
    currentUser: User
    users(limit: Int, offset: Int): UserConnection!
  }
  
  type User {
    id: ID!
    email: String!
    name: String
    role: UserRole!
    createdAt: DateTime!
  }
  
  enum UserRole {
    ADMIN
    EDITOR
    VIEWER
  }
  ```

- [ ] **1.1.3. RBAC directives**
  ```graphql
  directive @requireAuth on FIELD_DEFINITION
  directive @requireRole(role: UserRole!) on FIELD_DEFINITION
  ```

- [ ] **1.1.4. Testing**
  - Unit tests для resolvers
  - Integration tests для auth flow

**Блокирует:** 1.2, 1.3

---

##### 1.2. Custom Libraries (Phase 1)

**Статус:** 🚧 В работе

###### 1.2.1. `leptos-forms` (Form Handling)

**Статус:** ⏳ TODO  
**Приоритет:** P0  
**Блокирует:** Login, Register forms

- [ ] **Core:**
  - [ ] `Form` component
  - [ ] `Field` component
  - [ ] `use_form()` hook
  - [ ] Validation logic

- [ ] **Validators:**
  - [ ] `required()`
  - [ ] `email()`
  - [ ] `min_length(n)`
  - [ ] `max_length(n)`
  - [ ] `pattern(regex)`
  - [ ] `custom(fn)`

- [ ] **Features:**
  - [ ] Per-field errors
  - [ ] Form-level errors
  - [ ] Submit handling (loading, error states)
  - [ ] Reactive validation (blur, change, submit)

- [ ] **Documentation:**
  - [ ] README.md
  - [ ] Examples (login, register, profile)
  - [ ] API reference

**Desired API:**
```rust
use leptos_forms::{use_form, Field, Validator};

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
        <button disabled=form.is_submitting>"Login"</button>
    </form>
}
```

**Зависимости:**
- `leptos`
- `serde`
- `thiserror`

**References:**
- React Hook Form
- Formik

---

###### 1.2.2. `leptos-ui` (UI Components - Phase 1)

**Статус:** 🚧 В работе  
**Приоритет:** P0  
**Блокирует:** Все UI

**Phase 1 Components:**

- [ ] **Button** (`src/button.rs`)
  - [ ] Variants: Primary, Secondary, Outline, Ghost, Destructive
  - [ ] Sizes: Sm, Md, Lg
  - [ ] Loading state
  - [ ] Disabled state
  - [ ] Icon support

- [ ] **Input** (`src/input.rs`)
  - [ ] Types: text, email, password, number
  - [ ] Error state
  - [ ] Disabled state
  - [ ] Icon support (left, right)
  - [ ] Placeholder

- [ ] **Label** (`src/label.rs`)
  - [ ] Required indicator
  - [ ] Error state

- [ ] **Card** (`src/card.rs`)
  - [ ] Card (container)
  - [ ] CardHeader
  - [ ] CardContent
  - [ ] CardFooter

- [ ] **Badge** (`src/badge.rs`)
  - [ ] Variants: Default, Primary, Success, Warning, Danger
  - [ ] Sizes: Sm, Md, Lg

- [ ] **Separator** (`src/separator.rs`)
  - [ ] Horizontal
  - [ ] Vertical

**Desired API:**
```rust
use leptos_ui::{Button, ButtonVariant, ButtonSize, Input, Label, Card};

view! {
    <Card>
        <CardHeader>
            <h2>"Login"</h2>
        </CardHeader>
        <CardContent>
            <Label>"Email"</Label>
            <Input type="email" placeholder="you@example.com" />
            
            <Button variant=ButtonVariant::Primary size=ButtonSize::Lg>
                "Sign In"
            </Button>
        </CardContent>
    </Card>
}
```

**Design Principles:**
- **DSD approach** (shadcn-style)
- **Copy-paste friendly**
- **Variants over composition**
- **Tailwind-first**
- **Accessibility** (ARIA)

**Зависимости:**
- `leptos`

**References:**
- shadcn/ui
- Radix UI (accessibility)

---

##### 1.3. Leptos Admin (Phase 1)

**Статус:** 🚧 В работе

###### 1.3.1. Auth Pages

- [ ] **Login Page** (`apps/admin/src/pages/login.rs`)
  - [ ] Form (email, password)
  - [ ] Validation (email, min_length)
  - [ ] Submit → `api::sign_in()`
  - [ ] Error handling
  - [ ] "Forgot password?" link
  - [ ] "Sign up" link

- [ ] **Register Page** (`apps/admin/src/pages/register.rs`)
  - [ ] Form (email, name, password, confirm_password)
  - [ ] Validation
  - [ ] Submit → `api::sign_up()`
  - [ ] Error handling
  - [ ] "Already have account?" link

- [ ] **Forgot Password** (`apps/admin/src/pages/forgot_password.rs`)
  - [ ] Form (email)
  - [ ] Submit → `api::forgot_password()`
  - [ ] Success message

- [ ] **Reset Password** (`apps/admin/src/pages/reset_password.rs`)
  - [ ] Form (new_password, confirm_password)
  - [ ] Token from URL params
  - [ ] Submit → `api::reset_password()`

**Uses:**
- `leptos-auth` (api functions, hooks)
- `leptos-forms` (form handling)
- `leptos-ui` (Button, Input, Card)

---

###### 1.3.2. App Shell

- [ ] **Layout** (`apps/admin/src/components/layouts/app_layout.rs`)
  - [ ] Sidebar (navigation)
  - [ ] Header (user menu, notifications)
  - [ ] Main content area
  - [ ] Footer (optional)

- [ ] **Sidebar** (`apps/admin/src/components/layouts/sidebar.rs`)
  - [ ] Navigation links
    - [ ] Dashboard
    - [ ] Users
    - [ ] Content (Posts, Pages)
    - [ ] Settings
  - [ ] Active link highlighting
  - [ ] Collapse/expand
  - [ ] Logo

- [ ] **Header** (`apps/admin/src/components/layouts/header.rs`)
  - [ ] User menu (Profile, Settings, Logout)
  - [ ] Notifications (badge count)
  - [ ] Tenant switcher (if multi-tenant)
  - [ ] Search (global, optional)

- [ ] **User Menu** (`apps/admin/src/components/features/auth/user_menu.rs`)
  - [ ] User avatar/name
  - [ ] Dropdown:
    - [ ] Profile
    - [ ] Settings
    - [ ] Logout

**Uses:**
- `leptos-auth` (use_current_user, use_auth)
- `leptos-ui` (Button, Badge, Dropdown)
- `leptos_router` (Link, use_location)

---

###### 1.3.3. Dashboard (Placeholder)

- [ ] **Dashboard Page** (`apps/admin/src/pages/dashboard.rs`)
  - [ ] Page header ("Dashboard")
  - [ ] Stats cards (placeholder data)
    - [ ] Total Users
    - [ ] Total Posts
    - [ ] Active Sessions
  - [ ] Recent activity (placeholder)

**Uses:**
- `leptos-ui` (Card, Badge)
- `leptos-graphql` (query stats)

---

##### 1.4. Next.js Admin (Phase 1)

**Статус:** ⏳ TODO

###### 1.4.1. Auth Pages

- [ ] **Login Page** (`apps/next-admin/app/(auth)/login/page.tsx`)
  - [ ] Form (react-hook-form)
  - [ ] Submit → GraphQL `signIn` mutation
  - [ ] Error handling
  - [ ] "Forgot password?" link

- [ ] **Register Page** (`apps/next-admin/app/(auth)/register/page.tsx`)
  - [ ] Form (react-hook-form)
  - [ ] Submit → GraphQL `signUp` mutation
  - [ ] Error handling

- [ ] **Forgot/Reset Password** (аналогично)

**Uses:**
- `react-hook-form` (form handling)
- `urql` or `@apollo/client` (GraphQL)
- `shadcn/ui` (Button, Input, Card)

---

###### 1.4.2. App Shell

- [ ] **Layout** (`apps/next-admin/app/(dashboard)/layout.tsx`)
  - [ ] Sidebar, Header, Main
  - [ ] Same structure as Leptos

- [ ] **Dashboard** (`apps/next-admin/app/(dashboard)/page.tsx`)
  - [ ] Placeholder stats

**Uses:**
- `shadcn/ui` components
- Next.js App Router

---

##### 1.5. Testing & QA

- [ ] **Backend:**
  - [ ] Unit tests (auth resolvers)
  - [ ] Integration tests (auth flow)

- [ ] **Leptos Admin:**
  - [ ] E2E tests (Playwright)
    - [ ] Login flow
    - [ ] Register flow
    - [ ] Logout
  - [ ] Unit tests (components)

- [ ] **Next.js Admin:**
  - [ ] E2E tests (Playwright)
    - [ ] Login flow
  - [ ] Unit tests (components)

- [ ] **Cross-browser:**
  - [ ] Chrome
  - [ ] Firefox
  - [ ] Safari

---

##### 1.6. Documentation

- [ ] **Phase 1 Summary** (`docs/UI/PHASE_1_COMPLETE.md`)
  - [ ] Что реализовано
  - [ ] Скриншоты
  - [ ] Known issues
  - [ ] Next steps

- [ ] **Update README** (`apps/admin/README.md`, `apps/next-admin/README.md`)
  - [ ] Auth flow описание
  - [ ] Скриншоты

---

#### Критерии завершения Phase 1:

- [ ] ✅ Backend GraphQL schema для auth
- [ ] ✅ `leptos-forms` реализован (базовая функциональность)
- [ ] ✅ `leptos-ui` (Button, Input, Label, Card, Badge)
- [ ] ✅ Leptos Admin: Login, Register, Dashboard работают
- [ ] ✅ Next.js Admin: Login, Register, Dashboard работают
- [ ] ✅ E2E tests проходят
- [ ] ✅ Документация обновлена

**Блокирует:** Phase 2 (CRUD Operations)

---

### ⏳ Фаза 2: CRUD Operations (Users, Posts)

**Статус:** ⏳ Запланировано  
**Приоритет:** P1  
**Длительность:** 5-7 дней

**Цель:** Реализовать базовый CRUD для Users и Posts

#### Задачи:

##### 2.1. Backend GraphQL Schema

- [ ] **Users CRUD:**
  ```graphql
  type Query {
    users(limit: Int, offset: Int, search: String): UserConnection!
    user(id: ID!): User
  }
  
  type Mutation {
    createUser(input: CreateUserInput!): User!
    updateUser(id: ID!, input: UpdateUserInput!): User!
    deleteUser(id: ID!): Boolean!
  }
  ```

- [ ] **Posts CRUD:**
  ```graphql
  type Query {
    posts(limit: Int, offset: Int, status: PostStatus): PostConnection!
    post(id: ID!): Post
  }
  
  type Mutation {
    createPost(input: CreatePostInput!): Post!
    updatePost(id: ID!, input: UpdatePostInput!): Post!
    deletePost(id: ID!): Boolean!
    publishPost(id: ID!): Post!
  }
  ```

---

##### 2.2. Custom Libraries (Phase 2)

###### 2.2.1. `leptos-table` (Data Tables)

**Статус:** ⏳ TODO  
**Приоритет:** P1  
**Блокирует:** Users list, Posts list

- [ ] **Core:**
  - [ ] `Table` component
  - [ ] `Column` config
  - [ ] `use_table()` hook
  - [ ] Pagination logic

- [ ] **Features:**
  - [ ] Server-side pagination (offset/limit)
  - [ ] Server-side sorting (field, direction)
  - [ ] Server-side filtering (search query)
  - [ ] Row selection (single, multiple)
  - [ ] Loading/empty states

- [ ] **Documentation:**
  - [ ] README.md
  - [ ] Examples (users table)

**Desired API:**
```rust
use leptos_table::{Table, Column, use_table};

let table = use_table::<User>()
    .query(fetch_users)
    .pagination(10)
    .sortable(true);

view! {
    <Table table=table>
        <Column field="email" label="Email" sortable=true />
        <Column field="name" label="Name" sortable=true />
        <Column render=|user| view! {
            <button on:click=move |_| edit_user(user.id)>"Edit"</button>
        } />
    </Table>
}
```

**Зависимости:**
- `leptos`
- `leptos-graphql`
- `leptos-shadcn-pagination` (UI)

---

###### 2.2.2. `leptos-toast` (Notifications)

**Статус:** ⏳ TODO  
**Приоритет:** P1  
**Блокирует:** User feedback

- [ ] **Core:**
  - [ ] `Toast` component
  - [ ] `ToastProvider`
  - [ ] `use_toast()` hook
  - [ ] Queue management

- [ ] **Features:**
  - [ ] Variants: success, error, info, warning
  - [ ] Auto-dismiss (timer)
  - [ ] Manual dismiss
  - [ ] Positioning (top-right, etc.)

**Desired API:**
```rust
use leptos_toast::{use_toast, ToastVariant};

let toast = use_toast();
toast.success("User created successfully");
toast.error("Failed to save changes");
```

---

###### 2.2.3. `leptos-modal` (Modals)

**Статус:** ⏳ TODO  
**Приоритет:** P1  
**Блокирует:** Edit/delete dialogs

- [ ] **Core:**
  - [ ] `Modal` component
  - [ ] `use_modal()` hook
  - [ ] Backdrop
  - [ ] Focus trap

- [ ] **Features:**
  - [ ] Click-outside close
  - [ ] ESC key close
  - [ ] Scroll lock
  - [ ] Sizes (sm, md, lg, xl)

**Desired API:**
```rust
use leptos_modal::{Modal, use_modal};

let modal = use_modal();

view! {
    <button on:click=move |_| modal.open()>"Delete"</button>
    
    <Modal open=modal.is_open on:close=modal.close>
        <h2>"Delete User?"</h2>
        <p>"Are you sure?"</p>
        <button on:click=modal.close>"Cancel"</button>
        <button on:click=move |_| { delete(); modal.close(); }>"Delete"</button>
    </Modal>
}
```

---

###### 2.2.4. `leptos-ui` (Phase 2 Components)

- [ ] **Table primitives** (Table, TableHeader, TableRow, TableCell)
- [ ] **Dropdown menu**
- [ ] **Dialog** (Modal)
- [ ] **Tabs**
- [ ] **Skeleton** (loading state)
- [ ] **Checkbox**
- [ ] **Textarea**
- [ ] **Select/Combobox**

---

##### 2.3. Leptos Admin (Phase 2)

- [ ] **Users List** (`apps/admin/src/pages/users.rs`)
  - [ ] Table с pagination
  - [ ] Search
  - [ ] Sort by email, name
  - [ ] Actions: Edit, Delete
  - [ ] "Create User" button

- [ ] **User Edit** (`apps/admin/src/pages/user_edit.rs`)
  - [ ] Form (email, name, role)
  - [ ] Submit → `updateUser` mutation
  - [ ] Toast на success/error

- [ ] **User Create** (`apps/admin/src/pages/user_create.rs`)
  - [ ] Form (email, name, password, role)
  - [ ] Submit → `createUser` mutation

- [ ] **Posts List** (аналогично Users)
- [ ] **Post Edit** (аналогично User Edit)
- [ ] **Post Create** (аналогично User Create)

**Uses:**
- `leptos-table` (data table)
- `leptos-forms` (forms)
- `leptos-toast` (notifications)
- `leptos-modal` (delete confirmation)
- `leptos-ui` (UI components)

---

##### 2.4. Next.js Admin (Phase 2)

- [ ] **Users List** (`apps/next-admin/app/(dashboard)/users/page.tsx`)
  - [ ] Table (TanStack Table)
  - [ ] Pagination, search, sort
  - [ ] Actions

- [ ] **User Edit** (`apps/next-admin/app/(dashboard)/users/[id]/edit/page.tsx`)
  - [ ] Form (react-hook-form)
  - [ ] Submit → GraphQL mutation

- [ ] **Posts** (аналогично Users)

**Uses:**
- `shadcn/ui` (Table, Dialog, Toast)
- `react-hook-form` (forms)
- `@tanstack/react-table` (table logic)

---

##### 2.5. Testing & QA

- [ ] E2E tests:
  - [ ] Create user flow
  - [ ] Edit user flow
  - [ ] Delete user flow
  - [ ] Search/filter/sort
  - [ ] Pagination

- [ ] Unit tests:
  - [ ] Form validation
  - [ ] Table logic

---

##### 2.6. Documentation

- [ ] **Phase 2 Summary** (`docs/UI/PHASE_2_COMPLETE.md`)
- [ ] **Update README**

---

#### Критерии завершения Phase 2:

- [ ] ✅ Backend GraphQL schema для Users, Posts
- [ ] ✅ `leptos-table`, `leptos-toast`, `leptos-modal` реализованы
- [ ] ✅ Leptos Admin: CRUD для Users, Posts работает
- [ ] ✅ Next.js Admin: CRUD для Users, Posts работает
- [ ] ✅ E2E tests проходят
- [ ] ✅ Документация обновлена

**Блокирует:** Phase 3 (Advanced Features)

---

### ⏳ Фаза 3: Advanced Features

**Статус:** ⏳ Запланировано  
**Приоритет:** P2  
**Длительность:** 7-10 дней

**Цель:** Расширенные возможности (i18n, file upload, permissions)

#### Задачи:

##### 3.1. Custom Libraries (Phase 3)

- [ ] **`leptos-i18n`** (Internationalization)
  - [ ] Translation files (JSON)
  - [ ] Locale context
  - [ ] `t!()` macro
  - [ ] Locale switching

- [ ] **`leptos-file-upload`** (File Upload)
  - [ ] File picker
  - [ ] Drag & drop
  - [ ] Progress bar
  - [ ] Preview

- [ ] **`leptos-routing`** (Extended Routing)
  - [ ] Breadcrumbs
  - [ ] Active link detection
  - [ ] Route guards
  - [ ] Query params helpers

---

##### 3.2. Features

- [ ] **Multi-language support** (i18n)
  - [ ] Locale switcher
  - [ ] Translations (en, ru)
  - [ ] RTL support (optional)

- [ ] **File upload** (Media management)
  - [ ] Upload images
  - [ ] Media library
  - [ ] Image preview

- [ ] **Permissions** (RBAC)
  - [ ] Role-based access control
  - [ ] Permission checks in UI
  - [ ] `@requireRole` directive

- [ ] **Breadcrumbs**
  - [ ] Auto-generation from routes
  - [ ] Manual override

---

##### 3.3. Testing & Documentation

- [ ] E2E tests (i18n, file upload, permissions)
- [ ] Phase 3 Summary documentation

---

#### Критерии завершения Phase 3:

- [ ] ✅ `leptos-i18n`, `leptos-file-upload`, `leptos-routing` реализованы
- [ ] ✅ Multi-language support работает
- [ ] ✅ File upload работает
- [ ] ✅ RBAC реализован
- [ ] ✅ E2E tests проходят

**Блокирует:** Phase 4 (Analytics & Polish)

---

### ⏳ Фаза 4: Analytics & Polish

**Статус:** ⏳ Запланировано  
**Приоритет:** P3  
**Длительность:** 5-7 дней

**Цель:** Аналитика, графики, полировка UX

#### Задачи:

##### 4.1. Custom Libraries (Phase 4)

- [ ] **`leptos-charts`** (Charting)
  - [ ] Line charts
  - [ ] Bar charts
  - [ ] Pie charts
  - [ ] Area charts

---

##### 4.2. Features

- [ ] **Analytics Dashboard**
  - [ ] User growth chart
  - [ ] Post activity chart
  - [ ] Traffic stats

- [ ] **UI Polish**
  - [ ] Animations (smooth transitions)
  - [ ] Skeleton loaders
  - [ ] Empty states
  - [ ] Error states
  - [ ] Loading states

- [ ] **Performance Optimization**
  - [ ] Lazy loading
  - [ ] Code splitting
  - [ ] Bundle size optimization
  - [ ] Lighthouse audit

---

##### 4.3. Testing & Documentation

- [ ] E2E tests (analytics)
- [ ] Performance tests
- [ ] Final documentation
- [ ] User guide
- [ ] Developer guide

---

#### Критерии завершения Phase 4:

- [ ] ✅ `leptos-charts` реализован
- [ ] ✅ Analytics dashboard работает
- [ ] ✅ UI polish завершён
- [ ] ✅ Performance оптимизирован
- [ ] ✅ Документация полная

**Результат:** 🎉 **Production-ready admin panels**

---

## 📊 Progress Tracking

### Phase Status

| Phase | Status | Progress | ETA |
|-------|--------|----------|-----|
| Phase 0: Setup | ✅ Завершена | 100% | 2026-02-13 |
| Phase 1: Auth + Nav | 🚧 В работе | 20% | 2026-02-18 |
| Phase 2: CRUD | ⏳ Запланирована | 0% | 2026-02-25 |
| Phase 3: Advanced | ⏳ Запланирована | 0% | 2026-03-07 |
| Phase 4: Polish | ⏳ Запланирована | 0% | 2026-03-14 |

### Custom Libraries Status

| Library | Status | Phase | Progress |
|---------|--------|-------|----------|
| `leptos-graphql` | ✅ Готово | Phase 0 | 100% |
| `leptos-auth` | ✅ Готово | Phase 0 | 100% |
| `leptos-forms` | 🚧 WIP | Phase 1 | 0% |
| `leptos-ui` | 🚧 WIP | Phase 1 | 0% |
| `leptos-table` | ⏳ TODO | Phase 2 | 0% |
| `leptos-toast` | ⏳ TODO | Phase 2 | 0% |
| `leptos-modal` | ⏳ TODO | Phase 2 | 0% |
| `leptos-i18n` | ⏳ TODO | Phase 3 | 0% |
| `leptos-file-upload` | ⏳ TODO | Phase 3 | 0% |
| `leptos-routing` | ⏳ TODO | Phase 3 | 0% |
| `leptos-charts` | ⏳ TODO | Phase 4 | 0% |

---

## 🔗 Related Documentation

### Core Docs
- [`QUICKSTART.md`](../../QUICKSTART.md) — быстрый старт
- [`docs/UI/README.md`](./README.md) — общая документация UI
- [`docs/UI/CRITICAL_WARNINGS.md`](./CRITICAL_WARNINGS.md) — критичные предупреждения

### Architecture
- [`docs/UI/GRAPHQL_ARCHITECTURE.md`](./GRAPHQL_ARCHITECTURE.md) — GraphQL архитектура
- [`docs/UI/DESIGN_SYSTEM_DECISION.md`](./DESIGN_SYSTEM_DECISION.md) — выбор DSD подхода
- [`docs/UI/CUSTOM_LIBRARIES_STATUS.md`](./CUSTOM_LIBRARIES_STATUS.md) — статус библиотек

### Workflow
- [`docs/UI/PARALLEL_DEVELOPMENT_WORKFLOW.md`](./PARALLEL_DEVELOPMENT_WORKFLOW.md) — параллельная разработка

### Phase Documentation
- [`docs/UI/PHASE_0_COMPLETE.md`](./PHASE_0_COMPLETE.md) — Phase 0 завершена
- [`docs/UI/PHASE_1_START.md`](./PHASE_1_START.md) — Phase 1 началась

---

## 📞 Contact & Support

**Maintainer:** CTO Agent  
**Last Updated:** 2026-02-13  
**Version:** 1.0.0

> 💡 **Tip:** Держите этот файл актуальным при переходе между фазами!

---

## 📝 Change Log

### 2026-02-13
- ✅ Phase 0 завершена
- 🚧 Phase 1 началась
- 📝 Создан Master Implementation Plan (объединение всех планов)
- 📝 Создан CUSTOM_LIBRARIES_STATUS.md (статус библиотек)
- 📝 Добавлен FSD/DSD дизайн-система в план
