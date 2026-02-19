# rustok-mcp

`rustok-mcp` is the MCP adapter crate for RusToK. It uses the official Rust SDK (`rmcp`) and
exposes RusToK tools/resources by wiring them to platform services.

## Goals

- Keep MCP support as a thin adapter layer.
- Reuse the `rmcp` SDK for protocol, schema, and transport handling.
- Expose domain operations via typed tools with generated JSON Schemas.
- Return tool payloads in a standard response envelope (`McpToolResponse`).

## Tooling overview

- `list_modules`: list all registered modules.
- `query_modules`: list modules with pagination and filters.
- `module_exists` / `module_details`: module lookups by slug.
- `content_module` / `blog_module` / `forum_module` / `pages_module`: domain module metadata.
- `mcp_health`: readiness snapshot for MCP server.

## Quick start

```rust
use rustok_core::registry::ModuleRegistry;
use rustok_mcp::McpServerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let registry = ModuleRegistry::new();
    let config = McpServerConfig::new(registry);
    rustok_mcp::serve_stdio(config).await
}
```

To enable a tool allow-list:

```rust
use std::collections::HashSet;

use rustok_core::registry::ModuleRegistry;
use rustok_mcp::McpServerConfig;

let registry = ModuleRegistry::new();
let enabled = HashSet::from([
    "list_modules".to_string(),
    "mcp_health".to_string(),
]);
let config = McpServerConfig::with_enabled_tools(registry, enabled);
```

For more details see `docs/implementation-plan.md`.


## Взаимодействие
- встроенный binary target `rustok-mcp-server`
- crates/rustok-core (registry/services)
- доменные модули через service layer

## Документация
- Локальная документация: `./docs/`
- План реализации MCP-модуля: `./docs/implementation-plan.md`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Адаптер MCP-инструментов поверх Rust SDK (`rmcp`) для RusToK сервисов.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - встроенный binary target `rustok-mcp-server`
  - crates/rustok-core (registry/services)
  - доменные модули через service layer
- **Точки входа:**
  - `crates/rustok-mcp/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

