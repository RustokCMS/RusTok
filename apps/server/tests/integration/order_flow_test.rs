//! # Order Flow Integration Tests
//!
//! Tests the complete order lifecycle:
//! 1. Create product
//! 2. Create order
//! 3. Submit order
//! 4. Process payment
//! 5. Verify order status changes
//! 6. Verify events are emitted

use rustok_test_utils::*;
use rustok_commerce::dto::{
    CreateProductInput, CreateOrderInput, OrderItemInput,
    ProcessPaymentInput, PaymentMethod
};
use rustok_commerce::entities::OrderStatus;
use std::time::Duration;

/// Test the complete order flow from product creation to payment
#[tokio::test]
async fn test_complete_order_flow() {
    let app = spawn_test_app().await;
    
    // 1. Create a product
    let product = app
        .create_product(CreateProductInput {
            sku: "TEST-ORDER-001".to_string(),
            title: "Test Order Product".to_string(),
            description: Some("Product for order flow test".to_string()),
            price: 1500, // $15.00
            currency: "USD".to_string(),
            inventory: 100,
            status: None,
        })
        .await
        .expect("Failed to create product");
    
    assert_eq!(product.sku, "TEST-ORDER-001");
    assert_eq!(product.price, 1500);
    assert_eq!(product.inventory, 100);
    
    // 2. Create an order with 2 items
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![
                OrderItemInput {
                    product_id: product.id,
                    quantity: 2,
                    price: Some(1500),
                }
            ],
        })
        .await
        .expect("Failed to create order");
    
    assert_eq!(order.status, OrderStatus::Draft.to_string());
    assert_eq!(order.total, 3000); // 2 * $15.00
    assert_eq!(order.customer_id, test_customer_id());
    
    // 3. Submit the order
    let submitted_order = app
        .submit_order(order.id)
        .await
        .expect("Failed to submit order");
    
    assert_eq!(submitted_order.status, OrderStatus::PendingPayment.to_string());
    
    // 4. Process payment
    let payment = app
        .process_payment(order.id, ProcessPaymentInput {
            method: PaymentMethod::Card,
            amount: 3000,
            currency: "USD".to_string(),
            card_token: "tok_test_visa".to_string(),
            metadata: None,
        })
        .await
        .expect("Failed to process payment");
    
    assert!(payment.success, "Payment should succeed");
    assert_eq!(payment.amount, 3000);
    assert_eq!(payment.currency, "USD");
    
    // 5. Verify order is now paid
    let paid_order = app
        .get_order(order.id)
        .await
        .expect("Failed to get order");
    
    assert_eq!(
        paid_order.status,
        OrderStatus::Paid.to_string(),
        "Order should be in Paid status after payment"
    );
    
    // 6. Verify events were emitted
    let events = app.get_events_for_order(order.id).await;
    
    assert!(
        events.iter().any(|e| matches!(e, rustok_core::events::types::DomainEvent::OrderCreated { .. })),
        "OrderCreated event should be emitted"
    );
    
    assert!(
        events.iter().any(|e| matches!(e, rustok_core::events::types::DomainEvent::OrderPaid { .. })),
        "OrderPaid event should be emitted"
    );
    
    // 7. Verify product inventory was updated
    let updated_product = app
        .get_product(product.id)
        .await
        .expect("Failed to get product");
    
    assert_eq!(
        updated_product.inventory,
        98, // 100 - 2 items
        "Product inventory should be decreased after order"
    );
}

/// Test order creation with multiple items
#[tokio::test]
async fn test_order_with_multiple_items() {
    let app = spawn_test_app().await;
    
    // Create multiple products
    let product1 = app
        .create_product(test_product_input_with_sku("MULTI-001"))
        .await
        .expect("Failed to create product 1");
    
    let product2 = app
        .create_product(test_product_input_with_sku("MULTI-002"))
        .await
        .expect("Failed to create product 2");
    
    let product3 = app
        .create_product(test_product_input_with_sku("MULTI-003"))
        .await
        .expect("Failed to create product 3");
    
    // Create order with multiple items
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![
                OrderItemInput {
                    product_id: product1.id,
                    quantity: 1,
                    price: Some(1000),
                },
                OrderItemInput {
                    product_id: product2.id,
                    quantity: 2,
                    price: Some(2000),
                },
                OrderItemInput {
                    product_id: product3.id,
                    quantity: 1,
                    price: Some(1500),
                },
            ],
        })
        .await
        .expect("Failed to create order");
    
    // Verify order total
    let expected_total = 1000 + (2 * 2000) + 1500; // 6500
    assert_eq!(order.total, expected_total);
    assert_eq!(order.items.len(), 3);
}

/// Test order validation (negative quantity, missing product, etc.)
#[tokio::test]
async fn test_order_validation() {
    let app = spawn_test_app().await;
    
    // Test with non-existent product
    let result = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![
                OrderItemInput {
                    product_id: test_uuid(), // Non-existent
                    quantity: 1,
                    price: Some(1000),
                }
            ],
        })
        .await;
    
    assert!(result.is_err(), "Order with non-existent product should fail");
    
    // Test with negative quantity
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let result = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![
                OrderItemInput {
                    product_id: product.id,
                    quantity: -1, // Invalid
                    price: Some(1000),
                }
            ],
        })
        .await;
    
    assert!(result.is_err(), "Order with negative quantity should fail");
}

/// Test order payment failure scenarios
#[tokio::test]
async fn test_order_payment_failure() {
    let app = spawn_test_app().await;
    
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: 1,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order");
    
    let order = app
        .submit_order(order.id)
        .await
        .expect("Failed to submit order");
    
    // Test with invalid card token
    let result = app
        .process_payment(order.id, ProcessPaymentInput {
            method: PaymentMethod::Card,
            amount: 1000,
            currency: "USD".to_string(),
            card_token: "tok_fail".to_string(), // Invalid token
            metadata: None,
        })
        .await;
    
    assert!(result.is_err(), "Payment with invalid token should fail");
    
    // Verify order is still pending payment
    let failed_order = app
        .get_order(order.id)
        .await
        .expect("Failed to get order");
    
    assert_eq!(
        failed_order.status,
        OrderStatus::PendingPayment.to_string(),
        "Order should remain in PendingPayment after failed payment"
    );
}

/// Test order retrieval and search
#[tokio::test]
async fn test_order_retrieval_and_search() {
    let app = spawn_test_app().await;
    
    // Create multiple orders
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let order1 = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: 1,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order 1");
    
    let order2 = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: 2,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order 2");
    
    // Retrieve individual orders
    let retrieved1 = app
        .get_order(order1.id)
        .await
        .expect("Failed to retrieve order 1");
    
    assert_eq!(retrieved1.id, order1.id);
    assert_eq!(retrieved1.total, 1000);
    
    let retrieved2 = app
        .get_order(order2.id)
        .await
        .expect("Failed to retrieve order 2");
    
    assert_eq!(retrieved2.id, order2.id);
    assert_eq!(retrieved2.total, 2000);
    
    // Search orders by product SKU
    let results = app
        .search_orders(&product.sku)
        .await
        .expect("Failed to search orders");
    
    assert!(
        results.len() >= 2,
        "Should find at least 2 orders with product SKU"
    );
    
    assert!(
        results.iter().any(|o| o.id == order1.id),
        "Should find order1 in search results"
    );
    
    assert!(
        results.iter().any(|o| o.id == order2.id),
        "Should find order2 in search results"
    );
}

/// Test order lifecycle state transitions
#[tokio::test]
async fn test_order_lifecycle_state_transitions() {
    let app = spawn_test_app().await;
    
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: 1,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order");
    
    // Verify initial state: Draft
    assert_eq!(order.status, OrderStatus::Draft.to_string());
    
    // Submit order: Draft -> PendingPayment
    let order = app
        .submit_order(order.id)
        .await
        .expect("Failed to submit order");
    assert_eq!(order.status, OrderStatus::PendingPayment.to_string());
    
    // Process payment: PendingPayment -> Paid
    let _payment = app
        .process_payment(order.id, test_payment_input())
        .await
        .expect("Failed to process payment");
    
    let order = app
        .get_order(order.id)
        .await
        .expect("Failed to get order");
    
    assert_eq!(order.status, OrderStatus::Paid.to_string());
    
    // Verify events were emitted for each state transition
    let events = app.get_events_for_order(order.id).await;
    
    let event_types: Vec<_> = events
        .iter()
        .map(|e| std::mem::discriminant(e))
        .collect();
    
    assert!(
        event_types.iter().any(|t| {
            t == &std::mem::discriminant(&rustok_core::events::types::DomainEvent::OrderCreated {
                order_id: test_uuid(),
                customer_id: test_uuid(),
                total: 0,
                currency: "".to_string(),
                tenant_id: "".to_string(),
            })
        }),
        "OrderCreated event should be present"
    );
}

/// Helper function to wait for events to propagate
async fn wait_for_events(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}
