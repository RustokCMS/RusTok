use rust_decimal::Decimal;
use rustok_cart::dto::{AddCartLineItemInput, CreateCartInput};
use rustok_cart::error::CartError;
use rustok_cart::services::CartService;
use rustok_test_utils::db::setup_test_db;
use std::str::FromStr;
use uuid::Uuid;

mod support;

async fn setup() -> CartService {
    let db = setup_test_db().await;
    support::ensure_cart_schema(&db).await;
    CartService::new(db)
}

fn create_cart_input() -> CreateCartInput {
    CreateCartInput {
        customer_id: Some(Uuid::new_v4()),
        email: Some("buyer@example.com".to_string()),
        currency_code: "usd".to_string(),
        metadata: serde_json::json!({ "source": "cart-test" }),
    }
}

fn line_item_input() -> AddCartLineItemInput {
    AddCartLineItemInput {
        product_id: Some(Uuid::new_v4()),
        variant_id: Some(Uuid::new_v4()),
        sku: Some("SKU-CART-1".to_string()),
        title: "Cart product".to_string(),
        quantity: 2,
        unit_price: Decimal::from_str("15.50").unwrap(),
        metadata: serde_json::json!({ "slot": 1 }),
    }
}

#[tokio::test]
async fn create_cart_and_add_line_item_updates_totals() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let cart = service
        .create_cart(tenant_id, create_cart_input())
        .await
        .unwrap();
    let updated = service
        .add_line_item(tenant_id, cart.id, line_item_input())
        .await
        .unwrap();

    assert_eq!(updated.status, "active");
    assert_eq!(updated.currency_code, "USD");
    assert_eq!(updated.line_items.len(), 1);
    assert_eq!(updated.total_amount, Decimal::from_str("31.00").unwrap());
}

#[tokio::test]
async fn update_and_remove_line_items_recalculate_cart() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let cart = service
        .create_cart(tenant_id, create_cart_input())
        .await
        .unwrap();
    let cart = service
        .add_line_item(tenant_id, cart.id, line_item_input())
        .await
        .unwrap();
    let line_item_id = cart.line_items[0].id;

    let cart = service
        .update_line_item_quantity(tenant_id, cart.id, line_item_id, 3)
        .await
        .unwrap();
    assert_eq!(cart.total_amount, Decimal::from_str("46.50").unwrap());

    let cart = service
        .remove_line_item(tenant_id, cart.id, line_item_id)
        .await
        .unwrap();
    assert_eq!(cart.line_items.len(), 0);
    assert_eq!(cart.total_amount, Decimal::ZERO);
}

#[tokio::test]
async fn completed_cart_rejects_mutations() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let cart = service
        .create_cart(tenant_id, create_cart_input())
        .await
        .unwrap();
    let cart = service
        .add_line_item(tenant_id, cart.id, line_item_input())
        .await
        .unwrap();
    let cart = service.complete_cart(tenant_id, cart.id).await.unwrap();
    assert_eq!(cart.status, "completed");

    let error = service
        .add_line_item(tenant_id, cart.id, line_item_input())
        .await
        .unwrap_err();
    match error {
        CartError::InvalidTransition { from, .. } => assert_eq!(from, "completed"),
        other => panic!("expected invalid transition, got {other:?}"),
    }
}
