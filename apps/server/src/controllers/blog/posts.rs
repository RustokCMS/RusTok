use axum::{
    extract::{Path, Query, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_blog::{CreatePostInput, PostService};
use rustok_content::dto::{NodeListItem, NodeResponse};
use rustok_content::{ListNodesFilter, NodeService, UpdateNodeInput};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::auth::CurrentUser;
use crate::services::event_bus::transactional_event_bus_from_context;

/// List blog posts
#[utoipa::path(
    get,
    path = "/api/blog/posts",
    tag = "blog",
    responses(
        (status = 200, description = "List of posts", body = Vec<NodeListItem>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_posts(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Query(mut filter): Query<ListNodesFilter>,
) -> Result<Json<Vec<NodeListItem>>> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    // Force kind="post"
    filter.kind = Some("post".to_string());
    let (items, _) = service
        .list_nodes(tenant.id, user.security_context(), filter)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(items))
}

/// Get a single blog post by ID
#[utoipa::path(
    get,
    path = "/api/blog/posts/{id}",
    tag = "blog",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post details", body = NodeResponse),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<NodeResponse>> {
    let service = NodeService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let node = service
        .get_node(id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    // Optional: check if node.kind == "post"
    if node.kind != "post" {
        return Err(loco_rs::Error::NotFound);
    }
    Ok(Json(node))
}

/// Create a new blog post
#[utoipa::path(
    post,
    path = "/api/blog/posts",
    tag = "blog",
    request_body = CreatePostInput,
    responses(
        (status = 201, description = "Post created", body = Uuid),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_post(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Json(input): Json<CreatePostInput>,
) -> Result<Json<Uuid>> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let post_id = service
        .create_post(tenant.id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(post_id))
}

/// Update an existing blog post
#[utoipa::path(
    put,
    path = "/api/blog/posts/{id}",
    tag = "blog",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    request_body = UpdateNodeInput,
    responses(
        (status = 200, description = "Post updated"),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateNodeInput>,
) -> Result<()> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .update_post(id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}

/// Delete a blog post
#[utoipa::path(
    delete,
    path = "/api/blog/posts/{id}",
    tag = "blog",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 204, description = "Post deleted"),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .delete_post(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
