# Module & Application Registry

This document provides a comprehensive map of all components within the RusToK ecosystem, including their relationships and responsibilities.

## High-Level Architecture

```mermaid
graph TD
    subgraph Applications
        SERVER[apps/server - Loco Server]
        ADMIN[apps/admin - Leptos Admin]
        SF[apps/storefront - Leptos Storefront]
        NEXT_ADMIN[apps/next-admin - Next.js Admin]
        NEXT_SF[apps/next-frontend - Next.js Storefront]
    end

    subgraph Domain Modules
        COMMERCE[crates/rustok-commerce]
        CONTENT[crates/rustok-content]
        BLOG[crates/rustok-blog]
        FORUM[crates/rustok-forum]
        PAGES[crates/rustok-pages]
        INDEX[crates/rustok-index]
        RBAC[crates/rustok-rbac]
        TENANT[crates/rustok-tenant]
    end

    subgraph Infrastructure
        CORE[crates/rustok-core]
        OUTBOX[crates/rustok-outbox]
        IGGY[crates/rustok-iggy]
        IGGY_CONN[crates/rustok-iggy-connector]
        MCP[crates/rustok-mcp]
        TELEMETRY[crates/rustok-telemetry]
        TEST_UTILS[crates/rustok-test-utils]
    end

    subgraph Frontend Libraries
        L_AUTH[crates/leptos-auth]
        L_UI[crates/leptos-ui]
        L_GRAPHQL[crates/leptos-graphql]
    end

    SERVER --> COMMERCE
    SERVER --> CONTENT
    SERVER --> RBAC
    SERVER --> TENANT
    
    ADMIN --> L_AUTH
    ADMIN --> L_UI
    SF --> L_UI
    
    COMMERCE --> CORE
    CONTENT --> CORE
    COMMERCE --> OUTBOX
    OUTBOX --> IGGY
    IGGY --> IGGY_CONN
    
    Domain Modules -.-> TELEMETRY
```

## Component Directory

### Applications (`apps/`)

| Path | Name | Description |
|------|------|-------------|
| `apps/server` | **Server** | Main API server built on Loco.rs. Orchestrates all domain modules. |
| `apps/admin` | **Admin Panel** | Back-office management interface built with Leptos. |
| `apps/storefront` | **Storefront** | Customer-facing web interface built with Leptos. |
| `apps/next-admin` | **Next Admin** | Modern React-based admin interface (Next.js). |
| `apps/next-frontend` | **Next Storefront** | Modern React-based storefront (Next.js). |

### Core & Infrastructure (`crates/`)

| Path | Name | Description |
|------|------|-------------|
| `crates/rustok-core` | **Core** | Shared traits, base entities, and common utilities. |
| `crates/rustok-outbox` | **Outbox** | Implementation of the Transactional Outbox pattern. |
| `crates/rustok-iggy` | **Iggy Transport** | EventTransport implementation with serialization, topology, DLQ, replay. |
| `crates/rustok-iggy-connector` | **Iggy Connector** | Embedded/Remote mode switching, connection lifecycle, message I/O. |
| `crates/rustok-mcp` | **MCP** | MCP adapter crate with embedded `rustok-mcp-server` binary. Exposes RusToK tools/resources via the MCP protocol using the `rmcp` SDK. |
| `crates/rustok-telemetry` | **Telemetry** | Observability setup (OTLP, Tracing, Metrics). |
| `crates/rustok-tenant` | **Tenant** | Multi-tenancy isolation and management logic. |
| `crates/rustok-rbac` | **RBAC** | Role-based access control engine. |
| `crates/rustok-test-utils` | **Test Utils** | Shared testing helpers and mocks. |

### Domain Modules (`crates/`)

| Path | Name | Description |
|------|------|-------------|
| `crates/rustok-commerce` | **Commerce** | Products, orders, payments, and checkout logic. |
| `crates/rustok-content` | **Content** | Unified content storage, nodes, and versioning. |
| `crates/rustok-blog` | **Blog** | Blogging module: posts, comments, categories, tags. Wrapper over content module with type-safe state machine. ✅ Production Ready |
| `crates/rustok-pages` | **Pages** | Static and dynamic page management. |
| `crates/rustok-forum` | **Forum** | Community discussion and forum features. |
| `crates/rustok-index` | **Index** | Search indices and read-model optimization. |
| `crates/alloy-scripting` | **Alloy** | Dynamic scripting and extension engine. |

### Libraries & UI Kits (`crates/`)

| Path | Name | Description |
|------|------|-------------|
| `crates/leptos-ui` | **Leptos UI** | Shared UI component library for Leptos apps. |
| `crates/leptos-auth` | **Leptos Auth** | Authentication hooks and components for Leptos. |
| `crates/leptos-forms` | **Leptos Forms** | Form abstractions for Leptos applications. |
| `crates/leptos-graphql` | **Leptos GraphQL** | GraphQL client integration for Leptos. |
| `crates/leptos-hook-form` | **Leptos Hook Form** | Hook-form style bindings for Leptos. |
| `crates/leptos-shadcn-pagination` | **Leptos Pagination** | Pagination components compatible with shadcn design. |
| `crates/leptos-table` | **Leptos Table** | Data table primitives for Leptos UIs. |
| `crates/leptos-zod` | **Leptos Zod** | Zod-like validation helpers for Leptos projects. |
| `crates/leptos-zustand` | **Leptos Zustand** | Lightweight state management utilities for Leptos. |
| `crates/utoipa-swagger-ui-vendored` | **Swagger** | Vendored UI for API documentation. |
| `crates/tailwind-rs` | **Tailwind RS** | Tailwind CSS utility generation core. |
| `crates/tailwind-css` | **Tailwind CSS** | CSS value and property types for Tailwind. |
| `crates/tailwind-ast` | **Tailwind AST** | AST and parser for Tailwind class expressions. |

## Maintenance Rule

> [!IMPORTANT]
> This registry must be kept up to date. AI Agents are required to update the **Mermaid diagram** and **Component Directory** whenever a new crate or application is added, renamed, or significantly restructured.
