use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use rustok_commerce::InventoryService;

use crate::common::{ApiErrorResponse, ApiResponse, RequestContext};
use crate::services::event_bus::event_bus_from_context;
use loco_rs::app::AppContext;

/// Get variant inventory info
#[utoipa::path(
    get,
    path = "/api/commerce/variants/{id}/inventory",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Variant ID")
    ),
    responses(
        (status = 200, description = "Inventory details", body = InventoryResponse),
        (status = 404, description = "Variant not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(super) async fn get_inventory(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<ApiResponse<InventoryResponse>>, ApiErrorResponse> {
    use rustok_commerce::entities::product_variant;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    let variant = product_variant::Entity::find_by_id(variant_id)
        .filter(product_variant::Column::TenantId.eq(request.tenant_id))
        .one(&ctx.db)
        .await
        .map_err(|err| {
            ApiErrorResponse::from((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("DB_ERROR", err.to_string())),
            ))
        })?
        .ok_or_else(|| {
            ApiErrorResponse::from((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(
                    "VARIANT_NOT_FOUND",
                    "Variant not found",
                )),
            ))
        })?;

    Ok(Json(ApiResponse::success(InventoryResponse {
        variant_id,
        quantity: variant.inventory_quantity,
        policy: variant.inventory_policy.clone(),
        in_stock: variant.inventory_quantity > 0 || variant.inventory_policy == "continue",
    })))
}

/// Adjust variant inventory quantity
#[utoipa::path(
    post,
    path = "/api/commerce/variants/{id}/inventory/adjust",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Variant ID")
    ),
    request_body = AdjustInput,
    responses(
        (status = 200, description = "Inventory adjusted successfully", body = InventoryResponse),
        (status = 400, description = "Insufficient inventory or invalid input"),
        (status = 404, description = "Variant not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(super) async fn adjust_inventory(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(variant_id): Path<Uuid>,
    Json(input): Json<AdjustInput>,
) -> Result<Json<ApiResponse<InventoryResponse>>, ApiErrorResponse> {
    let user_id = request.require_user()?;

    let service = InventoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
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
            ApiErrorResponse::from((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(code, err.to_string())),
            ))
        })?;

    get_inventory(State(ctx), request, Path(variant_id)).await
}

/// Set absolute inventory quantity
#[utoipa::path(
    post,
    path = "/api/commerce/variants/{id}/inventory/set",
    tag = "commerce",
    params(
        ("id" = Uuid, Path, description = "Variant ID")
    ),
    request_body = SetInventoryInput,
    responses(
        (status = 200, description = "Inventory set successfully", body = InventoryResponse),
        (status = 404, description = "Variant not found"),
        (status = 401, description = "Unauthorized")
    )
)]
pub(super) async fn set_inventory(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Path(variant_id): Path<Uuid>,
    Json(input): Json<SetInventoryInput>,
) -> Result<Json<ApiResponse<InventoryResponse>>, ApiErrorResponse> {
    let user_id = request.require_user()?;

    let service = InventoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
    service
        .set_inventory(request.tenant_id, user_id, variant_id, input.quantity)
        .await
        .map_err(|err| {
            ApiErrorResponse::from((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("INVENTORY_ERROR", err.to_string())),
            ))
        })?;

    get_inventory(State(ctx), request, Path(variant_id)).await
}

/// Batch check inventory availability
#[utoipa::path(
    post,
    path = "/api/commerce/inventory/check",
    tag = "commerce",
    request_body = CheckAvailabilityInput,
    responses(
        (status = 200, description = "Availability results", body = Vec<AvailabilityResult>),
        (status = 401, description = "Unauthorized")
    )
)]
pub(super) async fn check_availability(
    State(ctx): State<AppContext>,
    request: RequestContext,
    Json(input): Json<CheckAvailabilityInput>,
) -> Result<Json<ApiResponse<Vec<AvailabilityResult>>>, ApiErrorResponse> {
    let service = InventoryService::new(ctx.db.clone(), event_bus_from_context(&ctx));
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

#[derive(Debug, Serialize, ToSchema)]
pub struct InventoryResponse {
    pub variant_id: Uuid,
    pub quantity: i32,
    pub policy: String,
    pub in_stock: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AdjustInput {
    pub adjustment: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetInventoryInput {
    pub quantity: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckAvailabilityInput {
    pub items: Vec<CheckItem>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CheckItem {
    pub variant_id: Uuid,
    pub quantity: i32,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AvailabilityResult {
    pub variant_id: Uuid,
    pub requested: i32,
    pub available: bool,
}
