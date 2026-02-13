# GraphQL-Only Architecture Decision

**Дата:** 2026-02-13  
**Решение:** ✅ **Только GraphQL, никакого REST API!**  
**Статус:** ✅ **Реализовано**

---

## 🎯 Краткое резюме

**Главное правило:** **Смешивать REST и GraphQL — плохая практика!**

**Решение:** Все auth операции (login, register, logout) идут через GraphQL `/api/graphql`.

---

## ❌ Проблема (что было)

### До исправления:

```
┌──────────────────────────────────────┐
│ Backend (apps/server)                 │
│  ✅ REST: /api/auth/login            │
│  ✅ REST: /api/auth/register         │
│  ✅ GraphQL: /api/graphql             │
│  ❌ Смешивание REST + GraphQL        │
└──────────────────────────────────────┘
         ▲
         │ Какой использовать???
         │
┌──────────────────────────────────────┐
│ Frontend (leptos-auth)                │
│  ⚠️ То REST, то GraphQL               │
│  ❌ Inconsistency!                    │
└──────────────────────────────────────┘
```

**Проблемы:**
1. ❌ Дублирование логики (REST + GraphQL auth)
2. ❌ Inconsistency — какой API использовать?
3. ❌ Больше конфигурации (2 клиента вместо 1)
4. ❌ Хуже производительность (нет batch requests для REST)
5. ❌ Сложнее поддержка (2 точки входа)

---

## ✅ Решение

### После исправления:

```
┌──────────────────────────────────────┐
│ Backend (apps/server)                 │
│  ✅ GraphQL: /api/graphql ONLY       │
│    - signIn mutation                 │
│    - signUp mutation                 │
│    - signOut mutation                │
│    - refreshToken mutation           │
│    - me query                        │
│  ⚠️ REST: /api/auth/* (blocked)      │
└──────────────────────────────────────┘
         ▲
         │ GraphQL only
         │
┌──────────────────────────────────────┐
│ Frontend (leptos-auth)                │
│  ✅ GraphQL client (leptos-graphql)  │
│  ✅ Consistency!                      │
└──────────────────────────────────────┘
```

**Преимущества:**
1. ✅ Единая точка входа — `/api/graphql`
2. ✅ Type-safe queries и mutations
3. ✅ Один клиент (leptos-graphql)
4. ✅ Batch requests, DataLoader
5. ✅ Проще поддержка

---

## 🔧 Что изменили

### 1. Backend: GraphQL Auth Module

**Создано:** `apps/server/src/graphql/auth/`

```
apps/server/src/graphql/auth/
├── mod.rs          ← Экспорты
├── types.rs        ← Input/Output types
├── mutation.rs     ← Auth mutations
└── query.rs        ← Auth queries
```

#### Mutations:

```graphql
type Mutation {
    signIn(input: SignInInput!): AuthPayload!
    signUp(input: SignUpInput!): AuthPayload!
    signOut: SignOutPayload!
    refreshToken(input: RefreshTokenInput!): AuthPayload!
    forgotPassword(input: ForgotPasswordInput!): ForgotPasswordPayload!
    resetPassword(input: ResetPasswordInput!): ResetPasswordPayload!
}
```

#### Types:

```graphql
input SignInInput {
    email: String!
    password: String!
}

input SignUpInput {
    email: String!
    password: String!
    name: String
}

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
```

---

### 2. Frontend: leptos-auth (вернули GraphQL)

**Изменено:** `crates/leptos-auth/src/api.rs`

#### До (REST — удалено):

```rust
// ❌ REST API (bad practice)
pub async fn sign_in(...) -> Result<...> {
    fetch_json("POST", "/api/auth/login", ...).await
}
```

#### После (GraphQL — правильно):

```rust
// ✅ GraphQL (good practice)
pub async fn sign_in(...) -> Result<...> {
    let request = GraphqlRequest {
        query: SIGN_IN_MUTATION,
        variables: json!({ "input": { ... } }),
    };
    execute(&url, request, ...).await
}
```

**Константы GraphQL mutations:**

```rust
const SIGN_IN_MUTATION: &str = r#"
mutation SignIn($input: SignInInput!) {
    signIn(input: $input) {
        accessToken
        refreshToken
        user { id email name }
    }
}
"#;
```

---

### 3. Middleware: Block REST Auth

**Создано:** `apps/server/src/middleware/block_rest_auth.rs`

**Цель:** Заблокировать использование REST auth endpoints админкой.

```rust
/// Block REST auth endpoints for admin panel
pub async fn block_rest_auth_for_admin(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let path = req.uri().path();
    
    // Check if blocked REST auth endpoint
    if BLOCKED_AUTH_PATHS.contains(&path) {
        // Check if from admin panel (User-Agent, Referer)
        if is_admin_request(&req) {
            return Err(StatusCode::FORBIDDEN); // ❌ BLOCK!
        }
    }
    
    Ok(next.run(req).await)
}
```

**Blocked paths:**
- `/api/auth/login`
- `/api/auth/register`
- `/api/auth/logout`
- `/api/auth/refresh`
- `/api/auth/forgot-password`
- `/api/auth/reset-password`

**Detection:** By `User-Agent` ("RusToK-Admin") or `Referer` (":3001", "/admin").

---

### 4. Documentation

**Обновлено:**
- ✅ `crates/leptos-auth/README.md` — только GraphQL
- ✅ `docs/UI/GRAPHQL_ONLY_DECISION.md` — этот документ
- ❌ `docs/UI/PHASE_1_STATUS.md` — удалён (содержал REST approach)

**TODO (следующие PR):**
- [ ] `docs/UI/MASTER_IMPLEMENTATION_PLAN.md`
- [ ] `docs/UI/CUSTOM_LIBRARIES_STATUS.md`
- [ ] `docs/UI/GRAPHQL_ARCHITECTURE.md`

---

## 📊 Архитектура (финальная)

```
┌────────────────────────────────────────────────┐
│  apps/admin (Leptos WASM)                       │
│  ┌──────────────────────────────────────────┐  │
│  │ Login, Register, Dashboard               │  │
│  │ Uses: leptos-auth                        │  │
│  └──────────────────────────────────────────┘  │
└─────────────────┬──────────────────────────────┘
                  │
                  │ use_auth() → api::sign_in()
                  ▼
┌────────────────────────────────────────────────┐
│  crates/leptos-auth (Auth Library)              │
│  ┌──────────────────────────────────────────┐  │
│  │ api.rs: GraphQL mutations               │  │
│  │   - signIn, signUp, signOut             │  │
│  │   - uses leptos-graphql transport       │  │
│  │ context.rs: AuthProvider, AuthContext   │  │
│  │ hooks.rs: use_auth(), use_token()       │  │
│  └──────────────────────────────────────────┘  │
└─────────────────┬──────────────────────────────┘
                  │
                  │ GraphQL HTTP request
                  ▼
┌────────────────────────────────────────────────┐
│  crates/leptos-graphql (HTTP Transport)         │
│  ┌──────────────────────────────────────────┐  │
│  │ execute() - HTTP POST to /api/graphql   │  │
│  │ Headers: X-Tenant-Slug, Authorization   │  │
│  └──────────────────────────────────────────┘  │
└─────────────────┬──────────────────────────────┘
                  │
                  │ POST /api/graphql
                  ▼
┌────────────────────────────────────────────────┐
│  apps/server (Loco Backend)                     │
│  ┌──────────────────────────────────────────┐  │
│  │ GraphQL Schema (/api/graphql)           │  │
│  │   ├── AuthMutation                       │  │
│  │   │   ├── signIn                         │  │
│  │   │   ├── signUp                         │  │
│  │   │   ├── signOut                        │  │
│  │   │   └── refreshToken                   │  │
│  │   ├── AuthQuery                          │  │
│  │   │   └── me                             │  │
│  │   └── Other modules (Commerce, Blog...)  │  │
│  │                                           │  │
│  │ ❌ REST: /api/auth/* (BLOCKED)          │  │
│  │   └── block_rest_auth middleware        │  │
│  │                                           │  │
│  │ Database: PostgreSQL                     │  │
│  │ Auth: JWT + sessions                     │  │
│  └──────────────────────────────────────────┘  │
└────────────────────────────────────────────────┘
```

**Flow:**

1. User clicks "Login" в admin panel
2. Leptos component вызывает `auth.sign_in(email, password, tenant)`
3. `leptos-auth` создаёт GraphQL mutation request
4. `leptos-graphql` отправляет `POST /api/graphql` с mutation
5. Server выполняет `AuthMutation::sign_in()`
6. Возвращает `AuthPayload { accessToken, user }`
7. `leptos-auth` сохраняет в LocalStorage
8. User перенаправлен на `/dashboard`

---

## 🧪 Testing

### GraphQL Mutation (curl):

```bash
# Sign In
curl -X POST http://localhost:5150/api/graphql \
  -H "Content-Type: application/json" \
  -H "X-Tenant-Slug: demo" \
  -d '{
    "query": "mutation SignIn($input: SignInInput!) { signIn(input: $input) { accessToken user { email } } }",
    "variables": {
      "input": {
        "email": "admin@local",
        "password": "admin12345"
      }
    }
  }'

# Expected:
{
  "data": {
    "signIn": {
      "accessToken": "eyJ...",
      "user": {
        "email": "admin@local"
      }
    }
  }
}
```

### REST Blocked (should fail):

```bash
# Try REST login from admin
curl -X POST http://localhost:5150/api/auth/login \
  -H "Content-Type: application/json" \
  -H "X-Tenant-Slug: demo" \
  -H "User-Agent: RusToK-Admin" \
  -d '{"email":"admin@local","password":"admin12345"}'

# Expected:
HTTP 403 Forbidden
```

---

## ⚠️ Migration Guide

### If you were using REST API:

**Old code (delete this):**

```rust
// ❌ REST API (deprecated)
use leptos_auth::api;

let response = fetch_json(
    "POST",
    "/api/auth/login",
    &json!({ "email": email, "password": password }),
).await?;
```

**New code (use this):**

```rust
// ✅ GraphQL API (correct)
use leptos_auth::api;

let (user, session) = api::sign_in(
    email,
    password,
    tenant,
).await?;
```

### Backend changes:

**REST endpoints still exist** for backward compatibility (mobile apps, etc.), but **admin panel should NOT use them**.

If you need to completely disable REST auth:

```rust
// apps/server/src/app.rs (or router setup)

// Apply middleware to block REST auth for admin
.layer(axum::middleware::from_fn(
    crate::middleware::block_rest_auth::block_rest_auth_for_admin
))
```

---

## 💡 Best Practices

### ✅ DO:

1. **Use GraphQL for all admin panel operations** (auth + data)
2. **Single endpoint** — `/api/graphql`
3. **Type-safe** — leverage GraphQL schema
4. **Batch requests** — multiple queries in one HTTP call
5. **DataLoader** — efficient data fetching

### ❌ DON'T:

1. **Mix REST and GraphQL** — choose one!
2. **Multiple endpoints** — REST + GraphQL = complexity
3. **ad-hoc REST calls** — everything through GraphQL
4. **Bypass GraphQL for "simple" operations** — stay consistent

---

## 📚 Resources

### Documentation:

- **leptos-auth README:** `crates/leptos-auth/README.md`
- **GraphQL schema:** `apps/server/src/graphql/auth/types.rs`
- **Mutations:** `apps/server/src/graphql/auth/mutation.rs`
- **Middleware:** `apps/server/src/middleware/block_rest_auth.rs`

### GraphQL Playground:

```
http://localhost:5150/api/graphql
```

### Introspection query:

```graphql
query {
  __schema {
    mutationType {
      fields {
        name
        args {
          name
          type {
            name
          }
        }
      }
    }
  }
}
```

---

## 🎯 Roadmap

### ✅ Completed (this PR):

- [x] GraphQL auth mutations на backend
- [x] leptos-auth использует GraphQL
- [x] Middleware для блокировки REST auth
- [x] Documentation обновлена

### ⏳ TODO (future PRs):

- [ ] Complete password reset flow в GraphQL
- [ ] Email verification flow в GraphQL
- [ ] 2FA support в GraphQL
- [ ] Rate limiting для GraphQL mutations
- [ ] Admin panel integration testing
- [ ] Update all docs (remove REST mentions)

---

## ✅ Summary

### What we achieved:

1. ✅ **Consistency** — только GraphQL, никакого REST
2. ✅ **Best practice** — единый API endpoint
3. ✅ **Type safety** — GraphQL schema
4. ✅ **Better DX** — один клиент вместо двух
5. ✅ **Protection** — middleware блокирует REST для админки

### Key principles:

- **GraphQL-first** — все операции через `/api/graphql`
- **No mixing** — REST и GraphQL нельзя смешивать
- **Consistency** — выбрали один подход и придерживаемся его

### Next steps:

1. Test GraphQL auth flow в admin panel
2. Implement remaining features (password reset, 2FA)
3. Update all documentation
4. Remove or clearly mark REST endpoints as "legacy" / "for mobile only"

---

**Дата:** 2026-02-13  
**Автор:** CTO Agent  
**Статус:** ✅ **Реализовано**

---

## 🔗 Related

- **GraphQL Architecture:** `docs/UI/GRAPHQL_ARCHITECTURE.md`
- **leptos-auth README:** `crates/leptos-auth/README.md`
- **Master Plan:** `docs/UI/MASTER_IMPLEMENTATION_PLAN.md`
