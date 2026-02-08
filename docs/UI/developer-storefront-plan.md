# Phase 1 (recovered)

## Принципы

- Мы **не клонируем** библиотеки целиком. Вместо этого делаем **минимальные адаптеры/обёртки** и закрываем пробелы **по мере работы** с админками/витриной.
- Приоритет — **готовые библиотеки и интеграции**; самопис — только если нет адекватного аналога.
- Любые отклонения фиксируем в UI‑документах и матрицах паритета.
- Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

См. базовые источники:
- [UI parity (admin + storefront)](./ui-parity.md)
- [Admin libraries parity](./admin-libraries-parity.md)
- [Admin auth phase 3 scope](./admin-auth-phase3.md)
- [Admin Phase 3 architecture](./admin-phase3-architecture.md)
- [Admin Phase 3 gap analysis](./admin-phase3-gap-analysis.md)
- [Admin template integration plan](./admin-template-integration-plan.md)
- [Admin reuse matrix](./admin-reuse-matrix.md)
- [Tech parity tracker](./tech-parity.md)
- [Storefront overview](./storefront.md)
- [Phase 2.1 — Users vertical slice](./phase2-users-vertical-slice.md)

---

## Phase 1 — чек‑лист (восстановлено по коду)

### Админки (Leptos + Next.js)

Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

| Работа | Leptos | Next |
| --- | --- | --- |
| Базовый layout и навигационный shell админки. | ✅ | ✅ |
| Dashboard/главная админки. | ✅ | ✅ |
| Страницы аутентификации: login / register / reset. | ✅ | ✅ |
| Страница Security. | ✅ | ✅ |
| Страница Profile. | ✅ | ✅ |
| Users list с фильтрами/поиском и пагинацией (REST + GraphQL запросы). | ✅ | ✅ |
| User details (карточка пользователя). | ✅ | ⬜ |
| Auth‑guard (защита приватных маршрутов). | ✅ | ✅ |
| Базовые UI‑примитивы (PageHeader, кнопки, инпуты) в shadcn‑style. | ✅ | ✅ |
| i18n (RU/EN). | ✅ | ✅ |

### Storefront (Leptos SSR + Next.js)

Перед разработкой **проверяем установленные библиотеки** и существующие компоненты, чтобы не писать лишний код.

| Работа | Leptos | Next |
| --- | --- | --- |
| Landing‑shell (hero + CTA + основной layout). | ✅ | ✅ |
| Блоки контента (карточки/фичи/коллекции). | ✅ | ✅ |
| Блоки маркетинга/инфо (alert/статы/история бренда/подписка). | ✅ | ✅ |
| i18n / локализация витрины. | ✅ | ✅ |
| Tailwind‑стили и базовая тема (DaisyUI/shadcn‑style). | ✅ | ✅ |
| SSR‑сервер + отдача CSS‑бандла. | ✅ | ⬜ |

---

## Phase 2.1 — Users vertical slice (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| i18n foundation: ключевые неймспейсы `app/auth/users/errors`. | ⬜ | ⬜ |
| i18n foundation: вынос строк в доменные модули/файлы. | ⬜ | ⬜ |
| i18n foundation: локализация API ошибок (`errors.*`). | ⬜ | ⬜ |
| Auth wiring: `POST /api/auth/login`. | ⬜ | ⬜ |
| Auth wiring: `GET /api/auth/me` (bootstrap). | ⬜ | ⬜ |
| Auth wiring: хранение токена (cookie/localStorage). | ⬜ | ⬜ |
| Users list: GraphQL `users` (pagination). | ⬜ | ⬜ |
| Users list: фильтры и поиск. | ⬜ | ⬜ |
| Users table parity: колонки `email/name/role/status/created_at`. | ⬜ | ⬜ |
| Users detail view: GraphQL `user(id)`. | ⬜ | ⬜ |
| Users CRUD: `createUser` mutation. | ⬜ | ⬜ |
| Users CRUD: `updateUser` mutation. | ⬜ | ⬜ |
| Users CRUD: `disableUser` mutation. | ⬜ | ⬜ |
| Users CRUD: формы, ошибки, тосты. | ⬜ | ⬜ |
| RBAC: права `users.create/users.update/users.manage`. | ⬜ | ⬜ |
| Shared UI/UX: layout/nav parity. | ⬜ | ⬜ |
| Shared UI/UX: breadcrumbs. | ⬜ | ⬜ |
| Shared UI/UX: form patterns. | ⬜ | ⬜ |
| Shared UI/UX: toasts/alerts. | ⬜ | ⬜ |

---

## Phase 3 — Admin Auth & User Security (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| Login page: tenant slug + email + password. | ⬜ | ⬜ |
| Login UX: ошибки/валидация, loading/empty states. | ⬜ | ⬜ |
| Language switch + persistence. | ⬜ | ⬜ |
| Password reset: request email. | ⬜ | ⬜ |
| Password reset: token + new password. | ⬜ | ⬜ |
| Password reset: token expiry UI. | ⬜ | ⬜ |
| Email verification: verify + resend action. | ⬜ | ⬜ |
| Registration: sign‑up (email + password + tenant). | ⬜ | ⬜ |
| Registration: optional name + password strength hints. | ⬜ | ⬜ |
| Invite onboarding: accept invite + expired invite UX. | ⬜ | ⬜ |
| Profile page: name/avatar/timezone/language. | ⬜ | ⬜ |
| Change password: current password + policy hints. | ⬜ | ⬜ |
| Active sessions list + “sign out all”. | ⬜ | ⬜ |
| Login history (success/failed, timestamps/IP). | ⬜ | ⬜ |
| Admin auth middleware/guard (private routes). | ⬜ | ⬜ |
| Token storage + refresh strategy (cookie/localStorage). | ⬜ | ⬜ |
| Logout flow (очистка токена/сессии). | ⬜ | ⬜ |
| RBAC checks for admin-only GraphQL/REST. | ⬜ | ⬜ |
| RU/EN coverage for auth/profile UI + validation. | ⬜ | ⬜ |
| Localized email templates: verify/reset/invite. | ⬜ | ⬜ |
| Admin route map: `/login` `/register` `/reset` `/profile` `/security`. | ⬜ | ⬜ |
| Audit events: login/password change/session invalidation/invite. | ⬜ | ⬜ |

---

## Phase 4 — Интеграция UI‑шаблона для админок (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| Подготовка: зафиксировать цели и scope Phase 3. | ⬜ | ⬜ |
| Инвентаризация шаблона: страницы, layout, компоненты, токены. | ⬜ | ⬜ |
| Инвентаризация текущих админок: маршруты, состояния, формы/таблицы. | ⬜ | ⬜ |
| Согласовать UI‑контракт (layout/sidebar/header, базовые компоненты). | ⬜ | ⬜ |
| Карта соответствий: страницы Template → RusToK. | ⬜ | ⬜ |
| Карта соответствий: компоненты Template → shadcn/ui/internal. | ⬜ | ⬜ |
| Карта токенов: цвета/отступы/типографика → дизайн‑токены. | ⬜ | ⬜ |
| Next.js: установить/синхронизировать зависимости шаблона. | ⬜ | ⬜ |
| Next.js: подключить layout/nav под контракт. | ⬜ | ⬜ |
| Next.js: перенести ключевые страницы (Login/Register/Reset/Profile/Security). | ⬜ | ⬜ |
| Next.js: Users list/details + Dashboard widgets. | ⬜ | ⬜ |
| Next.js: синхронизировать i18n под новые UI блоки. | ⬜ | ⬜ |
| Next.js: подключить API-клиенты и состояния (loading/error/empty). | ⬜ | ⬜ |
| Leptos: создать эквиваленты шаблонных компонентов. | ⬜ | ⬜ |
| Leptos: выровнять layout/nav с Next.js. | ⬜ | ⬜ |
| Leptos: перенести страницы тем же приоритетом. | ⬜ | ⬜ |
| Leptos: синхронизировать i18n. | ⬜ | ⬜ |
| Leptos: подключить API-слой и состояния. | ⬜ | ⬜ |
| Паритет: визуальный parity (Login/Users/Profile). | ⬜ | ⬜ |
| QA: поведение (валидация/ошибки/загрузка). | ⬜ | ⬜ |
| QA: доступность (контраст, фокус, aria). | ⬜ | ⬜ |
| QA: производительность (bundle/рендер). | ⬜ | ⬜ |
| План внедрения: порядок этапов + демонстрации. | ⬜ | ⬜ |
| План отката: фича‑флаг/релизный переключатель. | ⬜ | ⬜ |
| Definition of Done (DoD): паритет, локализация, без регрессий, документация. | ⬜ | ⬜ |
