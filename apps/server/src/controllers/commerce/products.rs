use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use utoipa::ToSchema;
use uuid::Uuid;

use rustok_commerce::dto::{CreateProductInput, ProductResponse, UpdateProductInput};
use rustok_commerce::CatalogService;

use crate::common::{PaginatedResponse, PaginationMeta, PaginationParams};
use crate::context::TenantContext;
use crate::extractors::rbac::{
    RequireProductsCreate, RequireProductsDelete, RequireProductsList, RequireProductsRead,
    RequireProductsUpdate,
};
use crate::services::event_bus::transactional_event_bus_from_context;

/// List commerce products
#[utoipa::path(
    get,
    path = "/api/commerce/products",
    tag = "commerce",
    params(
        ListProductsParams
    ),
    responses(
        (status = 200, description = "List of products", body = PaginatedResponse<ProductListItem>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn list_products(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsList(_user): RequireProductsList,
    Query(params): Query<ListProductsParams>,
) -> Result<Json<PaginatedResponse<ProductListItem>>> {
    use rustok_commerce::entities::{product, product_translation};

    let pagination = params.pagination.unwrap_or_default();
    let locale = params.locale.as_deref().unwrap_or("en");

    let mut query = product::Entity::find().filter(product::Column::TenantId.eq(tenant.id));

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
            .filter(product_translation::Column::Locale.eq(locale))
            .filter(product_translation::Column::Title.contains(search))
            .all(&ctx.db)
            .await
            .map_err(|err| Error::BadRequest(err.to_string()))?
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
        .map_err(|err| Error::BadRequest(err.to_string()))?;

    let products = query
        .order_by_desc(product::Column::CreatedAt)
        .offset(pagination.offset())
        .limit(pagination.limit())
        .all(&ctx.db)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;

    let product_ids: Vec<Uuid> = products.iter().map(|product| product.id).collect();
    let translations = product_translation::Entity::find()
        .filter(product_translation::Column::ProductId.is_in(product_ids.clone()))
        .filter(product_translation::Column::Locale.eq(locale))
        .all(&ctx.db)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;

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
                status: product.status.to_string(),
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

/// Create a new commerce product
#[utoipa::path(
    post,
    path = "/api/commerce/products",
    tag = "commerce",
    request_body = CreateProductInput,
    responses(
        (status = 201, description = "Product created successfully", body = ProductResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn create_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsCreate(user): RequireProductsCreate,
    Json(input): Json<CreateProductInput>,
) -> Result<(StatusCode, Json<ProductResponse>)> {
    let service = CatalogService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let product = service
        .create_product(tenant.id, user.user.id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(product)))
}

/// Get product details
#[utoipa::path(
    get,
    path = "/api/commerce/products/{id}",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product details", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn show_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: RequireProductsRead,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>> {
    let service = CatalogService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let product = service
        .get_product(tenant.id, id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(Json(product))
}

/// Update an existing product
#[utoipa::path(
    put,
    path = "/api/commerce/products/{id}",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    request_body = UpdateProductInput,
    responses(
        (status = 200, description = "Product updated successfully", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn update_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsUpdate(user): RequireProductsUpdate,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateProductInput>,
) -> Result<Json<ProductResponse>> {
    let service = CatalogService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let product = service
        .update_product(tenant.id, user.user.id, id, input)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(Json(product))
}

/// Delete a product
#[utoipa::path(
    delete,
    path = "/api/commerce/products/{id}",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 204, description = "Product deleted successfully"),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn delete_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsDelete(user): RequireProductsDelete,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let service = CatalogService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .delete_product(tenant.id, user.user.id, id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Publish a product
#[utoipa::path(
    post,
    path = "/api/commerce/products/{id}/publish",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product published successfully", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn publish_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsUpdate(user): RequireProductsUpdate,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>> {
    let service = CatalogService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let product = service
        .publish_product(tenant.id, user.user.id, id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(Json(product))
}

/// Unpublish a product
#[utoipa::path(
    post,
    path = "/api/commerce/products/{id}/unpublish",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Product unpublished successfully", body = ProductResponse),
        (status = 404, description = "Product not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub(super) async fn unpublish_product(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    RequireProductsUpdate(user): RequireProductsUpdate,
    Path(id): Path<Uuid>,
) -> Result<Json<ProductResponse>> {
    let service = CatalogService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let product = service
        .unpublish_product(tenant.id, user.user.id, id)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(Json(product))
}

#[derive(Debug, serde::Deserialize, ToSchema, utoipa::IntoParams)]
pub struct ListProductsParams {
    #[serde(flatten)]
    pub pagination: Option<PaginationParams>,
    pub status: Option<String>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub search: Option<String>,
    pub locale: Option<String>,
}

#[derive(Debug, serde::Serialize, ToSchema)]
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
