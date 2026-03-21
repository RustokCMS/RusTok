# План реализации `rustok-mcp`

## Назначение документа

Этот документ фиксирует только RusToK-специфичный слой реализации `crates/rustok-mcp`:

- что мы уже реализовали поверх `rmcp`;
- какие capability MCP мы реально подняли в RusToK;
- какие архитектурные пробелы ещё остаются до platform-grade MCP слоя.

Документ не должен пересказывать или переписывать спецификацию MCP.

## Источник истины по протоколу и SDK

Если вопрос касается самого протокола MCP, поведения SDK, security или authorization flow,
источником истины считаются внешние документы:

- MCP docs: [modelcontextprotocol.io/docs](https://modelcontextprotocol.io/docs)
- MCP spec: [modelcontextprotocol.io/specification](https://modelcontextprotocol.io/specification/2025-03-26)
- `rmcp` docs: [docs.rs/rmcp](https://docs.rs/rmcp/latest/rmcp/)
- Rust SDK repo: [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
- Authorization: [Understanding Authorization in MCP](https://modelcontextprotocol.io/docs/tutorials/security/authorization)
- Security: [Security Best Practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)

Локальная документация должна ссылаться на эти источники, а не копировать их содержимое.

## Текущая архитектурная роль `rustok-mcp`

- `rustok-mcp` остаётся thin adapter crate поверх `rmcp`;
- доменная логика не живёт внутри MCP-слоя и остаётся в platform/domain services;
- Alloy подключается как capability через `AlloyMcpState`, а не как отдельный MCP runtime;
- локальный MCP-слой сейчас покрывает typed tools, response envelope, access policy, runtime binding и первый реальный Alloy product-slice.

## Что реально реализовано

### Foundation

- [x] crate shape: `lib`, `server`, `tools`, tests
- [x] интеграция с официальным Rust SDK `rmcp`
- [x] dual-mode delivery: library + binary (`rustok-mcp-server`)
- [x] стабильный envelope `McpToolResponse`
- [x] compatibility policy для protocol version и response envelope

### Tool surface

- [x] module discovery tools: `list_modules`, `query_modules`, `module_exists`, `module_details`
- [x] domain module metadata tools: `content_module`, `blog_module`, `forum_module`, `pages_module`
- [x] readiness tool: `mcp_health`
- [x] Alloy script/runtime tools при наличии `AlloyMcpState`
- [x] `alloy_scaffold_module` как первый реальный `AI -> MCP -> Alloy -> Platform` tool
- [x] review/apply boundary через `alloy_review_module_scaffold` и `alloy_apply_module_scaffold`
- [x] persisted server-side scaffold draft control plane в `apps/server`
- [x] pluggable runtime draft-store contract и live binding к persisted server-side scaffold drafts через `DbBackedMcpRuntimeBridge`

### Operational baseline

- [x] allow-list инструментов через `enabled_tools`
- [x] базовый structured logging вокруг tool calls
- [x] schema-driven argument parsing через `schemars`
- [x] identity/policy foundation: `McpIdentity`, `McpAccessContext`, `McpAccessPolicy`
- [x] permission-aware authorization для tool calls с compatibility shim поверх legacy `enabled_tools`
- [x] introspection tool `mcp_whoami`
- [x] session-start runtime binding hooks: `McpSessionContext`, `McpAccessResolver`, `McpRuntimeBinding`
- [x] runtime allow/deny audit hook через `McpAuditSink` и `McpToolCallAuditEvent`

## Первый реальный Alloy product-slice

`alloy_scaffold_module` нужен не ради ещё одного tool, а как первая честная вертикаль между AI и
платформой.

Что он делает сейчас:

- принимает структурированный `ScaffoldModuleRequest`;
- возвращает preview набора файлов draft-модуля и staging draft id;
- отдаёт отдельный review step через `alloy_review_module_scaffold`;
- выносит реальную запись в workspace в `alloy_apply_module_scaffold` с `confirm=true`;
- не перезаписывает существующий crate;
- не регистрирует модуль автоматически в runtime.

Что это означает:

- это уже не “заглушка ради демонстрации”;
- это ещё не полноценная автоматическая генерация production-модуля;
- это управляемый scaffolding-step с явной review/apply границей, на котором можно строить persisted codegen pipeline дальше.

## Что не реализовано

Следующие части MCP-экосистемы пока не подняты в RusToK как production-ready surface:

- [ ] `resources`
- [ ] `prompts`
- [ ] `roots`
- [ ] `sampling`
- [ ] `logging` как полноценная MCP capability
- [ ] `completions`
- [ ] subscriptions/streaming surface за пределами текущего tool model

Это значит, что `rustok-mcp` сегодня нельзя описывать как “полную реализацию MCP для RusToK”.
Корректное описание: это governed MCP tool adapter поверх `rmcp` с Alloy-related extensions.

## Критический архитектурный gap: identities и authorization

Статус: **foundation реализован, но platform-grade remote MCP surface ещё не завершён**.

### Что уже есть

- persisted модели MCP client/token/policy/audit в `apps/server`;
- management API для клиентов, токенов, policy и аудита в `apps/server` (REST `/api/mcp/*` + GraphQL `mcp*`);
- server-owned `DbBackedMcpRuntimeBridge`, который резолвит plaintext MCP token в `McpAccessContext` на старте stdio-сессии;
- runtime audit allow/deny для tool invocations через `McpAuditSink` -> `mcp_audit_logs`.

### Чего сейчас нет

- server-owned remote MCP transport/session bootstrap beyond текущего stdio adapter path;
- admin UI поверх management API;
- полная product-модель consent/delegation для human-linked clients и model agents.

### Что это означает на практике

Текущий foundation уже включает identity + policy + permission mapping в runtime `rustok-mcp`, но
этого всё ещё недостаточно для platform-grade MCP access management. Он не заменяет:

- authorization из официальной MCP security model;
- tenant-aware access control;
- per-client/per-model consent и auditability.

Для remote MCP сценариев проектироваться нужно с опорой на официальный authorization/security
guidance, а не на локальные упрощённые допущения.

## Следующие целевые срезы

Следующие правильные слои развития `rustok-mcp` и Alloy integration:

1. Поднять server-owned remote MCP transport/session bootstrap поверх уже существующего runtime binding и persisted draft-store contract.
2. Довести audit trail до более богатого execution telemetry поверх текущего allow/deny слоя.
3. Добавить UI-слой для администрирования MCP доступа и Alloy draft review.
4. Расширить Alloy surface от draft scaffolding к более богатому codegen pipeline: permission/resource generation, runtime wiring hints, дальнейшая компиляция/публикация.

## Правило сопровождения документации

При изменениях в `crates/rustok-mcp/**`:

1. Сначала сверить изменения с официальными MCP/rmcp документами.
2. Обновить этот файл только в части RusToK integration behavior.
3. Обновить [`../README.md`](../README.md), если изменилось публичное поведение crate.
4. Обновить [`../../../docs/references/mcp/README.md`](../../../docs/references/mcp/README.md),
   если изменился локальный reference-index.
5. Обновить [`../../../docs/index.md`](../../../docs/index.md), если изменилась карта документации.
