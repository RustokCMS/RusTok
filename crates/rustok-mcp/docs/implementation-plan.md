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

### Phase 1 — Contract hardening (done)

- [x] Freeze tool naming conventions and argument schemas (constants in `tools.rs`).
- [x] Define response/error envelope policy for MCP tools (`McpToolResponse`).
- [x] Add compatibility matrix for client versions (see below).
- [x] Expand integration tests for schema and transport behavior.

### Compatibility matrix

| Component | Version | Notes |
| --- | --- | --- |
| MCP protocol | 2024-11-05 | Matches `rmcp::model::ProtocolVersion::V_2024_11_05`. |
| Tool response envelope | v1 | `McpToolResponse` with `ok/data/error`. |

### Phase 2 — Domain expansion (done)

- [x] Add content/page/blog/forum domain MCP tools (`content_module`, `blog_module`, `forum_module`, `pages_module`).
- [x] Introduce pagination/filter standards across tool outputs (`query_modules`).
- [x] Add observability defaults (structured logs via `tracing` around tool calls).
- [x] Define module-level ownership and release gates for each new tool group.

**Ownership and release gates**

| Tool group | Owner | Release gate |
| --- | --- | --- |
| Module discovery (`list_modules`, `query_modules`, `module_*`) | Platform foundation | Requires registry schema review + contract test updates. |
| Domain module tools (`content_module`, `blog_module`, `forum_module`, `pages_module`) | Domain module owners | Requires service-layer sign-off + changelog entry. |
| Health/ops (`mcp_health`) | Platform foundation | Requires runbook update + metrics review. |

### Phase 3 — Productionization (done)

- [x] Add rollout strategy (capability gates via `enabled_tools` in `McpServerConfig`).
- [x] Finalize security hardening checklist for tool execution.
- [x] Add SLO-aligned readiness checks and operational runbook (`mcp_health`).
- [x] Complete production support policy and upgrade playbook.

**Security hardening checklist**

- Tool allow-list enforced via configuration (`enabled_tools`).
- Consistent response envelope for errors (`McpToolResponse`).
- Tool argument validation enforced via JSON schema parsing.
- Unknown tools fail with protocol error.

**Operational runbook (summary)**

- Use `mcp_health` for readiness checks.
- Review `enabled_tools` before deployments.
- Monitor tool call logs (`tracing`) for error patterns.

**Production support policy**

- Maintain compatibility matrix for protocol + response envelope.
- Backward-compatible tool changes only; breaking changes require new tool names.
- Support window aligns with RusToK minor versions.

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

## Checklist

- [x] контрактные тесты покрывают все публичные use-case.

