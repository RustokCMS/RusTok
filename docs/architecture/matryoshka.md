# RusToK — Matryoshka Architecture (7-Layer Platform Model)

> **Date:** 2026-03-01
> **Status:** Foundational. This document describes the core architectural vision of RusToK.
> **Authors:** Human & Claude AI — Co-Founders of the RusToK architectural concept.

---

## Overview

RusToK follows a **Matryoshka Principle** — a 7-layer nested architecture where each layer builds upon the previous one, creating a complete platform ecosystem. Like the Russian nesting doll, each layer encapsulates the ones below it, adding new capabilities while maintaining the integrity of the whole.

This architecture has no direct analogue in the industry. It is the first attempt to build a universal, layered SaaS platform model in Rust — from bare metal to federation.

```text
┌─────────────────────────────────────────────────────────┐
│  Layer 7: Inter-Network (Federation / Mesh)             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Layer 6: Interaction Bus (Fast Index / Event Hub)│  │
│  │  ┌─────────────────────────────────────────────┐  │  │
│  │  │  Layer 5: Unified UI (Technology-Agnostic)  │  │  │
│  │  │  ┌───────────────────────────────────────┐  │  │  │
│  │  │  │  Layer 4: Shared Capabilities         │  │  │  │
│  │  │  │  ┌─────────────────────────────────┐  │  │  │  │
│  │  │  │  │  Layer 3: Sub-Modules           │  │  │  │  │
│  │  │  │  │  ┌───────────────────────────┐  │  │  │  │  │
│  │  │  │  │  │  Layer 2: Modules         │  │  │  │  │  │
│  │  │  │  │  │  ┌─────────────────────┐  │  │  │  │  │  │
│  │  │  │  │  │  │  Layer 1: Core      │  │  │  │  │  │  │
│  │  │  │  │  │  │  Platform           │  │  │  │  │  │  │
│  │  │  │  │  │  └─────────────────────┘  │  │  │  │  │  │
│  │  │  │  │  └───────────────────────────┘  │  │  │  │  │
│  │  │  │  └─────────────────────────────────┘  │  │  │  │
│  │  │  └───────────────────────────────────────┘  │  │  │
│  │  └─────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## Layer 1: Core Platform (Rust SaaS Starter)

**The Foundation. The Engine. The Tank.**

There is no Rust SaaS starter platform on the market. RusToK fills this gap. Layer 1 is a fully functional, bare platform with:

- Multi-tenant architecture with isolation
- Authentication & authorization (JWT, RBAC)
- Admin panel infrastructure
- Event-driven architecture (EventBus)
- CQRS-lite (write/read model separation)
- Database migrations framework
- Configuration management
- API layer (GraphQL + REST)

**What it gives you:** Take this layer and build *any* SaaS on top of it. E-commerce, CMS, LMS, CRM — anything with data. This is the starter that nobody built for Rust yet.

**Implementation:** `rustok-core`, `rustok-tenant`, `rustok-rbac`, `apps/server`

---

## Layer 2: Modules (Plug-and-Play Business Verticals)

**Independent bricks. Compile-time safe. Zero runtime cost.**

The modular architecture means you can attach any business vertical to the core platform. Each module is an independent crate with its own:

- Database tables and migrations
- Domain services and business logic
- GraphQL resolvers and REST endpoints
- Event publishers and subscribers
- DTOs and validation rules

| Module | Purpose |
|--------|---------|
| `rustok-content` | Universal CMS (nodes, bodies, categories, tags) |
| `rustok-commerce` | E-commerce (products, variants, orders, inventory, prices) |
| `rustok-blog` | Blogging (wraps content with post semantics) |
| `rustok-forum` | Community discussions (wraps content with topic semantics) |
| `rustok-pages` | Static pages, menus, blocks |
| `alloy-scripting` | Rhai scripting engine for automation |

**What it gives you:** Pick the modules you need. Blog? Commerce? Forum? All of them? Just add to your `modules.toml` and rebuild.

**Implementation:** `crates/rustok-*`, `modules.toml` manifest

---

## Layer 3: Sub-Modules (Extensions Within Modules)

**Depth without complexity. Plugins for plugins.**

Sub-modules extend the base functionality of Layer 2 modules without touching the core. They add specialized behavior within a module's domain:

- **Commerce extensions:** Payment gateways, shipping calculators, tax engines, discount rules
- **Content extensions:** SEO analyzers, media processors, version diff viewers
- **Blog extensions:** RSS generators, newsletter integrators, social sharing
- **Forum extensions:** Reputation systems, moderation queues, thread pinning

**Principle:** A sub-module depends only on its parent module and `rustok-core`. It never reaches into other domain modules.

**What it gives you:** Fine-grained extensibility. You don't just add "commerce" — you configure exactly which commerce capabilities you need.

---

## Layer 4: Shared Capabilities (Cross-Module Services)

**One implementation. Every context. Zero duplication.**

This is the key insight that separates RusToK from other platforms. Many features are identical across modules but traditionally get reimplemented for each context:

| Capability | Used In |
|------------|---------|
| Emoji/Reactions | Messages, Blog comments, Forum posts, Product reviews |
| Tags/Labels | Content nodes, Products, Forum topics, Pages |
| Comments/Threads | Blog posts, Products, Pages, Forum topics |
| Media/Attachments | Any entity across any module |
| Notifications | Order updates, Blog mentions, Forum replies |
| Search | Products, Content, Forum topics, Pages |
| Ratings/Reviews | Products, Blog posts, Forum answers |
| Activity Feed | User actions across all modules |

**What it gives you:** Write the emoji system once. It works everywhere — in chat, in blog comments, in the corporate forum, in product reviews. Same UX, same data model, same API.

**Implementation:** Shared capability crates that integrate via `rustok-core` events and interfaces, not direct module dependencies.

---

## Layer 5: Unified UI (Technology-Agnostic Design System)

**One look. One feel. Any technology.**

Everything must look the same regardless of the frontend technology. The UI layer provides:

- A unified design language and component system
- Consistent typography, spacing, colors, and interactions
- Technology-agnostic specification (works with Leptos, React/Next.js, HTMX, or any future framework)
- Admin and storefront share the same design DNA
- Responsive, accessible, and localizable by default

| Frontend | Technology | Status |
|----------|------------|--------|
| Admin Panel | Leptos CSR (WASM) | Active |
| Storefront (Rust) | Leptos SSR | Active |
| Storefront (JS) | Next.js App Router | Active |
| Mobile | Future | Planned |

**What it gives you:** Your platform looks professional and consistent. Switch frontend technologies without redesigning. The design system is the contract.

**Implementation:** Shared CSS/Tailwind tokens, component specifications, `apps/admin`, `apps/storefront`, `apps/next-frontend`

---

## Layer 6: Interaction Layer (Internal Communication Bus)

**The nervous system. Fast index. Event hub.**

Modules already communicate through the EventBus. But Layer 6 extracts their common interactions into a dedicated, high-speed index — a unified communication layer that:

- Aggregates cross-module data into composite read models
- Provides a single query interface for complex cross-domain searches
- Eliminates N×N dependencies between modules
- Enables real-time synchronization without direct module coupling
- Supports backpressure control and circuit breakers

```text
Module A ──┐                    ┌── Composite Index
Module B ──┤── Interaction ────┤── Cross-domain Search
Module C ──┤     Layer          ├── Activity Feeds
Module D ──┘                    └── Analytics Aggregates
```

**What it gives you:** Instead of modules talking to each other directly (creating a tangled web), they all publish to and read from a unified fast index. Clean architecture at scale.

**Implementation:** `rustok-index` (CQRS read models), EventBus aggregation, composite indexers

---

## Layer 7: Inter-Network (Federation / Mesh)

**The final frontier. Platform-to-platform communication.**

Layer 7 enables different RusToK instances to communicate with each other across networks:

- **Federation Protocol:** RusToK instances can share content, users, and data
- **Mesh Architecture:** Decentralized network of cooperating platforms
- **Cross-Platform Commerce:** Shared product catalogs, distributed orders
- **Content Syndication:** Publish once, appear everywhere
- **Identity Federation:** Single sign-on across instances
- **Event Propagation:** Domain events flow between instances

This is the layer that nobody has built for a universal SaaS platform. ActivityPub exists for social networks. RusToK's Layer 7 is federation for *everything*.

**What it gives you:** Your RusToK instance is not an island. It's a node in a network. Shops can share catalogs. Blogs can syndicate content. Forums can federate discussions. The possibilities are infinite.

**Implementation:** Future — `rustok-federation`, `rustok-mesh` (Graal vision)

---

## The Three Pillars

The Matryoshka architecture is realized through three interconnected projects:

| Project | Layers | Role |
|---------|--------|------|
| **RusToK** | 1-3 | Core platform, modules, and sub-modules |
| **Alloy** | 4-6 | Shared capabilities, unified UI, interaction layer |
| **Graal** | 7 | Inter-network federation and mesh |

```text
RusToK (Foundation)  →  Alloy (Glue & UI)  →  Graal (Network)
   Layers 1-3              Layers 4-6            Layer 7
```

---

## Why This Hasn't Been Done Before

1. **Rust barrier:** Building a full SaaS platform in Rust requires deep systems knowledge. Most teams choose easier languages and sacrifice performance.

2. **Scope:** Most platforms stop at Layer 2 (modules). Layers 3-7 require architectural vision that goes beyond "just add plugins."

3. **Cross-module problem:** Layer 4 (shared capabilities) is the hardest to get right. It requires careful abstraction without creating god-objects.

4. **Federation complexity:** Layer 7 requires protocol design, consensus mechanisms, and distributed systems expertise.

5. **Performance requirements:** Only Rust can deliver the performance needed for Layers 6-7 to work at scale without becoming bottlenecks.

---

## Analogy: The OSI Model for SaaS

Just as the OSI model standardized network communication into 7 layers, the Matryoshka architecture standardizes SaaS platform construction:

| OSI Layer | Matryoshka Layer | Parallel |
|-----------|------------------|----------|
| Physical | Core Platform | The foundation everything runs on |
| Data Link | Modules | Direct data handling |
| Network | Sub-Modules | Routing within domains |
| Transport | Shared Capabilities | Reliable cross-domain delivery |
| Session | Unified UI | Consistent user sessions |
| Presentation | Interaction Layer | Data format & aggregation |
| Application | Inter-Network | End-to-end communication |

---

## See Also

- [Architecture Overview](overview.md) — Technical system design
- [Architecture Principles](principles.md) — Core tenets and patterns
- [RUSTOK_MANIFEST.md](../../RUSTOK_MANIFEST.md) — System manifest
- [Documentation Map](../index.md) — current documentation entry point

---

> This document captures the foundational architectural vision of RusToK.
> The Matryoshka Principle was conceived and designed collaboratively by Human & Claude AI.
> It represents a new approach to building SaaS platforms — one that nobody has attempted before.
