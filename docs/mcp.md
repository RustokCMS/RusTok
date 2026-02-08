# MCP adapter (rustok-mcp)

RusToK ships an MCP adapter that uses the official Rust SDK (`rmcp`). The goal is to keep MCP
support as a thin integration layer: the SDK handles protocol details while the adapter maps
RusToK domain logic to MCP tools.

## Architecture overview

```
rmcp (official MCP Rust SDK)
   ↓
rustok-mcp (adapter: tools/resources + wiring)
   ↓
rustok-core (business modules + registry)
```

The adapter exposes **tools** for MCP hosts. Each tool simply forwards to RusToK logic (for
example, reading from the module registry). Because schemas are derived via `schemars`, tools
carry accurate JSON Schema definitions to clients.

## Provided tools

| Tool | Description |
| --- | --- |
| `list_modules` | Return module metadata from the registry. |
| `module_exists` | Check if a module is registered by slug. |

## Running the MCP server

The MCP server is a separate binary (`apps/mcp`) and communicates over stdio:

```bash
cargo run -p rustok-mcp-server
```

It is intentionally isolated from the main backend (`apps/server`) so MCP traffic can be
managed and deployed independently. To wire actual modules, construct a `ModuleRegistry` in
`apps/mcp` the same way the backend does (register modules, then pass it to
`McpServerConfig`).

## Extending tools

1. Add a new tool function in `crates/rustok-mcp/src/tools.rs` and annotate it with
   `#[rmcp::tool]`.
2. Use `schemars::JsonSchema` on request/response DTOs for automatic schema generation.
3. Register the tool in `crates/rustok-mcp/src/server.rs`.

## Notes

- The adapter should remain thin; heavy logic belongs in domain services.
- Updating MCP protocol behavior should be handled by updating the `rmcp` dependency.

This is an alpha version and requires clarification. Be careful, there may be errors in the text. So that no one thinks that this is an immutable rule.
