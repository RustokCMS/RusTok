# Dual UI Strategy: Leptos (Primary) + Next.js (Batteries Included)

- Date: 2026-03-17
- Status: Accepted

## Context

RusTok поддерживает два UI стека:
- **Leptos** — primary, компилируется в единый бинарник с Rust backend
- **Next.js** — secondary, для JS-разработчиков, знакомых с React-экосистемой

Изначально Next.js UI для модулей (`@rustok/blog-admin`, `@rustok/workflow-admin`,
`@rustok/blog-frontend`) хранился как отдельные npm-пакеты внутри крейтов
(`crates/rustok-blog/ui/admin/`, `crates/rustok-workflow/ui/admin/`, и т.д.).

## Проблема

Подход с npm-пакетами в крейтах создавал ложное ощущение "marketplace-auto-install"
для Next.js, которого на самом деле нет — Next.js требует ручного npm install + rebuild.
При этом:
- Код жил в `crates/` среди Rust-кода (путаница)
- Для добавления модуля нужно было менять два места (package.json + modules/index.ts)
- Разрыв между "как выглядит" (отдельный пакет) и "как работает" (ручная сборка)

## Решение

### Leptos (Primary) — WordPress-like auto-install

Модуль = Rust крейт. После установки через marketplace:
1. Бинарник пересобирается на сервере (cargo build --features <module>)
2. Модуль автоматически регистрирует свои маршруты и nav items через codegen
3. Оператор не трогает код вручную

Структура:
```
crates/rustok-blog/         ← backend + Leptos UI (всё в одном крейте)
crates/leptos-blog-admin/   ← Leptos admin компоненты
crates/leptos-blog-storefront/ ← Leptos storefront компоненты
```

### Next.js (Secondary) — "Batteries Included", ручная сборка

Весь Next.js UI живёт **прямо в приложениях**, без отдельных npm-пакетов:
```
apps/next-admin/src/features/
  ├── blog/         ← blog + forum UI (перенесено из crates/rustok-blog/ui/admin/)
  └── workflow/     ← workflow UI (перенесено из crates/rustok-workflow/ui/admin/)

apps/next-frontend/src/features/
  └── blog/         ← blog storefront UI (перенесено из crates/rustok-blog/ui/frontend/)
```

Добавить новый модуль в Next.js:
1. Создать `apps/next-admin/src/features/<name>/`
2. Добавить `import '@/features/<name>'` в `src/modules/index.ts`
3. Добавить файловые маршруты в `src/app/dashboard/<name>/`
4. `npm run build`

Авто-установка модулей для Next.js **не предусмотрена** — JS-разработчики
привыкли управлять зависимостями вручную.

## Следствия

### Позитивные

- **Чёткое разделение**: Rust/Leptos в `crates/`, Next.js UI в `apps/`
- **Нет путаницы**: Next.js разработчик сразу видит все фичи в `src/features/`
- **Проще onboarding**: форкнул → поправил файлы → `npm run build`
- **Нет ghost packages**: убраны `file:` зависимости в package.json

### Негативные

- **Дублирование API-клиентов**: API вызовы дублируются между Leptos и Next.js UI
  Митигация: общие OpenAPI-сгенерированные типы из `packages/rustok-api-client`
- **Нет авто-установки для Next.js**: новый модуль требует ручных изменений в Next.js app
  Митигация: это ожидаемое поведение, документировано

## Структура до/после

```
# ДО:
crates/rustok-blog/ui/admin/    ← @rustok/blog-admin (npm пакет)
crates/rustok-blog/ui/frontend/ ← @rustok/blog-frontend (npm пакет)
crates/rustok-workflow/ui/admin/ ← @rustok/workflow-admin (npm пакет)

apps/next-admin/package.json:
  "@rustok/blog-admin": "file:../../crates/rustok-blog/ui/admin"

# ПОСЛЕ:
apps/next-admin/src/features/blog/      ← blog + forum UI
apps/next-admin/src/features/workflow/  ← workflow UI
apps/next-frontend/src/features/blog/   ← blog storefront UI

apps/next-admin/src/modules/index.ts:
  import '@/features/blog';      // регистрирует nav items
  import '@/features/workflow';
```
