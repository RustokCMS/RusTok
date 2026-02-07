# Developer Storefront Plan — Phase 1 (recovered)

## Принципы (сверху, как договорились)

- Мы **не клонируем** библиотеки целиком. Вместо этого делаем **минимальные адаптеры/обёртки** и закрываем пробелы **по мере работы** с админками/витриной.
- Приоритет — **готовые библиотеки и интеграции**; самопис — только если нет адекватного аналога.
- Любые отклонения фиксируем в UI‑документах и матрицах паритета.

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

| Работа | Leptos | Next |
| --- | --- | --- |
| Базовый layout и навигационный shell админки. | [x] | [x] |
| Dashboard/главная админки. | [x] | [x] |
| Страницы аутентификации: login / register / reset. | [x] | [x] |
| Страница Security. | [x] | [x] |
| Страница Profile. | [x] | [x] |
| Users list с фильтрами/поиском и пагинацией (REST + GraphQL запросы). | [x] | [x] |
| User details (карточка пользователя). | [x] | [ ] |
| Auth‑guard (защита приватных маршрутов). | [x] | [x] |
| Базовые UI‑примитивы (PageHeader, кнопки, инпуты) в shadcn‑style. | [x] | [x] |
| i18n (RU/EN). | [x] | [x] |

### Storefront (Leptos SSR + Next.js)

| Работа | Leptos | Next |
| --- | --- | --- |
| Landing‑shell (hero + CTA + основной layout). | [x] | [x] |
| Блоки контента (карточки/фичи/коллекции). | [x] | [x] |
| Блоки маркетинга/инфо (alert/статы/история бренда/подписка). | [x] | [x] |
| i18n / локализация витрины. | [x] | [x] |
| Tailwind‑стили и базовая тема (DaisyUI/shadcn‑style). | [x] | [x] |
| SSR‑сервер + отдача CSS‑бандла. | [x] | [ ] |

---

## Phase 2.1 — Users vertical slice (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| i18n foundation (ключевые неймспейсы и единые ключи). | [ ] | [ ] |
| Auth wiring (REST: login/me, хранение токена, guard). | [ ] | [ ] |
| Users list + pagination + filters (GraphQL users query). | [ ] | [ ] |
| Users detail view (GraphQL user query). | [ ] | [ ] |
| Users CRUD (create/update/disable + формы и ошибки). | [ ] | [ ] |
| Shared UI/UX (layout/nav, breadcrumbs, toasts, form patterns). | [ ] | [ ] |

---

## Phase 3 — Admin Auth & User Security (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| Admin auth middleware/guard (требовать auth на приватных маршрутах). | [ ] | [ ] |
| Login flow (REST: `POST /api/auth/login`). | [ ] | [ ] |
| Session bootstrap (REST: `GET /api/auth/me`). | [ ] | [ ] |
| Token storage + refresh strategy (cookie/localStorage). | [ ] | [ ] |
| Logout flow (очистка токена/сессии). | [ ] | [ ] |
| Password reset flow (request + confirm). | [ ] | [ ] |
| Security settings screen (пароли, сессии, 2FA placeholder). | [ ] | [ ] |
| RBAC checks for admin-only GraphQL/REST. | [ ] | [ ] |
| Error mapping для auth (errors.*). | [ ] | [ ] |

---

## Phase 4 — Интеграция UI‑шаблона для админок (только работы)

| Работа | Leptos | Next |
| --- | --- | --- |
| Подготовка и аудит: цели, инвентаризация шаблона и текущих админок, UI контракт. | [ ] | [ ] |
| Карта соответствий (Template → RusToK): страницы, компоненты, токены. | [ ] | [ ] |
| Интеграция шаблона в Next.js админку: зависимости, layout/nav, страницы, i18n, API‑состояния. | [ ] | [ ] |
| Интеграция шаблона в Leptos админку: компоненты, layout/nav, страницы, i18n, API‑состояния. | [ ] | [ ] |
| Паритет и QA: визуальный паритет, поведение, доступность, производительность. | [ ] | [ ] |
| План внедрения/отката и DoD. | [ ] | [ ] |
