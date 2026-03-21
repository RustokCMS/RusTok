use rustok_core::module::RusToKModule;
use rustok_core::registry::ModuleRegistry;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const TOOL_LIST_MODULES: &str = "list_modules";
pub const TOOL_QUERY_MODULES: &str = "query_modules";
pub const TOOL_MODULE_EXISTS: &str = "module_exists";
pub const TOOL_MODULE_DETAILS: &str = "module_details";
pub const TOOL_CONTENT_MODULE: &str = "content_module";
pub const TOOL_BLOG_MODULE: &str = "blog_module";
pub const TOOL_FORUM_MODULE: &str = "forum_module";
pub const TOOL_PAGES_MODULE: &str = "pages_module";
pub const TOOL_MCP_HEALTH: &str = "mcp_health";
pub const TOOL_MCP_WHOAMI: &str = "mcp_whoami";

pub const MODULE_CONTENT: &str = "content";
pub const MODULE_BLOG: &str = "blog";
pub const MODULE_FORUM: &str = "forum";
pub const MODULE_PAGES: &str = "pages";

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

/// Request to filter and page through modules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleQueryRequest {
    /// Optional slug prefix filter
    pub slug_prefix: Option<String>,
    /// Optional dependency filter
    pub dependency: Option<String>,
    /// Max number of items to return
    pub limit: Option<usize>,
    /// Offset into the module list
    pub offset: Option<usize>,
}

/// Response indicating whether a module exists
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleLookupResponse {
    /// The slug that was queried
    pub slug: String,
    /// Whether the module exists
    pub exists: bool,
}

/// Response containing module details, if found
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ModuleDetailsResponse {
    /// The slug that was queried
    pub slug: String,
    /// Module details when present
    pub module: Option<ModuleInfo>,
}

/// Standard response envelope for MCP tool responses
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolResponse<T> {
    /// Indicates whether the tool executed successfully
    pub ok: bool,
    /// Payload for successful responses
    pub data: Option<T>,
    /// Error details for unsuccessful responses
    pub error: Option<McpToolError>,
}

/// Error payload for MCP tool responses
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolError {
    /// Machine-readable error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

/// Health response for MCP readiness checks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpHealthResponse {
    /// Readiness status
    pub status: String,
    /// MCP protocol version
    pub protocol_version: String,
    /// Number of registered tools
    pub tool_count: usize,
    /// List of enabled tools (when configured)
    pub enabled_tools: Option<Vec<String>>,
    /// Effective authorization mode for this server.
    pub access_mode: String,
    /// Attached MCP identity, when configured.
    pub identity: Option<crate::access::McpIdentity>,
}

impl<T> McpToolResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            data: None,
            error: Some(McpToolError {
                code: code.into(),
                message: message.into(),
            }),
        }
    }
}

fn to_module_info(module: &dyn RusToKModule) -> ModuleInfo {
    ModuleInfo {
        slug: module.slug().to_string(),
        name: module.name().to_string(),
        description: module.description().to_string(),
        version: module.version().to_string(),
        dependencies: module
            .dependencies()
            .iter()
            .map(|dep| dep.to_string())
            .collect(),
    }
}

/// List all registered modules
pub async fn list_modules(state: &McpState) -> ModuleListResponse {
    let modules = state
        .registry
        .list()
        .into_iter()
        .map(to_module_info)
        .collect();

    ModuleListResponse { modules }
}

/// List registered modules with filtering and pagination
pub async fn list_modules_filtered(
    state: &McpState,
    request: ModuleQueryRequest,
) -> ModuleListResponse {
    let modules = state.registry.list();
    let filtered = modules.into_iter().filter(|module| {
        let slug = module.slug();
        if let Some(prefix) = request.slug_prefix.as_deref() {
            if !slug.starts_with(prefix) {
                return false;
            }
        }
        if let Some(dependency) = request.dependency.as_deref() {
            if !module.dependencies().iter().any(|dep| dep == &dependency) {
                return false;
            }
        }
        true
    });

    let offset = request.offset.unwrap_or(0);
    let limit = request.limit.unwrap_or(usize::MAX);
    let modules = filtered
        .skip(offset)
        .take(limit)
        .map(to_module_info)
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

/// Fetch module details by slug
pub async fn module_details(
    state: &McpState,
    request: ModuleLookupRequest,
) -> ModuleDetailsResponse {
    module_details_by_slug(state, &request.slug)
}

/// Fetch module details by slug string
pub fn module_details_by_slug(state: &McpState, slug: &str) -> ModuleDetailsResponse {
    let module = state.registry.get(slug).map(to_module_info);

    ModuleDetailsResponse {
        slug: slug.to_string(),
        module,
    }
}
