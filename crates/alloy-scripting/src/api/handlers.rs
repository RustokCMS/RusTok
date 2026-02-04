use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::error::ScriptError;
use crate::model::{EntityProxy, Script, ScriptStatus};
use crate::runner::ScriptOrchestrator;
use crate::storage::{ScriptQuery, ScriptRegistry};

use super::dto::*;

/// Shared state для handlers
pub struct AppState<S: ScriptRegistry> {
    pub registry: Arc<S>,
    pub orchestrator: Arc<ScriptOrchestrator<S>>,
}

type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code.as_str() {
            "not_found" => StatusCode::NOT_FOUND,
            "validation" => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

impl From<ScriptError> for ApiError {
    fn from(e: ScriptError) -> Self {
        match e {
            ScriptError::NotFound { name } => ApiError {
                error: format!("Script not found: {name}"),
                code: "not_found".to_string(),
            },
            _ => ApiError {
                error: e.to_string(),
                code: "internal".to_string(),
            },
        }
    }
}

// ============ CRUD Handlers ============

/// GET /scripts
pub async fn list_scripts<S: ScriptRegistry>(
    State(state): State<Arc<AppState<S>>>,
) -> ApiResult<Json<ListScriptsResponse>> {
    let scripts = state
        .registry
        .find(ScriptQuery::ByStatus(ScriptStatus::Active))
        .await
        .map_err(ApiError::from)?;

    let total = scripts.len();
    let scripts: Vec<ScriptResponse> = scripts.into_iter().map(Into::into).collect();

    Ok(Json(ListScriptsResponse { scripts, total }))
}

/// GET /scripts/:id
pub async fn get_script<S: ScriptRegistry>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ScriptResponse>> {
    let script = state.registry.get(id).await.map_err(ApiError::from)?;
    Ok(Json(script.into()))
}

/// POST /scripts
pub async fn create_script<S: ScriptRegistry>(
    State(state): State<Arc<AppState<S>>>,
    Json(req): Json<CreateScriptRequest>,
) -> ApiResult<(StatusCode, Json<ScriptResponse>)> {
    let mut script = Script::new(req.name, req.code, req.trigger);
    script.description = req.description;
    script.permissions = req.permissions;
    script.run_as_system = req.run_as_system;

    let saved = state.registry.save(script).await.map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(saved.into())))
}

/// PUT /scripts/:id
pub async fn update_script<S: ScriptRegistry>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateScriptRequest>,
) -> ApiResult<Json<ScriptResponse>> {
    let mut script = state.registry.get(id).await.map_err(ApiError::from)?;

    if let Some(name) = req.name {
        script.name = name;
    }
    if let Some(desc) = req.description {
        script.description = Some(desc);
    }
    if let Some(code) = req.code {
        script.code = code;
    }
    if let Some(trigger) = req.trigger {
        script.trigger = trigger;
    }
    if let Some(status) = req.status {
        script.status = status;
    }
    if let Some(perms) = req.permissions {
        script.permissions = perms;
    }

    let saved = state.registry.save(script).await.map_err(ApiError::from)?;
    Ok(Json(saved.into()))
}

/// DELETE /scripts/:id
pub async fn delete_script<S: ScriptRegistry>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.registry.delete(id).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============ Execution Handlers ============

/// POST /scripts/:id/run
pub async fn run_script<S: ScriptRegistry>(
    State(state): State<Arc<AppState<S>>>,
    Path(id): Path<Uuid>,
    Json(req): Json<RunScriptRequest>,
) -> ApiResult<Json<RunScriptResponse>> {
    let script = state.registry.get(id).await.map_err(ApiError::from)?;

    let params = req
        .params
        .into_iter()
        .map(|(k, v)| (k, json_to_dynamic(v)))
        .collect();

    let entity = req.entity.map(|entity_input| {
        let data = entity_input
            .data
            .into_iter()
            .map(|(k, v)| (k, json_to_dynamic(v)))
            .collect();

        EntityProxy::new(entity_input.id, entity_input.entity_type, data)
    });

    let result = state
        .orchestrator
        .run_manual_with_entity(&script.name, params, entity, None)
        .await
        .map_err(ApiError::from)?;

    let (success, error, changes, return_value) = match result.outcome {
        crate::runner::ExecutionOutcome::Success {
            return_value,
            entity_changes,
        } => (
            true,
            None,
            Some(convert_map(entity_changes)),
            return_value
                .map(dynamic_to_json)
                .unwrap_or(serde_json::Value::Null),
        ),
        crate::runner::ExecutionOutcome::Aborted { reason } => {
            (false, Some(reason), None, serde_json::Value::Null)
        }
        crate::runner::ExecutionOutcome::Failed { error } => (
            false,
            Some(error.to_string()),
            None,
            serde_json::Value::Null,
        ),
    };

    Ok(Json(RunScriptResponse {
        execution_id: result.execution_id.to_string(),
        success,
        duration_ms: result.duration_ms(),
        error,
        changes,
        return_value,
    }))
}

// ============ Helpers ============

fn json_to_dynamic(v: serde_json::Value) -> rhai::Dynamic {
    match v {
        serde_json::Value::Null => rhai::Dynamic::UNIT,
        serde_json::Value::Bool(b) => rhai::Dynamic::from(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rhai::Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                rhai::Dynamic::from(f)
            } else {
                rhai::Dynamic::UNIT
            }
        }
        serde_json::Value::String(s) => rhai::Dynamic::from(s),
        serde_json::Value::Array(arr) => {
            let vec: Vec<rhai::Dynamic> = arr.into_iter().map(json_to_dynamic).collect();
            rhai::Dynamic::from(vec)
        }
        serde_json::Value::Object(map) => {
            let mut rhai_map = rhai::Map::new();
            for (k, v) in map {
                rhai_map.insert(k.into(), json_to_dynamic(v));
            }
            rhai::Dynamic::from(rhai_map)
        }
    }
}

fn dynamic_to_json(d: rhai::Dynamic) -> serde_json::Value {
    if d.is_unit() {
        serde_json::Value::Null
    } else if let Some(b) = d.clone().try_cast::<bool>() {
        serde_json::Value::Bool(b)
    } else if let Some(i) = d.clone().try_cast::<i64>() {
        serde_json::Value::Number(i.into())
    } else if let Some(f) = d.clone().try_cast::<f64>() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
    } else if let Some(s) = d.clone().try_cast::<String>() {
        serde_json::Value::String(s)
    } else if let Some(arr) = d.clone().try_cast::<rhai::Array>() {
        serde_json::Value::Array(arr.into_iter().map(dynamic_to_json).collect())
    } else if let Some(map) = d.clone().try_cast::<rhai::Map>() {
        let mut json_map = serde_json::Map::new();
        for (k, v) in map {
            json_map.insert(k.to_string(), dynamic_to_json(v));
        }
        serde_json::Value::Object(json_map)
    } else {
        serde_json::Value::String(d.to_string())
    }
}

fn convert_map(map: HashMap<String, rhai::Dynamic>) -> HashMap<String, serde_json::Value> {
    map.into_iter()
        .map(|(k, v)| (k, dynamic_to_json(v)))
        .collect()
}
