use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::model::{Script, ScriptId, ScriptStatus, ScriptTrigger};

// ============ Requests ============

#[derive(Debug, Deserialize)]
pub struct CreateScriptRequest {
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub trigger: ScriptTrigger,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub run_as_system: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScriptRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub code: Option<String>,
    pub trigger: Option<ScriptTrigger>,
    pub status: Option<ScriptStatus>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RunScriptRequest {
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    pub entity: Option<EntityInput>,
}

#[derive(Debug, Deserialize)]
pub struct EntityInput {
    pub id: String,
    pub entity_type: String,
    pub data: HashMap<String, serde_json::Value>,
}

// ============ Responses ============

#[derive(Debug, Serialize)]
pub struct ScriptResponse {
    pub id: ScriptId,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub trigger: ScriptTrigger,
    pub status: ScriptStatus,
    pub version: u32,
    pub error_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Script> for ScriptResponse {
    fn from(s: Script) -> Self {
        Self {
            id: s.id,
            name: s.name,
            description: s.description,
            code: s.code,
            trigger: s.trigger,
            status: s.status,
            version: s.version,
            error_count: s.error_count,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RunScriptResponse {
    pub execution_id: String,
    pub success: bool,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub changes: Option<HashMap<String, serde_json::Value>>,
    pub return_value: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ListScriptsResponse {
    pub scripts: Vec<ScriptResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ApiError {
    pub error: String,
    pub code: String,
}
