use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use rustok_commerce::dto::{CreateProductInput, ProductResponse, UpdateProductInput};
use rustok_commerce::CatalogService;
use rustok_core::EventBus;

use crate::common::{ApiResponse, PaginatedResponse, PaginationMeta, PaginationParams, RequestContext};
use crate::models::AppContext;

pub(super) async fn list_products(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Query(params): Query<ListProductsParams>,
) -> Result<Json<PaginatedResponse<ProductListItem>>, (StatusCode, Json<ApiResponse<()>>)> {
    use rustok_commerce::entities::{product, product_translation};

    let pagination = params.pagination.unwrap_or_default();

    let mut query = product::Entity::find().filter(product::Column::TenantId.eq(request.tenant_id));

    if let Some(status) = &params.status {
        query = query.filter(product::Column::Status.eq(status));
    }
    if let Some(vendor) = &params.vendor {
        query = query.filter(product::Column::Vendor.eq(vendor));
    }
    if let Some(product_type) = &params.product_type {
        query = query.filter(product::Column::ProductType.eq(product_type));
    }

    if let Some(search) = &params.search {
        let search_ids: Vec<Uuid> = product_translation::Entity::find()
            .filter(product_translation::Column::Locale.eq(&request.locale))
            .filter(product_translation::Column::Title.contains(search))
            .all(&ctx.db)
            .await
            .map_err(|err| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error("DB_ERROR", err.to_string())),
                )
            })?
            .into_iter()
            .map(|translation| translation.product_id)
            .collect();

        if search_ids.is_empty() {
            return Ok(Json(PaginatedResponse {
                data: Vec::new(),
                meta: PaginationMeta::new(pagination.page, pagination.per_page, 0),
            }));
        }

        query = query.filter(product::Column::Id.is_in(search_ids));
    }

    let total = query
        .clone()
        .count(&ctx.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", err.to_string())),
            )
        })?;

    let products = query
        .order_by_desc(product::Column::CreatedAt)
        .offset(pagination.offset())
        .limit(pagination.limit())
        .all(&ctx.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", err.to_string())),
            )
        })?;

    let product_ids: Vec<Uuid> = products.iter().map(|product| product.id).collect();
    let translations = product_translation::Entity::find()
        .filter(product_translation::Column::ProductId.is_in(product_ids.clone()))
        .filter(product_translation::Column::Locale.eq(&request.locale))
        .all(&ctx.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", err.to_string())),
            )
        })?;

    let translation_map: std::collections::HashMap<Uuid, _> = translations
        .into_iter()
        .map(|translation| (translation.product_id, translation))
        .collect();

    let items = products
        .into_iter()
        .map(|product| {
            let translation = translation_map.get(&product.id);
            ProductListItem {
                id: product.id,
                status: product.status,
                title: translation
                    .map(|value| value.title.clone())
                    .unwrap_or_default(),
                handle: translation
                    .map(|value| value.handle.clone())
                    .unwrap_or_default(),
                vendor: product.vendor,
                product_type: product.product_type,
                created_at: product.created_at.to_rfc3339(),
                published_at: product.published_at.map(|value| value.to_rfc3339()),
            }
        })
        .collect();

    Ok(Json(PaginatedResponse {
        data: items,
        meta: PaginationMeta::new(pagination.page, pagination.per_page, total),
    }))
}

pub(super) async fn create_product(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Json(input): Json<CreateProductInput>,
) -> Result<(StatusCode, Json<ApiResponse<ProductResponse>>), (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    let product = service
        .create_product(request.tenant_id, user_id, input)
        .await
        .map_err(|err| err.into())?;

    Ok((StatusCode::CREATED, Json(ApiResponse::success(product))))
}

pub(super) async fn show_product(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    let product = service
        .get_product(request.tenant_id, id)
        .await
        .map_err(|err| err.into())?;

    Ok(Json(ApiResponse::success(product)))
}

pub(super) async fn update_product(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProductInput>,
) -> Result<Json<ApiResponse<ProductResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    let product = service
        .update_product(request.tenant_id, user_id, id, input)
        .await
        .map_err(|err| err.into())?;

    Ok(Json(ApiResponse::success(product)))
}

pub(super) async fn delete_product(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    service
        .delete_product(request.tenant_id, user_id, id)
        .await
        .map_err(|err| err.into())?;

    Ok(StatusCode::NO_CONTENT)
}

pub(super) async fn publish_product(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    let product = service
        .publish_product(request.tenant_id, user_id, id)
        .await
        .map_err(|err| err.into())?;

    Ok(Json(ApiResponse::success(product)))
}

pub(super) async fn unpublish_product(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    let product = service
        .unpublish_product(request.tenant_id, user_id, id)
        .await
        .map_err(|err| err.into())?;

    Ok(Json(ApiResponse::success(product)))
}

#[derive(Debug, serde::Deserialize)]
pub struct ListProductsParams {
    #[serde(flatten)]
    pub pagination: Option<PaginationParams>,
    pub status: Option<String>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ProductListItem {
    pub id: Uuid,
    pub status: String,
    pub title: String,
    pub handle: String,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub created_at: String,
    pub published_at: Option<String>,
}
