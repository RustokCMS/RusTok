# RBAC verification plan for server and runtime modules

- **Status:** In progress
- **Mode:** Rolling verification
- **Full cadence:** once per week, plus any time RBAC, server transport, or runtime-module contracts change
- **Goal:** keep the live RBAC contract aligned across `apps/server`, `rustok-rbac`, and the runtime modules registered in `build_registry()`

---

## 1. Scope

### 1.1 Server surfaces

- `apps/server`
- REST RBAC extractors
- GraphQL mutations, queries, and subscriptions
- `RbacService`
- permission-aware `SecurityContext` creation

### 1.2 Runtime modules from `build_registry()`

- `auth`
- `cache`
- `email`
- `index`
- `tenant`
- `rbac`
- `content`
- `commerce`
- `blog`
- `forum`
- `pages`
- `workflow`

### 1.3 Capability surfaces checked together with RBAC

- Alloy transport/runtime (`rustok-alloy` + `alloy-scripting`)

---

## 2. Source of truth

Every verification pass must treat these contracts as canonical:

1. `RusToKModule::permissions()`
2. `RusToKModule::dependencies()`
3. `crates/<module>/README.md` -> `## Interactions`
4. `apps/server` RBAC path
5. `rustok-core::Permission`, `Resource`, `Action`

For Alloy capability surfaces, use:

1. `rustok-alloy` GraphQL/REST guards
2. `alloy-scripting` README/docs
3. live server composition in `apps/server`

---

## 3. Invariants

The agent must verify that these invariants still hold:

- No live server authorization path uses hardcoded `UserRole::Admin` or `UserRole::SuperAdmin`
  as a substitute for permission checks.
- `infer_user_role_from_permissions()` is used only for presentation, compatibility, or telemetry.
- Runtime modules with an RBAC surface publish it through `permissions()`.
- Runtime modules document their interactions in root `README.md`.
- `pages` depends on `content`.
- `blog` depends on `content`.
- `forum` depends on `content`.
- `workflow` has no runtime dependency on Alloy.
- Alloy does not participate in tenant module lifecycle.
- `apps/server` creates `SecurityContext` from resolved permissions, not from inferred role only.
- Flex field-definition mutations use explicit typed permissions, not role inference.

---

## 4. Weekly checklist

### A. Registry and module contract

- [ ] Run `cargo test -p rustok-server modules::contract_tests --lib`
- [ ] Verify every runtime module README still contains `## Interactions`
- [ ] Verify `permissions()` is non-empty for modules that expose RBAC-managed functionality:
  `auth`, `tenant`, `rbac`, `content`, `commerce`, `blog`, `forum`, `pages`, `workflow`
- [ ] Verify dependency edges are still correct:
  `blog -> content`, `forum -> content`, `pages -> content`
- [ ] Verify Alloy capability docs still state that Alloy is not a runtime module

### B. Server authorization path

- [ ] Search `apps/server/src` for forbidden role-based authorization patterns
- [ ] Verify GraphQL and REST entry points still go through `RbacService`, RBAC extractors,
  or permission-aware `SecurityContext`
- [ ] Verify Alloy GraphQL/REST still checks `scripts:*` without `tenant_modules.is_enabled("alloy")`

### C. Typed permission vocabulary

- [ ] Verify server-only RBAC surfaces still use typed permissions from `rustok-core`
- [ ] Verify no new ad-hoc string-based permission names or local role aliases were introduced
- [ ] Verify module ownership of permissions still matches current server callsites

### D. Runtime behavior

- [ ] Run focused RBAC/metrics/server slices
- [ ] Run full `cargo test -p rustok-server --lib`
- [ ] Confirm failures, if any, are triaged as RBAC drift, module contract drift, capability drift, or unrelated

### E. Documentation freshness

- [ ] Verify `docs/modules/registry.md` still matches runtime registry vs capability split
- [ ] Verify `docs/modules/crates-registry.md` still matches crate ownership
- [ ] Verify local README/docs for `rustok-alloy`, `alloy-scripting`, and `rustok-workflow` still match code

---

## 5. Commands

### 5.1 Contract and grep checks

```powershell
cargo test -p rustok-server modules::contract_tests --lib
git grep -n "infer_user_role_from_permissions" -- apps/server/src
git grep -n "UserRole::Admin\|UserRole::SuperAdmin" -- apps/server/src
git grep -n "require_module_enabled(.*alloy" -- crates/rustok-alloy apps/server/src
```

### 5.2 Focused runtime slices

```powershell
cargo test -p rustok-server rbac --lib
cargo test -p rustok-server metrics --lib
cargo test -p rustok-server flex --lib
```

### 5.3 Full server gate

```powershell
cargo test -p rustok-server --lib
```

### 5.4 Optional workspace spot-checks after RBAC changes

```powershell
cargo test -p rustok-core --lib
cargo test -p rustok-rbac --lib
cargo test -p rustok-blog --lib
cargo test -p rustok-forum --lib
cargo test -p rustok-pages --lib
cargo test -p rustok-workflow --lib
cargo test -p rustok-alloy --lib
```

---

## 6. Evidence to keep

Each verification run should leave a short evidence bundle with:

- date
- branch or commit
- commands executed
- pass/fail result
- list of RBAC drift findings
- list of module/capability doc drift findings
- list of fixes made
- remaining blockers

Preferred location:

- `artifacts/verification/rbac-server-modules/<yyyy-mm-dd>.md`

---

## 7. Stop-the-line conditions

Treat any of the following as blocking drift:

- a live server path authorizes by role shortcut instead of explicit permissions
- a runtime module with RBAC-managed behavior has empty or stale `permissions()`
- a runtime module README no longer documents `## Interactions`
- Alloy capability paths reintroduce tenant module gating
- `SecurityContext` is built without resolved permissions on the server path
- full `cargo test -p rustok-server --lib` fails due to RBAC or module-contract regressions
