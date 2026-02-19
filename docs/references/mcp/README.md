# MCP Reference-пакет (RusToK)

Дата последней актуализации: **2026-02-19**.

> Пакет фиксирует рабочие паттерны для `crates/rustok-mcp` (адаптер MCP сервера поверх `rmcp`) и защищает от ложных переносов из ad-hoc JSON-RPC/CLI интеграций.

## 1) Минимальный рабочий пример: запуск MCP stdio сервера

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

## 2) Минимальный рабочий пример: allow-list инструментов

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

## 3) Актуальные сигнатуры API (в репозитории)

- `pub fn new(registry: ModuleRegistry) -> Self` (`McpServerConfig`)
- `pub fn with_enabled_tools<I, S>(registry: ModuleRegistry, enabled_tools: I) -> Self where I: IntoIterator<Item = S>, S: Into<String>` (`McpServerConfig`)
- `pub fn new(registry: ModuleRegistry) -> Self` (`RusToKMcpServer`)
- `pub fn with_enabled_tools(registry: ModuleRegistry, enabled_tools: HashSet<String>) -> Self` (`RusToKMcpServer`)
- `pub async fn serve_stdio(config: McpServerConfig) -> Result<()>`
- `pub async fn list_modules(state: &McpState) -> ModuleListResponse`
- `pub async fn module_exists(state: &McpState, request: ModuleLookupRequest) -> ModuleLookupResponse`
- `pub async fn module_details(state: &McpState, request: ModuleLookupRequest) -> ModuleDetailsResponse`

## 4) Чего делать нельзя (типичные ложные паттерны)

1. **Нельзя обходить typed tools и отдавать произвольный JSON без `McpToolResponse<T>`.**
2. **Нельзя хардкодить доступ ко всем инструментам, если нужен ограниченный контур.** Использовать allow-list через `with_enabled_tools`.
3. **Нельзя дублировать бизнес-логику в MCP-слое.** MCP слой — адаптер над сервисами/registry, а не отдельный доменный модуль.
4. **Нельзя придумывать transport-handshake поверх stdio.** Для сервера использовать `serve_stdio(...)` и контракт `rmcp`.

## 5) Синхронизация с кодом (регламент)

- При изменениях в `crates/rustok-mcp/**`:
  1) обновить примеры и сигнатуры;
  2) обновить дату в шапке;
  3) проверить, что anti-patterns соответствуют текущей реализации.
