# rustok-rbac docs

В этой папке хранится документация модуля `crates/rustok-rbac`.

## Documents

- [Implementation plan](./implementation-plan.md)

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
  4. Module docs are updated (`crates/rustok-rbac/docs/*`) and, when migration milestones change, central architecture docs are synced (`docs/architecture/rbac-relation-migration-plan.md`).


## Runtime posture

Current operating model for integration layers (`apps/server` and future transports):

1. Runtime executes through `casbin_only`.
2. Rollout-mode env switching is removed from the live contract.
3. Integrations should treat Casbin as the only supported authorization engine.
