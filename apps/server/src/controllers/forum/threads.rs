use axum::{
    extract::{Path, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_core::EventBus;
use rustok_forum::{CreateThreadInput, ThreadResponse, ThreadService, UpdateThreadInput};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::auth::CurrentUser;

#[utoipa::path(
    get,
    path = "/api/forum/threads",
    tag = "forum",
    responses(
        (status = 200, description = "List of threads", body = Vec<rustok_forum::ThreadListItem>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_threads(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
) -> Result<Json<Vec<rustok_forum::ThreadListItem>>> {
    let service = ThreadService::new(ctx.db.clone(), EventBus::default());
    let threads = service
        .list_threads(tenant.id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(threads))
}

#[utoipa::path(
    get,
    path = "/api/forum/threads/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    responses(
        (status = 200, description = "Thread details", body = ThreadResponse),
        (status = 404, description = "Thread not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_thread(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ThreadResponse>> {
    let service = ThreadService::new(ctx.db.clone(), EventBus::default());
    let thread = service
        .get_thread(id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(thread))
}

#[utoipa::path(
    post,
    path = "/api/forum/threads",
    tag = "forum",
    request_body = CreateThreadInput,
    responses(
        (status = 201, description = "Thread created", body = Uuid),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_thread(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Json(input): Json<CreateThreadInput>,
) -> Result<Json<Uuid>> {
    let service = ThreadService::new(ctx.db.clone(), EventBus::default());
    let thread_id = service
        .create_thread(tenant.id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(thread_id))
}

#[utoipa::path(
    put,
    path = "/api/forum/threads/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    request_body = UpdateThreadInput,
    responses(
        (status = 200, description = "Thread updated"),
        (status = 404, description = "Thread not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_thread(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateThreadInput>,
) -> Result<()> {
    let service = ThreadService::new(ctx.db.clone(), EventBus::default());
    service
        .update_thread(id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}

#[utoipa::path(
    delete,
    path = "/api/forum/threads/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Thread ID")
    ),
    responses(
        (status = 204, description = "Thread deleted"),
        (status = 404, description = "Thread not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_thread(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = ThreadService::new(ctx.db.clone(), EventBus::default());
    service
        .delete_thread(id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
