# RusToK System Manifest

This document defines the current high-level identity, boundaries, and architecture invariants of the platform. It is intentionally stricter and more technical than the root README files.

<a id="table-of-contents"></a>

## Table of Contents

- [Identity](#identity)
- [Documentation Hub](#documentation-hub)
- [Current System Shape](#current-system-shape)
  - [Applications](#applications)
  - [Platform Modules](#platform-modules)
  - [Support and Capability Crates](#support-and-capability-crates)
- [Core Philosophy](#core-philosophy)
- [Technology Stack](#technology-stack)
- [Boundary Rules](#boundary-rules)
- [Runtime Composition](#runtime-composition)
- [API and Request Model](#api-and-request-model)
- [Data, Events, and Read Models](#data-events-and-read-models)
- [Documentation and Change Discipline](#documentation-and-change-discipline)

<a id="identity"></a>

## Identity

| Property | Value |
|---|---|
| Name | RusToK |
| Platform type | Event-driven modular platform |
| Runtime shape | Modular monolith with manifest-driven composition |
| Primary language | Rust |
| Repository | <https://github.com/RustokCMS/RusToK> |
| Primary composition root | `apps/server` |

RusToK is currently implemented as a single repository that combines:

- a shared server/runtime host in `apps/server`;
- integrated Leptos hosts in `apps/admin` and `apps/storefront`;
- headless or experimental Next.js hosts in `apps/next-admin` and `apps/next-frontend`;
- platform modules declared in `modules.toml`;
- shared and capability crates that sit outside the `Core` / `Optional` module taxonomy.

<a id="documentation-hub"></a>

## Documentation Hub

Start with the documentation map and then branch into the relevant subsystem.

Core entry points:

- [docs/index.md](docs/index.md)
- [docs/architecture/overview.md](docs/architecture/overview.md)
- [docs/modules/registry.md](docs/modules/registry.md)
- [docs/modules/manifest.md](docs/modules/manifest.md)
- [docs/modules/_index.md](docs/modules/_index.md)
- [docs/references/mcp/README.md](docs/references/mcp/README.md)
- [docs/verification/PLATFORM_VERIFICATION_PLAN.md](docs/verification/PLATFORM_VERIFICATION_PLAN.md)
- [DECISIONS/README.md](DECISIONS/README.md)

<a id="current-system-shape"></a>

## Current System Shape

<a id="applications"></a>

### Applications

| Path | Role |
|---|---|
| `apps/server` | Composition root, transport/runtime host, manifest validation, event runtime bootstrap |
| `apps/admin` | Primary integrated Leptos admin host |
| `apps/storefront` | Primary integrated Leptos storefront host |
| `apps/next-admin` | Headless or experimental Next.js admin host |
| `apps/next-frontend` | Headless or experimental Next.js storefront host |

Leptos hosts are the primary integrated UI path today. Next.js hosts exist in parallel, but should not be described as if they already share the same generated host contract as the Leptos applications.

<a id="platform-modules"></a>

### Platform Modules

`modules.toml` is the source of truth for platform module membership and build composition.

Core modules:

| Slug | Crate | Role |
|---|---|---|
| `auth` | `rustok-auth` | Authentication lifecycle, credentials, token flows |
| `cache` | `rustok-cache` | Cache backend factory and shared cache runtime |
| `channel` | `rustok-channel` | Channel context, resolution, channel-aware bindings |
| `email` | `rustok-email` | Email delivery and template-backed transactional flows |
| `index` | `rustok-index` | Read-model substrate and cross-domain indexing |
| `search` | `rustok-search` | Product-facing search, dictionaries, query rules, engine selection |
| `outbox` | `rustok-outbox` | Transactional event persistence, relay, retry, delivery contracts |
| `tenant` | `rustok-tenant` | Tenant lifecycle and tenant module enablement |
| `rbac` | `rustok-rbac` | Permission vocabulary and authorization runtime |

Optional modules:

| Slug | Crate | Role |
|---|---|---|
| `content` | `rustok-content` | Shared content helpers, locale/slug/rich-text helpers, orchestration state and bridge contracts |
| `comments` | `rustok-comments` | Generic comments domain for classic non-forum comments |
| `blog` | `rustok-blog` | Blog domain with module-owned posts, categories, tags, and comment integration |
| `forum` | `rustok-forum` | Forum domain with module-owned categories, topics, replies, and moderation |
| `pages` | `rustok-pages` | Module-owned pages, blocks, menus, and public/admin surfaces |
| `media` | `rustok-media` | Media lifecycle, uploads, metadata, and storage-facing APIs |
| `workflow` | `rustok-workflow` | Workflow execution, templates, and webhook ingress |
| `cart` | `rustok-cart` | Commerce family cart domain |
| `customer` | `rustok-customer` | Commerce family customer domain |
| `product` | `rustok-product` | Commerce family catalog domain |
| `profiles` | `rustok-profiles` | Public profile domain |
| `region` | `rustok-region` | Commerce family region and currency domain |
| `pricing` | `rustok-pricing` | Commerce family pricing domain |
| `inventory` | `rustok-inventory` | Commerce family stock and reservation domain |
| `order` | `rustok-order` | Commerce family order lifecycle domain |
| `payment` | `rustok-payment` | Commerce family payment lifecycle domain |
| `fulfillment` | `rustok-fulfillment` | Commerce family fulfillment domain |
| `commerce` | `rustok-commerce` | Umbrella/root module for the commerce family and orchestration layer |

Key boundary clarifications:

- `rustok-content` is no longer the public storage or transport owner for `blog`, `forum`, or `pages`. Those domains now own their product-facing persistence and surfaces.
- `rustok-comments` is a separate module and must not be collapsed back into `blog` or `forum`.
- `rustok-commerce` is not the only commerce crate. It sits above the split commerce family as the umbrella/root module.

<a id="support-and-capability-crates"></a>

### Support and Capability Crates

These crates are important to the architecture, but they are not part of the `Core` / `Optional` platform module taxonomy.

Shared/support crates:

- `rustok-core`
- `rustok-api`
- `rustok-events`
- `rustok-storage`
- `rustok-commerce-foundation`
- `rustok-test-utils`
- `rustok-telemetry`

Capability/runtime crates:

- `rustok-mcp`
- `alloy`
- `alloy-scripting`
- `flex`
- `rustok-iggy`
- `rustok-iggy-connector`

Boundary rule:

- `rustok-mcp`, `alloy`, `alloy-scripting`, and `flex` are capability layers or support systems.
- They must not be documented as tenant-toggle platform modules unless `modules.toml` and the runtime taxonomy actually change.

<a id="core-philosophy"></a>

## Core Philosophy

### Tank Strategy

The platform still follows the same high-level stance:

- stability before runtime plug-in flexibility;
- compile-time validation before dynamic wiring by convention;
- one repo with explicit boundaries instead of a loose collection of ad hoc services;
- module composition through manifest plus rebuild rather than hot-plug runtime mutation.

### Platform Intent

Current system intent can be summarized as:

- domain-owned write models;
- event-driven downstream effects;
- prepared read models and focused query surfaces for public reads;
- multilingual and channel-aware behavior as first-class platform concerns;
- host applications that stay thin while module crates own their domain transport and UI slices.

<a id="technology-stack"></a>

## Technology Stack

| Layer | Technology | Details |
|---|---|---|
| Repository | Cargo workspace | Monorepo for apps and crates |
| Runtime | Tokio | Async runtime |
| Server foundation | Loco.rs | App boot, auth/session foundation, workers, mailers |
| HTTP | Axum | REST and middleware through Loco |
| GraphQL | async-graphql | Shared schema composition and module-owned GraphQL surfaces |
| Database | PostgreSQL | Primary transactional store |
| ORM | SeaORM | Typed async ORM |
| Raw SQL / migrations | SQLx + SeaORM migrations | Lower-level queries and migration support |
| Events | in-memory + outbox + Iggy | Layered event transport strategy |
| Cache | `rustok-cache` + shared cache contracts | Redis or in-memory depending on runtime config |
| Search | `rustok-search` + Postgres-first strategy | Dictionaries, rules, optional engine evolution |
| Storage | `rustok-storage` and object-store-backed adapters | Shared storage abstractions |
| Observability | tracing + Prometheus + telemetry crates | Logs, metrics, traces |
| UI hosts | Leptos and Next.js | Integrated Leptos path plus headless/experimental Next.js path |

<a id="boundary-rules"></a>

## Boundary Rules

The platform currently relies on these invariants:

- `apps/server` is the shared composition root. Domain modules should keep domain transport logic in module crates while the server remains a thin host and wiring layer.
- Platform modules are only `Core` or `Optional`. There is no third architectural module kind created by code generation, registry wiring, or bootstrap logic.
- Domain modules must not directly own or mutate each other's tables as an integration strategy. Cross-domain work must go through shared contracts, bridge interfaces, or events.
- Build composition and tenant enablement are separate:
  - build composition is controlled by `modules.toml` and manifest-derived host wiring;
  - tenant enablement controls which compiled optional modules are active for a tenant.
- Leptos host wiring is manifest-driven. Do not describe the current system as if manual route registration in `app.rs` were the primary module integration path.
- Next.js hosts remain manual/headless companions and are not yet the canonical generated-host contract.

<a id="runtime-composition"></a>

## Runtime Composition

The live runtime composition flow is:

```text
modules.toml
  -> module metadata / rustok-module.toml
  -> build.rs code generation for host wiring
  -> apps/server manifest validation
  -> ModuleRegistry and runtime bootstrap
```

Operational implications:

- `modules.toml` is the source of truth for platform module membership.
- `apps/server/src/modules/manifest.rs` validates runtime/module alignment against the manifest.
- Host applications use manifest-derived wiring rather than ad hoc domain-specific bootstrap branches.
- `apps/server` is responsible for starting shared runtime services such as event transport and other cross-cutting runtime infrastructure.
- `rustok-cache`, `rustok-auth`, and `rustok-storage` are real crates in the current architecture and must not be described as nonexistent or unnecessary abstractions.

<a id="api-and-request-model"></a>

## API and Request Model

The platform currently uses a hybrid API model:

- GraphQL for UI-facing domain contracts and shared schema composition
- REST for operational endpoints, integration flows, and some resource-oriented module surfaces
- WebSocket transport for live runtime scenarios such as GraphQL subscriptions

Request/runtime context is assembled in the shared host layer and then passed into module-owned logic. In the current server pipeline, channel resolution is part of the live request path and follows the documented flow:

```text
header -> query -> built-in host slice -> tenant-scoped typed policies -> default
```

The request model must stay aligned across:

- tenant resolution;
- auth/session state;
- RBAC enforcement;
- channel-aware context;
- locale-aware reads and fallbacks.

<a id="data-events-and-read-models"></a>

## Data, Events, and Read Models

RusToK uses a write/read split:

- write-side state lives in module-owned normalized tables and services;
- downstream effects are propagated through transactional outbox and event transport;
- read-side aggregation and fast discovery flow through `rustok-index` and `rustok-search`.

Current invariants:

- Public read performance should come from prepared read models, indexed tables, or focused module queries rather than heavy cross-domain joins.
- Event publication and downstream processing should remain aligned with the outbox contract and shared event runtime behavior.
- Multilingual behavior is a first-class platform concern. Locale-aware reads and fallback behavior must be documented and implemented consistently across modules.

<a id="documentation-and-change-discipline"></a>

## Documentation and Change Discipline

When the architecture, runtime boundaries, module taxonomy, or public host contracts change:

1. Update the affected local docs in the relevant `apps/*` or `crates/*` component.
2. Update the central docs in `docs/`.
3. Update [docs/index.md](docs/index.md) if the documentation map changes.
4. Update [docs/modules/registry.md](docs/modules/registry.md) if the component map or taxonomy changes.
5. Record non-trivial boundary changes in [DECISIONS/README.md](DECISIONS/README.md) via an ADR.

This manifest should remain stable, high-level, and code-aligned. It is not the place for speculative claims, obsolete bootstrap examples, or duplicated subsystem detail that already belongs in component docs.
