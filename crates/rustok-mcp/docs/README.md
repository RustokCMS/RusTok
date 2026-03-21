# Документация `rustok-mcp`

В этой папке хранится только локальная документация по интеграции MCP в RusToK.

Важно: эта документация не должна подменять собой официальный spec/SDK MCP. Если речь идёт о
протоколе, capability surface, security или authorization flow, источником истины считаются
официальные документы MCP/rmcp, а локальные файлы фиксируют только:

- как именно RusToK использует MCP;
- что уже реализовано в `crates/rustok-mcp`;
- как session-start runtime binding и allow/deny audit связываются с persisted server-layer;
- как первый реальный Alloy product-slice (`alloy_scaffold_module`) и его review/apply boundary вписываются в MCP transport;
- как persisted scaffold draft control plane в `apps/server` подключается к runtime MCP flow через pluggable draft-store contract;
- какие платформенные ограничения и gap-ы у нас ещё открыты.

## Внешние источники истины

- Официальная документация MCP: [modelcontextprotocol.io/docs](https://modelcontextprotocol.io/docs)
- Официальная спецификация MCP: [modelcontextprotocol.io/specification](https://modelcontextprotocol.io/specification/2025-03-26)
- Документация Rust SDK `rmcp`: [docs.rs/rmcp](https://docs.rs/rmcp/latest/rmcp/)
- Репозиторий Rust SDK: [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
- Authorization guide: [Understanding Authorization in MCP](https://modelcontextprotocol.io/docs/tutorials/security/authorization)
- Security guide: [Security Best Practices](https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices)

## Содержимое

- [План реализации](./implementation-plan.md)
- [Центральный reference-индекс MCP](../../../docs/references/mcp/README.md)
