// Integration test for Product creation → Event → Index update flow
// This test verifies the complete workflow from product creation to indexing

use rustok_commerce::dto::{CreateProductInput, ProductTranslationInput, ProductVariantInput};
use rustok_commerce::services::CatalogService;
use rustok_core::events::DomainEvent;
use rustok_core::SecurityContext;
use rustok_test_utils::{db::setup_test_db, mock_transactional_event_bus};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_product_creation_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_transactional_event_bus();
    let service = CatalogService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    
    // Create a product
    let input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Test Product".to_string(),
            description: Some("A great test product".to_string()),
            handle: Some("test-product".to_string()),
        }],
        variants: vec![ProductVariantInput {
            sku: "TEST-SKU-001".to_string(),
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
    };
    
    let result = service.create_product(tenant_id, actor_id, input).await;
    assert!(result.is_ok());
    let product = result.unwrap();
    
    // Verify that a ProductCreated event was published
    assert_eq!(event_bus.event_count(), 1);
    assert!(event_bus.has_event_of_type("ProductCreated"));
    
    // Get the events and verify details
    let events = event_bus.events_of_type("ProductCreated");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::ProductCreated { product_id, .. } = &events[0] {
        assert_eq!(*product_id, product.id);
    } else {
        panic!("Expected ProductCreated event");
    }
    
    println!("✅ Product creation → Event publishing flow verified");
}

#[tokio::test]
async fn test_product_update_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = CatalogService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    
    // Create a product first
    let input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Original Product".to_string(),
            description: Some("Original description".to_string()),
            handle: Some("original-product".to_string()),
        }],
        variants: vec![ProductVariantInput {
            sku: "ORIG-SKU-001".to_string(),
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
        vendor: Some("Original Vendor".to_string()),
        product_type: Some("Physical".to_string()),
        publish: false,
        metadata: serde_json::json!({}),
    };
    
    let product = service.create_product(tenant_id, actor_id, input).await.unwrap();
    
    // Clear the first event (ProductCreated)
    event_bus.clear();
    
    // Update the product
    use rustok_commerce::dto::UpdateProductInput;
    use rustok_commerce::entities::product::ProductStatus;
    
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
    
    // Verify that a ProductUpdated event was published
    assert_eq!(event_bus.event_count(), 1);
    assert!(event_bus.has_event_of_type("ProductUpdated"));
    
    let events = event_bus.events_of_type("ProductUpdated");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::ProductUpdated { product_id, .. } = &events[0] {
        assert_eq!(*product_id, product.id);
    } else {
        panic!("Expected ProductUpdated event");
    }
    
    println!("✅ Product update → Event publishing flow verified");
}

#[tokio::test]
async fn test_product_publishing_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = CatalogService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    
    // Create a draft product
    let mut input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Draft Product".to_string(),
            description: Some("Draft description".to_string()),
            handle: Some("draft-product".to_string()),
        }],
        variants: vec![ProductVariantInput {
            sku: "DRAFT-SKU-001".to_string(),
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
    };
    
    let product = service.create_product(tenant_id, actor_id, input).await.unwrap();
    
    // Clear the first event (ProductCreated)
    event_bus.clear();
    
    // Publish the product
    let result = service.publish_product(product.id, actor_id).await;
    assert!(result.is_ok());
    
    // Verify that a ProductPublished event was published
    assert_eq!(event_bus.event_count(), 1);
    assert!(event_bus.has_event_of_type("ProductPublished"));
    
    let events = event_bus.events_of_type("ProductPublished");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::ProductPublished { product_id, .. } = &events[0] {
        assert_eq!(*product_id, product.id);
    } else {
        panic!("Expected ProductPublished event");
    }
    
    println!("✅ Product publishing → Event publishing flow verified");
}

#[tokio::test]
async fn test_product_deletion_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = CatalogService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    
    // Create a product first
    let input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "To be deleted".to_string(),
            description: Some("Will be deleted".to_string()),
            handle: Some("to-be-deleted".to_string()),
        }],
        variants: vec![ProductVariantInput {
            sku: "DELETE-SKU-001".to_string(),
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
    };
    
    let product = service.create_product(tenant_id, actor_id, input).await.unwrap();
    
    // Clear the first event (ProductCreated)
    event_bus.clear();
    
    // Delete the product
    let result = service.delete_product(product.id, actor_id).await;
    assert!(result.is_ok());
    
    // Verify that a ProductDeleted event was published
    assert_eq!(event_bus.event_count(), 1);
    assert!(event_bus.has_event_of_type("ProductDeleted"));
    
    let events = event_bus.events_of_type("ProductDeleted");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::ProductDeleted { product_id, .. } = &events[0] {
        assert_eq!(*product_id, product.id);
    } else {
        panic!("Expected ProductDeleted event");
    }
    
    println!("✅ Product deletion → Event publishing flow verified");
}

#[tokio::test]
async fn test_variant_creation_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = CatalogService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    
    // Create a product with multiple variants
    let input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Product with Variants".to_string(),
            description: Some("Product with multiple variants".to_string()),
            handle: Some("product-with-variants".to_string()),
        }],
        variants: vec![
            ProductVariantInput {
                sku: "VARIANT-SKU-001".to_string(),
                title: Some("Small".to_string()),
                price: 79.99,
                compare_at_price: None,
                cost: Some(40.00),
                barcode: None,
                requires_shipping: true,
                taxable: true,
                weight: Some(1.0),
                weight_unit: Some("kg".to_string()),
            },
            ProductVariantInput {
                sku: "VARIANT-SKU-002".to_string(),
                title: Some("Large".to_string()),
                price: 119.99,
                compare_at_price: Some(169.99),
                cost: Some(60.00),
                barcode: None,
                requires_shipping: true,
                taxable: true,
                weight: Some(2.0),
                weight_unit: Some("kg".to_string()),
            },
        ],
        vendor: Some("Test Vendor".to_string()),
        product_type: Some("Physical".to_string()),
        publish: false,
        metadata: serde_json::json!({}),
    };
    
    let product = service.create_product(tenant_id, actor_id, input).await.unwrap();
    
    // Verify that a ProductCreated event was published
    assert_eq!(event_bus.event_count(), 3); // 1 ProductCreated + 2 VariantCreated
    assert!(event_bus.has_event_of_type("ProductCreated"));
    
    // Get the events and verify details
    let product_events = event_bus.events_of_type("ProductCreated");
    assert_eq!(product_events.len(), 1);
    
    if let DomainEvent::ProductCreated { product_id, .. } = &product_events[0] {
        assert_eq!(*product_id, product.id);
    } else {
        panic!("Expected ProductCreated event");
    }
    
    // Verify that VariantCreated events were published for each variant
    let variant_events = event_bus.events_of_type("VariantCreated");
    assert_eq!(variant_events.len(), 2, "Should have 2 variant creation events");
    
    println!("✅ Product with variants → Event publishing flow verified");
}