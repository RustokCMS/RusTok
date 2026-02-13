# leptos-auth Integration Guide

**Дата:** 2026-02-13  
**Статус:** ✅ **IN PROGRESS** (Phase 1)  
**Цель:** Интеграция библиотеки `leptos-auth` в `apps/admin`

---

## 📋 Обзор изменений

### Что было сделано

1. ✅ **Обновлён `leptos-auth/api.rs`** — переход с `web-sys::fetch` на `reqwest`
   - Причина: `apps/admin` уже использует `reqwest` для всех HTTP вызовов
   - Консистентность кодовой базы
   - Добавлена поддержка `X-Tenant-Slug` header

2. ✅ **Обновлён `leptos-auth/Cargo.toml`**
   - Добавлен `reqwest` в зависимости
   - Убраны `wasm-bindgen`, `serde-wasm-bindgen`, лишние `web-sys` features

3. ✅ **Создан compatibility layer** — `apps/admin/src/providers/auth_new.rs`
   - Wrapper для постепенной миграции
   - `LegacyAuthContext` для совместимости с существующим кодом
   - Re-exports всех компонентов из `leptos-auth`

4. ✅ **Обновлён `apps/admin/src/app.rs`**
   - Добавлен `<AuthProvider>` wrapper вокруг Router
   - Старый `provide_auth_context()` оставлен для совместимости

---

## 🏗️ Архитектура интеграции

### Текущая структура (2 параллельных провайдера)

```
App
 ├─ provide_auth_context() [СТАРЫЙ] ← src/providers/auth.rs
 └─ <AuthProvider> [НОВЫЙ]           ← leptos-auth library
     └─ Router
         └─ Routes
             ├─ Login (использует СТАРЫЙ)
             ├─ Register (использует СТАРЫЙ)
             └─ ProtectedRoute (использует СТАРЫЙ)
                 └─ Dashboard, Profile, etc.
```

**Проблема:** Два провайдера работают независимо, нужна синхронизация или миграция.

---

### Целевая структура (после миграции)

```
App
 └─ <AuthProvider> [НОВЫЙ] ← только leptos-auth
     └─ Router
         └─ Routes
             ├─ <GuestRoute> (обёртка для Login)
             ├─ <GuestRoute> (обёртка для Register)
             └─ <ProtectedRoute> (из leptos-auth)
                 └─ Dashboard, Profile, etc.
```

---

## 🔄 План миграции (поэтапный)

### Фаза 1: Подготовка (✅ DONE)

- [x] Обновить `leptos-auth` под `reqwest`
- [x] Добавить `<AuthProvider>` в `app.rs`
- [x] Создать compatibility layer `auth_new.rs`

---

### Фаза 2: Миграция Login page (⬜ TODO)

**Файл:** `apps/admin/src/pages/login.rs`

**Текущий код:**
```rust
use crate::providers::auth::{use_auth, User};

let auth = use_auth();  // старый контекст
auth.set_token.set(Some(token));
auth.set_user.set(Some(user));
auth.set_tenant_slug.set(Some(tenant));
```

**Новый код:**
```rust
use crate::providers::auth_new::{use_leptos_auth, AuthError};

let auth = use_leptos_auth();  // новый контекст

spawn_local(async move {
    match auth.sign_in(email, password, tenant).await {
        Ok(_) => {
            // Session/user automatically saved to localStorage
            navigate("/dashboard", Default::default());
        }
        Err(AuthError::InvalidCredentials) => {
            set_error.set(Some("Invalid credentials"));
        }
        Err(_) => {
            set_error.set(Some("Network error"));
        }
    }
});
```

**Преимущества:**
- ✅ Меньше кода (не нужно вручную set_token, set_user, set_tenant_slug)
- ✅ Typed errors (`AuthError::InvalidCredentials` вместо HTTP status)
- ✅ Автоматическое сохранение в localStorage

**Изменения:**
1. Удалить `rest_post` вызов
2. Использовать `auth.sign_in()` из библиотеки
3. Убрать ручное сохранение в context signals

---

### Фаза 3: Миграция Register page (⬜ TODO)

**Файл:** `apps/admin/src/pages/register.rs`

**Аналогично Login:**
```rust
let auth = use_leptos_auth();

spawn_local(async move {
    match auth.sign_up(email, password, Some(name), tenant).await {
        Ok(_) => navigate("/dashboard", Default::default()),
        Err(e) => set_error.set(Some(format!("{:?}", e))),
    }
});
```

---

### Фаза 4: Миграция ProtectedRoute (⬜ TODO)

**Файл:** `apps/admin/src/components/protected_route.rs`

**Текущий код:**
```rust
use crate::providers::auth::use_auth;

let auth = use_auth();
if auth.token.get().is_none() {
    navigate("/login", Default::default());
}
```

**Новый код (Option A — использовать библиотечный компонент):**

В `app.rs` заменить:
```rust
<ParentRoute path=path!("") view=ProtectedRoute>
```

На:
```rust
use crate::providers::auth_new::LeptosProtectedRoute;

<ParentRoute path=path!("") view=LeptosProtectedRoute>
```

**Новый код (Option B — обновить существующий компонент):**
```rust
use crate::providers::auth_new::use_is_authenticated;

let is_authenticated = use_is_authenticated();

Effect::new(move |_| {
    if !is_authenticated.get() {
        navigate("/login", Default::default());
    }
});
```

**Рекомендация:** Option B (обновить существующий), так как там уже используется `<Outlet />` для nested routes.

---

### Фаза 5: Миграция остальных страниц (⬜ TODO)

**Файлы:**
- `apps/admin/src/pages/reset.rs` — забыл пароль / reset
- `apps/admin/src/pages/profile.rs` — использует `auth.user`
- `apps/admin/src/pages/security.rs` — использует `auth.user`
- `apps/admin/src/pages/users.rs` — использует `auth.token` для API вызовов
- `apps/admin/src/pages/dashboard.rs` — использует `auth.user`

**Pattern для миграции:**

Старый код:
```rust
use crate::providers::auth::use_auth;
let auth = use_auth();
let user = auth.user.get();
let token = auth.token.get();
```

Новый код:
```rust
use crate::providers::auth_new::{use_current_user, use_token};
let user = use_current_user();
let token = use_token();
```

**Преимущества:**
- ✅ Более гранулярные subscriptions (только нужные данные)
- ✅ Меньше re-renders (Signal::derive оптимизирован)

---

### Фаза 6: Cleanup (⬜ TODO)

После миграции всех страниц:

1. ✅ Удалить `apps/admin/src/providers/auth.rs`
2. ✅ Переименовать `auth_new.rs` → `auth.rs`
3. ✅ Удалить `provide_auth_context()` вызов из `app.rs`
4. ✅ Удалить старый `components/protected_route.rs` (если используем библиотечный)
5. ✅ Обновить imports по всему проекту

---

## 📦 API Mapping (старое → новое)

| Старый API | Новый API | Комментарий |
|------------|-----------|-------------|
| `use_auth()` | `use_leptos_auth()` | Возвращает `AuthContext` |
| `auth.user.get()` | `use_current_user().get()` | Reactive signal |
| `auth.token.get()` | `use_token().get()` | Reactive signal |
| `auth.tenant_slug.get()` | `use_tenant().get()` | Reactive signal |
| `auth.set_token.set(Some(t))` | `auth.sign_in(...).await` | Метод вместо ручного set |
| `auth.set_user.set(Some(u))` | Автоматически в `sign_in()` | Не нужно вручную |
| `rest_post("/api/auth/login", ...)` | `auth.sign_in(email, pwd, tenant)` | Typed API |
| `LocalStorage::set("rustok-admin-token", ...)` | Автоматически | Библиотека сама сохраняет |
| `if auth.token.get().is_some()` | `use_is_authenticated().get()` | Более семантично |

---

## 🔧 Response Format Compatibility

### Backend Response (текущий)

```json
{
  "access_token": "eyJhbGc...",
  "user": {
    "id": "123",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "admin"
  }
}
```

### leptos-auth ожидает

```rust
struct SignInResponse {
    #[serde(rename = "access_token")]
    pub token: String,  // ← переименован в token
    pub user: AuthUser,
}
```

**Вывод:** ✅ Совместимо! Используем `#[serde(rename = "access_token")]`

---

## 🧪 Testing Plan

### Manual Testing Checklist

**Login Flow:**
- [ ] Войти с валидными credentials → успех
- [ ] Войти с невалидными credentials → показать ошибку
- [ ] После входа → redirect на `/dashboard`
- [ ] Проверить localStorage → token/user/tenant сохранены
- [ ] Refresh страницы → сессия восстановлена

**Protected Routes:**
- [ ] Открыть `/dashboard` без auth → redirect на `/login`
- [ ] Войти → открыть `/dashboard` → показать контент
- [ ] Sign out → redirect на `/login`
- [ ] Проверить localStorage → всё очищено

**Register Flow:**
- [ ] Зарегистрировать нового пользователя
- [ ] После регистрации → auto sign-in → redirect на `/dashboard`

**Reset Password:**
- [ ] Запросить reset → показать success
- [ ] Ввести новый пароль → показать success

---

## 📊 Метрики миграции

| Метрика | До | После |
|---------|-----|-------|
| **Строк кода (auth logic)** | ~200 | ~50 |
| **API вызовов вручную** | 3-4 на страницу | 0 (библиотека) |
| **localStorage вызовов** | 6-8 на страницу | 0 (автоматически) |
| **Type safety** | Partial (status codes) | Full (AuthError enum) |
| **Reactive subscriptions** | Manual signals | Optimized hooks |

---

## 🚨 Потенциальные проблемы

### 1. Конфликт двух провайдеров

**Проблема:** Сейчас работают два независимых auth контекста.

**Решение:** После миграции страниц удалить старый `provide_auth_context()`.

---

### 2. API URL mismatch

**Проблема:**
- Старый код: `REST_API_URL = "http://localhost:3000"`
- Новый код: `API_BASE = "/api/auth"` (relative URL)

**Решение:**
- ✅ Relative URLs работают лучше (поддержка разных environments)
- ✅ В production работает из коробки (нет hardcode localhost)

---

### 3. Tenant header

**Проблема:** Старый код использует `X-Tenant-Slug` header.

**Решение:**
- ✅ Новый `api.rs` тоже использует `X-Tenant-Slug`
- ✅ Совместимость гарантирована

---

### 4. Token format

**Проблема:** Backend возвращает `access_token`, а не `token`.

**Решение:**
- ✅ Используем `#[serde(rename = "access_token")]`
- ✅ Совместимость гарантирована

---

## 🎯 Next Steps

### Immediate (текущая сессия)

1. ✅ Finish integration основы (DONE)
2. ⬜ Мигрировать Login page
3. ⬜ Мигрировать Register page
4. ⬜ Мигрировать ProtectedRoute
5. ⬜ Manual testing

### Short-term (1-2 дня)

6. ⬜ Мигрировать остальные страницы
7. ⬜ Cleanup старого кода
8. ⬜ Обновить documentation

### Long-term (1 неделя)

9. ⬜ Добавить unit tests
10. ⬜ Добавить integration tests
11. ⬜ Performance optimization

---

## 📝 Code Review Checklist

При review PR проверить:

- [ ] Все страницы используют новый API
- [ ] Старые auth providers удалены
- [ ] Нет дублирования кода
- [ ] localStorage keys совпадают
- [ ] Error handling корректный
- [ ] Typed errors используются
- [ ] Tests проходят (если есть)
- [ ] Manual testing выполнен

---

## 🔗 Related Documents

- [leptos-auth README](../../crates/leptos-auth/README.md)
- [leptos-auth Implementation Report](./LEPTOS_AUTH_IMPLEMENTATION.md)
- [Phase 1 Plan](./PHASE_1_START.md)

---

**Status:** 🚧 In Progress  
**Phase:** 1 of 8  
**Completion:** ~40% (integration started, pages not migrated yet)
