# RusToK Next.js Storefront

Скелет витрины (storefront) на Next.js (App Router) с DaisyUI, shadcn/ui и локализацией.

## Быстрый старт

```bash
npm install
npm run dev
```

## Основные библиотеки

- Next.js + React
- TailwindCSS + DaisyUI
- shadcn/ui (базовые компоненты)
- React Query
- Zod
- Next SEO
- next-intl (ru/en)

## Расширенный стандартный стек

- Auth: next-auth
- State: zustand
- GraphQL: graphql-request
- REST/OpenAPI: openapi-fetch
- Tables: @tanstack/react-table
- Dates: date-fns
- Analytics: posthog-js
- Monitoring: @sentry/nextjs
- Notifications: react-hot-toast

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.

## Взаимодействие
- apps/server (витринный API)
- crates/rustok-commerce (доменные данные через backend)
- crates/rustok-content (контент через backend)

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Next.js storefront для клиентской витрины и пользовательских сценариев.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - apps/server (витринный API)
  - crates/rustok-commerce и rustok-content через backend
  - общая дизайн-система с apps/admin
- **Точки входа:**
  - `apps/next-frontend/app/*`
  - `apps/next-frontend/components/*`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

