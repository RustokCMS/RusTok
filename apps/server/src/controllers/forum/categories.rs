use axum::{
    extract::{Path, Query, State},
    Json,
};
use loco_rs::prelude::*;
use rustok_forum::{
    CategoryListItem, CategoryResponse, CategoryService, CreateCategoryInput, UpdateCategoryInput,
};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::context::TenantContext;
use crate::extractors::auth::CurrentUser;
use crate::services::event_bus::event_bus_from_context;

#[derive(Debug, Deserialize, IntoParams)]
pub struct CategoryListParams {
    pub locale: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/forum/categories",
    tag = "forum",
    params(CategoryListParams),
    responses(
        (status = 200, description = "List of categories", body = Vec<CategoryListItem>),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn list_categories(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Query(params): Query<CategoryListParams>,
) -> Result<Json<Vec<CategoryListItem>>> {
    let locale = params.locale.unwrap_or_else(|| "en".to_string());
    let service = CategoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let categories = service
        .list(tenant.id, user.security_context(), &locale)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(categories))
}

#[utoipa::path(
    get,
    path = "/api/forum/categories/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Category ID"),
        ("locale" = Option<String>, Query, description = "Locale")
    ),
    responses(
        (status = 200, description = "Category details", body = CategoryResponse),
        (status = 404, description = "Category not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn get_category(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Query(params): Query<CategoryListParams>,
) -> Result<Json<CategoryResponse>> {
    let locale = params.locale.unwrap_or_else(|| "en".to_string());
    let service = CategoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let category = service
        .get(tenant.id, id, &locale)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(category))
}

#[utoipa::path(
    post,
    path = "/api/forum/categories",
    tag = "forum",
    request_body = CreateCategoryInput,
    responses(
        (status = 201, description = "Category created", body = CategoryResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn create_category(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Json(input): Json<CreateCategoryInput>,
) -> Result<Json<CategoryResponse>> {
    let service = CategoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let category = service
        .create(tenant.id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(category))
}

#[utoipa::path(
    put,
    path = "/api/forum/categories/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Category ID")
    ),
    request_body = UpdateCategoryInput,
    responses(
        (status = 200, description = "Category updated", body = CategoryResponse),
        (status = 404, description = "Category not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn update_category(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCategoryInput>,
) -> Result<Json<CategoryResponse>> {
    let service = CategoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    let category = service
        .update(tenant.id, id, user.security_context(), input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(Json(category))
}

#[utoipa::path(
    delete,
    path = "/api/forum/categories/{id}",
    tag = "forum",
    params(
        ("id" = Uuid, Path, description = "Category ID")
    ),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 404, description = "Category not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub async fn delete_category(
    State(ctx): State<AppContext>,
    _tenant: TenantContext,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<()> {
    let service = CategoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    service
        .delete(id, user.security_context())
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;
    Ok(())
}
