# rustok-rbac docs

В этой папке хранится документация модуля `crates/rustok-rbac`.

## Documents

- [Implementation plan](./implementation-plan.md)

## Event contracts

- [Event flow contract (central)](../../../docs/architecture/event-flow-contract.md)


## Runtime contracts

- `PermissionResolver` / `PermissionResolution` define tenant-aware RBAC resolver contract for adapter-based integrations (`apps/server` and future transports).
- `permission_policy` and `permission_evaluator` remain canonical policy/evaluation helpers for allow/deny semantics.
