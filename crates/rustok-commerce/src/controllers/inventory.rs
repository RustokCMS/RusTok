use axum::{
    extract::{Path, State},
    Json,
};
use loco_rs::{app::AppContext, Error, Result};
use rustok_api::{loco::transactional_event_bus_from_context, AuthContext, TenantContext};
use rustok_core::Permission;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::InventoryService;

use super::common::ensure_permissions;

/// Get variant inventory info
#[utoipa::path(
    get,
    path = "/api/commerce/variants/{id}/inventory",
    tag = "commerce",
    params(("id" = Uuid, Path, description = "Variant ID")),
    responses(
        (status = 200, description = "Inventory details", body = InventoryResponse),
        (status = 404, description = "Variant not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn get_inventory(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Path(variant_id): Path<Uuid>,
) -> Result<Json<InventoryResponse>> {
    ensure_permissions(
        &auth,
        &[Permission::INVENTORY_READ],
        "Permission denied: inventory:read required",
    )?;

    let response = load_inventory_response(&ctx.db, tenant.id, variant_id).await?;
    Ok(Json(response))
}

/// Adjust variant inventory quantity
#[utoipa::path(
    post,
    path = "/api/commerce/variants/{id}/inventory/adjust",
    tag = "commerce",
    params(("id" = Uuid, Path, description = "Variant ID")),
    request_body = AdjustInput,
    responses(
        (status = 200, description = "Inventory adjusted successfully", body = InventoryResponse),
        (status = 400, description = "Insufficient inventory or invalid input"),
        (status = 404, description = "Variant not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn adjust_inventory(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Path(variant_id): Path<Uuid>,
    Json(input): Json<AdjustInput>,
) -> Result<Json<InventoryResponse>> {
    ensure_permissions(
        &auth,
        &[Permission::INVENTORY_UPDATE],
        "Permission denied: inventory:update required",
    )?;

    let service = InventoryService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .adjust_inventory(
            tenant.id,
            auth.user_id,
            crate::dto::AdjustInventoryInput {
                variant_id,
                adjustment: input.adjustment,
                reason: input.reason,
            },
        )
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;

    let response = load_inventory_response(&ctx.db, tenant.id, variant_id).await?;
    Ok(Json(response))
}

/// Set absolute inventory quantity
#[utoipa::path(
    post,
    path = "/api/commerce/variants/{id}/inventory/set",
    tag = "commerce",
    params(("id" = Uuid, Path, description = "Variant ID")),
    request_body = SetInventoryInput,
    responses(
        (status = 200, description = "Inventory set successfully", body = InventoryResponse),
        (status = 404, description = "Variant not found"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn set_inventory(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Path(variant_id): Path<Uuid>,
    Json(input): Json<SetInventoryInput>,
) -> Result<Json<InventoryResponse>> {
    ensure_permissions(
        &auth,
        &[Permission::INVENTORY_UPDATE],
        "Permission denied: inventory:update required",
    )?;

    let service = InventoryService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    service
        .set_inventory(tenant.id, auth.user_id, variant_id, input.quantity)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?;

    let response = load_inventory_response(&ctx.db, tenant.id, variant_id).await?;
    Ok(Json(response))
}

/// Batch check inventory availability
#[utoipa::path(
    post,
    path = "/api/commerce/inventory/check",
    tag = "commerce",
    request_body = CheckAvailabilityInput,
    responses(
        (status = 200, description = "Availability results", body = Vec<AvailabilityResult>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden")
    )
)]
pub async fn check_availability(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    auth: AuthContext,
    Json(input): Json<CheckAvailabilityInput>,
) -> Result<Json<Vec<AvailabilityResult>>> {
    ensure_permissions(
        &auth,
        &[Permission::INVENTORY_LIST],
        "Permission denied: inventory:list required",
    )?;

    let service = InventoryService::new(ctx.db.clone(), transactional_event_bus_from_context(&ctx));
    let mut results = Vec::new();

    for item in input.items {
        let available = service
            .check_availability(tenant.id, item.variant_id, item.quantity)
            .await
            .unwrap_or(false);

        results.push(AvailabilityResult {
            variant_id: item.variant_id,
            requested: item.quantity,
            available,
        });
    }

    Ok(Json(results))
}

async fn load_inventory_response(
    db: &DatabaseConnection,
    tenant_id: Uuid,
    variant_id: Uuid,
) -> Result<InventoryResponse> {
    use crate::entities::product_variant;

    let variant = product_variant::Entity::find_by_id(variant_id)
        .filter(product_variant::Column::TenantId.eq(tenant_id))
        .one(db)
        .await
        .map_err(|err| Error::BadRequest(err.to_string()))?
        .ok_or(Error::NotFound)?;

    Ok(InventoryResponse {
        variant_id,
        quantity: variant.inventory_quantity,
        policy: variant.inventory_policy.clone(),
        in_stock: variant.inventory_quantity > 0 || variant.inventory_policy == "continue",
    })
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
