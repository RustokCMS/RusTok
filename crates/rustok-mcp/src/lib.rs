//! RusToK MCP Server
//!
//! This crate provides a Model Context Protocol (MCP) server for exploring
//! and interacting with RusToK modules, including Alloy scripting management.

pub mod alloy_tools;
pub mod server;
pub mod tools;

pub use alloy_tools::{
    AlloyMcpState, AlloyScriptInfo, ALL_ALLOY_TOOLS, TOOL_ALLOY_CREATE_SCRIPT,
    TOOL_ALLOY_DELETE_SCRIPT, TOOL_ALLOY_GET_SCRIPT, TOOL_ALLOY_LIST_ENTITY_TYPES,
    TOOL_ALLOY_LIST_SCRIPTS, TOOL_ALLOY_RUN_SCRIPT, TOOL_ALLOY_SCRIPT_HELPERS,
    TOOL_ALLOY_UPDATE_SCRIPT, TOOL_ALLOY_VALIDATE_SCRIPT,
};
pub use server::{serve_stdio, McpServerConfig, RusToKMcpServer};
pub use tools::{
    McpHealthResponse, McpState, McpToolError, McpToolResponse, ModuleDetailsResponse, ModuleInfo,
    ModuleListResponse, ModuleLookupRequest, ModuleLookupResponse, ModuleQueryRequest, MODULE_BLOG,
    MODULE_CONTENT, MODULE_FORUM, MODULE_PAGES, TOOL_BLOG_MODULE, TOOL_CONTENT_MODULE,
    TOOL_FORUM_MODULE, TOOL_LIST_MODULES, TOOL_MCP_HEALTH, TOOL_MODULE_DETAILS, TOOL_MODULE_EXISTS,
    TOOL_PAGES_MODULE, TOOL_QUERY_MODULES,
};
