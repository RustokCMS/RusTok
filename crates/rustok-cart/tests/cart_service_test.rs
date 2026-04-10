use rust_decimal::Decimal;
use rustok_cart::dto::{
    AddCartLineItemInput, CartShippingSelectionInput, CreateCartInput, SetCartAdjustmentInput,
    UpdateCartContextInput,
};
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
        region_id: None,
        country_code: None,
        locale_code: None,
        selected_shipping_option_id: None,
        currency_code: "usd".to_string(),
        metadata: serde_json::json!({ "source": "cart-test" }),
    }
}

fn line_item_input() -> AddCartLineItemInput {
    AddCartLineItemInput {
        product_id: Some(Uuid::new_v4()),
        variant_id: Some(Uuid::new_v4()),
        shipping_profile_slug: None,
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
    assert_eq!(updated.subtotal_amount, Decimal::from_str("31.00").unwrap());
    assert_eq!(updated.adjustment_total, Decimal::ZERO);
    assert_eq!(updated.total_amount, Decimal::from_str("31.00").unwrap());
}

#[tokio::test]
async fn set_adjustments_recalculates_cart_total_without_localized_labels() {
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

    let updated = service
        .set_adjustments(
            tenant_id,
            cart.id,
            vec![SetCartAdjustmentInput {
                line_item_id: Some(line_item_id),
                source_type: "Promotion".to_string(),
                source_id: Some("promo-spring".to_string()),
                amount: Decimal::from_str("5.00").unwrap(),
                metadata: serde_json::json!({
                    "rule_code": "spring",
                    "label": "Spring sale"
                }),
            }],
        )
        .await
        .unwrap();

    assert_eq!(updated.subtotal_amount, Decimal::from_str("31.00").unwrap());
    assert_eq!(updated.adjustment_total, Decimal::from_str("5.00").unwrap());
    assert_eq!(updated.total_amount, Decimal::from_str("26.00").unwrap());
    assert_eq!(updated.adjustments.len(), 1);
    assert_eq!(updated.adjustments[0].line_item_id, Some(line_item_id));
    assert_eq!(updated.adjustments[0].source_type, "promotion");
    assert_eq!(updated.adjustments[0].currency_code, "USD");
    assert!(updated.adjustments[0].metadata.get("label").is_none());
}

#[tokio::test]
async fn create_cart_persists_multilingual_context_snapshot() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let region_id = Uuid::new_v4();
    let shipping_option_id = Uuid::new_v4();

    let cart = service
        .create_cart(
            tenant_id,
            CreateCartInput {
                customer_id: Some(Uuid::new_v4()),
                email: Some("buyer@example.com".to_string()),
                region_id: Some(region_id),
                country_code: Some("de".to_string()),
                locale_code: Some("pt_BR".to_string()),
                selected_shipping_option_id: Some(shipping_option_id),
                currency_code: "eur".to_string(),
                metadata: serde_json::json!({ "source": "locale-test" }),
            },
        )
        .await
        .unwrap();

    assert_eq!(cart.region_id, Some(region_id));
    assert_eq!(cart.country_code.as_deref(), Some("DE"));
    assert_eq!(cart.locale_code.as_deref(), Some("pt-br"));
    assert_eq!(cart.selected_shipping_option_id, Some(shipping_option_id));
    assert_eq!(cart.currency_code, "EUR");
}

#[tokio::test]
async fn create_cart_with_channel_persists_channel_snapshot() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let channel_id = Uuid::new_v4();

    let cart = service
        .create_cart_with_channel(
            tenant_id,
            create_cart_input(),
            Some(channel_id),
            Some("web-store".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(cart.channel_id, Some(channel_id));
    assert_eq!(cart.channel_slug.as_deref(), Some("web-store"));
}

#[tokio::test]
async fn update_cart_context_rewrites_snapshot_fields() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let initial_region_id = Uuid::new_v4();
    let updated_region_id = Uuid::new_v4();
    let initial_shipping_option_id = Uuid::new_v4();
    let updated_shipping_option_id = Uuid::new_v4();

    let cart = service
        .create_cart(
            tenant_id,
            CreateCartInput {
                customer_id: Some(Uuid::new_v4()),
                email: Some("buyer@example.com".to_string()),
                region_id: Some(initial_region_id),
                country_code: Some("de".to_string()),
                locale_code: Some("de".to_string()),
                selected_shipping_option_id: Some(initial_shipping_option_id),
                currency_code: "eur".to_string(),
                metadata: serde_json::json!({ "source": "update-context-test" }),
            },
        )
        .await
        .unwrap();

    let updated = service
        .update_context(
            tenant_id,
            cart.id,
            UpdateCartContextInput {
                email: Some("checkout@example.com".to_string()),
                region_id: Some(updated_region_id),
                country_code: Some("pl".to_string()),
                locale_code: Some("pl_PL".to_string()),
                selected_shipping_option_id: Some(updated_shipping_option_id),
                shipping_selections: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.email.as_deref(), Some("checkout@example.com"));
    assert_eq!(updated.region_id, Some(updated_region_id));
    assert_eq!(updated.country_code.as_deref(), Some("PL"));
    assert_eq!(updated.locale_code.as_deref(), Some("pl-pl"));
    assert_eq!(
        updated.selected_shipping_option_id,
        Some(updated_shipping_option_id)
    );
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

#[tokio::test]
async fn checkout_lifecycle_uses_checking_out_before_completion() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let cart = service
        .create_cart(tenant_id, create_cart_input())
        .await
        .unwrap();
    let checking_out = service.begin_checkout(tenant_id, cart.id).await.unwrap();
    assert_eq!(checking_out.status, "checking_out");

    let error = service
        .add_line_item(tenant_id, cart.id, line_item_input())
        .await
        .unwrap_err();
    match error {
        CartError::InvalidTransition { from, .. } => assert_eq!(from, "checking_out"),
        other => panic!("expected invalid transition, got {other:?}"),
    }

    let reopened = service.release_checkout(tenant_id, cart.id).await.unwrap();
    assert_eq!(reopened.status, "active");

    let checking_out = service.begin_checkout(tenant_id, cart.id).await.unwrap();
    let completed = service
        .complete_cart(tenant_id, checking_out.id)
        .await
        .unwrap();
    assert_eq!(completed.status, "completed");
}

#[tokio::test]
async fn seller_aware_delivery_groups_split_same_shipping_profile() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let seller_a_option_id = Uuid::new_v4();
    let seller_b_option_id = Uuid::new_v4();
    let seller_a_id = "seller-a-id";
    let seller_b_id = "seller-b-id";

    let cart = service
        .create_cart(tenant_id, create_cart_input())
        .await
        .unwrap();
    let cart = service
        .add_line_item(
            tenant_id,
            cart.id,
            AddCartLineItemInput {
                metadata: serde_json::json!({
                    "seller": {
                        "id": seller_a_id,
                        "scope": "seller-a"
                    }
                }),
                ..line_item_input()
            },
        )
        .await
        .unwrap();
    let cart = service
        .add_line_item(
            tenant_id,
            cart.id,
            AddCartLineItemInput {
                sku: Some("SKU-CART-2".to_string()),
                metadata: serde_json::json!({
                    "seller": {
                        "id": seller_b_id,
                        "scope": "seller-b"
                    }
                }),
                ..line_item_input()
            },
        )
        .await
        .unwrap();

    assert_eq!(cart.delivery_groups.len(), 2);
    assert!(cart
        .delivery_groups
        .iter()
        .all(|group| group.shipping_profile_slug == "default"));

    let seller_ids = cart
        .line_items
        .iter()
        .map(|item| item.seller_id.clone().expect("seller id should be present"))
        .collect::<Vec<_>>();
    assert!(cart.line_items.iter().all(|item| {
        item.metadata
            .get("seller")
            .and_then(|seller| seller.get("label"))
            .is_none()
            && item.metadata.get("seller_label").is_none()
    }));
    assert!(seller_ids.contains(&seller_a_id.to_string()));
    assert!(seller_ids.contains(&seller_b_id.to_string()));

    let updated = service
        .update_context(
            tenant_id,
            cart.id,
            UpdateCartContextInput {
                email: cart.email.clone(),
                region_id: cart.region_id,
                country_code: cart.country_code.clone(),
                locale_code: cart.locale_code.clone(),
                selected_shipping_option_id: None,
                shipping_selections: Some(vec![
                    CartShippingSelectionInput {
                        shipping_profile_slug: "default".to_string(),
                        seller_id: Some(seller_a_id.to_string()),
                        seller_scope: None,
                        selected_shipping_option_id: Some(seller_a_option_id),
                    },
                    CartShippingSelectionInput {
                        shipping_profile_slug: "default".to_string(),
                        seller_id: Some(seller_b_id.to_string()),
                        seller_scope: None,
                        selected_shipping_option_id: Some(seller_b_option_id),
                    },
                ]),
            },
        )
        .await
        .unwrap();

    let delivery_groups = updated
        .delivery_groups
        .iter()
        .map(|group| {
            (
                group.shipping_profile_slug.clone(),
                group
                    .seller_id
                    .clone()
                    .expect("seller id should be present"),
                group
                    .seller_scope
                    .clone()
                    .expect("seller scope should be present"),
                group.selected_shipping_option_id,
            )
        })
        .collect::<Vec<_>>();
    assert!(delivery_groups.contains(&(
        String::from("default"),
        seller_a_id.to_string(),
        String::from("seller-a"),
        Some(seller_a_option_id),
    )));
    assert!(delivery_groups.contains(&(
        String::from("default"),
        seller_b_id.to_string(),
        String::from("seller-b"),
        Some(seller_b_option_id),
    )));
}

#[tokio::test]
async fn legacy_seller_scope_still_splits_delivery_groups_without_seller_id() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let cart = service
        .create_cart(tenant_id, create_cart_input())
        .await
        .unwrap();
    let cart = service
        .add_line_item(
            tenant_id,
            cart.id,
            AddCartLineItemInput {
                metadata: serde_json::json!({
                    "seller": {
                        "scope": "legacy-seller-a"
                    }
                }),
                ..line_item_input()
            },
        )
        .await
        .unwrap();
    let cart = service
        .add_line_item(
            tenant_id,
            cart.id,
            AddCartLineItemInput {
                sku: Some("SKU-CART-LEGACY-2".to_string()),
                metadata: serde_json::json!({
                    "seller": {
                        "scope": "legacy-seller-b"
                    }
                }),
                ..line_item_input()
            },
        )
        .await
        .unwrap();

    assert_eq!(cart.delivery_groups.len(), 2);
    assert!(cart
        .delivery_groups
        .iter()
        .all(|group| group.seller_id.is_none()));
    assert!(cart
        .delivery_groups
        .iter()
        .any(|group| { group.seller_scope.as_deref() == Some("legacy-seller-a") }));
    assert!(cart
        .delivery_groups
        .iter()
        .any(|group| { group.seller_scope.as_deref() == Some("legacy-seller-b") }));
}
