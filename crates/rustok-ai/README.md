# rustok-ai

`rustok-ai` is RusToK's AI host/orchestrator capability crate.

It sits above `rustok-mcp`, keeps model-provider orchestration out of `rustok-mcp`, and owns the
typed runtime contracts for provider profiles, task profiles, hybrid direct/MCP execution,
chat sessions, runs, traces, and approval-gated tool execution.

Current implementation includes:
- multiprovider routing for `OpenAI-compatible`, `Anthropic`, and `Gemini`
- AI-task RBAC permissions consumed from `rustok-core` / `rustok-rbac`
- multilingual locale-aware session/run contracts with arbitrary BCP-47-style locale tags
- direct task-job execution for first-party verticals `alloy_code`, `image_asset`, and `product_copy`
- shared admin surfaces for Leptos and Next.js hosts

## Responsibilities

- Expose a provider-agnostic AI runtime centered on the `ModelProvider` trait.
- Ship native provider adapters for `OpenAI-compatible`, `Anthropic`, and `Gemini`.
- Orchestrate chat runs, direct-vs-MCP execution selection, MCP tool calls, and approval flows.
- Own task-profile-driven routing through `AiRouter` and typed execution decisions.
- Persist requested/resolved locale metadata on AI sessions and runs.
- Support direct Alloy Script Assist jobs (`list_scripts`, `get_script`, `validate_script`, `run_script`).
- Support direct media image generation jobs that persist assets through `rustok-media`.
- Support direct localized product-copy jobs that persist translations through `rustok-commerce` /
  `CatalogService`.
- Provide the canonical persisted control-plane service layer used by `apps/server`.
- Enforce the AI-host boundary separately from the MCP server boundary owned by `rustok-mcp`.
- Consume RBAC permissions from `rustok-core`/`rustok-rbac` instead of owning authorization.

## Interactions

- Uses `rustok-mcp` as the MCP server/tool surface.
- Uses direct execution mode for first-party platform workflows and MCP execution mode for
  tool/agent boundaries.
- Direct first-party verticals currently include:
  `alloy_code` for Alloy Script Assist, `image_asset` for image generation + media persistence,
  and `product_copy` for tenant-locale-bound commerce translation updates.
- Uses `apps/server` as the persisted control plane for provider profiles, tool profiles, sessions,
  task profiles, runs, traces, and approvals.
- Ships a Leptos admin UI package in `crates/rustok-ai/admin`.
- Ships a Next.js admin UI package through `apps/next-admin/packages/rustok-ai`.

## Entry points

- `ModelProvider`
- `OpenAiCompatibleProvider`
- `AnthropicProvider`
- `GeminiProvider`
- `AiRuntime`
- `AiRouter`
- `McpClientAdapter`
- `ToolExecutionPolicy`
- `ProviderProfile`, `TaskProfile`, `ExecutionMode`, `ExecutionOverride`
- `ChatSession`, `ChatMessage`, `ChatRun`
- `ToolTrace`
- `ApprovalRequest`, `ApprovalDecision`
- `AiManagementService` (`server` feature)

## Documentation

- Local component docs: [`./docs/`](./docs/)
- Leptos admin UI package: [`./admin/README.md`](./admin/README.md)
- Platform docs map: [`../../docs/index.md`](../../docs/index.md)
