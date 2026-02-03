use anyhow::Result;

use rustok_core::registry::ModuleRegistry;

use crate::tools::{list_modules, module_exists, McpState};

pub struct McpServerConfig {
    pub registry: ModuleRegistry,
}

impl McpServerConfig {
    pub fn new(registry: ModuleRegistry) -> Self {
        Self { registry }
    }
}

pub async fn serve_stdio(config: McpServerConfig) -> Result<()> {
    let state = McpState {
        registry: config.registry,
    };

    rmcp::McpServer::new()
        .register_tool(list_modules)
        .register_tool(module_exists)
        .with_state(state)
        .serve(rmcp::transport::stdio::StdioTransport::default())
        .await?;

    Ok(())
}
