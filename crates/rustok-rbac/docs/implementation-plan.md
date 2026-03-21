# rustok-rbac module implementation plan (`rustok-rbac`)

## Scope and objective

This document captures the finalized implementation status for `rustok-rbac` in RusToK and
records the migration outcome for the module-level RBAC runtime contract.

Primary objective: keep `rustok-rbac` aligned with platform-level contracts while
documenting the completed transition to the Casbin-backed runtime.

## Target architecture

- `rustok-rbac` remains focused on its bounded context and public crate API.
- Integrations with other modules go through stable interfaces in `rustok-core`
  (or dedicated integration crates where applicable).
- Behavior changes are introduced through additive, backward-compatible steps.
- Observability and operability requirements are part of delivery readiness.

## Delivery phases

### Phase 0 — Foundation (done)

- [x] Baseline crate/module structure is in place.
- [x] Base docs and registry presence are established.
- [x] Core compile-time integration with the workspace is available.

### Phase 1 — Contract hardening (done)

- [x] Freeze initial public RBAC runtime API: exported `permission_policy`/`permission_evaluator` + trait contract `PermissionResolver`/`PermissionResolution` with default use-case methods (`has_*`) for adapter-driven integrations.
- [x] Introduce shared permission-policy helpers (`permission_policy`) and start consuming them from `apps/server` extractors/service wiring to reduce server-owned policy logic.
- [x] Introduce shared permission evaluation API (`permission_evaluator`) and move allow/deny + missing-permissions outcome assembly from server-side RBAC wiring into `rustok-rbac`.
- [x] Align error/validation conventions with platform guidance and then collapse the temporary rollout-mode parsing surface once the single-engine Casbin cutover was finalized.
- [x] Expand automated tests around core invariants and boundary behavior (including stable normalized permission payload from both relation and cache paths, empty-requirements decision contract, and resolver error propagation in `permission_authorizer`).

### Phase 2 — Domain expansion (done)

- [x] Implement prioritized domain capabilities for `rustok-rbac` (module now owns `permission_authorizer` use-case evaluation, relation-resolve orchestration via `RelationPermissionStore`, shared cache-aware resolver path (`resolve_permissions_with_cache` + `PermissionCache`) and runtime resolver service `RuntimePermissionResolver` with assignment contract `RoleAssignmentStore` (including role-assignment removal operations); `apps/server` consumes module runtime resolver instead of local `ServerPermissionResolver`).
- [x] Remove rollout-mode parsing from the live module contract after finalizing the Casbin-only runtime.
- [x] Keep internal permission-check shape primitives inside `rustok-rbac` so `apps/server` keeps only transport/observability concerns.
- [x] Route module-level authorization through the real Casbin library so `apps/server` does not keep a local engine implementation.
- [x] Standardize cross-module integration points and events (published canonical RBAC role-assignment integration event contract: `RbacRoleAssignmentEvent` + `RbacIntegrationEventKind` + stable `rbac.*` event-type constants).
- [x] Document ownership and release gates for new capabilities (added module owner, review boundaries, and release-gate checklist to `crates/rustok-rbac/docs/README.md`).

### Phase 3 — Productionization (done for current migration scope)

- [x] Finalize rollout and migration strategy for incremental adoption (documented final single-engine runtime posture and compatibility behavior in `crates/rustok-rbac/docs/README.md`).
- [x] Complete security/tenancy/rbac checks relevant to the migration contract.
- [x] Validate observability, runbooks, and operational readiness for the Casbin-backed runtime contract.

## Current status

- `rustok-rbac` is the canonical module boundary for RBAC runtime logic.
- Runtime authorization executes through the real `casbin` library.
- Authorization runtime is fixed to `casbin_only`; the crate no longer exposes a rollout-mode switch.
- Migration backlog is closed. Ongoing work is limited to steady-state hardening and drift prevention.

## Ongoing hardening backlog

### Phase 4 - Verification and drift prevention

- [ ] Keep the periodic verification cycle green using `docs/verification/rbac-server-modules-verification-plan.md`.
- [ ] Continue removing authorization drift where presentation-oriented role inference still exists outside the primary RBAC path.
- [ ] Keep runtime-module `permissions()` / `dependencies()` / `README.md -> Interactions` aligned as modules evolve.
- [ ] Expand module-level and server-level guardrails whenever a new RBAC-managed surface is added.

## Tracking and updates

When updating `rustok-rbac` architecture, API contracts, tenancy behavior, routing,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-rbac/README.md` and `crates/rustok-rbac/docs/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.
5. If the live RBAC contract changes, update `apps/server/docs/README.md` and the RBAC verification plan.

## Checklist

- [x] контрактные тесты покрывают все публичные use-case.


