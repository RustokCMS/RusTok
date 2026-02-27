use axum::{
    extract::{Query, State},
    Json,
};
use loco_rs::prelude::*;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use rustok_pages::{CreatePageInput, PageResponse, PageService};

use crate::context::TenantContext;
use crate::extractors::rbac::{RequirePagesCreate, RequirePagesRead};
use crate::services::event_bus::transactional_event_bus_from_context;

#[derive(Debug, Deserialize, IntoParams, ToSchema)]
pub struct GetPageParams {
    pub slug: Option<String>,
    pub locale: Option<String>,
}

/// Get a page by slug
#[utoipa::path(
    get,
    path = "/api/pages",
    tag = "pages",
    params(GetPageParams),
    responses(
        (status = 200, description = "Page content", body = PageResponse),
        (status = 404, description = "Page not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_page(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequirePagesRead(user): RequirePagesRead,
    Query(params): Query<GetPageParams>,
) -> Result<Json<PageResponse>> {
    let slug = params.slug.unwrap_or_else(|| "home".to_string());
    let locale = params.locale.unwrap_or_else(|| "en".to_string());

    let service = PageService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let page = service
        .get_by_slug(tenant.id, user.security_context(), &locale, &slug)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;

    match page {
        Some(page) => Ok(Json(page)),
        None => Err(Error::NotFound),
    }
}

/// Create a new page
#[utoipa::path(
    post,
    path = "/api/admin/pages",
    tag = "pages",
    request_body = CreatePageInput,
    responses(
        (status = 201, description = "Page created", body = PageResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn create_page(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequirePagesCreate(user): RequirePagesCreate,
    Json(input): Json<CreatePageInput>,
) -> Result<Json<PageResponse>> {
    let service = PageService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let page = service
        .create(tenant.id, user.security_context(), input)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;
    Ok(Json(page))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api")
        .add("/pages", axum::routing::get(get_page))
        .add("/admin/pages", axum::routing::post(create_page))
}
