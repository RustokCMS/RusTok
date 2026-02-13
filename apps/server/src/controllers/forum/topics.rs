use axum::{
    extract::{Path, Query, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_forum::{
    CreateTopicInput, ListTopicsFilter, TopicListItem, TopicResponse, TopicService,
    UpdateTopicInput,
};
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::auth::CurrentUser;
use crate::services::event_bus::event_bus_from_context;

#[utoipa::path(
    get,
    path = "/api/forum/topics",
    tag = "forum",
    params(ListTopicsFilter),
    responses(
        (status = 200, description = "List of topics", body = Vec<TopicListItem>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_topics(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Query(filter): Query<ListTopicsFilter>,
) -> Result<Json<Vec<TopicListItem>>> {
    let service = TopicService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let (topics, _) = service
        .list(tenant.id, user.security_context(), filter)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(topics))
}

#[utoipa::path(
    get,
    path = "/api/forum/topics/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Topic ID"),
        ("locale" = Option<String>, Query, description = "Locale")
    ),
    responses(
        (status = 200, description = "Topic details", body = TopicResponse),
        (status = 404, description = "Topic not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_topic(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Query(filter): Query<ListTopicsFilter>,
) -> Result<Json<TopicResponse>> {
    let locale = filter.locale.unwrap_or_else(|| "en".to_string());
    let service = TopicService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let topic = service
        .get(id, &locale)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(topic))
}

#[utoipa::path(
    post,
    path = "/api/forum/topics",
    tag = "forum",
    request_body = CreateTopicInput,
    responses(
        (status = 201, description = "Topic created", body = TopicResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_topic(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Json(input): Json<CreateTopicInput>,
) -> Result<Json<TopicResponse>> {
    let service = TopicService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let topic = service
        .create(tenant.id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(topic))
}

#[utoipa::path(
    put,
    path = "/api/forum/topics/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Topic ID")
    ),
    request_body = UpdateTopicInput,
    responses(
        (status = 200, description = "Topic updated", body = TopicResponse),
        (status = 404, description = "Topic not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_topic(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTopicInput>,
) -> Result<Json<TopicResponse>> {
    let service = TopicService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let topic = service
        .update(id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(topic))
}

#[utoipa::path(
    delete,
    path = "/api/forum/topics/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Topic ID")
    ),
    responses(
        (status = 204, description = "Topic deleted"),
        (status = 404, description = "Topic not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_topic(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = TopicService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    service
        .delete(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
