use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use loco_rs::{app::AppContext, Error, Result};
use rustok_api::{
    has_any_effective_permission, loco::transactional_event_bus_from_context, AuthContext,
    RequestContext, TenantContext,
};
use rustok_core::Permission;
use rustok_telemetry::metrics;
use serde::Deserialize;
use std::time::Instant;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{
    CreateTopicInput, ListTopicsFilter, TopicListItem, TopicResponse, TopicService,
    UpdateTopicInput,
};

#[derive(Debug, Clone, Copy, Deserialize, IntoParams, ToSchema)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PaginationParams {
    pub fn limit(&self) -> u64 {
        self.per_page.min(100)
    }
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    20
}

#[utoipa::path(
    get,
    path = "/api/forum/topics",
    tag = "forum",
    params(ListTopicsFilter),
    responses(
        (status = 200, description = "List of topics", body = Vec<TopicListItem>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn list_topics(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    request_context: RequestContext,
    Query(mut filter): Query<ListTopicsFilter>,
) -> Result<Json<Vec<TopicListItem>>> {
    ensure_forum_permission(
        &auth,
        &[Permission::FORUM_TOPICS_LIST],
        "Permission denied: forum_topics:list required",
    )?;

    filter.locale = filter.locale.or(Some(request_context.locale.clone()));
    let requested_limit = Some(filter.per_page);
    let effective_limit = filter.per_page.min(100);
    let service = TopicService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let list_started_at = Instant::now();
    let (topics, _) = service
        .list_with_locale_fallback(
            tenant.id,
            auth.security_context(),
            filter,
            Some(tenant.default_locale.as_str()),
        )
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;
    metrics::record_read_path_query(
        "http",
        "forum.list_topics",
        "service_list",
        list_started_at.elapsed().as_secs_f64(),
        topics.len() as u64,
    );

    metrics::record_read_path_budget(
        "http",
        "forum.list_topics",
        requested_limit,
        effective_limit,
        topics.len(),
    );

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
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_topic(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    request_context: RequestContext,
    Path(id): Path<Uuid>,
    Query(filter): Query<ListTopicsFilter>,
) -> Result<Json<TopicResponse>> {
    ensure_forum_permission(
        &auth,
        &[Permission::FORUM_TOPICS_READ],
        "Permission denied: forum_topics:read required",
    )?;

    let locale = filter
        .locale
        .unwrap_or_else(|| request_context.locale.clone());
    let service = TopicService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let topic = service
        .get_with_locale_fallback(tenant.id, id, &locale, Some(tenant.default_locale.as_str()))
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;
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
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_topic(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Json(input): Json<CreateTopicInput>,
) -> Result<(StatusCode, Json<TopicResponse>)> {
    ensure_forum_permission(
        &auth,
        &[Permission::FORUM_TOPICS_CREATE],
        "Permission denied: forum_topics:create required",
    )?;

    let service = TopicService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let topic = service
        .create(tenant.id, auth.security_context(), input)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;
    Ok((StatusCode::CREATED, Json(topic)))
}

#[utoipa::path(
    put,
    path = "/api/forum/topics/{id}",
    tag = "forum",
    params(("id" = Uuid, Path, description = "Topic ID")),
    request_body = UpdateTopicInput,
    responses(
        (status = 200, description = "Topic updated", body = TopicResponse),
        (status = 404, description = "Topic not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn update_topic(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTopicInput>,
) -> Result<Json<TopicResponse>> {
    ensure_forum_permission(
        &auth,
        &[Permission::FORUM_TOPICS_UPDATE],
        "Permission denied: forum_topics:update required",
    )?;

    let service = TopicService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let topic = service
        .update(tenant.id, id, auth.security_context(), input)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;
    Ok(Json(topic))
}

#[utoipa::path(
    delete,
    path = "/api/forum/topics/{id}",
    tag = "forum",
    params(("id" = Uuid, Path, description = "Topic ID")),
    responses(
        (status = 204, description = "Topic deleted"),
        (status = 404, description = "Topic not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn delete_topic(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    ensure_forum_permission(
        &auth,
        &[Permission::FORUM_TOPICS_DELETE],
        "Permission denied: forum_topics:delete required",
    )?;

    let service = TopicService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .delete(tenant.id, id, auth.security_context())
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

fn ensure_forum_permission(
    auth: &AuthContext,
    permissions: &[Permission],
    message: &str,
) -> Result<()> {
    if !has_any_effective_permission(&auth.permissions, permissions) {
        return Err(Error::Unauthorized(message.to_string()));
    }

    Ok(())
}
