use axum::{
    extract::{Path, Query, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_content::dto::{NodeListItem, NodeResponse};
use rustok_content::{CreateNodeInput, ListNodesFilter, NodeService, UpdateNodeInput};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::rbac::{
    RequireNodesCreate, RequireNodesDelete, RequireNodesList, RequireNodesRead, RequireNodesUpdate,
};
use crate::services::event_bus::transactional_event_bus_from_context;

/// List content nodes
#[utoipa::path(
    get,
    path = "/api/content/nodes",
    tag = "content",
    params(
        ListNodesFilter
    ),
    responses(
        (status = 200, description = "List of nodes", body = Vec<NodeListItem>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_nodes(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireNodesList(user): RequireNodesList,
    Query(filter): Query<ListNodesFilter>,
) -> Result<Json<Vec<NodeListItem>>> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let (items, _) = service
        .list_nodes(tenant.id, user.security_context(), filter)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(items))
}

/// Get a single content node by ID
#[utoipa::path(
    get,
    path = "/api/content/nodes/{id}",
    tag = "content",
    params(
        ("id" = Uuid, Path, description = "Node ID")
    ),
    responses(
        (status = 200, description = "Node details", body = NodeResponse),
        (status = 404, description = "Node not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_node(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: RequireNodesRead,
    Path(id): Path<Uuid>,
) -> Result<Json<NodeResponse>> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let node = service
        .get_node(id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(node))
}

/// Create a new content node
#[utoipa::path(
    post,
    path = "/api/content/nodes",
    tag = "content",
    request_body = CreateNodeInput,
    responses(
        (status = 201, description = "Node created", body = NodeResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_node(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireNodesCreate(user): RequireNodesCreate,
    Json(input): Json<CreateNodeInput>,
) -> Result<Json<NodeResponse>> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let node = service
        .create_node(tenant.id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(node))
}

/// Update an existing content node
#[utoipa::path(
    put,
    path = "/api/content/nodes/{id}",
    tag = "content",
    params(
        ("id" = Uuid, Path, description = "Node ID")
    ),
    request_body = UpdateNodeInput,
    responses(
        (status = 200, description = "Node updated", body = NodeResponse),
        (status = 404, description = "Node not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_node(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    RequireNodesUpdate(user): RequireNodesUpdate,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNodeInput>,
) -> Result<Json<NodeResponse>> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let node = service
        .update_node(id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(node))
}

/// Delete a content node
#[utoipa::path(
    delete,
    path = "/api/content/nodes/{id}",
    tag = "content",
    params(
        ("id" = Uuid, Path, description = "Node ID")
    ),
    responses(
        (status = 204, description = "Node deleted"),
        (status = 404, description = "Node not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn delete_node(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    RequireNodesDelete(user): RequireNodesDelete,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .delete_node(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
