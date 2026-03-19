# RusToK Database Schema

> Current-state schema map for the main platform tables and major module-owned schemas.  
> Updated: 2026-03-19

This document is a high-level guide, not the canonical migration source. Source of truth remains:

- SeaORM entities under `apps/server/src/models/_entities` and module crates;
- migrations in `apps/server/migration` and module-owned migration sources;
- module docs for storage/index/workflow-specific schemas.

---

## Foundation Tables

### `tenants`

Platform tenant registry.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `name` | TEXT/VARCHAR | Display name |
| `slug` | VARCHAR | Stable tenant slug |
| `domain` | VARCHAR nullable | Optional host/domain binding |
| `settings` | JSONB | Tenant-scoped opaque settings |
| `default_locale` | VARCHAR | Default locale used by request fallback chain |
| `is_active` | BOOL | Tenant activity flag |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | Last update timestamp |

### `users`

Tenant-scoped user identity table.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `tenant_id` | UUID | FK to tenant |
| `email` | VARCHAR | Login/identity email |
| `password_hash` | VARCHAR | Password hash |
| `name` | VARCHAR nullable | Display name |
| `status` | ENUM/text | Account status |
| `email_verified_at` | TIMESTAMPTZ nullable | Email verification timestamp |
| `last_login_at` | TIMESTAMPTZ nullable | Last login timestamp |
| `metadata` | JSONB | Additional profile metadata |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | Last update timestamp |

### `sessions`

Auth/session lifecycle table.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `tenant_id` | UUID | FK to tenant |
| `user_id` | UUID | FK to user |
| `token_hash` | VARCHAR | Refresh/session token hash |
| `ip_address` | VARCHAR nullable | Source IP |
| `user_agent` | TEXT/VARCHAR nullable | User agent |
| `last_used_at` | TIMESTAMPTZ nullable | Last use timestamp |
| `expires_at` | TIMESTAMPTZ | Expiration timestamp |
| `revoked_at` | TIMESTAMPTZ nullable | Soft-revoke marker |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | Last update timestamp |

### `platform_settings`

Per-tenant platform configuration overrides.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `tenant_id` | UUID | FK to tenant |
| `category` | VARCHAR | Category such as `email`, `rate_limit`, `events`, `oauth` |
| `settings` | JSONB | Stored category payload |
| `schema_version` | INT | Schema version for validation/migration |
| `updated_by` | UUID nullable | User who last updated the record |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | Last update timestamp |

### `tenant_modules`

Per-tenant module toggle and module-scoped settings.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `tenant_id` | UUID | FK to tenant |
| `module_slug` | VARCHAR | Module identifier |
| `enabled` | BOOL | Runtime enablement flag |
| `settings` | JSONB | Module-owned opaque settings |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `updated_at` | TIMESTAMPTZ | Last update timestamp |

### `oauth_apps`

Tenant-scoped OAuth application registry for provider/client management.

This table exists in the live schema and is used by GraphQL OAuth admin flows.

### `sys_events`

Transactional outbox and delivery state.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Primary key |
| `payload` | JSONB | Serialized event envelope |
| `status` | VARCHAR | Delivery state (`pending`, `dispatched`, `failed`, etc.) |
| `created_at` | TIMESTAMPTZ | Creation timestamp |
| `dispatched_at` | TIMESTAMPTZ nullable | Dispatch timestamp |

---

## RBAC Tables

RBAC source of truth remains relation tables:

- `roles`
- `permissions`
- `user_roles`
- `role_permissions`

These tables back the Casbin-only runtime through resolver/adapters; they remain authoritative for permission data.

---

## Content and Commerce Tables

### Content

Core content schema remains centered around:

- `nodes`
- `node_translations`
- `bodies`

This supports locale-aware content storage with explicit fallback behavior at the service/request layer.

### Commerce

Core commerce schema remains centered around:

- `products`
- `product_translations`
- `product_variants`
- `variant_translations`
- `prices`
- `product_images`
- `product_options`

---

## Index Tables

`rustok-index` owns the denormalized read models used by the CQRS read path.

### `index_content`

Implemented and maintained by content indexers.

Representative columns:

- `tenant_id`
- `node_id`
- `locale`
- `kind`
- `slug`
- `title`
- `excerpt`
- `search_vector`
- `indexed_at`

### `index_products`

Implemented and maintained by product indexers.

Representative columns:

- `tenant_id`
- `product_id`
- `locale`
- `handle`
- `title`
- `description`
- `price_min`
- `price_max`
- `attributes`
- `search_vector`
- `indexed_at`

---

## Workflow Tables

`rustok-workflow` owns its own module tables and they are implemented in the live schema:

- `workflows`
- `workflow_steps`
- `workflow_executions`
- `workflow_step_executions`
- `workflow_versions`

Notable runtime fields include:

- workflow trigger config (`trigger_config`)
- failure tracking (`failure_count`, `auto_disabled_at`)
- webhook trigger support (`webhook_slug`, `webhook_secret`)
- execution context and step I/O payloads (`context`, `input`, `output`)

---

## Media and Storage

Media metadata is module-owned while file bytes are handled through the shared storage runtime.

Key media tables:

- `media`
- `media_translations`

Storage backend configuration is not modeled as per-file SQL schema; it is runtime-configured through typed settings and `platform_settings`.

---

## Notes

- `tenant_id` remains the primary isolation boundary for platform and module data.
- JSONB is used intentionally for module/platform settings, workflow configuration, and flexible metadata.
- Read-model tables are denormalized on purpose and should not be treated as authoritative write-side state.
- For exact column/index/constraint details, prefer module migrations and generated entities over this summary doc.

---

## See Also

- [i18n.md](./i18n.md)
- [rbac.md](./rbac.md)
- [events.md](./events.md)
- [workflow.md](./workflow.md)
- [modules.md](./modules.md)
