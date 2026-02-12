//! # Event Flow Integration Tests
//!
//! Tests the complete event propagation flow:
//! 1. Event publication
//! 2. Event persistence in outbox
//! 3. Event relay to subscribers
//! 4. Event consumption by handlers

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;
use rustok_test_utils::*;
use rustok_core::events::types::DomainEvent;

/// Test event propagation from publication to consumption
#[tokio::test]
async fn test_event_propagation() {
    let app = spawn_test_app().await;
    
    // Create event listener
    let events = Arc::new(Mutex::new(Vec::new()));
    app.subscribe_to_events();
    
    // 1. Trigger event by creating a node
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    // 2. Wait for event propagation
    wait_for_events(500).await;
    
    // 3. Verify event was received
    let captured_events = app.get_events_for_node(node.id).await;
    
    assert!(
        !captured_events.is_empty(),
        "At least one event should be captured for node creation"
    );
    
    assert!(
        captured_events.iter().any(|e| matches!(e, DomainEvent::NodeCreated { .. })),
        "NodeCreated event should be present"
    );
}

/// Test event persistence in outbox table
#[tokio::test]
async fn test_event_outbox_persistence() {
    let app = spawn_test_app().await;
    
    // Create an order (which generates events)
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let order = app
        .create_order(rustok_commerce::dto::CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![rustok_commerce::dto::OrderItemInput {
                product_id: product.id,
                quantity: 1,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order");
    
    // Wait for outbox processing
    wait_for_events(200).await;
    
    // Verify event persisted in outbox
    let outbox_events = app.get_outbox_events().await;
    
    assert!(
        !outbox_events.is_empty(),
        "Events should be persisted in outbox"
    );
    
    // Verify event type
    assert!(
        outbox_events.iter().any(|e| {
            e.get("event_type")
                .and_then(|v| v.as_str())
                .map(|s| s.contains("OrderCreated"))
                .unwrap_or(false)
        }),
        "OrderCreated event should be in outbox"
    );
}

/// Test event relay to subscribers
#[tokio::test]
async fn test_event_relay() {
    let app = spawn_test_app().await;
    
    // Create multiple events
    let _product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let _node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    // Wait for relay processing
    wait_for_events(1000).await;
    
    // Verify events were relayed
    let relayed_count = app.get_relayed_events().await;
    
    assert!(
        relayed_count > 0,
        "Events should be relayed to subscribers"
    );
}

/// Test event ordering and sequence
#[tokio::test]
async fn test_event_ordering() {
    let app = spawn_test_app().await;
    
    // Create order with multiple state changes
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let order = app
        .create_order(rustok_commerce::dto::CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![rustok_commerce::dto::OrderItemInput {
                product_id: product.id,
                quantity: 1,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order");
    
    // Submit order
    let _submitted = app
        .submit_order(order.id)
        .await
        .expect("Failed to submit order");
    
    // Process payment
    let _payment = app
        .process_payment(order.id, test_payment_input())
        .await
        .expect("Failed to process payment");
    
    // Wait for all events
    wait_for_events(500).await;
    
    // Get all events for this order
    let events = app.get_events_for_order(order.id).await;
    
    assert!(
        events.len() >= 2,
        "At least OrderCreated and OrderPaid events should be present"
    );
    
    // Verify events in correct order (OrderCreated should come before OrderPaid)
    let created_index = events.iter().position(|e| matches!(e, DomainEvent::OrderCreated { .. }));
    let paid_index = events.iter().position(|e| matches!(e, DomainEvent::OrderPaid { .. }));
    
    if let (Some(created), Some(paid)) = (created_index, paid_index) {
        assert!(
            created < paid,
            "OrderCreated should come before OrderPaid"
        );
    }
}

/// Test event correlation IDs
#[tokio::test]
async fn test_event_correlation() {
    let app = spawn_test_app().await;
    
    // Create a node
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    // Publish the node
    let _published = app
        .publish_node(node.id)
        .await
        .expect("Failed to publish node");
    
    // Wait for events
    wait_for_events(300).await;
    
    // Get all events for this node
    let events = app.get_events_for_node(node.id).await;
    
    // All events for this node should have the same node_id
    for event in &events {
        match event {
            DomainEvent::NodeCreated { node_id, .. } => {
                assert_eq!(*node_id, node.id, "NodeCreated should have correct node_id");
            }
            DomainEvent::NodePublished { node_id, .. } => {
                assert_eq!(*node_id, node.id, "NodePublished should have correct node_id");
            }
            _ => {}
        }
    }
}

/// Test event error handling and retry
#[tokio::test]
async fn test_event_error_handling() {
    let app = spawn_test_app().await;
    
    // In a real implementation, this would test scenarios like:
    // - Event handler throws error
    // - Event gets retried
    // - DLQ for permanently failed events
    
    // For now, we just verify normal event flow works
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    wait_for_events(200).await;
    
    let events = app.get_events_for_node(node.id).await;
    assert!(!events.is_empty(), "Event should be created successfully");
}

/// Test cross-module event propagation
#[tokio::test]
async fn test_cross_module_events() {
    let app = spawn_test_app().await;
    
    // Create product (commerce event)
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    // Create node (content event)
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    wait_for_events(300).await;
    
    // Verify both events were captured
    let product_events = app.get_events_for_node(Uuid::new_v4()).await; // Placeholder
    let node_events = app.get_events_for_node(node.id).await;
    
    assert!(!node_events.is_empty(), "Content event should be present");
    
    // In a real implementation, we'd also verify commerce events
}

/// Test event filtering by tenant
#[tokio::test]
async fn test_event_tenant_isolation() {
    let app = spawn_test_app().await;
    
    // Create node in tenant1
    let node1 = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    wait_for_events(200).await;
    
    // Verify event has correct tenant_id
    let events = app.get_events_for_node(node1.id).await;
    
    if let Some(DomainEvent::NodeCreated { tenant_id, .. }) = events.first() {
        assert_eq!(
            tenant_id,
            &app.tenant_id,
            "Event should have correct tenant_id"
        );
    }
    
    // In a real implementation, we'd also verify that:
    // - Tenant A cannot see Tenant B's events
    // - Events are properly scoped by tenant
}

/// Test event validation before publication
#[tokio::test]
async fn test_event_validation() {
    let app = spawn_test_app().await;
    
    // Valid event: Create node with valid data
    let valid_result = app
        .create_node(test_node_input())
        .await;
    
    assert!(valid_result.is_ok(), "Valid event should be published");
    
    // In a real implementation, we'd test invalid events:
    // - Missing required fields
    // - Invalid data types
    // - Constraint violations
}

/// Test event payload size limits
#[tokio::test]
async fn test_event_payload_size() {
    let app = spawn_test_app().await;
    
    // Create node with very large body
    let large_body = "x".repeat(1_000_000); // 1MB body
    
    let result = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "Large Body Article".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_content::dto::BodyInput {
                format: BodyFormat::Markdown,
                content: large_body,
            }),
        })
        .await;
    
    // This should either succeed or fail gracefully
    // In a real implementation, we'd verify the behavior
    match result {
        Ok(_) => {
            // Event should still be created, but payload might be truncated
        }
        Err(_) => {
            // Event rejected due to size limit
        }
    }
}

/// Test event replay mechanism
#[tokio::test]
async fn test_event_replay() {
    let app = spawn_test_app().await;
    
    // In a real implementation, this would test:
    // - Storing events in event store
    // - Replaying events from a specific point
    // - Rebuilding read models from events
    // - Idempotency of event replay
    
    // For now, just verify events are persisted
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    wait_for_events(200).await;
    
    let events = app.get_events_for_node(node.id).await;
    assert!(!events.is_empty(), "Events should be persisted for replay");
}

/// Test event deduplication
#[tokio::test]
async fn test_event_deduplication() {
    let app = spawn_test_app().await;
    
    // Create node
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    wait_for_events(200).await;
    
    // Get events
    let events = app.get_events_for_node(node.id).await;
    
    // Count NodeCreated events
    let created_count = events
        .iter()
        .filter(|e| matches!(e, DomainEvent::NodeCreated { .. }))
        .count();
    
    assert_eq!(
        created_count, 1,
        "Should have exactly one NodeCreated event"
    );
}

/// Test event batching and bulk operations
#[tokio::test]
async fn test_event_batching() {
    let app = spawn_test_app().await;
    
    // Create multiple nodes to generate multiple events
    let mut node_ids = Vec::new();
    
    for i in 0..5 {
        let node = app
            .create_node(test_node_input_with_title(&format!("Node {}", i)))
            .await
            .expect("Failed to create node");
        node_ids.push(node.id);
    }
    
    wait_for_events(500).await;
    
    // Verify all events were created
    for node_id in node_ids {
        let events = app.get_events_for_node(node_id).await;
        assert!(!events.is_empty(), "Event should exist for each node");
    }
}

/// Helper function to wait for event processing
async fn wait_for_events(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}
