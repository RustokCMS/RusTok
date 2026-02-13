use axum::{
    extract::{Path, Query, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_forum::{
    CreateReplyInput, ReplyListItem, ReplyResponse, ReplyService, UpdateReplyInput,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::auth::CurrentUser;
use crate::services::event_bus::event_bus_from_context;

#[derive(Deserialize)]
pub struct ReplyListParams {
    pub locale: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/forum/topics/{id}/replies",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Topic ID"),
        ("locale" = Option<String>, Query, description = "Locale")
    ),
    responses(
        (status = 200, description = "List of replies", body = Vec<ReplyListItem>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_replies(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Path(topic_id): Path<Uuid>,
    Query(params): Query<ReplyListParams>,
) -> Result<Json<Vec<ReplyListItem>>> {
    let locale = params.locale.unwrap_or_else(|| "en".to_string());
    let service = ReplyService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let replies = service
        .list_for_topic(tenant.id, user.security_context(), topic_id, &locale)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(replies))
}

#[utoipa::path(
    get,
    path = "/api/forum/replies/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Reply ID"),
        ("locale" = Option<String>, Query, description = "Locale")
    ),
    responses(
        (status = 200, description = "Reply details", body = ReplyResponse),
        (status = 404, description = "Reply not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_reply(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Query(params): Query<ReplyListParams>,
) -> Result<Json<ReplyResponse>> {
    let locale = params.locale.unwrap_or_else(|| "en".to_string());
    let service = ReplyService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let reply = service
        .get(id, &locale)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(reply))
}

#[utoipa::path(
    post,
    path = "/api/forum/topics/{id}/replies",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Topic ID")
    ),
    request_body = CreateReplyInput,
    responses(
        (status = 201, description = "Reply created", body = ReplyResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_reply(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Path(topic_id): Path<Uuid>,
    Json(input): Json<CreateReplyInput>,
) -> Result<Json<ReplyResponse>> {
    let service = ReplyService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let reply = service
        .create(tenant.id, user.security_context(), topic_id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(reply))
}

#[utoipa::path(
    put,
    path = "/api/forum/replies/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Reply ID")
    ),
    request_body = UpdateReplyInput,
    responses(
        (status = 200, description = "Reply updated", body = ReplyResponse),
        (status = 404, description = "Reply not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_reply(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateReplyInput>,
) -> Result<Json<ReplyResponse>> {
    let service = ReplyService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let reply = service
        .update(id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(reply))
}

#[utoipa::path(
    delete,
    path = "/api/forum/replies/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Reply ID")
    ),
    responses(
        (status = 204, description = "Reply deleted"),
        (status = 404, description = "Reply not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_reply(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = ReplyService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    service
        .delete(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
