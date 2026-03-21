# MCP identity и tool policy foundation в `rustok-mcp`

- Date: 2026-03-19
- Status: Accepted

## Context

`rustok-mcp` уже использовался как thin adapter поверх официального Rust SDK `rmcp`, но до этого
контроль доступа фактически сводился к coarse-grained allow-list через `enabled_tools`.

Для RusToK этого недостаточно:

- нужно различать human, service и model actors;
- нужен явный слой identity/scopes/permissions для MCP boundary;
- нельзя превращать `enabled_tools` в ложную замену полноценной authz-модели;
- при этом не хочется ломать существующий stdio/runtime surface и создавать новый dependency cycle.

## Decision

Принято решение заложить foundation access-layer прямо внутри `rustok-mcp`, не превращая его в
runtime-модуль и не вынося пока в отдельный persisted management subsystem.

В рамках foundation:

- `rustok-mcp` остаётся capability/adaptor crate поверх `rmcp`;
- вводятся публичные типы `McpIdentity`, `McpAccessContext`, `McpAccessPolicy`,
  `McpToolRequirement`, `McpWhoAmIResponse`;
- authorization tool calls строится как:
  1. legacy `enabled_tools`;
  2. затем identity/policy/permissions/scopes через `McpAccessContext`;
- добавляется introspection tool `mcp_whoami`;
- `enabled_tools` сохраняется как compatibility shim, а не как долгосрочная authz-модель.

Persisted clients/tokens/policies/audit trail и management API/UI остаются следующим слоем и не
считаются частью этого решения.

## Consequences

- У RusToK появляется реальный MCP identity/policy foundation без слома текущих клиентов.
- Появляется явная точка интеграции для будущего management API и admin UI.
- Документация должна ссылаться на официальный MCP/rmcp upstream как на источник истины по
  протоколу, security и authorization semantics.
- Следующим шагом нужно проектировать persisted модели MCP clients/tokens/policies/audit и их API.
