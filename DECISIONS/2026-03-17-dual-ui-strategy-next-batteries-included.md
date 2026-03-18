# UI Strategy: Leptos (Primary) + Next.js (Modular Packages) + Deployment Modes

- Date: 2026-03-17
- Amended: 2026-03-18
- Status: Accepted

## Context

RusTok поддерживает два UI стека:

- **Leptos** — primary, компилируется в WASM, авто-деплой при install/uninstall модулей
- **Next.js** — secondary, для JS-разработчиков, знакомых с React-экосистемой

История изменений подхода к структуре UI-пакетов:
1. *(до 2026-03-17)* Next.js UI хранился как npm-пакеты внутри крейтов (`crates/rustok-blog/ui/admin/`)
2. *(2026-03-17)* Next.js переведён в «batteries included» — весь UI прямо в `apps/next-admin/src/features/`; Leptos UI через feature flags в `src/admin/` одного крейта
3. *(2026-03-18)* **Текущее решение**: оба стека получают отдельные publishable пакеты, но структурированы по-разному. Подробно — ниже.

## Решение

### 1. Leptos UI — живёт внутри модульного крейта через feature flags

Leptos UI располагается внутри директории модульного крейта в поддиректориях `admin/` и `storefront/` (которые сами являются publishable крейтами). Активация UI в приложении `apps/admin` происходит через подключение соответствующих крейтов.

В основном коде модуля (backend) могут присутствовать feature flags для логической связи с UI, но физически это отдельные крейты.

```text
crates/rustok-blog/
  Cargo.toml           # rustok-blog (backend)
  src/
  admin/               # rustok-blog-admin → crates.io
    Cargo.toml
    src/
  storefront/         # rustok-blog-storefront → crates.io
    Cargo.toml
    src/
```

`apps/admin/Cargo.toml` зависит от `rustok-blog-admin`, `rustok-commerce-admin` и т.д.
Архитектура позволяет публиковать UI независимо, но в коде модуля они логически связаны через feature flags.

### 2. Режимы деплоя

Каждый бинарник собирается с нужным набором крейтов:

```bash
# чистый API
cargo build -p rustok-server --release

# API + Leptos Admin WASM
cargo build -p rustok-admin --release
# (rustok-admin зависит от rustok-blog-admin, rustok-commerce-admin, ...)

# API + Leptos Storefront SSR
cargo build -p rustok-storefront --release

# всё вместе (monolith)
cargo build --workspace --release
```

Конкретная топология деплоя — решение оператора: monolith, headless,
раздельные серверы для API/admin/storefront, мультитенант, edge.

### 3. Next.js UI — модульные пакеты внутри приложения

Next.js UI каждого модуля живёт как **отдельный npm-пакет** внутри папки `packages/` самого приложения:

```text
apps/next-admin/
  packages/
    blog/              # @rustok/blog-admin
      package.json
      src/
    commerce/          # @rustok/commerce-admin
      package.json
      src/
  src/                 # само приложение — импортирует из packages/*
  package.json         # зависит от всех packages/*

apps/next-frontend/
  packages/
    blog/              # @rustok/blog-frontend
      package.json
      src/
    commerce/          # @rustok/commerce-frontend
      package.json
      src/
  src/
  package.json
```

`apps/next-admin/package.json` зависит от всех `packages/*` по умолчанию.

Убрать модуль из Next.js:

1. Удалить `apps/next-admin/packages/<module>/`
2. Убрать зависимость из `apps/next-admin/package.json`
3. `npm install && npm run build`

> [!IMPORTANT]
> Авто-установка через marketplace **не предусмотрена** для Next.js.
> Пересборка выполняется вручную. BuildExecutor управляет только Leptos-стеком.

### 4. rustok-module.toml — объявление UI-крейтов/пакетов

```toml
# crates/rustok-blog/rustok-module.toml
[provides.admin_ui]
leptos_crate = "rustok-blog-admin"   # Cargo crate name
next_package = "@rustok/blog-admin"  # npm package name

[provides.storefront_ui]
leptos_crate = "rustok-blog-storefront"
next_package = "@rustok/blog-frontend"
```

## Следствия

### Позитивные

- **Publishable**: оба стека могут публиковаться в реестры (crates.io / npm)
- **Headless-friendly**: UI деплоится независимо от backend
- **Granular**: оператор выбирает только нужные модули
- **Co-located**: UI-пакет рядом с модулем/приложением — удобно разрабатывать
- **Авто-деплой** для Leptos через BuildExecutor

### Негативные

- **Больше крейтов/пакетов** — 3 сущности на Leptos-модуль (backend + admin + storefront)
- **Next.js — ручное управление** — нет авто-деплоя
- **Дублирование API-клиентов** между стеками
  Митигация: OpenAPI-сгенерированные типы из `packages/rustok-api-client`

## История изменений ADR

### 2026-03-18 (текущая правка)

- Leptos UI: вынесен из `src/admin/` (feature flags) в отдельные sub-crates `admin/` и `storefront/`
- Next.js UI: вынесен из `apps/next-admin/src/features/` в `apps/next-admin/packages/<module>/`
- Оба стека теперь publishable (crates.io и npm соответственно)
- Добавлено поле `rustok-module.toml` для объявления UI-крейтов/пакетов

### 2026-03-17 (первая редакция)

- Next.js переведён в batteries included (`apps/next-admin/src/features/`)
- Leptos: feature flags внутри одного крейта (`src/admin/`)
- Авто-установка только для Leptos

### до 2026-03-17

- Next.js UI как npm-пакеты внутри `crates/rustok-<m>/ui/admin/`
- Leptos UI в отдельных крейтах `crates/leptos-blog-admin/`
