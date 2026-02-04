use anyhow::{bail, Result};

use rustok_core::registry::ModuleRegistry;

use crate::tools::McpState;

pub struct McpServerConfig {
    pub registry: ModuleRegistry,
}

impl McpServerConfig {
    pub fn new(registry: ModuleRegistry) -> Self {
        Self { registry }
    }
}

pub async fn serve_stdio(config: McpServerConfig) -> Result<()> {
    let _state = McpState {
        registry: config.registry,
    };

    bail!("rmcp server entrypoint not available in the current SDK version");
}
