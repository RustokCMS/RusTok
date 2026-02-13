use rustok_core::registry::ModuleRegistry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// State for MCP tools
#[derive(Clone)]
pub struct McpState {
    pub registry: ModuleRegistry,
}

/// Information about a RusToK module
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleInfo {
    /// Unique slug identifier for the module
    pub slug: String,
    /// Human-readable name of the module
    pub name: String,
    /// Description of the module's functionality
    pub description: String,
    /// Version of the module
    pub version: String,
    /// List of module dependencies
    pub dependencies: Vec<String>,
}

/// Response containing a list of modules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleListResponse {
    /// List of available modules
    pub modules: Vec<ModuleInfo>,
}

/// Request to check if a module exists
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleLookupRequest {
    /// The slug of the module to look up
    pub slug: String,
}

/// Response indicating whether a module exists
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleLookupResponse {
    /// The slug that was queried
    pub slug: String,
    /// Whether the module exists
    pub exists: bool,
}

/// List all registered modules
pub async fn list_modules(state: &McpState) -> ModuleListResponse {
    let modules = state
        .registry
        .list()
        .into_iter()
        .map(|module| ModuleInfo {
            slug: module.slug().to_string(),
            name: module.name().to_string(),
            description: module.description().to_string(),
            version: module.version().to_string(),
            dependencies: module
                .dependencies()
                .iter()
                .map(|dep| dep.to_string())
                .collect(),
        })
        .collect();

    ModuleListResponse { modules }
}

/// Check if a module exists by slug
pub async fn module_exists(state: &McpState, request: ModuleLookupRequest) -> ModuleLookupResponse {
    let exists = state.registry.contains(&request.slug);

    ModuleLookupResponse {
        slug: request.slug,
        exists,
    }
}
