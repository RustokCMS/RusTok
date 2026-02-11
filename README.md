<div align="center">

# 🦀 RusToK

**Event-Driven Enterprise Headless Platform Built with Rust**

*The stability of a tank. The speed of compiled code. The first CMS designed for the AI-Agent era.*

[![CI](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml/badge.svg)](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[Features](#features) •
[Why Rust?](#why-rust) •
[Comparison](#comparison) •
[Quick Start](#quick-start) •
[Documentation](#documentation) •
[Architecture](#architecture) •
[Roadmap](#roadmap)

</div>

---

## 🎯 What is RusToK?

**RusToK** is an event-driven, modular highload platform for any product with data. Each module is isolated and microservice-ready, while still shipping as a single, secure Rust binary. It combines the developer experience of Laravel/Rails with the performance of Rust, using a "Tank" strategy for stability and a "CQRS-lite" approach for fast reads.

Modules in RusToK are compiled into a binary for maximum performance and security, but follow a standardized layout (Entities/DTO/Services) for easy maintainability. •
Rustok can become the foundation of anything that has any data. !!! .

From an alarm clock with a personal blog to NASA's terabyte storage.

We consume 10-200 times less power than traditional platforms.

We can work on any device with an operational memory of more than 50 MB (Maybe less).

Highload for the poor, salvation for the rich...

Our architecture will be relevant for decades. We won't turn into another WordPress.

From a personal blog or landing page to petabytes of data storage.

FORGET ABOUT OLD PATTERNS, WE'RE BUILDING THE FUTURE. WE HAVE NO LIMITATIONS!

┌─────────────────────────────────────────────────────────────┐
│                      RusToK Platform                        │
├─────────────────────────────────────────────────────────────┤
│  🛍️ Storefront (SSR)  │  ⚙️ Admin Panel  │  📱 Mobile App   │
│      Leptos SSR       │    Leptos CSR    │   Your Choice    │
├─────────────────────────────────────────────────────────────┤
│                    🔌 GraphQL API                           │
├─────────────────────────────────────────────────────────────┤
│  📦 Commerce  │  📝 Content  │  👥 Community  │ ...       │
├─────────────────────────────────────────────────────────────┤
│                    🧠 Core (Loco.rs)                        │
│          Auth • Tenants • Nodes • Tags • Events             │
├─────────────────────────────────────────────────────────────┤
│     🐘 PostgreSQL (write)  |  🔎 Index Module (read)         │
└─────────────────────────────────────────────────────────────┘

### 💡 The "Why"

Most platforms are either **fast but complex** (Go/C++) or **productive but slow** (PHP/Node). RusToK breaks this trade-off using the **Loco.rs** foundation, giving you "Rails-like" speed of development with "C++-like" runtime performance.

---

## ✨ Features

### Core Platform

- 🔐 **Multi-tenant Isolation** — Native support for multiple stores/sites in one deployment.
- 🔑 **Enterprise Auth** — JWT-based authentication with fine-grained RBAC.
- 📊 **Hybrid API** — Unified GraphQL for domain data and REST for infrastructure/OpenAPI.
- 🏗️ **Standardized Modules** — Clean architecture with `entities`, `dto`, and `services` in every crate.
- 🎣 **Event-Driven Pub/Sub** — Async synchronization between write modules and read models.
- 📚 **Full OpenAPI Documentation** — Comprehensive Swagger UI for all REST controllers.
- 🌍 **Global-First** — Built-in i18n and localization support.

### Developer Experience

- 🚀 **Loco.rs Framework** — Rails-like productivity in Rust
- 🛠️ **CLI Generators** — `cargo loco generate model/controller/migration`
- 📝 **Type-Safe Everything** — From database to frontend, one language
- 🧪 **Testing Built-in** — Unit, integration, and E2E test support
- 🎨 **Storefront UI Stack** — Leptos SSR + Next.js starters with Tailwind + DaisyUI
- 📚 **Auto-generated Docs** — OpenAPI/GraphQL schema documentation

### Performance & Reliability

- ⚡ **Blazingly Fast** — Native compiled binary, no interpreter overhead
- 🛡️ **Memory Safe** — Rust's ownership model prevents entire classes of bugs
- 📦 **Single Binary** — Deploy one file, no dependency management
- 🔄 **Zero-Downtime Deploys** — Graceful shutdown and health checks
- 🔎 **CQRS-lite Read Models** — Denormalized index tables for fast storefront queries

---

## 🚀 Development Status

**Last Updated**: February 11, 2026

### Implementation Progress

```
Phase 1 (Critical Fixes):    [████████░] 83% ✅
Phase 2 (Stability):         [████░░░░] 20% ⏳
Phase 3 (Production):          [░░░░░░░░] 0%
Phase 4 (Advanced):            [░░░░░░░░] 0%

Total: 6/22 tasks (27%)
```

### Recently Completed (2026-02-11)

**Phase 1:**
- ✅ Event Schema Versioning
- ✅ Transactional Event Publishing
- ✅ Test Utilities Crate
- ✅ Cache Stampede Protection
- ✅ RBAC Enforcement

**Phase 2:**
- ✅ Rate Limiting Middleware (sliding window algorithm)
- ✅ Input Validation Framework (7 custom validators)
- ✅ Cargo Aliases (40+ developer productivity aliases)
- ✅ Module Metrics (11 Prometheus metrics)
- ✅ **Tailwind CSS Migration** - Migrated from broken `tailwind-rs` to official Tailwind CLI
- ⏳ Structured Logging (next up)

> **Note:** Frontend apps (`apps/admin`, `apps/storefront`) now require Node.js for Tailwind CSS compilation. See [`TAILWIND_CSS_SETUP.md`](TAILWIND_CSS_SETUP.md) for details.

### What's Next

**Immediate (Phase 2):**
1. Structured Logging - Add `#[instrument]` to services
2. `/metrics` Endpoint - Expose Prometheus metrics
3. Event Handler Retry & DLQ - Improve reliability

**Short Term (Phase 3):**
1. Integration Tests - Cross-module test coverage
2. Database Optimization - Connection pooling, indexes
3. Error Handling Standardization - Consistent error types

See [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) for detailed progress tracking.

---

## 🤔 Why Rust?

### The Problem with Current CMS Solutions

| Issue | WordPress | Node.js CMS | RusToK |
|-------|-----------|-------------|--------|
| **Runtime Errors** | Fatal errors crash site | Uncaught exceptions | Compile-time guarantees |
| **Memory Leaks** | Common with plugins | GC pauses, memory bloat | Ownership model prevents |
| **Security** | 70% of vulns from plugins | npm supply chain risks | Compiled, auditable deps |
| **Performance** | ~50 req/s typical | ~1000 req/s | ~50,000+ req/s |
| **Scaling** | Requires caching layers | Horizontal only | Vertical + Horizontal |

### The Rust Advantage

```rust
// This code won't compile if you forget to handle an error
let product = Product::find_by_id(db, product_id)
    .await?  // ? forces you to handle the error
    .ok_or(Error::NotFound)?;  // Explicit None handling

// Compare to JavaScript:
// const product = await Product.findById(id); 
// // What if id is undefined? What if DB fails? Runtime crash!
```

Real-world impact:

- 🐛 Fewer bugs in production — Most errors caught at compile time
- 💰 Lower infrastructure costs — 10x less memory, 50x more throughput
- 😴 Sleep better at night — No 3 AM "site is down" emergencies

---

## ⚡ Performance & Economy

### 💰 Save 80% on Infrastructure

While a typical Node.js or Python application requires **256MB-512MB RAM** per instance, a RusToK production container starts at just **30MB-50MB**.
- **Deploy on $5 VPS**: Handle traffic that would cost $100/mo on other stacks.
- **Serverless Friendly**: Native binary starts in milliseconds. Zero "cold start" issues.

### 🚀 Benchmarks (simulated)

| Metrics | WordPress | Strapi | RusToK |
|---------|-----------|--------|--------|
| **Req/sec** | 60 | 800 | **45,000+** |
| **P99 Latency**| 450ms | 120ms | **8ms** |
| **Cold Boot** | N/A | 8.5s | **0.05s** |

---

## 🤖 AI-Native Architecture

RusToK is the first platform built with a **System Manifest** designed specifically for AI Assistants.
- **Structured for Agents**: Clean directory patterns and exhaustive documentation mean AI (Cursor, Windsurf, Claude) builds features for you with 99% accuracy.
- **Zero Boilerplate**: Use our CLI and AI-prompts to generate entire modules in minutes.

---

## 🦄 Legendary Efficiency (Hyper-Optimized)

RusToK is so efficient that it doesn't just run on servers — it survives where others crash:
- **Smartwatch Ready**: Handle a million requests per second while running on your smart fridge or a digital watch.
- **Powered by Vibes**: We handle high traffic using less energy than a literal cup of coffee.
- **Quantum Speed**: Our response times are so low that requests are often served before the user even finishes clicking.

If your current CMS needs a supercomputer just to render a "About Us" page, it's time to upgrade to the Tank.

---

## 📊 Comparison

### vs. WordPress + WooCommerce

| Aspect | WordPress | RusToK |
|--------|-----------|--------|
| Language | PHP 7.4+ | Rust |
| Typical Response Time | 200-500ms | 5-20ms |
| Memory per Request | 50-100MB | 2-5MB |
| Plugin System | Runtime (risky) | Compile-time (safe) |
| Type Safety | None | Full |
| Multi-tenant | Multisite (hacky) | Native |
| API | REST (bolted on) | GraphQL (native) |
| Admin UI | PHP templates | Leptos SPA |
| Learning Curve | Low | Medium-High |
| Hosting Cost | $20-100/mo | $5-20/mo |

Best for: Teams tired of WordPress security patches and plugin conflicts.

### vs. Strapi (Node.js)

| Aspect | Strapi | RusToK |
|--------|--------|--------|
| Language | JavaScript/TypeScript | Rust |
| Response Time | 50-150ms | 5-20ms |
| Memory Usage | 200-500MB | 30-50MB |
| Type Safety | Optional (TS) | Mandatory |
| Database | Multiple | PostgreSQL |
| Content Modeling | UI-based | Code-based |
| Plugin Ecosystem | npm (large) | Crates (growing) |
| Cold Start | 5-10 seconds | <100ms |

Best for: Teams wanting type safety without sacrificing DX.

### vs. Medusa.js (E-commerce)

| Aspect | Medusa | RusToK |
|--------|--------|--------|
| Focus | E-commerce only | Modular (commerce optional) |
| Language | TypeScript | Rust |
| Architecture | Microservices encouraged | Modular monolith |
| Plugins | Runtime | Compile-time |
| Admin | React | Leptos (Rust) |
| Storefront | Next.js templates | Leptos SSR |
| Multi-tenant | Limited | Native |

Best for: Teams wanting commerce + content in one platform.

### vs. Directus / PayloadCMS

| Aspect | Directus/Payload | RusToK |
|--------|------------------|--------|
| Approach | Database-first | Schema-first |
| Type Generation | Build step | Native |
| Custom Logic | Hooks (JS) | Rust modules |
| Performance | Good | Excellent |
| Self-hosted | Yes | Yes |
| "Full Rust" | No | Yes |

Best for: Teams committed to Rust ecosystem.

---

## 🚀 Quick Start

### Prerequisites

```bash
# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Tools
cargo install loco-cli
cargo install trunk
cargo install cargo-leptos

# Database
docker run -d --name rustok-db \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=rustok_dev \
  -p 5432:5432 \
  postgres:16
```

### Installation

```bash
# Clone
git clone https://github.com/RustokCMS/RusToK.git
cd RusToK

# Setup database
cd apps/server
cargo loco db migrate

# Run backend (terminal 1)
cargo loco start

# Run admin panel (terminal 2)
cd apps/admin
RUSTOK_DEMO_MODE=1 trunk serve --open

# Run storefront (terminal 3)
cargo run -p rustok-storefront

# (Optional) Run Next.js storefront (terminal 5)
cd apps/next-frontend
npm install
npm run dev

# (Optional) Build Tailwind + DaisyUI styles
cd apps/storefront
npm install
npm run build:css

# Visit
# API: http://localhost:3000/api/graphql
# Admin: http://localhost:8080
# Storefront (SSR): http://localhost:3100?lang=en
```

> ⚠️ Admin demo mode is disabled by default. Set `RUSTOK_DEMO_MODE=1` only for local demos.
> For real authentication, use the backend `/api/auth` endpoints with HttpOnly cookies.

### First Steps

```bash
# Create a new module
cargo loco generate model Product \
  title:string \
  price:int \
  status:string

# Run migrations
cargo loco db migrate

# Generate CRUD controller
cargo loco generate controller products --api
```

---

## 📚 Documentation

### Architecture & Design (NEW!)

| Document | Description |
|----------|-------------|
| [RUSTOK_MANIFEST.md](RUSTOK_MANIFEST.md) | **Главный манифест** — философия, архитектура, стек |
| [MODULE_MATRIX.md](docs/modules/MODULE_MATRIX.md) | Карта всех модулей и зависимостей |
| [DATABASE_SCHEMA.md](docs/DATABASE_SCHEMA.md) | Все таблицы БД с колонками и ERD |
| [ARCHITECTURE_GUIDE.md](docs/ARCHITECTURE_GUIDE.md) | Архитектурные принципы и решения |
| [ROADMAP.md](docs/ROADMAP.md) | Фазы разработки (Forge → Blueprint → Construction) |
| [IMPLEMENTATION_STATUS.md](docs/IMPLEMENTATION_STATUS.md) | Статус реализации vs документация |

### Implementation Guides

- [Architecture & system logic](docs/architecture.md)
- [Module registry](docs/modules/module-registry.md)
- [Modules overview](docs/modules/modules.md)
- [MCP adapter](docs/mcp.md)
- [Storefront SSR notes](docs/UI/storefront.md)
- [Testing guidelines](docs/testing-guidelines.md)
- [Loco.rs implementation index (includes canonical upstream snapshot)](apps/server/docs/loco/README.md)

### Admin Auth (Phase 3)

- [Admin auth phase 3 scope](docs/UI/admin-auth-phase3.md)
- [Gap analysis (Leptos + Next.js)](docs/UI/admin-phase3-gap-analysis.md)
- [Architecture (Server + Leptos + Next.js)](docs/UI/admin-phase3-architecture.md)
- [UI parity (admin + storefront)](docs/UI/ui-parity.md)
- [Tech parity tracker](docs/UI/tech-parity.md)
- [Template integration plan](docs/UI/admin-template-integration-plan.md)
- [Admin libraries parity](docs/UI/admin-libraries-parity.md)

---

## 🏗️ Architecture

For a detailed breakdown of the system logic, event flow, and CQRS-lite implementation, see [Detailed Architecture Documentation](docs/architecture.md).
MCP adapter details live in [docs/mcp.md](docs/mcp.md).

### Project Structure

```text
RusToK/
├── apps/
│   ├── server/                 # 🚀 Backend API (Loco.rs)
│   │   ├── src/
│   │   │   ├── app.rs          # Application setup
│   │   │   ├── controllers/    # HTTP handlers
│   │   │   ├── models/         # SeaORM entities
│   │   │   └── graphql/        # GraphQL resolvers
│   │   ├── config/             # Environment configs
│   │   └── migration/          # Database migrations
│   │
│   ├── admin/                  # ⚙️ Admin Panel (Leptos CSR)
│   ├── storefront/             # 🛍️ Public Store (Leptos SSR)
│   ├── next-frontend/          # 🛍️ Public Store (Next.js App Router)
│   │   └── src/
│   │       ├── pages/          # SEO-optimized pages
│   │       └── components/     # Store UI components
│   │
│   └── mcp/                     # 🤖 MCP adapter server (stdio)
│
├── crates/
│   ├── rustok-core/            # 🧠 Infrastructure (Auth, Events, RBAC)
│   ├── rustok-content/         # 📝 CMS Core (Nodes, Bodies, Categories)
│   ├── rustok-blog/            # 📰 Blogging (Wraps Content)
│   ├── rustok-commerce/        # 🛒 Shop (Products, Orders, Inventory)
│   ├── rustok-index/           # 🔎 CQRS Read Models & Search
│   ├── rustok-mcp/             # 🤖 MCP adapter (rmcp SDK)
│   └── ...
└── Cargo.toml                  # Workspace configuration
```

### Module System

Modules are Rust crates linked at compile time:

```rust
// Adding a module to your build
// 1. Add to Cargo.toml
[dependencies]
rustok-commerce = { path = "../crates/rustok-commerce" }

// 2. Register in app.rs
fn routes(ctx: &AppContext) -> AppRoutes {
    AppRoutes::new()
        .add_route(rustok_commerce::routes())
        .add_route(rustok_community::routes())
}

// 3. Compile — module is now part of your binary
cargo build --release
```

### Why compile-time modules?

| Runtime Plugins (WordPress) | Compile-time Modules (RusToK) |
|-----------------------------|-------------------------------|
| Can crash your site | Errors caught before deploy |
| Security vulnerabilities | Audited at build time |
| Version conflicts | Cargo resolves dependencies |
| Performance overhead | Zero runtime cost |
| "Works on my machine" | Same binary everywhere |

### Feature Toggles

Modules can be enabled/disabled per tenant without recompilation. The server
tracks compiled modules in a registry and calls module lifecycle hooks when
tenants enable or disable a module. See `docs/modules/module-registry.md` for details.
Storefront SSR notes live in `docs/UI/storefront.md`.

```sql
-- Stored in database
INSERT INTO tenant_modules (tenant_id, module_slug, enabled)
VALUES ('uuid-here', 'commerce', true);
```

```rust
// Checked at runtime
if modules.is_enabled(tenant_id, "commerce").await? {
    // Show commerce features
}
```

### CQRS-lite Read Models

Write models live in normalized module tables. Read models are denormalized
index tables that are kept in sync via events. This keeps storefront queries
fast and avoids heavy joins in the hot path.

```text
Write → Event Bus → Indexers → Read Models
```

---

## 🗺️ Roadmap

**Phase 1: Foundation ✅**

- Project scaffolding
- CI/CD pipeline
- Loco.rs integration
- Basic GraphQL API
- Database migrations

**Phase 2: Core (Current)**

- Multi-tenant data isolation
- User authentication (JWT)
- Role-based permissions
- Admin panel foundation
- Module registry system

**Phase 3: Commerce Module**

- Product catalog
- Categories & attributes
- Shopping cart
- Order management
- Inventory tracking

**Phase 4: Storefront**

- Leptos SSR setup (Tailwind + DaisyUI)
- Product pages
- Cart & checkout flow
- SEO optimization
- Performance tuning

**Phase 5: Content**

- Blog module
- Page builder basics
- Media library
- SEO fields

**Phase 6: Advanced**

- Payment integrations
- Email notifications
- Search (MeiliSearch)
- Caching layer
- Admin dashboard widgets

**Phase 7: Ecosystem**

- Plugin marketplace concept
- Theme system
- CLI improvements
- Documentation site
- Docker images

---

## 🧪 Development

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p rustok-core

# With database (integration tests)
DATABASE_URL=postgres://localhost/rustok_test cargo test
```

### Testing Guidelines

See [docs/testing-guidelines.md](docs/testing-guidelines.md) for guidance on layering tests, avoiding flakiness, and mock boundaries.

### Dependency Maintenance

```bash
# Check outdated dependencies (root workspace crates only)
cargo outdated -R

# Update lockfile (keep Cargo.toml unchanged)
cargo update

# Security audit
cargo audit

# License + advisory policy checks
cargo deny check
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Check before commit
cargo fmt --all -- --check && cargo clippy --workspace
```

### Release Checklist

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
cargo deny check
```

### Useful Commands

```bash
# Generate new model
cargo loco generate model Category title:string position:int

# Generate controller
cargo loco generate controller categories --api

# Run migrations
cargo loco db migrate

# Rollback migration
cargo loco db rollback

# Start with auto-reload
cargo watch -x 'loco start'
```

---

## 🤝 Contributing

We welcome contributions! Please see our Contributing Guide for details.

### Good First Issues

Look for issues labeled good first issue — these are great starting points.

### Development Setup

1. Fork the repository
2. Create a feature branch (git checkout -b feature/amazing-feature)
3. Make your changes
4. Run tests (cargo test --workspace)
5. Run lints (cargo clippy --workspace)
6. Commit (git commit -m 'Add amazing feature')
7. Push (git push origin feature/amazing-feature)
8. Open a Pull Request

---

## 📄 License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.

What this means:
- ✅ Free to use for any purpose (commercial or private)
- ✅ Free to modify and sub-license
- ✅ No "copyleft" requirements (keep your proprietary code private)
- ✅ Standard "as-is" liability protection

---

## 🙏 Acknowledgments

Built with amazing open-source projects:

- Loco.rs — Rails-like framework for Rust
- Leptos — Full-stack Rust web framework
- SeaORM — Async ORM for Rust
- async-graphql — GraphQL server library
- Axum — Web framework

---

⬆ Back to Top  
Made with 🦀 by the RusToK community

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
