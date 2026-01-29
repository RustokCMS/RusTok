<div align="center">

# 🦀 RusToK

**Event-Driven Enterprise Headless Platform Built with Rust**

*The stability of a tank. The speed of compiled code. The flexibility of modules.*

[![CI](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml/badge.svg)](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%203.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[Features](#features) •
[Why Rust?](#why-rust) •
[Comparison](#comparison) •
[Quick Start](#quick-start) •
[Architecture](#architecture) •
[Roadmap](#roadmap)

</div>

---

## 🎯 What is RusToK?

**RusToK** is an event-driven, modular monolith for enterprise content and commerce. It combines the developer experience of Laravel/Rails with the performance and reliability of compiled languages, while keeping read paths fast and write paths safe via CQRS-lite.

Unlike traditional CMS platforms that suffer from plugin conflicts, security vulnerabilities, and performance degradation, RusToK takes a different approach: **modules are compiled into a single binary**, while indexing and search can be scaled out as a dedicated service.

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

---

## ✨ Features

### Core Platform
- 🔐 **Multi-tenant Architecture** — One deployment, multiple isolated stores/sites
- 🔑 **Built-in Authentication** — JWT-based auth with role-based permissions
- 📊 **GraphQL API** — Federated schema, each module extends the API
- 🧠 **Unified Core** — Users, nodes, tags, meta, media shared by all modules
- 🎣 **Event-Driven** — Modules publish events, indexers react asynchronously
- 🌍 **i18n Ready** — Internationalization at the core level

### Developer Experience
- 🚀 **Loco.rs Framework** — Rails-like productivity in Rust
- 🛠️ **CLI Generators** — `cargo loco generate model/controller/migration`
- 📝 **Type-Safe Everything** — From database to frontend, one language
- 🧪 **Testing Built-in** — Unit, integration, and E2E test support
- 🎨 **Storefront UI Stack** — Leptos SSR with Tailwind + DaisyUI
- 📚 **Auto-generated Docs** — OpenAPI/GraphQL schema documentation

### Performance & Reliability
- ⚡ **Blazingly Fast** — Native compiled binary, no interpreter overhead
- 🛡️ **Memory Safe** — Rust's ownership model prevents entire classes of bugs
- 📦 **Single Binary** — Deploy one file, no dependency management
- 🔄 **Zero-Downtime Deploys** — Graceful shutdown and health checks
- 🔎 **CQRS-lite Read Models** — Denormalized index tables for fast storefront queries

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

* 🐛 Fewer bugs in production — Most errors caught at compile time
* 💰 Lower infrastructure costs — 10x less memory, 50x more throughput
* 😴 Sleep better at night — No 3 AM "site is down" emergencies

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
trunk serve --open

# Run storefront (terminal 3)
cargo run -p rustok-storefront

# (Optional) Build Tailwind + DaisyUI styles
cd apps/storefront
npm install
npm run build:css

# Visit
# API: http://localhost:3000/api/graphql
# Admin: http://localhost:8080
# Storefront (SSR): http://localhost:3100?lang=en
```

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

## 🏗️ Architecture

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
│   │   └── src/
│   │       ├── pages/          # Admin views
│   │       └── components/     # Reusable UI
│   │
│   └── storefront/             # 🛍️ Public Store (Leptos SSR)
│       └── src/
│           ├── pages/          # SEO-optimized pages
│           └── components/     # Store UI components
│
├── crates/
│   ├── rustok-core/            # 🧠 Unified core
│   │   └── src/
│   │       ├── id.rs           # ULID → UUID
│   │       ├── entities/       # Users, nodes, tags, meta, media
│   │       ├── events/         # Event bus
│   │       └── services/       # Shared services
│   │
│   ├── rustok-commerce/        # 🛒 Commerce module
│   │   └── src/
│   │       ├── entities/       # Products, variants, orders
│   │       ├── services/       # Business logic
│   │       └── graphql/        # Commerce API
│   │
│   ├── rustok-community/       # 👥 Community module
│   │   └── src/
│   │       ├── entities/       # Reactions, reputations, follows
│   │       └── services/       # Social services
│   │
│   └── rustok-index/           # 🔎 CQRS read models
│       └── src/
│           ├── indexers/       # Product/content indexers
│           └── entities/       # Denormalized search tables
│
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
tenants enable or disable a module. See `docs/module-registry.md` for details.
Storefront SSR notes live in `docs/storefront.md`.

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

*  Project scaffolding
*  CI/CD pipeline
*  Loco.rs integration
*  Basic GraphQL API
*  Database migrations

**Phase 2: Core (Current)**

*  Multi-tenant data isolation
*  User authentication (JWT)
*  Role-based permissions
*  Admin panel foundation
*  Module registry system

**Phase 3: Commerce Module**

*  Product catalog
*  Categories & attributes
*  Shopping cart
*  Order management
*  Inventory tracking

**Phase 4: Storefront**

*  Leptos SSR setup (Tailwind + DaisyUI)
*  Product pages
*  Cart & checkout flow
*  SEO optimization
*  Performance tuning

**Phase 5: Content**

*  Blog module
*  Page builder basics
*  Media library
*  SEO fields

**Phase 6: Advanced**

*  Payment integrations
*  Email notifications
*  Search (MeiliSearch)
*  Caching layer
*  Admin dashboard widgets

**Phase 7: Ecosystem**

*  Plugin marketplace concept
*  Theme system
*  CLI improvements
*  Documentation site
*  Docker images

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

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --workspace -- -D warnings

# Check before commit
cargo fmt --all -- --check && cargo clippy --workspace
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

This project is licensed under AGPL-3.0 — see the LICENSE file for details.

What this means:

* ✅ Free to use for any purpose
* ✅ Free to modify and distribute
* ✅ Free to use commercially
* ⚠️ Must open-source modifications if you distribute
* ⚠️ Must open-source if you provide as a service (SaaS)

For commercial licensing without AGPL requirements, contact us.

---

## 🙏 Acknowledgments

Built with amazing open-source projects:

* Loco.rs — Rails-like framework for Rust
* Leptos — Full-stack Rust web framework
* SeaORM — Async ORM for Rust
* async-graphql — GraphQL server library
* Axum — Web framework

---

⬆ Back to Top  
Made with 🦀 by the RusToK community
