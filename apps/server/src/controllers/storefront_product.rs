use axum::{
    extract::{Path, Query, State},
    routing::get,
};
use loco_rs::prelude::*;
use serde::Deserialize;

use crate::extractors::tenant::CurrentTenant;
use rustok_index::product::{ProductQueryBuilder, ProductQueryService, ProductSortBy, SortOrder};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct StorefrontProductListParams {
    pub locale: Option<String>,
    pub category_id: Option<Uuid>,
    pub tag: Option<String>,
    pub search: Option<String>,
    pub in_stock: Option<bool>,
    pub price_min: Option<i64>,
    pub price_max: Option<i64>,
    pub sort_by: Option<ProductSortBy>,
    pub sort_order: Option<SortOrder>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct StorefrontProductShowParams {
    pub locale: Option<String>,
}

async fn list_products(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Query(params): Query<StorefrontProductListParams>,
) -> Result<Response> {
    let locale = params.locale.as_deref().unwrap_or("en");
    let mut builder = ProductQueryBuilder::new(tenant.id, locale);

    if let Some(category_id) = params.category_id {
        builder = builder.category(category_id);
    }

    if let Some(tag) = params.tag {
        builder = builder.tag(tag);
    }

    if let Some(search) = params.search {
        builder = builder.search(search);
    }

    if let Some(in_stock) = params.in_stock {
        builder = builder.in_stock(in_stock);
    }

    builder = builder.price_range(params.price_min, params.price_max);

    if let (Some(sort_by), Some(sort_order)) = (params.sort_by, params.sort_order) {
        builder = builder.sort(sort_by, sort_order);
    }

    let limit = params.limit.unwrap_or(20);
    let offset = params.offset.unwrap_or(0);
    let query = builder.paginate(limit, offset).build();

    let query_service = ProductQueryService::new(ctx.db.clone());
    let products = query_service
        .find(query)
        .await
        .map_err(|err| Error::InternalServerError(err.to_string()))?;

    format::json(products)
}

async fn show_product(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Path(handle): Path<String>,
    Query(params): Query<StorefrontProductShowParams>,
) -> Result<Response> {
    let locale = params.locale.as_deref().unwrap_or("en");
    let query_service = ProductQueryService::new(ctx.db.clone());
    let product = query_service
        .find_by_handle(tenant.id, locale, &handle)
        .await
        .map_err(|err| Error::InternalServerError(err.to_string()))?;

    match product {
        Some(product) => format::json(product),
        None => Err(Error::NotFound("Product not found".into())),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/storefront/products")
        .add("/", get(list_products))
        .add("/:handle", get(show_product))
}
