# rustok-rbac docs

В этой папке хранится документация модуля `crates/rustok-rbac`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)


## Runtime contracts

- `PermissionResolver` / `PermissionResolution` define tenant-aware RBAC resolver contract for adapter-based integrations (`apps/server` and future transports), including default `has_permission/has_any_permission/has_all_permissions` use-cases powered by module evaluator APIs.
- `permission_policy` and `permission_evaluator` remain canonical policy/evaluation helpers for allow/deny semantics.
- `RuntimePermissionResolver` composes relation-store/cache/role-assignment adapters and supports unified resolver error mapping (`Into<E>`) so integration layers can keep their own adapter-specific error types; the same resolver contract is used for both read-path permission resolution and write-path role assignment/replacement use-cases.
- `RbacAuthzMode` defines shared rollout-mode parsing for relation-only/dual-read authorization execution (`RUSTOK_RBAC_AUTHZ_MODE`).

- `shadow_decision` exports legacy-vs-relation shadow comparison helpers (`ShadowCheck` + `compare_shadow_decision`, as well as `compare_single_permission/compare_any_permissions/compare_all_permissions`) to keep dual-read decision semantics in the RBAC module; `ShadowCheck::as_str()` provides a stable mode label (`single|any|all`) for adapter-level observability tags.
- `shadow_dual_read` exports `evaluate_dual_read` + `DualReadOutcome` to centralize dual-read orchestration (`skipped` + compared decision with match/mismatch semantics) and keep transport layers adapter-only.
