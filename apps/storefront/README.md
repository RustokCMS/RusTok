# RusToK Leptos Storefront

`apps/storefront` — Leptos SSR витрина RusToK (Rust-first вариант storefront).

## Роль в платформе

- SSR storefront для витринных сценариев;
- параллельная реализация к `apps/next-frontend` для технологического паритета;
- проверка Rust UI/SSR пайплайна в единой платформе.

## Архитектурный контур

- entrypoint: `src/main.rs`
- модульные расширения витрины: `src/modules/*` (registry/slots)
- стили: Tailwind + статическая сборка `static/app.css`
- FSD не применяется (single-file storefront), но модульная расширяемость через slot-систему

## Соглашения об именовании (Naming Conventions)

В проекте приняты следующие соглашения для обеспечения чистоты кода и соблюдения стандартов Rust:

- **Компоненты (функции)**: Все Leptos-компоненты именуются в `snake_case` (например, `storefront_shell`, `product_card`). Использование `PascalCase` для функций-компонентов не рекомендуется.
- **Shared UI**: Общие UI-компоненты в `shared/ui/` (если появятся) имеют префикс `ui_` (например, `ui_button`).
- **Модули**: Компоненты в `src/modules/` также используют `snake_case`.

## Библиотеки

### Ядро

- `leptos`, `leptos_router` — UI и SSR-рендеринг (чистый SSR, без hydration)
- `axum`, `tokio` — HTTP сервер

### i18n

- `leptos_i18n` 0.6 (feature `ssr`) — compile-time многоязычность через `t_string!()` макрос;
- `leptos_i18n_build` — кодогенерация i18n-модуля из `locales/*.json` через `build.rs`;
- файлы локалей: `locales/en.json`, `locales/ru.json`;
- выбор языка: query-параметр `?lang=ru`.

### Внутренние crates

- `leptos-auth`, `leptos-graphql` — auth/GraphQL контракты
- `leptos-table`, `leptos-hook-form`, `leptos-zod`, `leptos-zustand` — формы/состояние
- `leptos-shadcn-pagination`, `leptos-next-metadata`, `leptos_query` — UI-утилиты

## Взаимодействие

- `apps/server` (API)
- доменные `crates/rustok-*` через backend
- общий UI-контур с `apps/admin` / `apps/next-frontend`

## Документация

- Платформенный контекст: `docs/UI/storefront.md`
- Общая карта: `docs/index.md`
