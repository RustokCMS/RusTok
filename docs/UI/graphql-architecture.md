# GraphQL Architecture for RusToK Admin

**Дата:** 2026-02-13  
**Главное правило:** ✅ **Только GraphQL, никакого REST API!**

---

## 🎯 Архитектурное решение

> **Админки используют ИСКЛЮЧИТЕЛЬНО GraphQL для всех операций (включая аутентификацию)**

### Почему только GraphQL?

1. **Единая точка входа** — один endpoint `/api/graphql` для всего
2. **Типобезопасность** — GraphQL schema как single source of truth
3. **Гибкость запросов** — клиент запрашивает только нужные поля
4. **Батчинг** — несколько операций в одном запросе
5. **Introspection** — автогенерация документации
6. **Простота** — не нужно поддерживать два API (REST + GraphQL)

---

## 📦 Архитектура слоёв

```
┌─────────────────────────────────────────────────┐
│         apps/admin (Leptos UI)                   │
│  ┌───────────────────────────────────────────┐  │
│  │ Pages: Login, Users, Dashboard            │  │
│  │ Uses: leptos-auth hooks & components      │  │
│  └───────────────────────────────────────────┘  │
└─────────────────┬───────────────────────────────┘
                  │ use_auth(), api::sign_in()
                  ▼
┌─────────────────────────────────────────────────┐
│     crates/leptos-auth (Auth Logic)              │
│  ┌───────────────────────────────────────────┐  │
│  │ api.rs: sign_in(), sign_up(), sign_out()  │  │
│  │ context.rs: AuthProvider, AuthContext     │  │
│  │ hooks.rs: use_auth(), use_token()         │  │
│  │ storage.rs: LocalStorage helpers          │  │
│  └───────────────────────────────────────────┘  │
└─────────────────┬───────────────────────────────┘
                  │ leptos_graphql::execute()
                  ▼
┌─────────────────────────────────────────────────┐
│   crates/leptos-graphql (Transport Layer)        │
│  ┌───────────────────────────────────────────┐  │
│  │ execute() - HTTP client для GraphQL       │  │
│  │ GraphqlRequest, GraphqlResponse           │  │
│  │ Error mapping (Network, Graphql, Http)    │  │
│  └───────────────────────────────────────────┘  │
└─────────────────┬───────────────────────────────┘
                  │ POST /api/graphql
                  ▼
┌─────────────────────────────────────────────────┐
│          apps/server (Backend)                   │
│  ┌───────────────────────────────────────────┐  │
│  │ GraphQL Schema (async-graphql)            │  │
│  │ Mutations: signIn, signUp, signOut        │  │
│  │ Queries: me, users, dashboardStats        │  │
│  └───────────────────────────────────────────┘  │
└────────────────▲────────────────────────────────┘
                 │ POST /api/graphql
┌────────────────┴────────────────────────────────┐
│      apps/next-admin (Next.js Admin UI)          │
│  ┌───────────────────────────────────────────┐  │
│  │ NextAuth (Credentials provider)           │  │
│  │ lib/graphql.ts — fetch-based GQL client   │  │
│  │ lib/auth-api.ts — signIn(), me()          │  │
│  │ middleware.ts — route protection          │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

**Принцип разделения:**

- `apps/admin` — UI logic, state management (leptos Resources)
- `leptos-auth` — auth-specific business logic, LocalStorage, context
- `leptos-graphql` — generic HTTP transport для GraphQL (reusable)
- `apps/server` — GraphQL resolvers, database, business logic

---

## 📦 Библиотеки

### 1. `leptos-graphql` — Transport Layer

**Назначение:** Низкоуровневый HTTP-клиент для GraphQL запросов

**Файл:** `crates/leptos-graphql/src/lib.rs`

**API:**

```rust
pub const GRAPHQL_ENDPOINT: &str = "/api/graphql";
pub const TENANT_HEADER: &str = "X-Tenant-Slug";
pub const AUTH_HEADER: &str = "Authorization";

pub struct GraphqlRequest<V> {
    pub query: String,
    pub variables: Option<V>,
    pub extensions: Option<Value>,
}

pub async fn execute<V, T>(
    endpoint: &str,
    request: GraphqlRequest<V>,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, GraphqlHttpError>
where
    V: Serialize,
    T: DeserializeOwned;
```

**Использование:**

```rust
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};

let query = r#"
query GetUser($id: ID!) {
    user(id: $id) {
        id
        email
        name
    }
}
"#;

let variables = serde_json::json!({"id": "123"});
let request = GraphqlRequest::new(query, Some(variables));

let response: UserData = execute(
    GRAPHQL_ENDPOINT,
    request,
    Some(token),
    Some(tenant),
).await?;
```

---

### 2. `leptos-auth` — Authentication via GraphQL

**Назначение:** Высокоуровневые функции для аутентификации через GraphQL

**Файл:** `crates/leptos-auth/src/api.rs`

**Transport:** Использует `leptos-graphql::execute()` для всех запросов

**GraphQL Mutations/Queries:**

#### Authentication

```graphql
# Вход в систему
mutation SignIn($email: String!, $password: String!) {
    signIn(email: $email, password: $password) {
        token
        user {
            id
            email
            name
        }
    }
}

# Регистрация
mutation SignUp($email: String!, $password: String!, $name: String) {
    signUp(email: $email, password: $password, name: $name) {
        token
        user {
            id
            email
            name
        }
    }
}

# Выход
mutation SignOut {
    signOut
}

# Текущий пользователь
query CurrentUser {
    currentUser {
        id
        email
        name
    }
}

# Обновить токен
mutation RefreshToken {
    refreshToken {
        token
    }
}

# Забыли пароль
mutation ForgotPassword($email: String!) {
    forgotPassword(email: $email)
}

# Сброс пароля
mutation ResetPassword($token: String!, $newPassword: String!) {
    resetPassword(token: $token, newPassword: $newPassword)
}
```

**Implementation:**

```rust
// leptos-auth использует leptos-graphql под капотом
async fn execute_graphql<V, T>(
    query: &str,
    variables: Option<V>,
    token: Option<String>,
    tenant: String,
) -> Result<T, AuthError> {
    let endpoint = "http://localhost:5150/api/graphql";
    let request = leptos_graphql::GraphqlRequest::new(query, variables);
    
    leptos_graphql::execute(endpoint, request, token, Some(tenant))
        .await
        .map_err(AuthError::from)
}
```

**API Functions:**

```rust
use leptos_auth::api;

// Login
let (user, session) = api::sign_in(
    email,
    password,
    tenant,
).await?;

// Register
let (user, session) = api::sign_up(
    email,
    password,
    Some(name),
    tenant,
).await?;

// Logout
api::sign_out(&token, &tenant).await?;

// Get current user
let user = api::get_current_user(&token, &tenant).await?;

// Refresh token
let new_token = api::refresh_token(&token, &tenant).await?;

// Password reset flow
api::forgot_password(email, tenant).await?;
api::reset_password(reset_token, new_password, tenant).await?;
```

---

## 🏗️ Backend GraphQL Schema

### Mutations для аутентификации

**Реализовано в backend (`apps/server/src/graphql/auth/mutation.rs` + auth types):**

```graphql
type Mutation {
  # Authentication (apps/server/src/graphql/auth/mutation.rs)
  signIn(input: SignInInput!): AuthPayload!
  signUp(input: SignUpInput!): AuthPayload!
  signOut: SignOutPayload!
  refreshToken(input: RefreshTokenInput!): AuthPayload!
  forgotPassword(input: ForgotPasswordInput!): ForgotPasswordPayload!
  resetPassword(input: ResetPasswordInput!): ResetPasswordPayload!
  
  # User management (apps/server/src/graphql/mutations.rs)
  createUser(input: CreateUserInput!): User!
  updateUser(id: UUID!, input: UpdateUserInput!): User!
  disableUser(id: UUID!): User!
  toggleModule(moduleSlug: String!, enabled: Boolean!): TenantModule!
}

type Query {
  # Health & info
  health: String!
  apiVersion: String!
  
  # Authentication
  me: User                   # Возвращает текущего пользователя (или null)
  
  # Tenancy
  currentTenant: Tenant!
  enabledModules: [String!]!
  moduleRegistry: [ModuleRegistryItem!]!
  tenantModules: [TenantModule!]!
  
  # User management (RBAC-protected)
  users(pagination: PaginationInput, filter: UsersFilter, search: String): UserConnection!
  user(id: UUID!): User
  
  # Dashboard
  dashboardStats: DashboardStats!
  recentActivity(limit: Int!): [ActivityItem!]!
}

# Auth Input types
input SignInInput {
  email: String!
  password: String!
}

input SignUpInput {
  email: String!
  password: String!
  name: String
}

input RefreshTokenInput {
  refreshToken: String!
}

# Auth Response types
type AuthPayload {
  accessToken: String!
  refreshToken: String!
  tokenType: String!
  expiresIn: Int!
  user: AuthUser!
}

type AuthUser {
  id: String!
  email: String!
  name: String
  role: String!
  status: String!
}

type SignOutPayload {
  success: Boolean!
}

type ForgotPasswordPayload {
  success: Boolean!
  message: String!
}

type ResetPasswordPayload {
  success: Boolean!
}

# User type (returned by me, users, etc.)
type User {
  id: ID!
  email: String!
  name: String
  role: String!
  status: String!
  createdAt: DateTime!
  tenantName: String         # Via DataLoader
}
```

> **Важно:** Tenant определяется через HTTP header `X-Tenant-Slug`, а не через аргумент мутации.
> Schema защищена лимитами: `depth=12`, `complexity=600`.

---

## 🔄 Authentication Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  Frontend (apps/admin)                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1. User enters email/password                               │
│  2. Call leptos_auth::api::sign_in()                         │
│     ↓                                                         │
│  3. Creates GraphQL mutation:                                │
│     mutation SignIn($email, $password) { ... }               │
│     ↓                                                         │
│  4. leptos-auth uses execute_graphql()                       │
│     ↓                                                         │
│  5. Send POST /api/graphql with:                             │
│     - query: "mutation SignIn..."                            │
│     - variables: { email, password }                         │
│     - header: X-Tenant-Slug: <tenant>                        │
│     ↓                                                         │
└─────┼───────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Backend (apps/server)                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  6. GraphQL resolver: signIn(email, password)                │
│     ↓                                                         │
│  7. Validate credentials (check DB)                          │
│     ↓                                                         │
│  8. Generate JWT token                                       │
│     ↓                                                         │
│  9. Return: { token, user { id, email, name } }              │
│     ↓                                                         │
└─────┼───────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────┐
│                  Frontend (apps/admin)                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  10. Receive response: { token, user }                       │
│  11. Save to localStorage (via storage.rs)                   │
│  12. Update AuthContext state                                │
│  13. Redirect to /dashboard                                  │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## 📁 Структура кода

```
crates/
├── leptos-graphql/           ← Transport layer (HTTP client)
│   ├── src/
│   │   └── lib.rs            ← execute(), GraphqlRequest, GraphqlResponse
│   └── Cargo.toml
│
├── leptos-auth/              ← Auth-specific GraphQL operations
│   ├── src/
│   │   ├── api.rs            ← sign_in(), sign_up(), sign_out() (GraphQL)
│   │   ├── context.rs        ← AuthProvider component
│   │   ├── hooks.rs          ← use_auth(), use_token(), use_tenant()
│   │   ├── storage.rs        ← LocalStorage helpers
│   │   ├── components.rs     ← ProtectedRoute, GuestRoute
│   │   └── lib.rs
│   └── Cargo.toml
│
apps/
├── admin/                    ← Leptos Admin Panel
│   ├── src/
│   │   ├── app.rs            ← Wraps in <AuthProvider>
│   │   ├── pages/
│   │   │   ├── login.rs      ← Uses leptos_auth::api::sign_in()
│   │   │   ├── register.rs   ← Uses leptos_auth::api::sign_up()
│   │   │   └── users.rs      ← Uses leptos_graphql::execute()
│   │   └── ...
│   └── Cargo.toml
│
├── server/                   ← Backend
│   ├── src/
│   │   ├── graphql/
│   │   │   ├── mutations.rs  ← signIn, signUp, signOut resolvers
│   │   │   ├── queries.rs    ← currentUser resolver
│   │   │   └── schema.rs     ← Schema composition
│   │   └── ...
│   └── Cargo.toml
│
└── next-admin/               ← Next.js Admin Panel
    ├── lib/
    │   ├── graphql/          ← GraphQL client (Apollo/urql)
    │   │   ├── auth.ts       ← signIn, signUp mutations
    │   │   └── users.ts      ← users queries
    │   └── auth/             ← Auth context
    └── ...
```

---

## 💻 Примеры использования

### 1. Login Page (Leptos)

```rust
// apps/admin/src/pages/login.rs
use leptos::*;
use leptos_auth::api;

#[component]
pub fn Login() -> impl IntoView {
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    let navigate = use_navigate();
    
    let login_action = create_action(|_| async move {
        set_error.set(None);
        
        match api::sign_in(
            email.get(),
            password.get(),
            "demo".to_string(), // tenant from config
        ).await {
            Ok((user, session)) => {
                // AuthContext will handle storage automatically
                navigate("/dashboard", Default::default());
            }
            Err(e) => {
                set_error.set(Some(e.to_string()));
            }
        }
    });
    
    view! {
        <form on:submit=|ev| {
            ev.prevent_default();
            login_action.dispatch(());
        }>
            <input 
                type="email"
                value=email
                on:input=move |ev| set_email.set(event_target_value(&ev))
                placeholder="Email"
            />
            <input 
                type="password"
                value=password
                on:input=move |ev| set_password.set(event_target_value(&ev))
                placeholder="Password"
            />
            <button type="submit">"Login"</button>
            
            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}
        </form>
    }
}
```

---

### 2. Users Page (Leptos)

```rust
// apps/admin/src/pages/users.rs
use leptos::*;
use leptos_graphql::{execute, GraphqlRequest, GRAPHQL_ENDPOINT};
use leptos_auth::{use_token, use_tenant};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
struct UsersData {
    users: UsersConnection,
}

#[derive(Deserialize, Clone)]
struct UsersConnection {
    items: Vec<User>,
    total: i32,
}

#[derive(Deserialize, Clone)]
struct User {
    id: String,
    email: String,
    name: Option<String>,
    role: String,
}

const GET_USERS_QUERY: &str = r#"
query GetUsers($limit: Int, $offset: Int) {
    users(limit: $limit, offset: $offset) {
        items {
            id
            email
            name
            role
        }
        total
    }
}
"#;

#[component]
pub fn Users() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();
    
    let users_resource = create_resource(
        move || (token.get(), tenant.get()),
        |(token, tenant)| async move {
            if token.is_none() || tenant.is_none() {
                return Err("Not authenticated".to_string());
            }
            
            let variables = serde_json::json!({
                "limit": 20,
                "offset": 0,
            });
            
            let request = GraphqlRequest::new(GET_USERS_QUERY, Some(variables));
            
            execute::<_, UsersData>(
                GRAPHQL_ENDPOINT,
                request,
                token,
                tenant,
            )
            .await
            .map_err(|e| e.to_string())
        },
    );
    
    view! {
        <div class="users-page">
            <h1>"Users"</h1>
            
            <Suspense fallback=|| view! { <p>"Loading users..."</p> }>
                {move || users_resource.get().map(|result| match result {
                    Ok(data) => view! {
                        <div class="users-list">
                            <p>"Total: " {data.users.total}</p>
                            <ul>
                                {data.users.items.iter().map(|user| view! {
                                    <li>
                                        {&user.email} " - " {&user.role}
                                    </li>
                                }).collect::<Vec<_>>()}
                            </ul>
                        </div>
                    },
                    Err(e) => view! {
                        <p class="error">{e}</p>
                    },
                })}
            </Suspense>
        </div>
    }
}
```

---

### 3. Create User (Leptos)

```rust
// apps/admin/src/pages/users.rs (continued)

const CREATE_USER_MUTATION: &str = r#"
mutation CreateUser($input: CreateUserInput!) {
    createUser(input: $input) {
        id
        email
        name
        role
    }
}
"#;

#[component]
pub fn CreateUserForm() -> impl IntoView {
    let token = use_token();
    let tenant = use_tenant();
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (name, set_name) = create_signal(String::new());
    let (error, set_error) = create_signal(None::<String>);
    
    let create_action = create_action(|_| async move {
        set_error.set(None);
        
        let variables = serde_json::json!({
            "input": {
                "email": email.get(),
                "password": password.get(),
                "name": name.get(),
            }
        });
        
        let request = GraphqlRequest::new(CREATE_USER_MUTATION, Some(variables));
        
        match execute(
            GRAPHQL_ENDPOINT,
            request,
            token.get(),
            tenant.get(),
        ).await {
            Ok(_) => {
                // Success - clear form
                set_email.set(String::new());
                set_password.set(String::new());
                set_name.set(String::new());
            }
            Err(e) => {
                set_error.set(Some(e.to_string()));
            }
        }
    });
    
    view! {
        <form on:submit=|ev| {
            ev.prevent_default();
            create_action.dispatch(());
        }>
            <input 
                type="email"
                value=email
                on:input=move |ev| set_email.set(event_target_value(&ev))
                placeholder="Email"
            />
            <input 
                type="password"
                value=password
                on:input=move |ev| set_password.set(event_target_value(&ev))
                placeholder="Password"
            />
            <input 
                type="text"
                value=name
                on:input=move |ev| set_name.set(event_target_value(&ev))
                placeholder="Name"
            />
            <button type="submit">"Create User"</button>
            
            {move || error.get().map(|e| view! { <p class="error">{e}</p> })}
        </form>
    }
}
```

---

## 📖 Best Practices

### 1. Используйте константы для queries

```rust
// ✅ ПРАВИЛЬНО
const GET_USERS_QUERY: &str = r#"
query GetUsers($limit: Int) {
    users(limit: $limit) {
        items { id email name }
    }
}
"#;

let request = GraphqlRequest::new(GET_USERS_QUERY, Some(variables));
```

```rust
// ❌ НЕПРАВИЛЬНО
let query = format!("query {{ users {{ id email }} }}");
```

### 2. Типизируйте ответы

```rust
// ✅ ПРАВИЛЬНО
#[derive(Deserialize)]
struct UsersData {
    users: UsersConnection,
}

let response: UsersData = execute(...).await?;
```

```rust
// ❌ НЕПРАВИЛЬНО
let response: serde_json::Value = execute(...).await?;
let users = response["users"]["items"].as_array().unwrap(); // паника!
```

### 3. Обрабатывайте ошибки

```rust
// ✅ ПРАВИЛЬНО
match execute(...).await {
    Ok(data) => { /* success */ },
    Err(GraphqlHttpError::Unauthorized) => { /* redirect to login */ },
    Err(GraphqlHttpError::Graphql(msg)) => { /* show error */ },
    Err(_) => { /* network error */ },
}
```

### 4. Используйте Leptos Resources

```rust
// ✅ ПРАВИЛЬНО - реактивность + suspense
let users = create_resource(
    move || (token.get(), tenant.get()),
    |(token, tenant)| async move {
        execute(...).await
    },
);

view! {
    <Suspense fallback=|| view! { <p>"Loading..."</p> }>
        {move || users.get().map(|data| /* render */)}
    </Suspense>
}
```

---

## ✅ Checklist

### Перед отправкой GraphQL запроса

- [ ] **Query/Mutation написан правильно?** (проверьте синтаксис GraphQL)
- [ ] **Используется `leptos-graphql::execute()`?** (а не прямой reqwest)
- [ ] **Добавлен `Authorization: Bearer <token>` header?** (если требуется)
- [ ] **Добавлен `X-Tenant-Slug` header?** (обязательно!)
- [ ] **Типы ответов соответствуют schema?** (используйте struct + Deserialize)
- [ ] **Обработаны GraphQL errors?** (Unauthorized, Graphql, Network)
- [ ] **Используется константа для query?** (а не строковая интерполяция)

---

## 🚀 Backend Requirements

### Нужно реализовать на backend

**Файл:** `apps/server/src/graphql/mutations.rs`

```rust
// Add these mutations to RootMutation

async fn sign_in(
    &self,
    ctx: &Context<'_>,
    email: String,
    password: String,
) -> Result<SignInPayload> {
    let tenant = ctx.data::<TenantContext>()?;
    let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
    
    // 1. Find user by email
    let user = users::Entity::find_by_email(&app_ctx.db, tenant.id, &email)
        .await?
        .ok_or_else(|| FieldError::new("Invalid credentials"))?;
    
    // 2. Verify password
    if !verify_password(&password, &user.password_hash)? {
        return Err(FieldError::new("Invalid credentials"));
    }
    
    // 3. Generate JWT token
    let token = encode_access_token(&user, tenant.id)?;
    
    Ok(SignInPayload {
        token,
        user: User::from(&user),
    })
}

async fn sign_up(
    &self,
    ctx: &Context<'_>,
    email: String,
    password: String,
    name: Option<String>,
) -> Result<SignUpPayload> {
    // Implementation...
}

async fn sign_out(&self, ctx: &Context<'_>) -> Result<bool> {
    // Invalidate token (if using token blacklist)
    Ok(true)
}

async fn refresh_token(&self, ctx: &Context<'_>) -> Result<RefreshTokenPayload> {
    let auth = ctx.data::<AuthContext>()?;
    // Generate new token
    let new_token = encode_access_token(&auth.user, auth.tenant_id)?;
    Ok(RefreshTokenPayload { token: new_token })
}

async fn forgot_password(&self, ctx: &Context<'_>, email: String) -> Result<bool> {
    // Send reset email
    Ok(true)
}

async fn reset_password(
    &self,
    ctx: &Context<'_>,
    token: String,
    new_password: String,
) -> Result<bool> {
    // Validate reset token and update password
    Ok(true)
}
```

**Файл:** `apps/server/src/graphql/queries.rs`

```rust
// Add to RootQuery

async fn current_user(&self, ctx: &Context<'_>) -> Result<User> {
    let auth = ctx.data::<AuthContext>()
        .map_err(|_| FieldError::new("Unauthorized"))?;
    
    Ok(User::from(&auth.user))
}
```

---

## 🔐 Persisted Queries

Для критических admin-операций используется механизм Persisted Queries.
Сервер проверяет наличие `sha256Hash` в `extensions.persistedQuery` для операций `Users` и `User`.

**Файл whitelist:** `apps/server/src/graphql/persisted.rs`

**Зарегистрированные хэши (Leptos admin):**

| Операция | Хэш | Источник |
|----------|------|----------|
| `Users` | `ff1e132e...` | `apps/admin/src/api/queries.rs` |
| `User` | `85f7f7b0...` | `apps/admin/src/api/queries.rs` |

**Как это работает:**

```rust
// Leptos admin отправляет persisted query hash
let request = GraphqlRequest::new(query, Some(variables))
    .with_extensions(persisted_query_extension(sha256_hash));
```

Сервер в `graphql_handler` проверяет:

1. Является ли операция критической (`Users` / `User`)
2. Содержит ли запрос валидный persisted query hash
3. Если хэш отсутствует или невалиден — возвращает ошибку

**Добавление нового хэша:**

1. Вычислить SHA-256 от GraphQL query строки
2. Добавить в `ADMIN_PERSISTED_QUERY_HASHES` в `persisted.rs`
3. Использовать `request_with_persisted()` или `with_extensions()` на клиенте

---

## 🌐 Next.js Admin — Auth Flow

**Приложение:** `apps/next-admin`

**Стек:** Next.js + NextAuth (Credentials provider) + TypeScript

**Ключевые файлы:**

- `src/auth.ts` — конфигурация NextAuth
- `src/lib/auth-api.ts` — GraphQL auth mutations
- `src/lib/graphql.ts` — HTTP-клиент для GraphQL
- `src/middleware.ts` — защита маршрутов
- `src/types/next-auth.d.ts` — расширение типов NextAuth

### Auth Flow

```
┌─────────────────────────────────────────────────────────────┐
│              Next.js Admin (apps/next-admin)                  │
├─────────────────────────────────────────────────────────────┤
│  1. User enters email, password, tenantSlug                  │
│  2. Next.js calls NextAuth signIn('credentials', ...)        │
│     ↓                                                         │
│  3. authorize() → calls auth-api.ts signIn()                 │
│     ↓                                                         │
│  4. graphqlRequest() sends:                                  │
│     POST /api/graphql                                        │
│     Headers: X-Tenant-Slug: <tenantSlug>                     │
│     Body: mutation SignIn($input: SignInInput!) { ... }       │
│     ↓                                                         │
│  5. Server returns AuthPayload:                              │
│     { accessToken, refreshToken, user { ... } }              │
│     ↓                                                         │
│  6. authorize() returns user object with rustokToken         │
│  7. NextAuth JWT callback saves token + role                 │
│  8. Session callback exposes data via useSession()           │
│  9. middleware.ts redirects unauthenticated → /auth/sign-in  │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### GraphQL Client

```typescript
// apps/next-admin/src/lib/graphql.ts
export async function graphqlRequest<V, T>(
  query: string,
  variables?: V,
  token?: string | null,    // → Authorization: Bearer <token>
  tenantSlug?: string | null // → X-Tenant-Slug: <slug>
): Promise<T>
```

### Данные из GraphQL

Next.js admin использует тот же GraphQL endpoint `/api/graphql` что и Leptos admin.
Все данные (пользователи, dashboard, модули) запрашиваются через GraphQL.

```typescript
// Пример: получение текущего пользователя
const CURRENT_USER_QUERY = `
query Me {
  me {
    id email name role status
  }
}
`;
```

---

## 📊 Summary

| Компонент | Назначение | Статус |
|-----------|------------|--------|
| `leptos-graphql` | HTTP transport для GraphQL | ✅ Готов |
| `leptos-auth` | Auth operations через GraphQL | ✅ Готов |
| Backend mutations | signIn, signUp, signOut, refreshToken, forgotPassword, resetPassword | ✅ Реализовано |
| Backend queries | me, users, dashboardStats, recentActivity | ✅ Реализовано |
| Next.js `auth-api.ts` | GraphQL auth client для Next.js admin | ✅ Исправлено |
| Next.js `graphql.ts` | HTTP transport для GraphQL | ✅ Готов |
| Persisted Queries | Whitelist хэшей для критических операций | ✅ Leptos admin |

---

**Статус:** ✅ Реализовано и задокументировано (GraphQL-only для обеих админок)  
**Критичность:** ⚡ Архитектурное ядро — все UI-клиенты зависят от GraphQL endpoint
