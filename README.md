<div align="center">

# 🦀 RusToK

**Enterprise-grade modular headless CMS/Commerce on Rust**

*Stability of a tank. Speed of compiled code. Flexibility of modules.*

[![CI](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml/badge.svg)](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

</div>

---

## Русская версия

### 🎯 Что такое RusToK

**RusToK** — модульная headless-платформа на Rust для построения CMS/Commerce-решений уровня enterprise. Здесь нет «адского плагинного хаоса» — модули компилируются в единый бинарник, что делает систему предсказуемой, стабильной и безопасной.

### ✨ Возможности

**Ядро платформы**
- 🔐 Multi-tenant архитектура
- 🔑 Встроенная аутентификация и роли
- 📊 GraphQL API (модули расширяют схему)
- 🎣 Hooks и событийная модель без жёстких зависимостей

**Developer Experience**
- 🚀 Loco.rs — Laravel/Rails DX в мире Rust
- 🛠️ Генераторы (models/controllers/migrations)
- 🧪 Интеграция с тестами и линтингом в CI

**Производительность и надежность**
- ⚡ Нативный бинарник без runtime-накладных расходов
- 🛡️ Безопасная работа с памятью (ownership модель)
- 📦 Один бинарник для деплоя

### 🤔 Почему Rust

- **Zero-cost abstractions** → высокая производительность без компромиссов.
- **Строгая типизация** → меньше runtime-ошибок в продакшене.
- **Асинхронность (Tokio)** → устойчивость под высокой нагрузкой.

### 📊 Сравнение с классическими CMS

| Критерий | RusToK | Классические CMS (PHP/монолитные) |
|---|---|---|
| Архитектура | Headless, модульная, GraphQL | Монолит, REST/HTML смешаны |
| Типизация | Compile-time, строгая | Динамическая/частичная |
| Модульность | Rust-крейты | Плагины разного качества |
| Безопасность | Memory-safe | Часто слабее контроль |
| Производительность | Высокая и стабильная | Зависит от runtime |

### 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────────┐
│                      RusToK Platform                        │
├─────────────────────────────────────────────────────────────┤
│  🛍️ Storefront (SSR)  │  ⚙️ Admin Panel  │  📱 Mobile App   │
│      Leptos SSR       │    Leptos CSR    │   Your Choice    │
├─────────────────────────────────────────────────────────────┤
│                    🔌 GraphQL API                           │
├─────────────────────────────────────────────────────────────┤
│  📦 Commerce  │  📝 Blog  │  📄 Pages  │  🎫 Tickets  │ ... │
├─────────────────────────────────────────────────────────────┤
│                    🧠 Core (Loco.rs)                        │
│            Auth • Tenants • Events • Hooks                  │
├─────────────────────────────────────────────────────────────┤
│                    🐘 PostgreSQL                            │
└─────────────────────────────────────────────────────────────┘
```

### 🗂️ Структура проекта

```text
rustok/
├── apps/
│   ├── server/        # Backend (Loco.rs + GraphQL)
│   ├── admin/         # Admin UI (Leptos CSR)
│   └── storefront/    # Storefront (Leptos SSR)
├── crates/            # Модули и ядро
│   ├── rustok-core/
│   ├── rustok-commerce/
│   └── rustok-blog/
└── Cargo.toml         # Workspace
```

### 📈 Нагрузки и память

- **Async-архитектура** эффективно обрабатывает множество параллельных запросов.
- **PostgreSQL** со строгой схемой обеспечивает предсказуемые запросы.

> ⚠️ Точные RPS/latency зависят от окружения и бизнес-логики. Официальные бенчмарки появятся позже.

Потребление памяти зависит от нагрузки, модулей и кэшей. Rust без GC дает более предсказуемые пики.

### 🚀 Быстрый старт

```bash
# База данных
docker run -d --name rustok-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustok_dev \
  -p 5432:5432 \
  postgres:16

# Миграции и запуск
cd apps/server
cargo loco db migrate
cargo loco start
```

---

## English version

### 🎯 What is RusToK

**RusToK** is a modular headless Rust platform for building enterprise-grade CMS/Commerce solutions. There is no runtime plugin chaos — modules are compiled into a single binary for predictability, stability, and safety.

### ✨ Features

**Core platform**
- 🔐 Multi-tenant architecture
- 🔑 Built-in authentication and roles
- 📊 GraphQL API (modules extend the schema)
- 🎣 Hooks + event-driven design without tight coupling

**Developer experience**
- 🚀 Loco.rs for Laravel/Rails-like DX in Rust
- 🛠️ Generators (models/controllers/migrations)
- 🧪 CI-ready testing and linting

**Performance & reliability**
- ⚡ Native binary, no interpreter overhead
- 🛡️ Memory safety via ownership model
- 📦 Single-binary deployments

### 🤔 Why Rust

- **Zero-cost abstractions** for high performance.
- **Strong typing** to reduce runtime errors.
- **Async (Tokio)** for scalable concurrency.

### 📊 Comparison with classic CMS

| Criteria | RusToK | Classic CMS (PHP/monoliths) |
|---|---|---|
| Architecture | Headless, modular, GraphQL | Monolithic, REST/HTML mixed |
| Typing | Compile-time, strong | Dynamic/partial |
| Modularity | Rust crates | Plugins of varying quality |
| Security | Memory-safe | Often weaker control |
| Performance | High and stable | Depends on runtime |

### 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      RusToK Platform                        │
├─────────────────────────────────────────────────────────────┤
│  🛍️ Storefront (SSR)  │  ⚙️ Admin Panel  │  📱 Mobile App   │
│      Leptos SSR       │    Leptos CSR    │   Your Choice    │
├─────────────────────────────────────────────────────────────┤
│                    🔌 GraphQL API                           │
├─────────────────────────────────────────────────────────────┤
│  📦 Commerce  │  📝 Blog  │  📄 Pages  │  🎫 Tickets  │ ... │
├─────────────────────────────────────────────────────────────┤
│                    🧠 Core (Loco.rs)                        │
│            Auth • Tenants • Events • Hooks                  │
├─────────────────────────────────────────────────────────────┤
│                    🐘 PostgreSQL                            │
└─────────────────────────────────────────────────────────────┘
```

### 🗂️ Project structure

```text
rustok/
├── apps/
│   ├── server/        # Backend (Loco.rs + GraphQL)
│   ├── admin/         # Admin UI (Leptos CSR)
│   └── storefront/    # Storefront (Leptos SSR)
├── crates/            # Modules and core
│   ├── rustok-core/
│   ├── rustok-commerce/
│   └── rustok-blog/
└── Cargo.toml         # Workspace
```

### 📈 Load & memory

- **Async architecture** handles many concurrent requests efficiently.
- **Strict PostgreSQL schema** ensures predictable queries.

> ⚠️ Exact RPS/latency depends on environment and business logic. Official benchmarks will follow.

Memory usage depends on load, enabled modules, and caches. Rust’s no-GC model typically yields more predictable peaks.

### 🚀 Quickstart

```bash
# Database
docker run -d --name rustok-db \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustok_dev \
  -p 5432:5432 \
  postgres:16

# Migrate and start
cd apps/server
cargo loco db migrate
cargo loco start
```
