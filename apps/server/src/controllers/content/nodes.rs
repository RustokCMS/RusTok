use axum::{
    extract::{Path, Query, State},
    routing::{get, post, put, delete}, // Added routing
    Json,
};
use loco_rs::prelude::*;
use rustok_content::{CreateNodeInput, ListNodesFilter, NodeService, UpdateNodeInput};
use rustok_core::{EventBus, SecurityContext};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::auth::CurrentUser;

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
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_nodes(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Query(filter): Query<ListNodesFilter>,
) -> Result<Json<Vec<rustok_content::dto::NodeListItem>>> {
    let security = SecurityContext::new(user.user.role, Some(user.user.id));
    let service = NodeService::new(ctx.db.clone(), EventBus::default());
    let (items, _) = service.list_nodes(tenant.id, security, filter).await?;
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
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_node(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<rustok_content::dto::NodeResponse>> {
    let service = NodeService::new(ctx.db.clone(), EventBus::default());
    let node = service.get_node(id).await?;
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
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_node(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Json(input): Json<CreateNodeInput>,
) -> Result<Json<rustok_content::dto::NodeResponse>> {
    let security = SecurityContext::new(user.user.role, Some(user.user.id));
    let service = NodeService::new(ctx.db.clone(), EventBus::default());
    let node = service
        .create_node(tenant.id, security, input)
        .await?;
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
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_node(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNodeInput>,
) -> Result<Json<rustok_content::dto::NodeResponse>> {
    let security = SecurityContext::new(user.user.role, Some(user.user.id));
    let service = NodeService::new(ctx.db.clone(), EventBus::default());
    let node = service
        .update_node(id, security, input)
        .await?;
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
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_node(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let security = SecurityContext::new(user.user.role, Some(user.user.id));
    let service = NodeService::new(ctx.db.clone(), EventBus::default());
    service.delete_node(id, security).await?;
    Ok(())
}
