use chrono::Utc;
use rust_decimal::Decimal;
use rustok_order::dto::{CreateOrderInput, CreateOrderLineItemInput};
use rustok_order::error::OrderError;
use rustok_order::services::OrderService;
use rustok_test_utils::{db::setup_test_db, mock_transactional_event_bus};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use std::str::FromStr;
use uuid::Uuid;

mod support;

mod order_field_definitions {
    rustok_core::define_field_definitions_entity!("order_field_definitions");
}

async fn setup() -> OrderService {
    let db = setup_test_db().await;
    support::ensure_order_schema(&db).await;
    OrderService::new(db, mock_transactional_event_bus())
}

fn create_order_input() -> CreateOrderInput {
    CreateOrderInput {
        customer_id: Some(Uuid::new_v4()),
        currency_code: "usd".to_string(),
        line_items: vec![
            CreateOrderLineItemInput {
                product_id: Some(Uuid::new_v4()),
                variant_id: Some(Uuid::new_v4()),
                shipping_profile_slug: "default".to_string(),
                seller_id: None,
                sku: Some("SKU-1".to_string()),
                title: "Test product".to_string(),
                quantity: 2,
                unit_price: Decimal::from_str("19.99").unwrap(),
                metadata: serde_json::json!({ "slot": 1 }),
            },
            CreateOrderLineItemInput {
                product_id: None,
                variant_id: None,
                shipping_profile_slug: "gift".to_string(),
                seller_id: None,
                sku: Some("SKU-2".to_string()),
                title: "Gift wrap".to_string(),
                quantity: 1,
                unit_price: Decimal::from_str("4.00").unwrap(),
                metadata: serde_json::json!({ "slot": 2 }),
            },
        ],
        metadata: serde_json::json!({ "source": "order-test" }),
    }
}

#[tokio::test]
async fn create_order_persists_snapshot_and_total() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let created = service
        .create_order(tenant_id, actor_id, create_order_input())
        .await
        .unwrap();

    assert_eq!(created.status, "pending");
    assert_eq!(created.currency_code, "USD");
    assert_eq!(created.line_items.len(), 2);
    assert_eq!(created.total_amount, Decimal::from_str("43.98").unwrap());
}

#[tokio::test]
async fn create_order_with_channel_persists_channel_snapshot() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let channel_id = Uuid::new_v4();

    let created = service
        .create_order_with_channel(
            tenant_id,
            actor_id,
            create_order_input(),
            Some(channel_id),
            Some("marketplace-eu".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(created.channel_id, Some(channel_id));
    assert_eq!(created.channel_slug.as_deref(), Some("marketplace-eu"));
}

#[tokio::test]
async fn order_lifecycle_transitions_persist_status_metadata() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let created = service
        .create_order(tenant_id, actor_id, create_order_input())
        .await
        .unwrap();

    let confirmed = service
        .confirm_order(tenant_id, actor_id, created.id)
        .await
        .unwrap();
    assert_eq!(confirmed.status, "confirmed");
    assert!(confirmed.confirmed_at.is_some());

    let paid = service
        .mark_paid(
            tenant_id,
            actor_id,
            created.id,
            "pay_123".to_string(),
            "manual".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(paid.status, "paid");
    assert_eq!(paid.payment_id.as_deref(), Some("pay_123"));

    let shipped = service
        .ship_order(
            tenant_id,
            actor_id,
            created.id,
            "trk_123".to_string(),
            "dhl".to_string(),
        )
        .await
        .unwrap();
    assert_eq!(shipped.status, "shipped");
    assert_eq!(shipped.tracking_number.as_deref(), Some("trk_123"));

    let delivered = service
        .deliver_order(
            tenant_id,
            actor_id,
            created.id,
            Some("front-desk".to_string()),
        )
        .await
        .unwrap();
    assert_eq!(delivered.status, "delivered");
    assert_eq!(delivered.delivered_signature.as_deref(), Some("front-desk"));
}

#[tokio::test]
async fn invalid_transition_is_rejected() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let created = service
        .create_order(tenant_id, actor_id, create_order_input())
        .await
        .unwrap();

    let error = service
        .ship_order(
            tenant_id,
            actor_id,
            created.id,
            "trk_123".to_string(),
            "dhl".to_string(),
        )
        .await
        .unwrap_err();

    match error {
        OrderError::InvalidTransition { from, to } => {
            assert_eq!(from, "pending");
            assert_eq!(to, "shipped");
        }
        other => panic!("expected invalid transition, got {other:?}"),
    }
}

#[tokio::test]
async fn localized_order_custom_fields_resolve_from_attached_values() {
    let db = setup_test_db().await;
    support::ensure_order_schema(&db).await;
    let service = OrderService::new(db.clone(), mock_transactional_event_bus());
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let now = Utc::now();

    order_field_definitions::ActiveModel {
        id: Set(rustok_core::generate_id()),
        tenant_id: Set(tenant_id),
        field_key: Set("gift_message".to_string()),
        field_type: Set("Text".to_string()),
        label: Set(serde_json::json!({ "en": "Gift message" })),
        description: Set(None),
        is_localized: Set(true),
        is_required: Set(false),
        default_value: Set(None),
        validation: Set(None),
        position: Set(0),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(&db)
    .await
    .expect("field definition should insert");

    let created = service
        .create_order(
            tenant_id,
            actor_id,
            CreateOrderInput {
                metadata: serde_json::json!({
                    "locale": "de-DE",
                    "gift_message": "Danke",
                    "source": "order-test"
                }),
                ..create_order_input()
            },
        )
        .await
        .expect("order should be created");

    assert_eq!(created.metadata["gift_message"], serde_json::json!("Danke"));
    assert_eq!(created.metadata["source"], serde_json::json!("order-test"));

    let storefront_view = service
        .get_order_with_locale_fallback(tenant_id, created.id, "de-DE", Some("en"))
        .await
        .expect("order should load with locale fallback");
    assert_eq!(
        storefront_view.metadata["gift_message"],
        serde_json::json!("Danke")
    );
    assert_eq!(
        storefront_view.metadata["source"],
        serde_json::json!("order-test")
    );
}
