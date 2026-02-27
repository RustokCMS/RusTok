use axum::{
    extract::{Path, Query, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_blog::{CreatePostInput, PostListQuery, PostResponse, PostService, UpdatePostInput};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::rbac::{
    RequireBlogPostsCreate, RequireBlogPostsDelete, RequireBlogPostsList, RequireBlogPostsPublish,
    RequireBlogPostsRead, RequireBlogPostsUpdate,
};
use crate::services::event_bus::transactional_event_bus_from_context;

/// List blog posts
#[utoipa::path(
    get,
    path = "/api/blog/posts",
    tag = "blog",
    params(PostListQuery),
    responses(
        (status = 200, description = "List of posts", body = rustok_blog::PostListResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_posts(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireBlogPostsList(user): RequireBlogPostsList,
    Query(query): Query<PostListQuery>,
) -> Result<Json<rustok_blog::PostListResponse>> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let result = service
        .list_posts(tenant.id, user.security_context(), query)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(result))
}

/// Get a single blog post by ID
#[utoipa::path(
    get,
    path = "/api/blog/posts/{id}",
    tag = "blog",
    params(
        ("id" = Uuid, Path, description = "Post ID"),
        ("locale" = Option<String>, Query, description = "Requested locale")
    ),
    responses(
        (status = 200, description = "Post details", body = PostResponse),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: RequireBlogPostsRead,
    Path(id): Path<Uuid>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PostResponse>> {
    let locale = params.get("locale").map(String::as_str).unwrap_or("en");
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let post = service
        .get_post(id, locale)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(post))
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
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_post(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireBlogPostsCreate(user): RequireBlogPostsCreate,
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
    request_body = UpdatePostInput,
    responses(
        (status = 200, description = "Post updated"),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    RequireBlogPostsUpdate(user): RequireBlogPostsUpdate,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdatePostInput>,
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
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn delete_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    RequireBlogPostsDelete(user): RequireBlogPostsDelete,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .delete_post(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}

/// Publish a blog post
#[utoipa::path(
    post,
    path = "/api/blog/posts/{id}/publish",
    tag = "blog",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post published"),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn publish_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    RequireBlogPostsPublish(user): RequireBlogPostsPublish,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .publish_post(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}

/// Unpublish a blog post
#[utoipa::path(
    post,
    path = "/api/blog/posts/{id}/unpublish",
    tag = "blog",
    params(
        ("id" = Uuid, Path, description = "Post ID")
    ),
    responses(
        (status = 200, description = "Post unpublished"),
        (status = 404, description = "Post not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn unpublish_post(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    RequireBlogPostsPublish(user): RequireBlogPostsPublish,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = PostService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .unpublish_post(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
