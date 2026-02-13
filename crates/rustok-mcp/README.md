# rustok-mcp

`rustok-mcp` is the MCP adapter crate for RusToK. It uses the official Rust SDK (`rmcp`) and
exposes RusToK tools/resources by wiring them to platform services.

## Goals

- Keep MCP support as a thin adapter layer.
- Reuse the `rmcp` SDK for protocol, schema, and transport handling.
- Expose domain operations via typed tools with generated JSON Schemas.

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

For more details see `docs/mcp.md`.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.

## Взаимодействие
- apps/mcp
- crates/rustok-core (registry/services)
- доменные модули через service layer

## Документация
- Локальная документация: `./docs/`
- Общая документация платформы: `/docs`

## Паспорт компонента
- **Роль в системе:** Адаптер MCP-инструментов поверх Rust SDK (`rmcp`) для RusToK сервисов.
- **Основные данные/ответственность:** бизнес-логика и API данного компонента; структура кода и документации в корне компонента.
- **Взаимодействует с:**
  - apps/mcp
  - crates/rustok-core (registry/services)
  - доменные модули через service layer
- **Точки входа:**
  - `crates/rustok-mcp/src/lib.rs`
- **Локальная документация:** `./docs/`
- **Глобальная документация платформы:** `/docs/`

