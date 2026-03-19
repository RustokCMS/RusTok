use axum::{
    extract::{Path, State},
    Json,
};
use loco_rs::app::AppContext;
use rustok_workflow::{WorkflowExecutionResponse, WorkflowService};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::error::{Error, Result};
use crate::extractors::rbac::{RequireWorkflowExecutionsList, RequireWorkflowExecutionsRead};

/// List workflow executions for a workflow.
pub async fn list_executions(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _auth: RequireWorkflowExecutionsList,
    Path(workflow_id): Path<Uuid>,
) -> Result<Json<Vec<WorkflowExecutionResponse>>> {
    let service = WorkflowService::new(ctx.db.clone());
    let executions = service
        .list_executions(tenant.id, workflow_id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(executions))
}

/// Get a single workflow execution by id.
pub async fn get_execution(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _auth: RequireWorkflowExecutionsRead,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<WorkflowExecutionResponse>> {
    let service = WorkflowService::new(ctx.db.clone());
    let execution = service
        .get_execution(tenant.id, execution_id)
        .await
        .map_err(|err| match err {
            rustok_workflow::WorkflowError::ExecutionNotFound(_) => Error::NotFound,
            other => Error::BadRequest(other.to_string()),
        })?;
    Ok(Json(execution))
}
