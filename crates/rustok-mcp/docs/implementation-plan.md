# MCP module implementation plan (`rustok-mcp`)

## Scope and objective

This document captures the current implementation plan for the MCP module in RusToK and
serves as the source of truth for rollout sequencing in `crates/rustok-mcp`.

Primary objective: incrementally expand MCP capabilities without coupling business domains
to transport/protocol details.

## Target architecture

- `rustok-mcp` stays a thin adapter over `rmcp`.
- Domain logic remains in platform/domain services (`rustok-core` + `rustok-*` modules).
- MCP exposes typed tools/resources with stable contracts and versioned evolution.
- Runtime supports dual-mode delivery:
  - library mode (`rustok_mcp` as embeddable crate);
  - binary mode (`rustok-mcp-server` for standalone stdio/server usage).

## Delivery phases

### Phase 0 — Foundation (done)

- [x] Baseline crate structure (`lib`, `server`, `tools`, tests).
- [x] Integrated official Rust MCP SDK (`rmcp`).
- [x] Introduced dual-mode packaging (library + binary).
- [x] Initial docs and integration points with module registry.
- [x] Module discovery tools (`list_modules`, `module_exists`, `module_details`).
- [x] Tests for module discovery tools.

### Phase 1 — Contract hardening (in progress)

- [ ] Freeze tool naming conventions and argument schemas.
- [ ] Define response/error envelope policy for MCP tools.
- [ ] Add compatibility matrix for client versions.
- [ ] Expand integration tests for schema and transport behavior.

### Phase 2 — Domain expansion (planned)

- [ ] Add content/page/blog/forum/domain-oriented MCP tools via service layer.
- [ ] Introduce pagination/filter standards across tool outputs.
- [ ] Add observability defaults (structured logs, tracing spans, basic metrics).
- [ ] Define module-level ownership and release gates for each new tool group.

### Phase 3 — Productionization (planned)

- [ ] Add rollout strategy (feature flags/capability gates).
- [ ] Finalize security hardening checklist for tool execution.
- [ ] Add SLO-aligned readiness checks and operational runbook.
- [ ] Complete production support policy and upgrade playbook.

## Status section: virtual users and RBAC access

> Status: **planned, not yet exposed as a production-ready MCP API in the current module**.

### What is planned (but not enabled as production MCP API)

- Virtual users model for non-human/automation MCP actors.
- RBAC-aware capability checks for MCP tool invocations.
- Role/scope mapping between MCP identities and RusToK permission model.
- Audit trail requirements for privileged tool execution under virtual identities.

### What is already completed for this stream

- ✅ Dual-mode module shape is in place (library + binary delivery model).
- ✅ Readiness posture is evaluated at planning level and included in rollout thinking.
- ✅ A detailed MCP + RBAC roadmap is now fixed in module documentation and can be
  tracked as part of module-level planning.

### Entry criteria for enabling production API

Before exposing virtual users + RBAC as production MCP API:

1. Permission model must be explicitly documented (roles/scopes/tenancy boundaries).
2. End-to-end authorization checks must be validated by tests.
3. Auditability requirements must be implemented and observable.
4. Backward-compatible migration path must be documented for existing MCP clients.

## Tracking and updates

When updating MCP architecture, API contracts, tenancy behavior, routing of tools,
or observability expectations:

1. Update this file first.
2. Update `crates/rustok-mcp/README.md` when public behavior changes.
3. Update `docs/index.md` links if documentation structure changes.
4. If module responsibilities change, update `docs/modules/registry.md` accordingly.
