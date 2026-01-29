use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rustok_commerce::InventoryService;
use rustok_core::EventBus;

use crate::common::{ApiResponse, RequestContext};
use crate::models::AppContext;

pub(super) async fn get_inventory(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<ApiResponse<InventoryResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    use rustok_commerce::entities::product_variant;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let variant = product_variant::Entity::find_by_id(variant_id)
        .filter(product_variant::Column::TenantId.eq(request.tenant_id))
        .one(&ctx.db)
        .await
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error("DB_ERROR", err.to_string())),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error("VARIANT_NOT_FOUND", "Variant not found")),
            )
        })?;

    Ok(Json(ApiResponse::success(InventoryResponse {
        variant_id,
        quantity: variant.inventory_quantity,
        policy: variant.inventory_policy.clone(),
        in_stock: variant.inventory_quantity > 0 || variant.inventory_policy == "continue",
    })))
}

pub(super) async fn adjust_inventory(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(variant_id): Path<Uuid>,
    Json(input): Json<AdjustInput>,
) -> Result<Json<ApiResponse<InventoryResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = InventoryService::new(ctx.db.clone(), EventBus::default());
    service
        .adjust_inventory(
            request.tenant_id,
            user_id,
            rustok_commerce::dto::AdjustInventoryInput {
                variant_id,
                adjustment: input.adjustment,
                reason: input.reason,
            },
        )
        .await
        .map_err(|err| {
            let code = match &err {
                rustok_commerce::CommerceError::InsufficientInventory { .. } => {
                    "INSUFFICIENT_INVENTORY"
                }
                _ => "INVENTORY_ERROR",
            };
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(code, err.to_string())),
            )
        })?;

    get_inventory(State(ctx), request, Path(variant_id)).await
}

pub(super) async fn set_inventory(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(variant_id): Path<Uuid>,
    Json(input): Json<SetInventoryInput>,
) -> Result<Json<ApiResponse<InventoryResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let user_id = request.require_user()?;

    let service = InventoryService::new(ctx.db.clone(), EventBus::default());
    service
        .set_inventory(request.tenant_id, user_id, variant_id, input.quantity)
        .await
        .map_err(|err| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("INVENTORY_ERROR", err.to_string())),
            )
        })?;

    get_inventory(State(ctx), request, Path(variant_id)).await
}

pub(super) async fn check_availability(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Json(input): Json<CheckAvailabilityInput>,
) -> Result<Json<ApiResponse<Vec<AvailabilityResult>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let service = InventoryService::new(ctx.db.clone(), EventBus::default());
    let mut results = Vec::new();

    for item in input.items {
        let available = service
            .check_availability(request.tenant_id, item.variant_id, item.quantity)
            .await
            .unwrap_or(false);

        results.push(AvailabilityResult {
            variant_id: item.variant_id,
            requested: item.quantity,
            available,
        });
    }

    Ok(Json(ApiResponse::success(results)))
}

#[derive(Debug, Serialize)]
pub struct InventoryResponse {
    pub variant_id: Uuid,
    pub quantity: i32,
    pub policy: String,
    pub in_stock: bool,
}

#[derive(Debug, Deserialize)]
pub struct AdjustInput {
    pub adjustment: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetInventoryInput {
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct CheckAvailabilityInput {
    pub items: Vec<CheckItem>,
}

#[derive(Debug, Deserialize)]
pub struct CheckItem {
    pub variant_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize)]
pub struct AvailabilityResult {
    pub variant_id: Uuid,
    pub requested: i32,
    pub available: bool,
}
