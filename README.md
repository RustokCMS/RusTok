<div align="center">

# <img src="assets/rustok-logo-512x512.png" width="72" align="center" /> RusToK

**Event-driven modular platform built with Rust**

*One repository for the server, integrated Leptos hosts, and headless/experimental Next.js hosts.*

[![CI](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml/badge.svg)](https://github.com/RustokCMS/RusToK/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

**[Russian version](README.ru.md)** | **[Quick Platform Info (RU)](PLATFORM_INFO_RU.md)**

</div>

RusToK is a Rust-first modular monolith for multi-tenant products that combine content, commerce, workflow, and integrations. The current platform centers on `apps/server` as the composition root, manifest-driven module builds, an event-driven write/read split, and two UI host strategies: Leptos hosts as the primary integrated path and Next.js hosts as headless or experimental companions.

<a id="table-of-contents"></a>

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Performance & Economy](#performance-and-economy)
- [Why Rust](#why-rust)
- [AI-Native Architecture](#ai-native-architecture)
- [Comparison](#comparison)
- [Architecture Snapshot](#architecture-snapshot)
  - [Applications](#applications)
  - [Module Taxonomy](#module-taxonomy)
- [Module System](#module-system)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Development](#development)
- [Current Focus](#current-focus)
- [Acknowledgments](#acknowledgments)
- [License](#license)

<a id="overview"></a>

## Overview

Current platform strengths:

- Manifest-driven composition from [`modules.toml`](modules.toml) into the runtime and host applications.
- Clear module boundaries between `Core`, `Optional`, and capability/support crates.
- Hybrid API model: GraphQL for UI-facing domain surfaces, REST for operational and integration flows, and WebSocket transport where live runtime behavior needs it.
- Event-driven write/read split with transactional outbox, `rustok-index`, and `rustok-search`.
- Dual UI host model: `apps/admin` and `apps/storefront` as integrated Leptos hosts, plus `apps/next-admin` and `apps/next-frontend` for headless or experimental frontend work.

The root README is intentionally brief. Treat it as a repo entry point, not as the full architecture spec.

<a id="features"></a>

## Features

### Core Platform

- Multi-tenant isolation with tenant-aware runtime contracts
- Hybrid API model with GraphQL, REST, and WebSocket transport where needed
- Manifest-driven module composition and per-tenant enablement
- Event-driven write/read split with transactional outbox
- Built-in localization, observability, and RBAC foundations

### Deployment Modes

| Mode | How it works | Auth | Use case |
|------|-------------|------|----------|
| **Monolith** | Server plus integrated Leptos admin/storefront hosts | Server sessions and shared runtime context | Self-hosted sites, integrated backoffice + storefront |
| **Headless** | `apps/server` exposes APIs while frontend lives separately | OAuth2, sessions, or mixed contracts depending on client | Mobile apps, external frontends, third-party integrations |
| **Mixed** | Integrated Leptos hosts plus external clients against the same runtime | Both | Built-in admin plus external apps and integrations |

### Capability Snapshot

| Capability | WordPress | Shopify | Strapi | Ghost | **RusToK** |
|---|---|---|---|---|---|
| Monolith deployment | yes | no | no | yes | **yes** |
| Headless API surface | partial | yes | yes | partial | **yes** |
| Mixed integrated + headless mode | hacks | partial | partial | limited | **yes** |
| Multi-tenant runtime | multisite | limited | no | no | **native** |
| Compile-time module composition | no | no | no | no | **yes** |
| Rust-first integrated UI path | no | no | no | no | **yes** |

### Developer Experience

- Loco.rs foundation for the shared server runtime
- Rust crates as explicit module boundaries
- Module-owned transport and UI slices instead of a giant central app dump
- Living docs indexed from `docs/index.md`

### Testing & Quality

- Workspace-wide Rust test flow with `cargo nextest`
- Manifest and dependency hygiene checks via `cargo machete`
- Platform verification plans for architecture, frontend, and quality contours

### Observability & Security

- Prometheus-style metrics and tracing stack
- Typed RBAC and permission-aware runtime contracts
- Tenant-aware request context and channel-aware request flow
- Shared validation and outbox/event-runtime guardrails

<a id="performance-and-economy"></a>

## Performance & Economy

The exact numbers depend on the deployment profile and enabled modules, but the platform is still positioned around compiled-runtime efficiency and denormalized read paths.

### Benchmarks (simulated)

| Metrics | WordPress | Strapi | RusToK |
|---------|-----------|--------|--------|
| **Req/sec** | 60 | 800 | **45,000+** |
| **P99 Latency**| 450ms | 120ms | **8ms** |
| **Cold Boot** | N/A | 8.5s | **0.05s** |

<a id="why-rust"></a>

## Why Rust

### The Problem with Current CMS Solutions

| Issue | WordPress | Node.js CMS | RusToK |
|-------|-----------|-------------|--------|
| **Runtime Errors** | Fatal errors crash site | Uncaught exceptions | Compile-time guarantees |
| **Memory Leaks** | Common with plugins | GC pauses, memory bloat | Ownership model prevents |
| **Security** | Large plugin attack surface | npm supply-chain risk | Compiled, auditable dependencies |
| **Scaling** | Cache-heavy layering | Mostly horizontal | Vertical and horizontal options |

### The Rust Advantage

```rust
let product = Product::find_by_id(db, product_id)
    .await?
    .ok_or(Error::NotFound)?;
```

The value proposition is still the same even as the architecture evolves:

- more errors are caught at compile time;
- domain contracts stay explicit across crates;
- runtime performance is predictable without interpreter overhead.

<a id="ai-native-architecture"></a>

## AI-Native Architecture

RusToK is documented and structured for agent-assisted work, but the current claim should be read narrowly: the repository favors explicit contracts, documentation hubs, module manifests, and predictable component boundaries rather than vague scaffolding conventions.

Practical AI-facing entry points:

- [Documentation map](docs/index.md)
- [System manifest](RUSTOK_MANIFEST.md)
- [Module registry](docs/modules/registry.md)
- [Agent rules](AGENTS.md)

<a id="comparison"></a>

## Comparison

### vs. WordPress + WooCommerce

| Aspect | WordPress | RusToK |
|--------|-----------|--------|
| Language | PHP 7.4+ | Rust |
| Plugin System | Runtime (risky) | Compile-time and manifest-driven |
| Type Safety | None | Full |
| Multi-tenant | Multisite (hacky) | Native |
| API | REST (bolted on) | GraphQL + REST |
| Admin UI | PHP templates | Leptos host |

Best for: teams that want stronger contracts than a plugin-first PHP stack.

### vs. Strapi (Node.js)

| Aspect | Strapi | RusToK |
|--------|--------|--------|
| Language | JavaScript/TypeScript | Rust |
| Content Modeling | UI-based | Code and module based |
| Plugin Ecosystem | npm | crates and workspace modules |
| Cold Start | Higher | Lower |

Best for: teams that want type safety and explicit domain ownership.

### vs. Medusa.js (E-commerce)

| Aspect | Medusa | RusToK |
|--------|--------|--------|
| Focus | E-commerce only | Commerce plus content/community/workflow |
| Language | TypeScript | Rust |
| Architecture | Microservices encouraged | Modular monolith |
| Storefront | Next.js templates | Leptos host plus Next.js companion paths |

Best for: teams that want commerce and non-commerce domains in one platform.

### vs. Directus / PayloadCMS

| Aspect | Directus/Payload | RusToK |
|--------|------------------|--------|
| Approach | Database-first | Schema-first and module-first |
| Type Generation | Build step | Native Rust types |
| Custom Logic | Hooks (JS) | Rust modules |
| Self-hosted | Yes | Yes |
| "Full Rust" | No | Yes |

Best for: teams committed to a Rust-centered platform stack.

<a id="architecture-snapshot"></a>

## Architecture Snapshot

<a id="applications"></a>

### Applications

| Path | Role |
|---|---|
| `apps/server` | Composition root, HTTP/GraphQL runtime host, auth/session/RBAC wiring, event runtime, manifest validation |
| `apps/admin` | Primary Leptos admin host |
| `apps/storefront` | Primary Leptos storefront host |
| `apps/next-admin` | Headless or experimental Next.js admin host |
| `apps/next-frontend` | Headless or experimental Next.js storefront host |

<a id="module-taxonomy"></a>

### Module Taxonomy

`modules.toml` is the source of truth for platform modules.

Core modules:

- `auth`
- `cache`
- `channel`
- `email`
- `index`
- `search`
- `outbox`
- `tenant`
- `rbac`

Optional modules:

- Content and community: `content`, `blog`, `comments`, `forum`, `pages`, `media`, `workflow`
- Commerce family: `cart`, `customer`, `product`, `profiles`, `region`, `pricing`, `inventory`, `order`, `payment`, `fulfillment`, `commerce`

Support and capability crates sit outside the `Core` / `Optional` taxonomy:

- Shared/support: `rustok-core`, `rustok-api`, `rustok-events`, `rustok-storage`, `rustok-commerce-foundation`, `rustok-test-utils`, `rustok-telemetry`
- Capability/runtime layers: `rustok-mcp`, `alloy`, `alloy-scripting`, `flex`, `rustok-iggy`, `rustok-iggy-connector`

Domain boundary highlights:

- `rustok-content` is now a shared helper and orchestration layer. It is no longer the product-facing storage or transport owner for `blog`, `forum`, or `pages`.
- `rustok-comments` is the generic comments module for classic non-forum comments.
- The commerce surface is split into dedicated family modules, with `rustok-commerce` acting as the umbrella/root module and orchestration layer.
- Channel-aware behavior is part of the live request/runtime pipeline through `rustok-channel` and shared request context contracts.

<a id="module-system"></a>

## Module System

The current module flow is manifest-driven:

```text
modules.toml
  -> build.rs code generation for host wiring
  -> apps/server manifest validation
  -> ModuleRegistry / runtime bootstrap
  -> per-tenant enablement for optional modules
```

Important rules:

- Do not treat manual route registration in `app.rs` as the primary module integration model.
- Host applications wire optional modules through generated contracts derived from `modules.toml` and module manifests.
- Build composition and tenant enablement are different concerns:
  - build composition decides what is compiled into the artifact;
  - tenant enablement decides which optional modules are active for a given tenant.
- Leptos hosts already consume module-owned UI packages through manifest-driven wiring.
- Next.js hosts remain manual/headless entry points and should not be described as if they already follow the same generated host contract.

For the full runtime map, see:

- [Architecture overview](docs/architecture/overview.md)
- [Module registry](docs/modules/registry.md)
- [Module docs index](docs/modules/_index.md)
- [Module manifest and rebuild lifecycle](docs/modules/manifest.md)

<a id="quick-start"></a>

## Quick Start

The current local-dev quickstart lives in [docs/guides/quickstart.md](docs/guides/quickstart.md).

Typical workflow:

```bash
./scripts/dev-start.sh
```

The current guide covers the full local stack:

- backend on `http://localhost:5150`
- Next.js admin on `http://localhost:3000`
- Leptos admin on `http://localhost:3001`
- Next.js storefront on `http://localhost:3100`
- Leptos storefront on `http://localhost:3101`

If you need the app-level details instead of the root overview, start with:

- [apps/server docs](apps/server/docs/README.md)
- [apps/admin docs](apps/admin/docs/README.md)
- [apps/storefront docs](apps/storefront/docs/README.md)
- [apps/next-admin docs](apps/next-admin/docs/README.md)
- [apps/next-frontend docs](apps/next-frontend/docs/README.md)

<a id="documentation"></a>

## Documentation

Canonical entry points:

- [Documentation map](docs/index.md)
- [Architecture overview](docs/architecture/overview.md)
- [Module and application registry](docs/modules/registry.md)
- [Module documentation index](docs/modules/_index.md)
- [MCP reference package](docs/references/mcp/README.md)
- [Testing guide](docs/guides/testing.md)
- [Module system plan](docs/modules/module-system-plan.md)
- [Platform verification plan](docs/verification/PLATFORM_VERIFICATION_PLAN.md)
- [System manifest](RUSTOK_MANIFEST.md)
- [Agent rules](AGENTS.md)

<a id="development"></a>

## Development

Recommended baseline:

- Rust toolchain from the repository configuration
- PostgreSQL for local runtime work
- Node.js or Bun for Next.js hosts
- `trunk` for Leptos hosts

Useful commands:

```bash
# full local stack
./scripts/dev-start.sh

# Rust tests
cargo nextest run --workspace --all-targets --all-features

# doc tests
cargo test --workspace --doc --all-features

# format and lint
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings

# dependency and policy checks
cargo deny check
cargo machete
```

For repo-wide contributor rules, see [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md).

<a id="current-focus"></a>

## Current Focus

Current priorities are documented in the living platform docs rather than in a separate root roadmap file:

- [Module system plan](docs/modules/module-system-plan.md)
- [Platform verification plan](docs/verification/PLATFORM_VERIFICATION_PLAN.md)
- [Architecture decisions](DECISIONS/README.md)

At a high level, the current codebase is focused on:

- keeping module boundaries honest as the platform evolves;
- expanding module-owned transport and UI surfaces without turning `apps/server` into a domain dump;
- preserving manifest-driven composition across server and Leptos hosts;
- keeping channel-aware, multilingual, and event-driven contracts aligned across domains.

<a id="acknowledgments"></a>

## Acknowledgments

Built with open-source foundations such as:

- Loco.rs
- Leptos
- SeaORM
- async-graphql
- Axum

<a id="license"></a>

## License

RusToK is released under the [MIT License](LICENSE).
