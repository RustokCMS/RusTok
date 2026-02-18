# RusToK Quickstart Guide

Быстрый старт для локальной разработки с двумя админками (Next.js + Leptos) и двумя витринами.

## 🚀 Запуск одной командой

```bash
# 1. Клонировать репозиторий (если еще не сделано)
git clone <repo-url>
cd RusTok

# 2. Запустить весь стек
./scripts/dev-start.sh
```

Скрипт автоматически:
- создаст `.env.dev` из `.env.dev.example` (если не существует);
- поднимет PostgreSQL;
- запустит backend (`apps/server`);
- запустит обе админки (Next.js на `:3000`, Leptos на `:3001`);
- запустит обе витрины (Next.js на `:3100`, Leptos на `:3101`).

Источник: [`scripts/dev-start.sh`](../../scripts/dev-start.sh).

## 📱 Доступ к сервисам

### Backend
- **API Server**: <http://localhost:5150>
- **GraphQL Endpoint**: <http://localhost:5150/api/graphql>
- **Health Check**: <http://localhost:5150/api/health>

### Админки
- **Next.js Admin**: <http://localhost:3000>
- **Leptos Admin**: <http://localhost:3001>

### Витрины
- **Next.js Storefront**: <http://localhost:3100>
- **Leptos Storefront**: <http://localhost:3101>

### База данных
- **PostgreSQL**: `localhost:5432`
- **Database**: `rustok_dev`
- **User**: `rustok`
- **Password**: `rustok`

## 🔑 Тестовые данные

Для входа в dev-окружение:

```text
Email:    admin@local
Password: admin12345
```

## 🛠 Полезные команды

```bash
# Остановить все сервисы
./scripts/dev-start.sh stop

# Перезапустить
./scripts/dev-start.sh restart

# Логи
./scripts/dev-start.sh logs
./scripts/dev-start.sh logs server

# Статус
./scripts/dev-start.sh status

# Запуск только админ-профиля
./scripts/dev-start.sh start admin

# Помощь
./scripts/dev-start.sh --help
```

## 🔧 Ручной запуск без Docker

### Требования
- Rust toolchain (см. `rust-toolchain.toml`)
- Node.js/Bun для Next.js приложений
- PostgreSQL
- Trunk для Leptos приложений (`cargo install trunk`)

### Запуск

```bash
# backend
cd apps/server
cargo run

# next admin
cd apps/next-admin
bun install
bun run dev

# leptos admin
cd apps/admin
trunk serve --port 3001
```

## 📚 Связанные документы

- [Docs index](../index.md)
- [UI documentation hub](../UI/README.md)
- [Admin ↔ Server connection](../UI/admin-server-connection-quickstart.md)
- [apps/next-admin README](../../apps/next-admin/README.md)
- [apps/admin docs](../../apps/admin/docs/README.md)
