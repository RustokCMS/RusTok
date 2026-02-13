// Comprehensive unit tests for CatalogService  
// These tests verify product CRUD, variants, translations,
// pricing, and publishing workflows.

use rustok_commerce::dto::{
    CreateProductInput, ProductTranslationInput, ProductVariantInput, UpdateProductInput,
};
use rustok_commerce::entities::product::ProductStatus;
use rustok_commerce::services::CatalogService;
use rustok_commerce::CommerceError;
use rustok_test_utils::{db::setup_test_db, mock_transactional_event_bus, helpers::unique_slug};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

async fn setup() -> (DatabaseConnection, CatalogService) {
    let db = setup_test_db().await;
    let event_bus = mock_transactional_event_bus();
    let service = CatalogService::new(db.clone(), event_bus);
    (db, service)
}

fn create_test_product_input() -> CreateProductInput {
    CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Test Product".to_string(),
            description: Some("A great test product".to_string()),
            handle: Some(unique_slug("test-product")),
        }],
        variants: vec![ProductVariantInput {
            sku: format!(
                "SKU-{}",
                Uuid::new_v4().to_string().split('-').next().unwrap()
            ),
            title: Some("Default".to_string()),
            price: 99.99,
            compare_at_price: Some(149.99),
            cost: Some(50.00),
            barcode: None,
            requires_shipping: true,
            taxable: true,
            weight: Some(1.5),
            weight_unit: Some("kg".to_string()),
        }],
        vendor: Some("Test Vendor".to_string()),
        product_type: Some("Physical".to_string()),
        publish: false,
        metadata: serde_json::json!({}),
    }
}

// =============================================================================
// Basic CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_success() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let input = create_test_product_input();

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.translations.len(), 1);
    assert_eq!(product.translations[0].title, "Test Product");
    assert_eq!(product.variants.len(), 1);
    assert_eq!(product.status, ProductStatus::Draft);
}

#[tokio::test]
async fn test_create_product_requires_translations() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.translations = vec![];

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CommerceError::Validation(msg) => {
            assert!(msg.contains("translation"));
        }
        _ => panic!("Expected validation error"),
    }
}

#[tokio::test]
async fn test_create_product_requires_variants() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants = vec![];

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CommerceError::NoVariants => {}
        _ => panic!("Expected NoVariants error"),
    }
}

#[tokio::test]
async fn test_get_product_success() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let created = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let result = service.get_product(created.id).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.id, created.id);
    assert_eq!(product.translations[0].title, "Test Product");
}

#[tokio::test]
async fn test_get_nonexistent_product() {
    let (_db, service) = setup().await;
    let fake_id = Uuid::new_v4();

    let result = service.get_product(fake_id).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CommerceError::NotFound(_) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_update_product_success() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let update_input = UpdateProductInput {
        translations: Some(vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Updated Product".to_string(),
            description: Some("Updated description".to_string()),
            handle: None,
        }]),
        vendor: Some("Updated Vendor".to_string()),
        product_type: Some("Digital".to_string()),
        status: Some(ProductStatus::Active),
        metadata: None,
    };

    let result = service
        .update_product(product.id, actor_id, update_input)
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.translations[0].title, "Updated Product");
    assert_eq!(updated.vendor, Some("Updated Vendor".to_string()));
    assert_eq!(updated.product_type, Some("Digital".to_string()));
    assert_eq!(updated.status, ProductStatus::Active);
}

#[tokio::test]
async fn test_delete_product_success() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let result = service.delete_product(product.id, actor_id).await;
    assert!(result.is_ok());

    let get_result = service.get_product(product.id).await;
    assert!(get_result.is_err());
}

// =============================================================================
// Multi-Language Translation Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_with_multiple_translations() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.translations.push(ProductTranslationInput {
        locale: "ru".to_string(),
        title: "Тестовый продукт".to_string(),
        description: Some("Отличный тестовый продукт".to_string()),
        handle: Some(unique_slug("test-product-ru")),
    });
    input.translations.push(ProductTranslationInput {
        locale: "de".to_string(),
        title: "Testprodukt".to_string(),
        description: Some("Ein großartiges Testprodukt".to_string()),
        handle: Some(unique_slug("test-product-de")),
    });

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.translations.len(), 3);

    let en_trans = product.translations.iter().find(|t| t.locale == "en");
    let ru_trans = product.translations.iter().find(|t| t.locale == "ru");
    let de_trans = product.translations.iter().find(|t| t.locale == "de");

    assert!(en_trans.is_some());
    assert!(ru_trans.is_some());
    assert!(de_trans.is_some());
    assert_eq!(en_trans.unwrap().title, "Test Product");
    assert_eq!(ru_trans.unwrap().title, "Тестовый продукт");
    assert_eq!(de_trans.unwrap().title, "Testprodukt");
}

// =============================================================================
// Product Variant Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_with_multiple_variants() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants.push(ProductVariantInput {
        sku: format!(
            "SKU-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        title: Some("Small".to_string()),
        price: 79.99,
        compare_at_price: None,
        cost: Some(40.00),
        barcode: None,
        requires_shipping: true,
        taxable: true,
        weight: Some(1.0),
        weight_unit: Some("kg".to_string()),
    });
    input.variants.push(ProductVariantInput {
        sku: format!(
            "SKU-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        title: Some("Large".to_string()),
        price: 119.99,
        compare_at_price: Some(169.99),
        cost: Some(60.00),
        barcode: None,
        requires_shipping: true,
        taxable: true,
        weight: Some(2.0),
        weight_unit: Some("kg".to_string()),
    });

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.variants.len(), 3);

    let small = product
        .variants
        .iter()
        .find(|v| v.title.as_deref() == Some("Small"));
    let large = product
        .variants
        .iter()
        .find(|v| v.title.as_deref() == Some("Large"));

    assert!(small.is_some());
    assert!(large.is_some());
    assert_eq!(small.unwrap().price, 79.99);
    assert_eq!(large.unwrap().price, 119.99);
}

#[tokio::test]
async fn test_variant_pricing() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants[0].price = 99.99;
    input.variants[0].compare_at_price = Some(149.99);
    input.variants[0].cost = Some(50.00);

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    let variant = &product.variants[0];

    assert_eq!(variant.price, 99.99);
    assert_eq!(variant.compare_at_price, Some(149.99));
    assert_eq!(variant.cost, Some(50.00));

    let discount = 149.99 - 99.99;
    let discount_percent = (discount / 149.99) * 100.0;
    assert!((discount_percent - 33.34).abs() < 0.1);
}

#[tokio::test]
async fn test_variant_shipping_properties() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants[0].requires_shipping = true;
    input.variants[0].weight = Some(2.5);
    input.variants[0].weight_unit = Some("kg".to_string());
    input.variants[0].taxable = true;

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    let variant = &product.variants[0];

    assert_eq!(variant.requires_shipping, true);
    assert_eq!(variant.weight, Some(2.5));
    assert_eq!(variant.weight_unit, Some("kg".to_string()));
    assert_eq!(variant.taxable, true);
}

#[tokio::test]
async fn test_variant_with_barcode() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants[0].barcode = Some("1234567890123".to_string());

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    let variant = &product.variants[0];

    assert_eq!(variant.barcode, Some("1234567890123".to_string()));
}

// =============================================================================
// Publishing & Status Tests
// =============================================================================

#[tokio::test]
async fn test_create_product_with_publish() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.publish = true;

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.status, ProductStatus::Active);
    assert!(product.published_at.is_some());
}

#[tokio::test]
async fn test_publish_product() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.publish = false;
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    assert_eq!(product.status, ProductStatus::Draft);
    assert!(product.published_at.is_none());

    let result = service.publish_product(product.id, actor_id).await;

    assert!(result.is_ok());
    let published = result.unwrap();
    assert_eq!(published.status, ProductStatus::Active);
    assert!(published.published_at.is_some());
}

#[tokio::test]
async fn test_unpublish_product() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.publish = true;
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    assert_eq!(product.status, ProductStatus::Active);

    let result = service.unpublish_product(product.id, actor_id).await;

    assert!(result.is_ok());
    let unpublished = result.unwrap();
    assert_eq!(unpublished.status, ProductStatus::Draft);
}

// =============================================================================
// Metadata Tests
// =============================================================================

#[tokio::test]
async fn test_product_with_metadata() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.metadata = serde_json::json!({
        "featured": true,
        "tags": ["new", "sale", "popular"],
        "color": "blue",
        "size_chart": "standard"
    });

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.metadata["featured"], true);
    assert!(product.metadata["tags"].is_array());
    assert_eq!(product.metadata["color"], "blue");
    assert_eq!(product.metadata["size_chart"], "standard");
}

#[tokio::test]
async fn test_update_product_metadata() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let update_input = UpdateProductInput {
        translations: None,
        vendor: None,
        product_type: None,
        status: None,
        metadata: Some(serde_json::json!({
            "featured": true,
            "priority": "high",
            "badge": "bestseller"
        })),
    };

    let result = service
        .update_product(product.id, actor_id, update_input)
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.metadata["featured"], true);
    assert_eq!(updated.metadata["priority"], "high");
    assert_eq!(updated.metadata["badge"], "bestseller");
}

// =============================================================================
// Vendor & Product Type Tests
// =============================================================================

#[tokio::test]
async fn test_product_with_vendor_and_type() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.vendor = Some("Acme Corp".to_string());
    input.product_type = Some("Electronics".to_string());

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.vendor, Some("Acme Corp".to_string()));
    assert_eq!(product.product_type, Some("Electronics".to_string()));
}

#[tokio::test]
async fn test_update_vendor() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let update_input = UpdateProductInput {
        translations: None,
        vendor: Some("New Vendor Inc".to_string()),
        product_type: None,
        status: None,
        metadata: None,
    };

    let result = service
        .update_product(product.id, actor_id, update_input)
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.vendor, Some("New Vendor Inc".to_string()));
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
async fn test_update_nonexistent_product() {
    let (_db, service) = setup().await;
    let actor_id = Uuid::new_v4();
    let fake_id = Uuid::new_v4();

    let update_input = UpdateProductInput {
        translations: Some(vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Updated".to_string(),
            description: None,
            handle: None,
        }]),
        vendor: None,
        product_type: None,
        status: None,
        metadata: None,
    };

    let result = service.update_product(fake_id, actor_id, update_input).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        CommerceError::NotFound(_) => {}
        _ => panic!("Expected NotFound error"),
    }
}

#[tokio::test]
async fn test_delete_nonexistent_product() {
    let (_db, service) = setup().await;
    let actor_id = Uuid::new_v4();
    let fake_id = Uuid::new_v4();

    let result = service.delete_product(fake_id, actor_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_publish_nonexistent_product() {
    let (_db, service) = setup().await;
    let actor_id = Uuid::new_v4();
    let fake_id = Uuid::new_v4();

    let result = service.publish_product(fake_id, actor_id).await;

    assert!(result.is_err());
}

// =============================================================================
// SKU & Handle Uniqueness Tests
// =============================================================================

#[tokio::test]
async fn test_unique_skus_per_product() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let skus: Vec<String> = product.variants.iter().map(|v| v.sku.clone()).collect();
    let unique_skus: std::collections::HashSet<_> = skus.iter().collect();

    assert_eq!(skus.len(), unique_skus.len(), "All SKUs should be unique");
}

// =============================================================================
// Additional Edge Case Tests
// =============================================================================

#[tokio::test]
async fn test_product_with_empty_vendor() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.vendor = None;

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.vendor, None);
}

#[tokio::test]
async fn test_variant_digital_product() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.product_type = Some("Digital".to_string());
    input.variants[0].requires_shipping = false;
    input.variants[0].weight = None;
    input.variants[0].weight_unit = None;

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.product_type, Some("Digital".to_string()));
    assert_eq!(product.variants[0].requires_shipping, false);
    assert_eq!(product.variants[0].weight, None);
}

#[tokio::test]
async fn test_create_archived_product() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let input = create_test_product_input();
    let product = service
        .create_product(tenant_id, actor_id, input)
        .await
        .unwrap();

    let update_input = UpdateProductInput {
        translations: None,
        vendor: None,
        product_type: None,
        status: Some(ProductStatus::Archived),
        metadata: None,
    };

    let result = service
        .update_product(product.id, actor_id, update_input)
        .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.status, ProductStatus::Archived);
}

#[tokio::test]
async fn test_variant_profit_margin() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants[0].price = 100.00;
    input.variants[0].cost = Some(40.00);

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    let variant = &product.variants[0];

    let profit = variant.price - variant.cost.unwrap();
    let margin = (profit / variant.price) * 100.0;

    assert_eq!(profit, 60.00);
    assert_eq!(margin, 60.0);
}

#[tokio::test]
async fn test_multiple_variants_different_prices() {
    let (_db, service) = setup().await;
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();

    let mut input = create_test_product_input();
    input.variants[0].price = 50.00;
    input.variants.push(ProductVariantInput {
        sku: format!(
            "SKU-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        ),
        title: Some("Premium".to_string()),
        price: 150.00,
        compare_at_price: None,
        cost: Some(80.00),
        barcode: None,
        requires_shipping: true,
        taxable: true,
        weight: Some(2.0),
        weight_unit: Some("kg".to_string()),
    });

    let result = service.create_product(tenant_id, actor_id, input).await;

    assert!(result.is_ok());
    let product = result.unwrap();
    assert_eq!(product.variants.len(), 2);

    let prices: Vec<f64> = product.variants.iter().map(|v| v.price).collect();
    assert_eq!(prices[0], 50.00);
    assert_eq!(prices[1], 150.00);
}
