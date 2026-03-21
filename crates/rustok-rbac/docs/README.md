# rustok-rbac docs

В этой папке хранится документация модуля `crates/rustok-rbac`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Canonical scope

This file is the canonical module-level documentation for `crates/rustok-rbac`.
Module-specific RBAC documentation must stay in this crate, not under `docs/architecture/`.

## Current module boundary

RusToK uses a single-engine RBAC runtime:

- relation tables `roles`, `permissions`, `user_roles`, `role_permissions` remain the source of truth for assignments;
- `crates/rustok-rbac` owns permission policy helpers, evaluator APIs, Casbin runtime helpers, resolver contracts, and integration event contracts;
- `apps/server` owns only the adapter and wiring layer: SeaORM stores, Moka cache, `RbacService`, transport-specific extractors, and observability;
- `rustok-core` owns the typed RBAC primitives (`Permission`, `Resource`, `Action`, `UserRole`, `SecurityContext`).

Live authorization no longer has a separate relation-only path, shadow-runtime path, or parity-gate logic.

## Permission model

- Canonical permission shape is `<resource>:<action>`.
- `manage` acts as a wildcard for the action set of a resource.
- Wildcard semantics live in `rustok-rbac`, not in server handlers.
- Tenant isolation is enforced by tenant-filtered relation-store queries and resolver contracts, not by permission-string prefixes.
- New permissions must be added through typed contracts in `rustok-core`.

## Request and integration path

Runtime decision path for REST/GraphQL integrations:

1. `apps/server` calls `RbacService`.
2. `RuntimePermissionResolver` loads effective permissions through relation-store and cache adapters.
3. `rustok-rbac` builds the Casbin-backed evaluator and returns `AuthorizationDecision`.
4. Server adapters emit decision, cache, and latency telemetry.

`apps/server` also builds `rustok_core::SecurityContext` from the resolved permission set, not from an inferred role alone. Derived role values are still allowed for presentation and compatibility, but live authorization must use typed permissions plus the permission-aware `SecurityContext`.

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)


## Runtime contracts

- `PermissionResolver` / `PermissionResolution` define tenant-aware RBAC resolver contract for adapter-based integrations (`apps/server` and future transports), including default `has_permission/has_any_permission/has_all_permissions` use-cases powered by module evaluator APIs.
- `permission_policy` and `permission_evaluator` remain canonical policy/evaluation helpers for allow/deny semantics.
- `RuntimePermissionResolver` composes relation-store/cache/role-assignment adapters and supports unified resolver error mapping (`Into<E>`) so integration layers can keep their own adapter-specific error types; the same resolver contract is used for both read-path permission resolution and write-path role assignment/replacement/removal use-cases.
- Runtime contract is single-engine and mode-free: authorization executes only through `casbin_only`, without rollout env switches or compatibility aliases in the live API surface.
- `permission_authorizer` routes allow/deny through the real Casbin library-backed evaluator and stamps the active engine into `AuthorizationDecision`.
- Internal Casbin permission evaluation now lives behind `casbin_evaluator` + `permission_check`; relation-vs-shadow rollout logic is no longer part of the module contract.
- `integration` exports canonical RBAC cross-module event contract for role-assignment change notifications: `RbacRoleAssignmentEvent`, `RbacIntegrationEventKind`, and stable event-type constants (`rbac.role_permissions_assigned`, `rbac.user_role_replaced`, `rbac.tenant_role_assignments_removed`, `rbac.user_role_assignment_removed`). Integration payloads are `serde`-serializable (`snake_case` enum tags) for transport-agnostic publish/consume flows.

## Observability

Canonical runtime signals for the live contract:

- `rustok_rbac_permission_cache_hits`
- `rustok_rbac_permission_cache_misses`
- `rustok_rbac_permission_checks_allowed`
- `rustok_rbac_permission_checks_denied`
- `rustok_rbac_claim_role_mismatch_total`
- `rustok_rbac_engine_decisions_casbin_total`
- `rustok_rbac_engine_eval_duration_ms_total`
- `rustok_rbac_engine_eval_duration_samples`

Consistency metrics over relation data remain valid operational signals because relation data is still the source of truth. Old parity telemetry from the cutover period is no longer part of the live runtime contract.


## Ownership and release gates

- **Module owner:** Platform foundation team (`apps/server` + `crates/rustok-core` ownership group from repository map).
- **Change scope requiring owner review:**
  - public API exports in `crates/rustok-rbac/src/lib.rs`;
  - resolver contracts (`PermissionResolver`, `RuntimePermissionResolver`, relation/cache assignment traits);
- public RBAC API exports and single-engine authorization contract;
  - integration event contracts under `integration` (`rbac.*` event-type constants and payload structs).
- **Release gate checklist for RBAC module changes:**
  1. Unit tests for changed domain logic are present/updated in `crates/rustok-rbac/src/**` (or explained why not needed).
  2. `rustfmt` passes for touched Rust files.
  3. `apps/server` adapter compatibility is validated (compile/tests in network-enabled CI or documented local limitation).
  4. Module docs are updated (`crates/rustok-rbac/docs/*`) and server/verification docs stay synced (`apps/server/docs/README.md`, `docs/verification/rbac-server-modules-verification-plan.md`).


## Runtime posture

Current operating model for integration layers (`apps/server` and future transports):

1. Runtime executes through `casbin_only`.
2. Rollout-mode env switching is removed from the live contract.
3. Integrations should treat Casbin as the only supported authorization engine.
