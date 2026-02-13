# RusToK Admin (Leptos)

Админ-панель на Leptos (CSR) для управления контентом и настройками RusToK.

## Документация

- [Admin auth phase 3 scope](../../docs/UI/admin-auth-phase3.md)
- [Admin Phase 3 architecture](../../docs/UI/admin-phase3-architecture.md)
- [Admin Phase 3 gap analysis](../../docs/UI/admin-phase3-gap-analysis.md)
- [UI parity (admin + storefront)](../../docs/UI/ui-parity.md)
- [Tech parity tracker](../../docs/UI/tech-parity.md)
- [Template integration plan](../../docs/UI/admin-template-integration-plan.md)
- [Admin libraries parity](../../docs/UI/admin-libraries-parity.md)

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.

## Взаимодействие
- apps/server (HTTP/GraphQL API)
- crates/rustok-core (контракты и события через API-слой)
- crates/rustok-rbac (авторизация/права через backend)

## Паспорт компонента
- **Роль в системе:** Админ-приложение для управления RusToK (контент, каталог, настройки, пользователи).
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - apps/server (REST/GraphQL API)
  - crates/leptos-* (UI/формы/таблицы)
  - crates/rustok-rbac (модель прав через backend)
- **Точки входа:**
  - `apps/admin/src/main.rs`
  - `apps/admin/src/pages/*`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`



## Библиотеки фронтенда и их роль (чтобы не потерять контекст)

### UI и приложение
- `leptos`, `leptos_router` — UI-компоненты, маршрутизация и реактивность.
- `tailwindcss` (+ PostCSS/Autoprefixer) — дизайн-система и стили.

### API и интеграции
- `leptos-graphql` (внутренний crate) — тонкий GraphQL transport/utils слой: request shape, persisted query extensions, auth/tenant headers, error mapping.
- `reqwest` — HTTP клиент под капотом transport-слоя и REST-вызовов.
- `graphql-client` (рекомендуемый опциональный слой) — codegen типизированных запросов/ответов; используем по мере подключения typed `.graphql` flow.

### Управление async-состоянием
- `Resource`/actions Leptos в страницах (`apps/admin/src/pages/*`) — loading/error/data lifecycle без отдельного Apollo-like runtime.

### Почему так
- Мы сознательно используем battle-tested библиотеки (`reqwest`, при необходимости `graphql-client`) и держим свой слой минимальным (`leptos-graphql`),
  чтобы не разрастать самописный монолитный GraphQL клиент.
