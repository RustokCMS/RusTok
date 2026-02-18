//! RusToK MCP Server
//!
//! This crate provides a Model Context Protocol (MCP) server for exploring
//! and interacting with RusToK modules.

pub mod server;
pub mod tools;

pub use server::{serve_stdio, McpServerConfig, RusToKMcpServer};
pub use tools::{
    McpState, ModuleDetailsResponse, ModuleInfo, ModuleListResponse, ModuleLookupRequest,
    ModuleLookupResponse,
};
