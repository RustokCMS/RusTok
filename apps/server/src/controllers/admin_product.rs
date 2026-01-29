use axum::{extract::State, routing::post, Json};
use loco_rs::prelude::*;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::extractors::{auth::CurrentUser, tenant::CurrentTenant};
use rustok_commerce::dto::{
    CreateProductInput, CreateVariantInput, PriceInput, ProductOptionInput,
    ProductTranslationInput,
};
use rustok_commerce::{CatalogService, CommerceError};
use rustok_core::{EventBus, Permission, Rbac};

#[derive(Debug, Deserialize)]
pub struct CreateProductParams {
    pub status: Option<String>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub title_en: String,
    pub price: Decimal,
}

fn ensure_permission(user: &CurrentUser, permission: Permission) -> Result<()> {
    if !Rbac::has_permission(&user.user.role, &permission) {
        return Err(Error::Unauthorized("Permission denied".into()));
    }
    Ok(())
}

fn map_commerce_error(error: CommerceError) -> Error {
    match error {
        CommerceError::ProductNotFound(_)
        | CommerceError::VariantNotFound(_)
        | CommerceError::DuplicateHandle { .. }
        | CommerceError::DuplicateSku(_)
        | CommerceError::InvalidPrice(_)
        | CommerceError::InsufficientInventory { .. }
        | CommerceError::InvalidOptionCombination
        | CommerceError::Validation(_)
        | CommerceError::NoVariants
        | CommerceError::CannotDeletePublished => Error::BadRequest(error.to_string()),
        CommerceError::Database(err) => Error::BadRequest(err.to_string()),
    }
}

async fn create_product(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    CurrentUser(current_user): CurrentUser,
    Json(params): Json<CreateProductParams>,
) -> Result<Response> {
    ensure_permission(&current_user, Permission::PRODUCTS_CREATE)?;

    let publish = matches!(params.status.as_deref(), Some("active" | "published"));

    let input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: params.title_en.clone(),
            handle: None,
            description: None,
            meta_title: None,
            meta_description: None,
        }],
        options: vec![ProductOptionInput {
            name: "Default".to_string(),
            values: vec!["Default".to_string()],
        }],
        variants: vec![CreateVariantInput {
            sku: None,
            barcode: None,
            option1: Some("Default".to_string()),
            option2: None,
            option3: None,
            prices: vec![PriceInput {
                currency_code: "USD".to_string(),
                amount: params.price,
                compare_at_amount: None,
            }],
            inventory_quantity: 100,
            inventory_policy: "deny".to_string(),
            weight: None,
            weight_unit: None,
        }],
        vendor: params.vendor,
        product_type: params.product_type,
        metadata: serde_json::json!({}),
        publish,
    };

    let service = CatalogService::new(ctx.db.clone(), EventBus::default());
    let product = service
        .create_product(tenant.id, current_user.user.id, input)
        .await
        .map_err(map_commerce_error)?;

    format::json(serde_json::json!({ "id": product.id }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/admin/products")
        .add("/", post(create_product))
}
