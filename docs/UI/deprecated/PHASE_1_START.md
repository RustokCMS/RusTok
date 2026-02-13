# Phase 1: Auth + Session Management — Start Report

**Дата:** 2026-02-13  
**Фаза:** 1 из 8  
**Статус:** 🚀 Начало разработки  
**Deadline:** 5-7 дней (Sprint 1)

---

## 📋 Scope Фазы 1

Реализовать полный Auth flow в обеих админках с приоритетом на паритет и переиспользование кода.

**Ключевые задачи:**
1. Sign In / Sign Out
2. Sign Up / Registration  
3. Password Reset Flow
4. Session Management (JWT refresh)
5. User Context / Auth State

---

## 🔍 Initial Audit

### Текущее состояние кодовой базы

**apps/admin (Leptos):**
- ✅ Структура создана
- ✅ Роутинг настроен
- ✅ Страницы существуют: `login.rs`, `register.rs`, `reset.rs`
- ✅ Компонент `ProtectedRoute` существует
- ⚠️ Интеграция с `leptos-auth` требует проверки

**apps/next-admin (Next.js):**
- ✅ Полная структура из starter template
- ✅ TypeScript + Tailwind + shadcn/ui
- ✅ App Router (Next.js 14+)
- ⚠️ Auth pages требуют адаптации под наш backend

**crates/leptos-auth:**
- ✅ Существует (`crates/leptos-auth/src/lib.rs`)
- ⚠️ Содержит только **типы данных** (AuthUser, AuthSession, AuthError)
- ❌ НЕТ хуков (use_auth, use_current_user)
- ❌ НЕТ компонентов (AuthProvider, ProtectedRoute)
- ❌ НЕТ функций (sign_in, sign_out, sign_up)

**Вывод:** `leptos-auth` — это **базовая библиотека типов**, а не полноценное auth решение. Нужно дописать.

---

## 📦 Необходимые библиотеки

### Leptos Stack (что есть / что нужно)

| Библиотека | Статус | Что есть | Что нужно дописать |
|------------|--------|----------|-------------------|
| `leptos-auth` | ⚠️ Частично | Типы данных (AuthUser, AuthSession) | Хуки, компоненты, API функции |
| `leptos-hook-form` | ❓ Не проверено | - | Проверить API |
| `leptos-zod` | ❓ Не проверено | - | Проверить API |
| `leptos-graphql` | ❓ Не проверено | - | Проверить API |
| `leptos-use` | ❓ Не проверено | - | localStorage/sessionStorage |

### Next.js Stack (starter template)

| Библиотека | Статус | Использование |
|------------|--------|---------------|
| `react-hook-form` | ✅ В starter | Формы |
| `zod` | ✅ В starter | Валидация |
| `@tanstack/react-query` | ✅ В starter | Data fetching |
| `zustand` | ✅ В starter | State management |
| `next-themes` | ✅ В starter | Theme switching |

---

## 🎯 План действий (приоритеты)

### Шаг 1: Дописать `leptos-auth` (critical path)

**Задача:** Превратить `leptos-auth` из библиотеки типов в полноценное auth решение.

**Что нужно добавить:**

1. **API functions** (`src/api.rs`):
   ```rust
   pub async fn sign_in(email: String, password: String) -> Result<AuthSession, AuthError>
   pub async fn sign_up(data: SignUpData) -> Result<AuthUser, AuthError>
   pub async fn sign_out() -> Result<(), AuthError>
   pub async fn get_current_user() -> Result<AuthUser, AuthError>
   pub async fn refresh_token() -> Result<AuthSession, AuthError>
   ```

2. **Context** (`src/context.rs`):
   ```rust
   #[derive(Clone)]
   pub struct AuthContext {
       pub user: Signal<Option<AuthUser>>,
       pub session: Signal<Option<AuthSession>>,
       pub is_loading: Signal<bool>,
   }
   
   #[component]
   pub fn AuthProvider(children: Children) -> impl IntoView
   ```

3. **Hooks** (`src/hooks.rs`):
   ```rust
   pub fn use_auth() -> AuthContext
   pub fn use_current_user() -> Signal<Option<AuthUser>>
   pub fn use_session() -> Signal<Option<AuthSession>>
   ```

4. **Components** (`src/components.rs`):
   ```rust
   #[component]
   pub fn ProtectedRoute(
       children: Children,
       fallback: impl Fn() -> View + 'static
   ) -> impl IntoView
   ```

5. **Storage helpers** (`src/storage.rs`):
   ```rust
   pub fn save_session(session: &AuthSession)
   pub fn load_session() -> Option<AuthSession>
   pub fn clear_session()
   ```

**Deadline:** 1-2 дня  
**Priority:** 🔥 CRITICAL (блокирует всю разработку)

---

### Шаг 2: Адаптировать Next.js auth pages

**Задача:** Взять страницы из starter template и адаптировать под наш backend API.

**Файлы для адаптации:**
- `apps/next-admin/src/app/auth/sign-in/page.tsx`
- `apps/next-admin/src/app/auth/sign-up/page.tsx`
- `apps/next-admin/src/app/auth/forgot-password/page.tsx`
- `apps/next-admin/src/app/auth/reset-password/[token]/page.tsx`

**Что изменить:**
- API endpoints → наш backend `/api/auth/*`
- Auth provider → наш JWT-based auth
- Storage → cookies или localStorage для JWT

**Deadline:** 1 день (параллельно с Шагом 1)

---

### Шаг 3: Интегрировать leptos-auth в Leptos Admin

**Задача:** Использовать обновленную `leptos-auth` в `apps/admin`.

**Файлы для обновления:**
- `apps/admin/src/main.rs` — добавить `<AuthProvider>`
- `apps/admin/src/pages/login.rs` — использовать `use_auth()`
- `apps/admin/src/pages/register.rs` — использовать `use_auth()`
- `apps/admin/src/pages/reset.rs` — использовать `use_auth()`
- `apps/admin/src/components/protected_route.rs` — использовать `<ProtectedRoute>` из библиотеки

**Deadline:** 1 день (после Шага 1)

---

### Шаг 4: Обеспечить паритет

**Задача:** Убедиться, что обе админки работают одинаково.

**Checklist:**
- [ ] Одинаковые API endpoints
- [ ] Одинаковая валидация (можно shared Zod schema?)
- [ ] Одинаковый UI/UX (Tailwind классы)
- [ ] Одинаковая обработка ошибок
- [ ] Одинаковый redirect flow

**Deadline:** 1 день (после Шагов 2-3)

---

### Шаг 5: Тестирование и документация

**Задача:** Проверить работоспособность и обновить документацию.

**Checklist:**
- [ ] Manual testing обеих админок
- [ ] Обновить `FRONTEND_DEVELOPMENT_LOG.md`
- [ ] Обновить `PROGRESS_SUMMARY.md`
- [ ] Обновить `admin-libraries-parity.md` (статус leptos-auth)
- [ ] Создать `crates/leptos-auth/README.md` с API документацией

**Deadline:** 1 день

---

## 📊 Estimated Timeline

| Шаг | Задача | Дни | Зависимости |
|-----|--------|-----|-------------|
| 1 | Дописать `leptos-auth` | 1-2 | - |
| 2 | Адаптировать Next.js auth | 1 | - |
| 3 | Интегрировать в Leptos | 1 | Шаг 1 |
| 4 | Обеспечить паритет | 1 | Шаги 2-3 |
| 5 | Тестирование + docs | 1 | Шаги 1-4 |
| **Total** | **5-6 дней** | - | - |

**Буфер:** 1 день для неожиданных проблем  
**Итого:** 6-7 дней (укладываемся в Sprint 1)

---

## 🚨 Risks & Mitigation

### Risk 1: leptos-auth требует больше времени чем ожидалось

**Вероятность:** Средняя  
**Impact:** Высокий (блокирует всю фазу)

**Mitigation:**
- Начинаем с leptos-auth в первую очередь
- Если застреваем >2 дней — создаем temporary workaround в `apps/admin/src/auth/`
- Параллельно продолжаем работу с Next.js

### Risk 2: API endpoints нашего backend не готовы

**Вероятность:** Низкая (предполагаем, что backend готов)  
**Impact:** Высокий

**Mitigation:**
- Проверить `/api/auth/*` endpoints в первый день
- Если не готовы — создать issue и использовать mock API
- После готовности backend — заменить mock на real API

### Risk 3: Сложности с паритетом между Next.js и Leptos

**Вероятность:** Средняя  
**Impact:** Средний

**Mitigation:**
- Использовать Tailwind классы (одинаковые в обоих фреймворках)
- Копировать UI structure из Next.js в Leptos
- Создавать parity-gap issues для отслеживания

---

## 📝 Next Steps

**Immediate (сегодня):**
1. ✅ Создать `FRONTEND_DEVELOPMENT_LOG.md`
2. ✅ Создать `PHASE_1_START.md` (этот документ)
3. ⬜ Commit изменений
4. ⬜ Начать работу над `leptos-auth`:
   - Создать план структуры (`src/api.rs`, `src/context.rs`, etc.)
   - Проверить зависимости (leptos-use для localStorage)
   - Начать реализацию API functions

**Tomorrow:**
- Продолжить `leptos-auth`
- Параллельно адаптировать Next.js auth pages
- Проверить backend API endpoints

---

## 📚 Документация для обновления

**После завершения Фазы 1:**
- [ ] `FRONTEND_DEVELOPMENT_LOG.md` — обновить progress table
- [ ] `PROGRESS_SUMMARY.md` — отметить Phase 1 как завершенную
- [ ] `admin-libraries-parity.md` — обновить статус `leptos-auth`: ⚠️ → ✅
- [ ] `crates/leptos-auth/README.md` — создать с API документацией
- [ ] `PHASE_1_COMPLETE.md` — создать отчет о завершении

---

**Версия:** 1.0  
**Автор:** CTO Agent  
**Статус:** 🚀 Ready to start
