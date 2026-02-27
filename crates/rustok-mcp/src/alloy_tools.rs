use std::collections::HashMap;
use std::sync::Arc;

use alloy_scripting::model::{Script, ScriptStatus, ScriptTrigger};
use alloy_scripting::runner::ExecutionOutcome;
use alloy_scripting::storage::{ScriptQuery, ScriptRegistry};
use alloy_scripting::utils::{dynamic_to_json, json_to_dynamic};
use alloy_scripting::{EntityProxy, ScriptEngine, ScriptOrchestrator};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const TOOL_ALLOY_LIST_SCRIPTS: &str = "alloy_list_scripts";
pub const TOOL_ALLOY_GET_SCRIPT: &str = "alloy_get_script";
pub const TOOL_ALLOY_CREATE_SCRIPT: &str = "alloy_create_script";
pub const TOOL_ALLOY_UPDATE_SCRIPT: &str = "alloy_update_script";
pub const TOOL_ALLOY_DELETE_SCRIPT: &str = "alloy_delete_script";
pub const TOOL_ALLOY_VALIDATE_SCRIPT: &str = "alloy_validate_script";
pub const TOOL_ALLOY_RUN_SCRIPT: &str = "alloy_run_script";
pub const TOOL_ALLOY_LIST_ENTITY_TYPES: &str = "alloy_list_entity_types";
pub const TOOL_ALLOY_SCRIPT_HELPERS: &str = "alloy_script_helpers";

pub const ALL_ALLOY_TOOLS: &[&str] = &[
    TOOL_ALLOY_LIST_SCRIPTS,
    TOOL_ALLOY_GET_SCRIPT,
    TOOL_ALLOY_CREATE_SCRIPT,
    TOOL_ALLOY_UPDATE_SCRIPT,
    TOOL_ALLOY_DELETE_SCRIPT,
    TOOL_ALLOY_VALIDATE_SCRIPT,
    TOOL_ALLOY_RUN_SCRIPT,
    TOOL_ALLOY_LIST_ENTITY_TYPES,
    TOOL_ALLOY_SCRIPT_HELPERS,
];

pub struct AlloyMcpState<R: ScriptRegistry + 'static> {
    pub registry: Arc<R>,
    pub engine: Arc<ScriptEngine>,
    pub orchestrator: Arc<ScriptOrchestrator<R>>,
}

impl<R: ScriptRegistry + 'static> Clone for AlloyMcpState<R> {
    fn clone(&self) -> Self {
        Self {
            registry: Arc::clone(&self.registry),
            engine: Arc::clone(&self.engine),
            orchestrator: Arc::clone(&self.orchestrator),
        }
    }
}

impl<R: ScriptRegistry + 'static> AlloyMcpState<R> {
    pub fn new(
        registry: Arc<R>,
        engine: Arc<ScriptEngine>,
        orchestrator: Arc<ScriptOrchestrator<R>>,
    ) -> Self {
        Self {
            registry,
            engine,
            orchestrator,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlloyScriptInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub trigger_type: String,
    pub status: String,
    pub version: u32,
    pub error_count: u32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Script> for AlloyScriptInfo {
    fn from(s: Script) -> Self {
        let trigger_type = match &s.trigger {
            ScriptTrigger::Event { entity_type, event } => {
                format!("event:{}:{}", entity_type, event.as_str())
            }
            ScriptTrigger::Cron { expression } => format!("cron:{}", expression),
            ScriptTrigger::Manual => "manual".to_string(),
            ScriptTrigger::Api { path, method } => format!("api:{}:{}", method.as_str(), path),
        };
        Self {
            id: s.id.to_string(),
            name: s.name,
            description: s.description,
            code: s.code,
            trigger_type,
            status: s.status.as_str().to_string(),
            version: s.version,
            error_count: s.error_count,
            created_at: s.created_at.to_rfc3339(),
            updated_at: s.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct ListScriptsRequest {
    /// Filter by status: draft, active, paused, disabled, archived. Omit to list all.
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListScriptsResponse {
    pub scripts: Vec<AlloyScriptInfo>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetScriptRequest {
    /// Script name (preferred)
    pub name: Option<String>,
    /// Script UUID
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateScriptRequest {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    /// JSON-encoded trigger object. Examples:
    /// `{"type":"manual"}`,
    /// `{"type":"cron","expression":"0 * * * * *"}`,
    /// `{"type":"event","entity_type":"order","event":"before_create"}`,
    /// `{"type":"api","path":"/my-endpoint","method":"POST"}`
    pub trigger: serde_json::Value,
    /// Initial status: "draft" (default) or "active"
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateScriptRequest {
    pub id: String,
    pub code: Option<String>,
    pub description: Option<String>,
    /// New status: draft, active, paused, disabled, archived
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteScriptRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateScriptRequest {
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateScriptResponse {
    pub valid: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunScriptRequest {
    pub name: String,
    /// Optional JSON object with script parameters
    pub params: Option<serde_json::Value>,
    /// Optional entity context for scripts working with entities
    pub entity: Option<RunScriptEntityInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunScriptEntityInput {
    pub id: String,
    pub entity_type: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunScriptResponse {
    pub execution_id: String,
    pub success: bool,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub return_value: Option<serde_json::Value>,
    pub changes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScriptHelpersResponse {
    pub helpers: Vec<ScriptHelperInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScriptHelperInfo {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub available_in: Vec<String>,
}

pub async fn alloy_list_scripts<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: ListScriptsRequest,
) -> Result<ListScriptsResponse, String> {
    let query = match request.status.as_deref() {
        Some(s) => match ScriptStatus::parse(s) {
            Some(status) => ScriptQuery::ByStatus(status),
            None => return Err(format!("Invalid status: {s}")),
        },
        None => ScriptQuery::All,
    };

    let scripts = state
        .registry
        .find(query)
        .await
        .map_err(|e| e.to_string())?;
    let total = scripts.len();
    Ok(ListScriptsResponse {
        scripts: scripts.into_iter().map(AlloyScriptInfo::from).collect(),
        total,
    })
}

pub async fn alloy_get_script<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: GetScriptRequest,
) -> Result<AlloyScriptInfo, String> {
    if let Some(name) = request.name {
        let script = state
            .registry
            .get_by_name(&name)
            .await
            .map_err(|e| e.to_string())?;
        return Ok(script.into());
    }
    if let Some(id_str) = request.id {
        let id = id_str.parse::<uuid::Uuid>().map_err(|e| e.to_string())?;
        let script = state.registry.get(id).await.map_err(|e| e.to_string())?;
        return Ok(script.into());
    }
    Err("Either name or id must be provided".to_string())
}

pub async fn alloy_create_script<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: CreateScriptRequest,
) -> Result<AlloyScriptInfo, String> {
    let trigger: ScriptTrigger =
        serde_json::from_value(request.trigger).map_err(|e| e.to_string())?;

    if let ScriptTrigger::Cron { ref expression } = trigger {
        alloy_scripting::utils::validate_cron_expression(expression).map_err(|e| e)?;
    }

    let mut scope = rhai::Scope::new();
    state
        .engine
        .compile(&request.name, &request.code, &mut scope)
        .map_err(|e| e.to_string())?;

    let mut script = Script::new(request.name, request.code, trigger);
    script.description = request.description;

    if let Some(status_str) = request.status {
        script.status = ScriptStatus::parse(&status_str)
            .ok_or_else(|| format!("Invalid status: {status_str}"))?;
    }

    let saved = state
        .registry
        .save(script)
        .await
        .map_err(|e| e.to_string())?;
    Ok(saved.into())
}

pub async fn alloy_update_script<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: UpdateScriptRequest,
) -> Result<AlloyScriptInfo, String> {
    let id = request
        .id
        .parse::<uuid::Uuid>()
        .map_err(|e| e.to_string())?;
    let mut script = state.registry.get(id).await.map_err(|e| e.to_string())?;

    if let Some(code) = request.code {
        let mut scope = rhai::Scope::new();
        state
            .engine
            .compile(&script.name, &code, &mut scope)
            .map_err(|e| e.to_string())?;
        state.engine.invalidate(&script.name);
        script.code = code;
    }

    if let Some(desc) = request.description {
        script.description = Some(desc);
    }

    if let Some(status_str) = request.status {
        script.status = ScriptStatus::parse(&status_str)
            .ok_or_else(|| format!("Invalid status: {status_str}"))?;
    }

    let saved = state
        .registry
        .save(script)
        .await
        .map_err(|e| e.to_string())?;
    Ok(saved.into())
}

pub async fn alloy_delete_script<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: DeleteScriptRequest,
) -> Result<bool, String> {
    let id = request
        .id
        .parse::<uuid::Uuid>()
        .map_err(|e| e.to_string())?;
    let script = state.registry.get(id).await.map_err(|e| e.to_string())?;
    state.engine.invalidate(&script.name);
    state.registry.delete(id).await.map_err(|e| e.to_string())?;
    Ok(true)
}

pub fn alloy_validate_script<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: ValidateScriptRequest,
) -> ValidateScriptResponse {
    let mut scope = rhai::Scope::new();
    match state
        .engine
        .compile("__mcp_validate__", &request.code, &mut scope)
    {
        Ok(_) => ValidateScriptResponse {
            valid: true,
            message: "Script compiles successfully".to_string(),
        },
        Err(e) => ValidateScriptResponse {
            valid: false,
            message: e.to_string(),
        },
    }
}

pub async fn alloy_run_script<R: ScriptRegistry>(
    state: &AlloyMcpState<R>,
    request: RunScriptRequest,
) -> Result<RunScriptResponse, String> {
    let params = match request.params {
        Some(serde_json::Value::Object(map)) => map
            .into_iter()
            .map(|(k, v)| (k, json_to_dynamic(v)))
            .collect(),
        _ => HashMap::new(),
    };

    let entity = request.entity.map(|e| {
        let data = match e.data {
            serde_json::Value::Object(map) => map
                .into_iter()
                .map(|(k, v)| (k, json_to_dynamic(v)))
                .collect(),
            _ => HashMap::new(),
        };
        EntityProxy::new(e.id, e.entity_type, data)
    });

    let result = state
        .orchestrator
        .run_manual_with_entity(&request.name, params, entity, None)
        .await
        .map_err(|e| e.to_string())?;

    let (success, error, return_value, changes) = match result.outcome {
        ExecutionOutcome::Success {
            return_value,
            entity_changes,
        } => {
            let rv = return_value.map(dynamic_to_json);
            let ch = serde_json::Value::Object(
                entity_changes
                    .into_iter()
                    .map(|(k, v)| (k, dynamic_to_json(v)))
                    .collect(),
            );
            (true, None, rv, Some(ch))
        }
        ExecutionOutcome::Aborted { reason } => (false, Some(reason), None, None),
        ExecutionOutcome::Failed { error } => (false, Some(error.to_string()), None, None),
    };

    Ok(RunScriptResponse {
        execution_id: result.execution_id.to_string(),
        success,
        duration_ms: result.duration_ms(),
        error,
        return_value,
        changes,
    })
}

pub fn alloy_list_entity_types() -> Vec<String> {
    vec![
        "order".to_string(),
        "product".to_string(),
        "content".to_string(),
        "page".to_string(),
        "blog_post".to_string(),
        "forum_topic".to_string(),
        "forum_post".to_string(),
        "user".to_string(),
        "tenant".to_string(),
    ]
}

pub fn alloy_script_helpers() -> ScriptHelpersResponse {
    ScriptHelpersResponse {
        helpers: vec![
            ScriptHelperInfo {
                name: "log".to_string(),
                signature: "log(message: string)".to_string(),
                description: "Log an info message".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "log_warn".to_string(),
                signature: "log_warn(message: string)".to_string(),
                description: "Log a warning message".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "log_error".to_string(),
                signature: "log_error(message: string)".to_string(),
                description: "Log an error message".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "abort".to_string(),
                signature: "abort(reason: string)".to_string(),
                description: "Abort execution and reject the operation with a reason".to_string(),
                available_in: vec!["before".to_string()],
            },
            ScriptHelperInfo {
                name: "now".to_string(),
                signature: "now() -> string".to_string(),
                description: "Return current UTC timestamp as RFC3339 string".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "now_unix".to_string(),
                signature: "now_unix() -> i64".to_string(),
                description: "Return current Unix timestamp in seconds".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "validate_email".to_string(),
                signature: "validate_email(email: string) -> bool".to_string(),
                description: "Validate an email address format".to_string(),
                available_in: vec![
                    "before".to_string(),
                    "manual".to_string(),
                    "scheduled".to_string(),
                ],
            },
            ScriptHelperInfo {
                name: "validate_required".to_string(),
                signature: "validate_required(value: string) -> bool".to_string(),
                description: "Check that a string is non-empty after trimming".to_string(),
                available_in: vec![
                    "before".to_string(),
                    "manual".to_string(),
                    "scheduled".to_string(),
                ],
            },
            ScriptHelperInfo {
                name: "validate_min_length".to_string(),
                signature: "validate_min_length(value: string, min: i64) -> bool".to_string(),
                description: "Check that a string has at least min characters".to_string(),
                available_in: vec![
                    "before".to_string(),
                    "manual".to_string(),
                    "scheduled".to_string(),
                ],
            },
            ScriptHelperInfo {
                name: "validate_max_length".to_string(),
                signature: "validate_max_length(value: string, max: i64) -> bool".to_string(),
                description: "Check that a string has at most max characters".to_string(),
                available_in: vec![
                    "before".to_string(),
                    "manual".to_string(),
                    "scheduled".to_string(),
                ],
            },
            ScriptHelperInfo {
                name: "validate_range".to_string(),
                signature: "validate_range(value: i64, min: i64, max: i64) -> bool".to_string(),
                description: "Check that a number is within [min, max]".to_string(),
                available_in: vec![
                    "before".to_string(),
                    "manual".to_string(),
                    "scheduled".to_string(),
                ],
            },
            ScriptHelperInfo {
                name: "format_money".to_string(),
                signature: "format_money(amount: i64) -> string".to_string(),
                description: "Format an integer amount with thousands separators".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "is_empty".to_string(),
                signature: "is_empty(value: Dynamic) -> bool".to_string(),
                description: "Check if a value is null, empty string, or empty array".to_string(),
                available_in: vec!["all".to_string()],
            },
            ScriptHelperInfo {
                name: "coalesce".to_string(),
                signature: "coalesce(value: Dynamic, default: Dynamic) -> Dynamic".to_string(),
                description: "Return value if non-null, otherwise return default".to_string(),
                available_in: vec!["all".to_string()],
            },
        ],
    }
}
