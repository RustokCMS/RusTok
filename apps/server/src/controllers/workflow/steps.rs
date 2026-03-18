use axum::{
    extract::{Path, State},
    Json,
};
use loco_rs::app::AppContext;
use rustok_workflow::{
    CreateWorkflowStepInput, UpdateWorkflowStepInput, WorkflowService,
};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::error::{Error, Result};
use crate::extractors::rbac::RequireWorkflowsUpdate;

/// Add a step to a workflow
pub async fn add_step(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _auth: RequireWorkflowsUpdate,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateWorkflowStepInput>,
) -> Result<Json<serde_json::Value>> {
    let service = WorkflowService::new(ctx.db.clone());
    let step_id = service
        .add_step(tenant.id, id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(serde_json::json!({ "id": step_id })))
}

/// Update a workflow step
pub async fn update_step(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _auth: RequireWorkflowsUpdate,
    Path((id, step_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateWorkflowStepInput>,
) -> Result<Json<serde_json::Value>> {
    let service = WorkflowService::new(ctx.db.clone());
    service
        .update_step(tenant.id, id, step_id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Delete a workflow step
pub async fn delete_step(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _auth: RequireWorkflowsUpdate,
    Path((id, step_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>> {
    let service = WorkflowService::new(ctx.db.clone());
    service
        .delete_step(tenant.id, id, step_id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(serde_json::json!({ "ok": true })))
}
