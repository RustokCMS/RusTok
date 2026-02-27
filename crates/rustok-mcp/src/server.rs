use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParams, CallToolResult, Implementation, ListToolsResult, ServerInfo},
    service::{RequestContext, RoleServer},
    transport::stdio,
    ServerHandler,
};
use rustok_core::registry::ModuleRegistry;

use crate::alloy_tools::{
    alloy_create_script, alloy_delete_script, alloy_get_script, alloy_list_entity_types,
    alloy_list_scripts, alloy_run_script, alloy_script_helpers, alloy_update_script,
    alloy_validate_script, AlloyMcpState, CreateScriptRequest, DeleteScriptRequest,
    GetScriptRequest, ListScriptsRequest, RunScriptRequest, UpdateScriptRequest,
    ValidateScriptRequest, ALL_ALLOY_TOOLS, TOOL_ALLOY_CREATE_SCRIPT, TOOL_ALLOY_DELETE_SCRIPT,
    TOOL_ALLOY_GET_SCRIPT, TOOL_ALLOY_LIST_ENTITY_TYPES, TOOL_ALLOY_LIST_SCRIPTS,
    TOOL_ALLOY_RUN_SCRIPT, TOOL_ALLOY_SCRIPT_HELPERS, TOOL_ALLOY_UPDATE_SCRIPT,
    TOOL_ALLOY_VALIDATE_SCRIPT,
};
use crate::tools::{
    list_modules, list_modules_filtered, module_details, module_details_by_slug, module_exists,
    McpHealthResponse, McpState, McpToolResponse, ModuleDetailsResponse, ModuleListResponse,
    ModuleLookupRequest, ModuleLookupResponse, ModuleQueryRequest, MODULE_BLOG, MODULE_CONTENT,
    MODULE_FORUM, MODULE_PAGES, TOOL_BLOG_MODULE, TOOL_CONTENT_MODULE, TOOL_FORUM_MODULE,
    TOOL_LIST_MODULES, TOOL_MCP_HEALTH, TOOL_MODULE_DETAILS, TOOL_MODULE_EXISTS, TOOL_PAGES_MODULE,
    TOOL_QUERY_MODULES,
};
use alloy_scripting::storage::ScriptRegistry;

/// Configuration for the MCP server
pub struct McpServerConfig {
    pub registry: ModuleRegistry,
    pub enabled_tools: Option<HashSet<String>>,
}

impl McpServerConfig {
    pub fn new(registry: ModuleRegistry) -> Self {
        Self {
            registry,
            enabled_tools: None,
        }
    }

    pub fn with_enabled_tools<I, S>(registry: ModuleRegistry, enabled_tools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            registry,
            enabled_tools: Some(enabled_tools.into_iter().map(Into::into).collect()),
        }
    }
}

/// MCP Server handler for RusToK modules
pub struct RusToKMcpServer<R: ScriptRegistry + 'static = alloy_scripting::InMemoryStorage> {
    state: Arc<McpState>,
    alloy: Option<Arc<AlloyMcpState<R>>>,
    enabled_tools: Option<Arc<HashSet<String>>>,
}

impl<R: ScriptRegistry + 'static> Clone for RusToKMcpServer<R> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            alloy: self.alloy.as_ref().map(Arc::clone),
            enabled_tools: self.enabled_tools.as_ref().map(Arc::clone),
        }
    }
}

impl RusToKMcpServer<alloy_scripting::InMemoryStorage> {
    pub fn new(registry: ModuleRegistry) -> Self {
        Self {
            state: Arc::new(McpState { registry }),
            alloy: None,
            enabled_tools: None,
        }
    }

    pub fn with_enabled_tools(registry: ModuleRegistry, enabled_tools: HashSet<String>) -> Self {
        Self {
            state: Arc::new(McpState { registry }),
            alloy: None,
            enabled_tools: Some(Arc::new(enabled_tools)),
        }
    }
}

impl<R: ScriptRegistry + 'static> RusToKMcpServer<R> {
    pub fn with_alloy(registry: ModuleRegistry, alloy: AlloyMcpState<R>) -> Self {
        Self {
            state: Arc::new(McpState { registry }),
            alloy: Some(Arc::new(alloy)),
            enabled_tools: None,
        }
    }

    pub fn with_alloy_and_enabled_tools(
        registry: ModuleRegistry,
        alloy: AlloyMcpState<R>,
        enabled_tools: HashSet<String>,
    ) -> Self {
        Self {
            state: Arc::new(McpState { registry }),
            alloy: Some(Arc::new(alloy)),
            enabled_tools: Some(Arc::new(enabled_tools)),
        }
    }

    /// List all registered modules
    async fn list_modules_internal(&self) -> ModuleListResponse {
        list_modules(&self.state).await
    }

    /// Check if a module exists by slug
    async fn module_exists_internal(&self, slug: &str) -> ModuleLookupResponse {
        module_exists(
            &self.state,
            ModuleLookupRequest {
                slug: slug.to_string(),
            },
        )
        .await
    }

    /// Fetch module details by slug
    async fn module_details_internal(&self, slug: &str) -> ModuleDetailsResponse {
        module_details(
            &self.state,
            ModuleLookupRequest {
                slug: slug.to_string(),
            },
        )
        .await
    }

    /// Filter modules with pagination
    async fn list_modules_filtered_internal(
        &self,
        request: ModuleQueryRequest,
    ) -> ModuleListResponse {
        list_modules_filtered(&self.state, request).await
    }

    /// Fetch module details by static slug
    fn module_details_by_slug_internal(&self, slug: &str) -> ModuleDetailsResponse {
        module_details_by_slug(&self.state, slug)
    }

    fn tool_allowed(&self, tool_name: &str) -> bool {
        match &self.enabled_tools {
            Some(enabled) => enabled.contains(tool_name) || tool_name == TOOL_MCP_HEALTH,
            None => true,
        }
    }

    fn health_response(&self, tool_count: usize) -> McpHealthResponse {
        McpHealthResponse {
            status: "ready".to_string(),
            protocol_version: format!("{:?}", rmcp::model::ProtocolVersion::V_2024_11_05),
            tool_count,
            enabled_tools: self
                .enabled_tools
                .as_ref()
                .map(|tools| tools.iter().cloned().collect::<Vec<String>>()),
        }
    }

    fn available_tool_names(&self) -> Vec<&'static str> {
        let mut tools = vec![
            TOOL_LIST_MODULES,
            TOOL_QUERY_MODULES,
            TOOL_MODULE_EXISTS,
            TOOL_MODULE_DETAILS,
            TOOL_CONTENT_MODULE,
            TOOL_BLOG_MODULE,
            TOOL_FORUM_MODULE,
            TOOL_PAGES_MODULE,
            TOOL_MCP_HEALTH,
        ];

        if self.alloy.is_some() {
            tools.extend_from_slice(ALL_ALLOY_TOOLS);
        }

        match &self.enabled_tools {
            Some(enabled) => tools
                .into_iter()
                .filter(|name| enabled.contains(*name) || *name == TOOL_MCP_HEALTH)
                .collect(),
            None => tools,
        }
    }

    fn serialize_response<T: serde::Serialize>(value: T) -> Result<String, rmcp::ErrorData> {
        serde_json::to_string(&value).map_err(|e| {
            rmcp::ErrorData::internal_error(format!("Failed to serialize response: {}", e), None)
        })
    }
}

impl<R: ScriptRegistry + Send + Sync + 'static> ServerHandler for RusToKMcpServer<R> {
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        tracing::info!(tool = %request.name, "MCP tool call");

        if !self.tool_allowed(request.name.as_ref()) {
            let content = Self::serialize_response(McpToolResponse::<()>::error(
                "tool_disabled",
                "Tool is disabled by configuration",
            ))?;
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                content,
            )]));
        }

        match request.name.as_ref() {
            TOOL_LIST_MODULES => {
                let result = self.list_modules_internal().await;
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_QUERY_MODULES => {
                let args = request
                    .arguments
                    .ok_or_else(|| rmcp::ErrorData::invalid_params("Missing arguments", None))?;
                let req: ModuleQueryRequest =
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
                        rmcp::ErrorData::invalid_params(format!("Invalid arguments: {}", e), None)
                    })?;
                let result = self.list_modules_filtered_internal(req).await;
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_MODULE_EXISTS => {
                let args = request
                    .arguments
                    .ok_or_else(|| rmcp::ErrorData::invalid_params("Missing arguments", None))?;
                let req: ModuleLookupRequest =
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
                        rmcp::ErrorData::invalid_params(format!("Invalid arguments: {}", e), None)
                    })?;
                let result = self.module_exists_internal(&req.slug).await;
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_MODULE_DETAILS => {
                let args = request
                    .arguments
                    .ok_or_else(|| rmcp::ErrorData::invalid_params("Missing arguments", None))?;
                let req: ModuleLookupRequest =
                    serde_json::from_value(serde_json::Value::Object(args)).map_err(|e| {
                        rmcp::ErrorData::invalid_params(format!("Invalid arguments: {}", e), None)
                    })?;
                let result = self.module_details_internal(&req.slug).await;
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_CONTENT_MODULE => {
                let result = self.module_details_by_slug_internal(MODULE_CONTENT);
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_BLOG_MODULE => {
                let result = self.module_details_by_slug_internal(MODULE_BLOG);
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_FORUM_MODULE => {
                let result = self.module_details_by_slug_internal(MODULE_FORUM);
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_PAGES_MODULE => {
                let result = self.module_details_by_slug_internal(MODULE_PAGES);
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }
            TOOL_MCP_HEALTH => {
                let tool_count = self.available_tool_names().len();
                let result = self.health_response(tool_count);
                let content = Self::serialize_response(McpToolResponse::success(result))?;
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    content,
                )]))
            }

            // ── Alloy tools ──────────────────────────────────────────────────────────
            name if ALL_ALLOY_TOOLS.contains(&name) => {
                let alloy = match &self.alloy {
                    Some(a) => Arc::clone(a),
                    None => {
                        let content = Self::serialize_response(McpToolResponse::<()>::error(
                            "not_configured",
                            "Alloy scripting is not configured in this MCP server",
                        ))?;
                        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]));
                    }
                };

                match name {
                    TOOL_ALLOY_LIST_SCRIPTS => {
                        let req: ListScriptsRequest = parse_optional_args(request.arguments)?;
                        let result = alloy_list_scripts(&alloy, req).await;
                        let content = Self::serialize_response(match result {
                            Ok(v) => McpToolResponse::success(v),
                            Err(e) => McpToolResponse::error("alloy_error", e),
                        })?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_GET_SCRIPT => {
                        let args = require_args(request.arguments)?;
                        let req: GetScriptRequest = serde_json::from_value(
                            serde_json::Value::Object(args),
                        )
                        .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                        let result = alloy_get_script(&alloy, req).await;
                        let content = Self::serialize_response(match result {
                            Ok(v) => McpToolResponse::success(v),
                            Err(e) => McpToolResponse::error("alloy_error", e),
                        })?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_CREATE_SCRIPT => {
                        let args = require_args(request.arguments)?;
                        let req: CreateScriptRequest = serde_json::from_value(
                            serde_json::Value::Object(args),
                        )
                        .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                        let result = alloy_create_script(&alloy, req).await;
                        let content = Self::serialize_response(match result {
                            Ok(v) => McpToolResponse::success(v),
                            Err(e) => McpToolResponse::error("alloy_error", e),
                        })?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_UPDATE_SCRIPT => {
                        let args = require_args(request.arguments)?;
                        let req: UpdateScriptRequest = serde_json::from_value(
                            serde_json::Value::Object(args),
                        )
                        .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                        let result = alloy_update_script(&alloy, req).await;
                        let content = Self::serialize_response(match result {
                            Ok(v) => McpToolResponse::success(v),
                            Err(e) => McpToolResponse::error("alloy_error", e),
                        })?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_DELETE_SCRIPT => {
                        let args = require_args(request.arguments)?;
                        let req: DeleteScriptRequest = serde_json::from_value(
                            serde_json::Value::Object(args),
                        )
                        .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                        let result = alloy_delete_script(&alloy, req).await;
                        let content = Self::serialize_response(match result {
                            Ok(v) => McpToolResponse::success(v),
                            Err(e) => McpToolResponse::error("alloy_error", e),
                        })?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_VALIDATE_SCRIPT => {
                        let args = require_args(request.arguments)?;
                        let req: ValidateScriptRequest = serde_json::from_value(
                            serde_json::Value::Object(args),
                        )
                        .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                        let result = alloy_validate_script(&alloy, req);
                        let content = Self::serialize_response(McpToolResponse::success(result))?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_RUN_SCRIPT => {
                        let args = require_args(request.arguments)?;
                        let req: RunScriptRequest = serde_json::from_value(
                            serde_json::Value::Object(args),
                        )
                        .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None))?;
                        let result = alloy_run_script(&alloy, req).await;
                        let content = Self::serialize_response(match result {
                            Ok(v) => McpToolResponse::success(v),
                            Err(e) => McpToolResponse::error("alloy_error", e),
                        })?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_LIST_ENTITY_TYPES => {
                        let result = alloy_list_entity_types();
                        let content = Self::serialize_response(McpToolResponse::success(result))?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    TOOL_ALLOY_SCRIPT_HELPERS => {
                        let result = alloy_script_helpers();
                        let content = Self::serialize_response(McpToolResponse::success(result))?;
                        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            content,
                        )]))
                    }
                    _ => unreachable!("ALL_ALLOY_TOOLS exhausted"),
                }
            }

            _ => Err(rmcp::ErrorData::new(
                rmcp::model::ErrorCode::METHOD_NOT_FOUND,
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, rmcp::ErrorData> {
        use rmcp::model::Tool;
        use schemars::schema_for;

        let empty_schema = match serde_json::to_value(schema_for!(())) {
            Ok(serde_json::Value::Object(map)) => map,
            _ => serde_json::Map::new(),
        };

        let module_exists_schema =
            match serde_json::to_value(schema_for!(crate::tools::ModuleLookupRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let module_query_schema =
            match serde_json::to_value(schema_for!(crate::tools::ModuleQueryRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let list_scripts_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::ListScriptsRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let get_script_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::GetScriptRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let create_script_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::CreateScriptRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let update_script_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::UpdateScriptRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let delete_script_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::DeleteScriptRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let validate_script_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::ValidateScriptRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let run_script_schema =
            match serde_json::to_value(schema_for!(crate::alloy_tools::RunScriptRequest)) {
                Ok(serde_json::Value::Object(map)) => map,
                _ => serde_json::Map::new(),
            };

        let mut tools = vec![
            Tool::new(
                TOOL_LIST_MODULES,
                "List all registered RusToK modules with their metadata",
                empty_schema.clone(),
            ),
            Tool::new(
                TOOL_QUERY_MODULES,
                "List modules with filters and pagination",
                module_query_schema,
            ),
            Tool::new(
                TOOL_MODULE_EXISTS,
                "Check if a module exists by its slug",
                module_exists_schema.clone(),
            ),
            Tool::new(
                TOOL_MODULE_DETAILS,
                "Fetch module metadata by slug",
                module_exists_schema,
            ),
            Tool::new(
                TOOL_CONTENT_MODULE,
                "Fetch content module metadata",
                empty_schema.clone(),
            ),
            Tool::new(
                TOOL_BLOG_MODULE,
                "Fetch blog module metadata",
                empty_schema.clone(),
            ),
            Tool::new(
                TOOL_FORUM_MODULE,
                "Fetch forum module metadata",
                empty_schema.clone(),
            ),
            Tool::new(
                TOOL_PAGES_MODULE,
                "Fetch pages module metadata",
                empty_schema.clone(),
            ),
            Tool::new(
                TOOL_MCP_HEALTH,
                "MCP readiness and configuration status",
                empty_schema.clone(),
            ),
        ];

        if self.alloy.is_some() {
            tools.extend([
                Tool::new(
                    TOOL_ALLOY_LIST_SCRIPTS,
                    "List Alloy scripts with optional status filter",
                    list_scripts_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_GET_SCRIPT,
                    "Get a single Alloy script by name or UUID",
                    get_script_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_CREATE_SCRIPT,
                    "Create a new Alloy Rhai script",
                    create_script_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_UPDATE_SCRIPT,
                    "Update an existing Alloy script (code, description, status)",
                    update_script_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_DELETE_SCRIPT,
                    "Delete an Alloy script by UUID",
                    delete_script_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_VALIDATE_SCRIPT,
                    "Validate Rhai script syntax without executing",
                    validate_script_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_RUN_SCRIPT,
                    "Execute an Alloy script manually with optional params and entity context",
                    run_script_schema,
                ),
                Tool::new(
                    TOOL_ALLOY_LIST_ENTITY_TYPES,
                    "List all known entity types in the platform",
                    empty_schema.clone(),
                ),
                Tool::new(
                    TOOL_ALLOY_SCRIPT_HELPERS,
                    "List available Rhai helper functions with signatures and descriptions",
                    empty_schema,
                ),
            ]);
        }

        if let Some(enabled) = &self.enabled_tools {
            tools.retain(|tool| enabled.contains(&tool.name) || tool.name == TOOL_MCP_HEALTH);
        }

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::V_2024_11_05,
            capabilities: rmcp::model::ServerCapabilities::default(),
            server_info: Implementation {
                name: "RusToK MCP Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("RusToK MCP Server".to_string()),
                description: Some(
                    "MCP server for exploring RusToK modules and managing Alloy scripts. Use list_modules to see available modules, alloy_list_scripts to manage scripts.".to_string(),
                ),
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "MCP server for RusToK. Use list_modules/module_exists for module discovery, alloy_* tools for script management.".to_string(),
            ),
        }
    }
}

/// Serve the MCP server over stdio
pub async fn serve_stdio(config: McpServerConfig) -> Result<()> {
    let server = match config.enabled_tools {
        Some(enabled_tools) => RusToKMcpServer::with_enabled_tools(config.registry, enabled_tools),
        None => RusToKMcpServer::new(config.registry),
    };

    stdio::serve(server)
        .await
        .map_err(|e| anyhow::anyhow!("MCP server error: {}", e))
}

fn require_args(
    args: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<serde_json::Map<String, serde_json::Value>, rmcp::ErrorData> {
    args.ok_or_else(|| rmcp::ErrorData::invalid_params("Missing arguments", None))
}

fn parse_optional_args<T: serde::de::DeserializeOwned + Default>(
    args: Option<serde_json::Map<String, serde_json::Value>>,
) -> Result<T, rmcp::ErrorData> {
    match args {
        Some(map) => serde_json::from_value(serde_json::Value::Object(map))
            .map_err(|e| rmcp::ErrorData::invalid_params(e.to_string(), None)),
        None => Ok(T::default()),
    }
}
