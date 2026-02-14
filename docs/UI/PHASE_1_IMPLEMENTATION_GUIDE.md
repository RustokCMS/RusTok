# Phase 1 Implementation Guide — Auth + Navigation + Base UI

**Дата:** 2026-02-14  
**Статус:** 🚧 В работе  
**Цель:** Реализовать базовую оболочку админок с авторизацией и навигацией

---

## 📋 Обзор Phase 1

### Что реализуем:

1. **Backend GraphQL Schema** — Auth mutations/queries, RBAC directives
2. **Custom Libraries (Phase 1)**:
   - `leptos-forms` — Form handling, validation
   - `leptos-ui` — UI components (Button, Input, Card, etc.)
3. **Leptos Admin** — Login, Register, App shell, Dashboard
4. **Next.js Admin** — Login, Register, App shell, Dashboard
5. **Testing & Documentation**

### Результат:

✅ Обе админки с рабочей авторизацией  
✅ Единый дизайн и функциональность  
✅ Базовые UI компоненты  
✅ GraphQL API готов

---

## 🎯 Задачи Phase 1

### 1. Backend GraphQL Schema ⏳ TODO

**Файл:** `apps/server/src/graphql/schema.rs`

#### 1.1. Auth Mutations

```graphql
type Mutation {
  signIn(email: String!, password: String!): SignInPayload!
  signUp(input: SignUpInput!): SignUpPayload!
  signOut: Boolean!
  refreshToken: RefreshTokenPayload!
  forgotPassword(email: String!): Boolean!
  resetPassword(token: String!, newPassword: String!): Boolean!
}

input SignUpInput {
  email: String!
  password: String!
  name: String
}

type SignInPayload {
  token: String!
  user: User!
  expiresAt: DateTime!
}

type SignUpPayload {
  token: String!
  user: User!
  expiresAt: DateTime!
}

type RefreshTokenPayload {
  token: String!
  expiresAt: DateTime!
}
```

#### 1.2. Auth Queries

```graphql
type Query {
  currentUser: User
  users(
    limit: Int = 10
    offset: Int = 0
    search: String
  ): UserConnection!
}

type User {
  id: ID!
  email: String!
  name: String
  role: UserRole!
  createdAt: DateTime!
  updatedAt: DateTime!
}

enum UserRole {
  ADMIN
  EDITOR
  VIEWER
}

type UserConnection {
  nodes: [User!]!
  totalCount: Int!
  pageInfo: PageInfo!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
}
```

#### 1.3. RBAC Directives

```graphql
directive @requireAuth on FIELD_DEFINITION
directive @requireRole(role: UserRole!) on FIELD_DEFINITION

# Usage example:
type Query {
  currentUser: User @requireAuth
  users: UserConnection! @requireRole(role: ADMIN)
}
```

#### Задачи Backend:

- [ ] Создать `SignInPayload`, `SignUpPayload` types
- [ ] Реализовать `sign_in` resolver
- [ ] Реализовать `sign_up` resolver
- [ ] Реализовать `sign_out` resolver
- [ ] Реализовать `refresh_token` resolver
- [ ] Реализовать `forgot_password` resolver
- [ ] Реализовать `reset_password` resolver
- [ ] Реализовать `current_user` query
- [ ] Реализовать `@requireAuth` directive
- [ ] Реализовать `@requireRole` directive
- [ ] Unit tests для resolvers
- [ ] Integration tests для auth flow

**Блокирует:** Весь Phase 1

---

### 2. Custom Library: `leptos-forms` 🚧 WIP

**Цель:** Form handling и validation для Leptos

#### Структура:

```
crates/leptos-forms/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           # Re-exports
    ├── form.rs          # FormContext, use_form hook
    ├── field.rs         # Field component
    ├── validator.rs     # Validation rules
    └── error.rs         # Error types
```

#### API Design:

```rust
use leptos_forms::{use_form, Field, Validator};

#[component]
fn LoginForm() -> impl IntoView {
    let form = use_form(|| LoginData::default())
        .field("email", Validator::email().required())
        .field("password", Validator::min_length(6).required())
        .on_submit(|data| async move {
            api::sign_in(&data.email, &data.password, &tenant).await
        });

    view! {
        <form on:submit=form.submit>
            <Field 
                form=form 
                name="email" 
                label="Email" 
                placeholder="you@example.com"
            />
            <Field 
                form=form 
                name="password" 
                label="Password" 
                type="password"
            />
            
            <button disabled=form.is_submitting>
                {move || if form.is_submitting() { "Loading..." } else { "Login" }}
            </button>
            
            {move || form.error().map(|err| view! {
                <div class="text-red-500">{err}</div>
            })}
        </form>
    }
}
```

#### Features:

- ✅ `use_form()` hook — form state management
- ✅ `Field` component — input with error display
- ✅ Validators:
  - `required()` — обязательное поле
  - `email()` — валидация email
  - `min_length(n)` — минимальная длина
  - `max_length(n)` — максимальная длина
  - `pattern(regex)` — регулярное выражение
  - `custom(fn)` — кастомная валидация
- ✅ Per-field errors
- ✅ Form-level errors
- ✅ Submit handling (loading, error states)
- ✅ Reactive validation (on blur, on change, on submit)

#### Задачи:

- [ ] Создать crate: `cargo new --lib crates/leptos-forms`
- [ ] Реализовать `FormContext` и `use_form()` hook
- [ ] Реализовать `Field` component
- [ ] Реализовать validators (required, email, min_length, etc.)
- [ ] Реализовать error handling
- [ ] Реализовать submit handling
- [ ] Написать README с примерами
- [ ] Создать example в `apps/admin/examples/`
- [ ] Unit tests

**Зависимости:**
```toml
[dependencies]
leptos = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
regex = "1"
```

**Блокирует:** Login/Register pages

---

### 3. Custom Library: `leptos-ui` 🚧 WIP

**Цель:** DSD-style UI components (shadcn approach)

#### Структура:

```
crates/leptos-ui/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs           # Re-exports
    ├── button.rs        # Button component
    ├── input.rs         # Input, Textarea
    ├── label.rs         # Label
    ├── card.rs          # Card, CardHeader, CardContent, CardFooter
    ├── badge.rs         # Badge
    ├── separator.rs     # Separator
    ├── alert.rs         # Alert
    └── types.rs         # Shared types (Size, Variant)
```

#### Components (Phase 1):

##### Button

```rust
use leptos_ui::{Button, ButtonVariant, ButtonSize};

view! {
    <Button variant=ButtonVariant::Primary size=ButtonSize::Md>
        "Click me"
    </Button>
    
    <Button variant=ButtonVariant::Outline loading=true>
        "Loading..."
    </Button>
}
```

**Props:**
- `variant`: Primary | Secondary | Outline | Ghost | Destructive
- `size`: Sm | Md | Lg
- `disabled`: bool
- `loading`: bool
- `icon_left`: Option<View>
- `icon_right`: Option<View>
- `on_click`: Callback

##### Input

```rust
use leptos_ui::Input;

view! {
    <Input 
        type="email"
        placeholder="you@example.com"
        value=email
        on_change=set_email
        error=Some("Invalid email")
    />
}
```

**Props:**
- `type`: text | email | password | number
- `placeholder`: String
- `value`: Signal<String>
- `on_change`: Callback<String>
- `error`: Option<String>
- `disabled`: bool
- `icon_left`: Option<View>
- `icon_right`: Option<View>

##### Card

```rust
use leptos_ui::{Card, CardHeader, CardContent, CardFooter};

view! {
    <Card>
        <CardHeader>
            <h2 class="text-2xl font-bold">"Login"</h2>
        </CardHeader>
        <CardContent>
            <p>"Please enter your credentials"</p>
        </CardContent>
        <CardFooter>
            <Button>"Submit"</Button>
        </CardFooter>
    </Card>
}
```

##### Label

```rust
use leptos_ui::Label;

view! {
    <Label required=true>"Email"</Label>
    <Input type="email" />
}
```

##### Badge

```rust
use leptos_ui::{Badge, BadgeVariant};

view! {
    <Badge variant=BadgeVariant::Success>"Active"</Badge>
    <Badge variant=BadgeVariant::Warning>"Pending"</Badge>
}
```

**Variants:** Default | Primary | Success | Warning | Danger

##### Separator

```rust
use leptos_ui::Separator;

view! {
    <Separator />  // horizontal
    <Separator orientation="vertical" />
}
```

#### Design Principles:

1. **DSD approach** (shadcn-style)
   - Copy-paste friendly
   - Variants over composition
   - Tailwind-first

2. **Accessibility**
   - ARIA attributes
   - Keyboard navigation
   - Focus management

3. **Consistency**
   - Единый API для всех компонентов
   - Shared types (Size, Variant)
   - Tailwind CSS classes

#### Задачи:

- [ ] Создать crate: `cargo new --lib crates/leptos-ui`
- [ ] Реализовать Button component
- [ ] Реализовать Input component
- [ ] Реализовать Label component
- [ ] Реализовать Card components (Card, CardHeader, CardContent, CardFooter)
- [ ] Реализовать Badge component
- [ ] Реализовать Separator component
- [ ] Написать README с примерами
- [ ] Создать Storybook-like example page
- [ ] Unit tests для каждого компонента

**Зависимости:**
```toml
[dependencies]
leptos = { workspace = true }
```

**Блокирует:** Все UI в Phase 1

---

### 4. Leptos Admin (Phase 1) 🚧 WIP

#### 4.1. Auth Pages

##### Login Page

**Файл:** `apps/admin/src/pages/auth/login.rs`

```rust
use leptos::*;
use leptos_router::*;
use leptos_auth::{use_auth, api};
use leptos_forms::{use_form, Field, Validator};
use leptos_ui::{Button, Card, CardHeader, CardContent, ButtonVariant};

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    
    let form = use_form(|| LoginFormData::default())
        .field("email", Validator::email().required())
        .field("password", Validator::min_length(6).required())
        .on_submit(move |data| async move {
            match api::sign_in(&data.email, &data.password, &"demo").await {
                Ok((user, session)) => {
                    auth.set_session(session);
                    navigate("/dashboard", Default::default());
                }
                Err(e) => {
                    form.set_error(e.to_string());
                }
            }
        });
    
    view! {
        <div class="min-h-screen flex items-center justify-center bg-gray-50">
            <Card class="w-full max-w-md">
                <CardHeader>
                    <h2 class="text-2xl font-bold">"Sign In"</h2>
                    <p class="text-gray-600">"Enter your credentials to continue"</p>
                </CardHeader>
                <CardContent>
                    <form on:submit=form.submit class="space-y-4">
                        <Field 
                            form=form 
                            name="email" 
                            label="Email" 
                            placeholder="you@example.com"
                        />
                        <Field 
                            form=form 
                            name="password" 
                            label="Password" 
                            type="password"
                            placeholder="••••••••"
                        />
                        
                        <div class="flex items-center justify-between">
                            <a href="/auth/forgot-password" class="text-sm text-blue-600 hover:underline">
                                "Forgot password?"
                            </a>
                        </div>
                        
                        <Button 
                            variant=ButtonVariant::Primary 
                            type="submit"
                            loading=form.is_submitting()
                            class="w-full"
                        >
                            "Sign In"
                        </Button>
                        
                        {move || form.error().map(|err| view! {
                            <div class="text-red-500 text-sm">{err}</div>
                        })}
                    </form>
                    
                    <div class="mt-4 text-center text-sm">
                        "Don't have an account? "
                        <a href="/auth/register" class="text-blue-600 hover:underline">
                            "Sign up"
                        </a>
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}

#[derive(Default, Clone)]
struct LoginFormData {
    email: String,
    password: String,
}
```

**Задачи:**
- [ ] Создать LoginPage component
- [ ] Интегрировать leptos-forms
- [ ] Интегрировать leptos-ui
- [ ] Обработка успешного login (redirect)
- [ ] Обработка ошибок
- [ ] Links: Forgot password, Sign up

##### Register Page

**Файл:** `apps/admin/src/pages/auth/register.rs`

```rust
#[component]
pub fn RegisterPage() -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    
    let form = use_form(|| RegisterFormData::default())
        .field("email", Validator::email().required())
        .field("name", Validator::required())
        .field("password", Validator::min_length(8).required())
        .field("confirm_password", Validator::custom(|value, form_data| {
            if value != form_data.password {
                Err("Passwords don't match")
            } else {
                Ok(())
            }
        }))
        .on_submit(move |data| async move {
            match api::sign_up(&data.email, &data.name, &data.password).await {
                Ok((user, session)) => {
                    auth.set_session(session);
                    navigate("/dashboard", Default::default());
                }
                Err(e) => {
                    form.set_error(e.to_string());
                }
            }
        });
    
    // ... view similar to LoginPage
}
```

**Задачи:**
- [ ] Создать RegisterPage component
- [ ] Form fields: email, name, password, confirm_password
- [ ] Password match validation
- [ ] Обработка успешного register (redirect)
- [ ] Link: Already have account? Sign in

##### Forgot Password Page

**Файл:** `apps/admin/src/pages/auth/forgot_password.rs`

**Задачи:**
- [ ] Создать ForgotPasswordPage component
- [ ] Form: email field
- [ ] Submit → `api::forgot_password()`
- [ ] Success message: "Check your email"

##### Reset Password Page

**Файл:** `apps/admin/src/pages/auth/reset_password.rs`

**Задачи:**
- [ ] Создать ResetPasswordPage component
- [ ] Get token from URL params
- [ ] Form: new_password, confirm_password
- [ ] Submit → `api::reset_password()`
- [ ] Success redirect to login

#### 4.2. App Shell

##### Layout

**Файл:** `apps/admin/src/components/layouts/app_layout.rs`

```rust
use leptos::*;
use leptos_router::Outlet;
use crate::components::layouts::{Sidebar, Header};

#[component]
pub fn AppLayout() -> impl IntoView {
    view! {
        <div class="flex h-screen bg-gray-100">
            <Sidebar />
            <div class="flex-1 flex flex-col overflow-hidden">
                <Header />
                <main class="flex-1 overflow-auto p-6">
                    <Outlet />
                </main>
            </div>
        </div>
    }
}
```

**Задачи:**
- [ ] Создать AppLayout component
- [ ] Responsive layout (sidebar collapse on mobile)
- [ ] Smooth transitions

##### Sidebar

**Файл:** `apps/admin/src/components/layouts/sidebar.rs`

```rust
use leptos::*;
use leptos_router::*;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    
    let is_active = move |path: &str| {
        location.pathname.get() == path
    };
    
    view! {
        <aside class="w-64 bg-white border-r border-gray-200">
            <div class="flex items-center h-16 px-6 border-b">
                <h1 class="text-xl font-bold">"RusToK Admin"</h1>
            </div>
            
            <nav class="p-4 space-y-2">
                <NavLink href="/dashboard" active=is_active("/dashboard")>
                    "Dashboard"
                </NavLink>
                <NavLink href="/users" active=is_active("/users")>
                    "Users"
                </NavLink>
                <NavLink href="/posts" active=is_active("/posts")>
                    "Posts"
                </NavLink>
                <NavLink href="/pages" active=is_active("/pages")>
                    "Pages"
                </NavLink>
                <NavLink href="/settings" active=is_active("/settings")>
                    "Settings"
                </NavLink>
            </nav>
        </aside>
    }
}

#[component]
fn NavLink(
    href: &'static str,
    active: impl Fn() -> bool + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <A 
            href=href
            class=move || {
                if active() {
                    "block px-4 py-2 rounded bg-blue-50 text-blue-600 font-medium"
                } else {
                    "block px-4 py-2 rounded hover:bg-gray-100"
                }
            }
        >
            {children()}
        </A>
    }
}
```

**Задачи:**
- [ ] Создать Sidebar component
- [ ] Navigation links с active state
- [ ] Logo/branding
- [ ] Collapse/expand на mobile

##### Header

**Файл:** `apps/admin/src/components/layouts/header.rs`

```rust
use leptos::*;
use leptos_auth::use_current_user;
use crate::components::features::auth::UserMenu;

#[component]
pub fn Header() -> impl IntoView {
    let current_user = use_current_user();
    
    view! {
        <header class="h-16 bg-white border-b border-gray-200 flex items-center justify-between px-6">
            <div class="flex items-center space-x-4">
                <h2 class="text-lg font-semibold">"Dashboard"</h2>
            </div>
            
            <div class="flex items-center space-x-4">
                // Notifications (placeholder)
                <button class="relative">
                    <span class="absolute -top-1 -right-1 bg-red-500 text-white text-xs rounded-full w-4 h-4 flex items-center justify-center">
                        "3"
                    </span>
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                    </svg>
                </button>
                
                // User menu
                <UserMenu user=current_user />
            </div>
        </header>
    }
}
```

**Задачи:**
- [ ] Создать Header component
- [ ] User menu (avatar, dropdown)
- [ ] Notifications badge (placeholder)
- [ ] Search (optional, Phase 2)

##### User Menu

**Файл:** `apps/admin/src/components/features/auth/user_menu.rs`

```rust
use leptos::*;
use leptos_router::use_navigate;
use leptos_auth::{use_auth, User};

#[component]
pub fn UserMenu(user: Signal<Option<User>>) -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();
    let (open, set_open) = create_signal(false);
    
    let handle_logout = move |_| {
        spawn_local(async move {
            let _ = auth.sign_out().await;
            navigate("/auth/login", Default::default());
        });
    };
    
    view! {
        <div class="relative">
            <button 
                on:click=move |_| set_open(!open.get())
                class="flex items-center space-x-2"
            >
                <div class="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center text-white">
                    {move || user.get().map(|u| u.name.chars().next().unwrap_or('U').to_string())}
                </div>
            </button>
            
            <Show when=move || open.get()>
                <div class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1">
                    <a href="/profile" class="block px-4 py-2 hover:bg-gray-100">
                        "Profile"
                    </a>
                    <a href="/settings" class="block px-4 py-2 hover:bg-gray-100">
                        "Settings"
                    </a>
                    <hr class="my-1" />
                    <button 
                        on:click=handle_logout
                        class="block w-full text-left px-4 py-2 hover:bg-gray-100 text-red-600"
                    >
                        "Logout"
                    </button>
                </div>
            </Show>
        </div>
    }
}
```

**Задачи:**
- [ ] Создать UserMenu component
- [ ] Dropdown: Profile, Settings, Logout
- [ ] Click outside to close
- [ ] User avatar/initial

#### 4.3. Dashboard (Placeholder)

**Файл:** `apps/admin/src/pages/dashboard.rs`

```rust
use leptos::*;
use leptos_ui::{Card, CardHeader, CardContent, Badge};

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="space-y-6">
            <div>
                <h1 class="text-3xl font-bold">"Dashboard"</h1>
                <p class="text-gray-600">"Welcome to RusToK Admin"</p>
            </div>
            
            // Stats cards
            <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
                <StatCard 
                    title="Total Users" 
                    value="1,234" 
                    trend="+12%"
                />
                <StatCard 
                    title="Total Posts" 
                    value="567" 
                    trend="+8%"
                />
                <StatCard 
                    title="Active Sessions" 
                    value="89" 
                    trend="+23%"
                />
                <StatCard 
                    title="Revenue" 
                    value="$12,345" 
                    trend="+15%"
                />
            </div>
            
            // Recent activity
            <Card>
                <CardHeader>
                    <h2 class="text-xl font-semibold">"Recent Activity"</h2>
                </CardHeader>
                <CardContent>
                    <div class="space-y-4">
                        <ActivityItem 
                            user="John Doe" 
                            action="created a new post" 
                            time="2 hours ago"
                        />
                        <ActivityItem 
                            user="Jane Smith" 
                            action="updated user profile" 
                            time="4 hours ago"
                        />
                    </div>
                </CardContent>
            </Card>
        </div>
    }
}

#[component]
fn StatCard(
    title: &'static str,
    value: &'static str,
    trend: &'static str,
) -> impl IntoView {
    view! {
        <Card>
            <CardContent class="p-6">
                <div class="flex items-center justify-between">
                    <div>
                        <p class="text-sm text-gray-600">{title}</p>
                        <p class="text-2xl font-bold">{value}</p>
                    </div>
                    <Badge variant=BadgeVariant::Success>{trend}</Badge>
                </div>
            </CardContent>
        </Card>
    }
}
```

**Задачи:**
- [ ] Создать DashboardPage component
- [ ] Stats cards (placeholder data)
- [ ] Recent activity list (placeholder)
- [ ] Charts (Phase 4)

---

### 5. Next.js Admin (Phase 1) ⏳ TODO

**Принцип:** Копируем функциональность из Leptos Admin, используя готовые React-библиотеки.

#### 5.1. Auth Pages

**Задачи:**
- [ ] Login page: `apps/next-admin/src/app/(auth)/login/page.tsx`
- [ ] Register page: `apps/next-admin/src/app/(auth)/register/page.tsx`
- [ ] Forgot password: `apps/next-admin/src/app/(auth)/forgot-password/page.tsx`
- [ ] Reset password: `apps/next-admin/src/app/(auth)/reset-password/page.tsx`

**Uses:**
- `react-hook-form` для form handling
- `zod` для validation
- `urql` или `@apollo/client` для GraphQL
- `shadcn/ui` components (Button, Input, Card)

#### 5.2. App Shell

**Задачи:**
- [ ] Layout: `apps/next-admin/src/app/(dashboard)/layout.tsx`
- [ ] Sidebar component
- [ ] Header component
- [ ] User menu component

#### 5.3. Dashboard

**Задачи:**
- [ ] Dashboard page: `apps/next-admin/src/app/(dashboard)/page.tsx`
- [ ] Stats cards
- [ ] Recent activity

---

### 6. Testing & QA ⏳ TODO

#### Backend Tests

**Файл:** `apps/server/tests/graphql_auth_tests.rs`

**Задачи:**
- [ ] Unit tests для auth resolvers
- [ ] Integration test: sign_in flow
- [ ] Integration test: sign_up flow
- [ ] Integration test: refresh_token
- [ ] Integration test: @requireAuth directive
- [ ] Integration test: @requireRole directive

#### Leptos Admin Tests

**Файл:** `apps/admin/tests/e2e/auth.spec.rs` (Playwright)

**Задачи:**
- [ ] E2E test: Login flow
- [ ] E2E test: Register flow
- [ ] E2E test: Logout flow
- [ ] E2E test: Forgot password
- [ ] E2E test: Protected route redirect

#### Next.js Admin Tests

**Файл:** `apps/next-admin/__tests__/e2e/auth.spec.ts` (Playwright)

**Задачи:**
- [ ] E2E test: Login flow
- [ ] E2E test: Register flow
- [ ] E2E test: Logout flow

#### Cross-browser Tests

**Задачи:**
- [ ] Chrome
- [ ] Firefox
- [ ] Safari (if available)

---

### 7. Documentation ⏳ TODO

**Задачи:**
- [ ] Update `CUSTOM_LIBRARIES_STATUS.md` — leptos-forms, leptos-ui status
- [ ] Create `PHASE_1_COMPLETE.md` — summary с screenshots
- [ ] Update `apps/admin/README.md` — auth flow guide
- [ ] Update `apps/next-admin/README.md` — auth flow guide

---

## 📊 Progress Tracking

### Phase 1 Status: 20%

| Task | Status | Blockers |
|------|--------|----------|
| Backend GraphQL Schema | ⏳ TODO | None |
| leptos-forms | 🚧 WIP | None |
| leptos-ui | 🚧 WIP | None |
| Leptos Admin: Auth pages | ⏳ TODO | leptos-forms, leptos-ui |
| Leptos Admin: App shell | ⏳ TODO | leptos-ui |
| Leptos Admin: Dashboard | ⏳ TODO | leptos-ui |
| Next.js Admin: Auth pages | ⏳ TODO | Backend GraphQL |
| Next.js Admin: App shell | ⏳ TODO | None |
| Next.js Admin: Dashboard | ⏳ TODO | None |
| Testing | ⏳ TODO | All above |
| Documentation | ⏳ TODO | All above |

---

## 🚀 Quick Start Guide

### Для разработчика (начало работы Phase 1)

1. **Проверить backend API:**
   ```bash
   cd apps/server
   cargo test graphql_auth
   ```

2. **Работать над leptos-forms:**
   ```bash
   cd crates/leptos-forms
   cargo test
   # Создать example в apps/admin/examples/forms.rs
   ```

3. **Работать над leptos-ui:**
   ```bash
   cd crates/leptos-ui
   cargo test
   # Создать Storybook-like page в apps/admin
   ```

4. **Работать над Leptos Admin:**
   ```bash
   cd apps/admin
   trunk serve --port 3001
   # Open http://localhost:3001
   ```

5. **Работать над Next.js Admin:**
   ```bash
   cd apps/next-admin
   bun dev
   # Open http://localhost:3000
   ```

---

## 🔗 Related Docs

- [MASTER_IMPLEMENTATION_PLAN.md](./MASTER_IMPLEMENTATION_PLAN.md) — General plan
- [CUSTOM_LIBRARIES_STATUS.md](./CUSTOM_LIBRARIES_STATUS.md) — Libraries status
- [PARALLEL_DEVELOPMENT_WORKFLOW.md](./PARALLEL_DEVELOPMENT_WORKFLOW.md) — Workflow guide
- [CRITICAL_WARNINGS.md](./CRITICAL_WARNINGS.md) — Do not delete libraries!

---

**Last Updated:** 2026-02-14  
**Maintainer:** CTO Agent
