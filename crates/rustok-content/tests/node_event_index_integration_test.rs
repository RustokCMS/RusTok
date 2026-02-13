// Integration test for Node creation → Event flow
// This test verifies the complete workflow from content creation to event publishing

use rustok_content::dto::{BodyInput, CreateNodeInput, NodeTranslationInput};
use rustok_content::services::NodeService;
use rustok_core::events::DomainEvent;
use rustok_core::{SecurityContext, UserRole};
use rustok_outbox::OutboxTransport;
use rustok_test_utils::{db::setup_test_db, events::mock_event_bus};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::test]
async fn test_node_creation_triggers_event_and_indexing() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = NodeService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let security = SecurityContext::new(UserRole::Admin, Some(user_id));
    
    // Create a node
    let input = CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![NodeTranslationInput {
            locale: "en".to_string(),
            title: Some("Test Post".to_string()),
            slug: Some("test-post".to_string()),
            excerpt: Some("Test excerpt".to_string()),
        }],
        bodies: vec![BodyInput {
            locale: "en".to_string(),
            body: Some("Hello, RusToK!".to_string()),
            format: Some("markdown".to_string()),
        }],
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    };
    
    let result = service.create_node(tenant_id, security, input).await;
    assert!(result.is_ok());
    let node = result.unwrap();
    
    // Verify that a NodeCreated event was published
    assert_eq!(event_bus.event_count(), 1);
    assert!(event_bus.has_event_of_type("NodeCreated"));
    
    // Get the events and verify details
    let events = event_bus.events_of_type("NodeCreated");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::NodeCreated { node_id, kind, author_id } = &events[0] {
        assert_eq!(*node_id, node.id);
        assert_eq!(kind, "post");
        assert_eq!(*author_id, Some(user_id));
    } else {
        panic!("Expected NodeCreated event");
    }
    
    println!("✅ Node creation → Event publishing flow verified");
}

#[tokio::test]
async fn test_node_update_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = NodeService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let security = SecurityContext::new(UserRole::Admin, Some(user_id));
    
    // Create a node first
    let input = CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![NodeTranslationInput {
            locale: "en".to_string(),
            title: Some("Original Title".to_string()),
            slug: Some("original-slug".to_string()),
            excerpt: Some("Original excerpt".to_string()),
        }],
        bodies: vec![BodyInput {
            locale: "en".to_string(),
            body: Some("Original content".to_string()),
            format: Some("markdown".to_string()),
        }],
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    };
    
    let node = service.create_node(tenant_id, security.clone(), input).await.unwrap();
    
    // Clear the first event (NodeCreated)
    event_bus.clear();
    
    // Update the node
    use rustok_content::dto::UpdateNodeInput;
    use rustok_content::entities::node::ContentStatus;
    
    let update_input = UpdateNodeInput {
        kind: None,
        translations: Some(vec![NodeTranslationInput {
            locale: "en".to_string(),
            title: Some("Updated Title".to_string()),
            slug: None,
            excerpt: Some("Updated excerpt".to_string()),
        }]),
        bodies: None,
        status: Some(ContentStatus::Published),
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: None,
        published_at: None,
    };
    
    let result = service.update_node(node.id, security, update_input).await;
    assert!(result.is_ok());
    
    // Verify that a NodeUpdated event was published
    assert_eq!(event_bus.event_count(), 2); // 1 NodeCreated + 1 NodeUpdated
    assert!(event_bus.has_event_of_type("NodeUpdated"));
    
    let events = event_bus.events_of_type("NodeUpdated");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::NodeUpdated { node_id, .. } = &events[0] {
        assert_eq!(*node_id, node.id);
    } else {
        panic!("Expected NodeUpdated event");
    }
    
    println!("✅ Node update → Event publishing flow verified");
}

#[tokio::test]
async fn test_node_deletion_triggers_event() {
    // Setup test database and services
    let db = setup_test_db().await;
    let event_bus = mock_event_bus();
    let service = NodeService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let security = SecurityContext::new(UserRole::Admin, Some(user_id));
    
    // Create a node first
    let input = CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![NodeTranslationInput {
            locale: "en".to_string(),
            title: Some("To be deleted".to_string()),
            slug: Some("to-be-deleted".to_string()),
            excerpt: Some("Will be deleted".to_string()),
        }],
        bodies: vec![BodyInput {
            locale: "en".to_string(),
            body: Some("Content to be deleted".to_string()),
            format: Some("markdown".to_string()),
        }],
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    };
    
    let node = service.create_node(tenant_id, security.clone(), input).await.unwrap();
    
    // Clear the first event (NodeCreated)
    event_bus.clear();
    
    // Delete the node
    let result = service.delete_node(node.id, security).await;
    assert!(result.is_ok());
    
    // Verify that a NodeDeleted event was published
    assert_eq!(event_bus.event_count(), 2); // 1 NodeCreated + 1 NodeDeleted
    assert!(event_bus.has_event_of_type("NodeDeleted"));
    
    let events = event_bus.events_of_type("NodeDeleted");
    assert_eq!(events.len(), 1);
    
    if let DomainEvent::NodeDeleted { node_id, .. } = &events[0] {
        assert_eq!(*node_id, node.id);
    } else {
        panic!("Expected NodeDeleted event");
    }
    
    println!("✅ Node deletion → Event publishing flow verified");
}

#[tokio::test]
async fn test_transactional_event_persistence() {
    // Setup test database and services
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let transport = Arc::new(OutboxTransport::new(db.clone()));
    let event_bus = rustok_core::events::TransactionalEventBus::new(transport);
    let mock_bus = mock_event_bus();
    let service = NodeService::new(db.clone(), event_bus);
    
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let security = SecurityContext::new(UserRole::Admin, Some(user_id));
    
    // Create a node (this should use transactional event publishing)
    let input = CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![NodeTranslationInput {
            locale: "en".to_string(),
            title: Some("Transactional Test".to_string()),
            slug: Some("transactional-test".to_string()),
            excerpt: Some("Testing transactional events".to_string()),
        }],
        bodies: vec![BodyInput {
            locale: "en".to_string(),
            body: Some("Transactional content".to_string()),
            format: Some("markdown".to_string()),
        }],
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    };
    
    let result = service.create_node(tenant_id, security, input).await;
    assert!(result.is_ok());
    
    // Verify that the event was persisted in the outbox
    use sea_orm::EntityTrait;
    use rustok_outbox::entity::sys_events;
    
    let events = sys_events::Entity::find()
        .all(&db)
        .await
        .unwrap();
    
    assert_eq!(events.len(), 1, "Event should be persisted in outbox");
    assert_eq!(events[0].event_type, "node.created");
    assert_eq!(events[0].schema_version, 1);
    
    // Also verify that the mock bus received the event
    assert_eq!(mock_bus.event_count(), 1);
    assert!(mock_bus.has_event_of_type("NodeCreated"));
    
    println!("✅ Transactional event persistence verified");
}