//! # Content Flow Integration Tests
//!
//! Tests the complete content node lifecycle:
//! 1. Create node
//! 2. Add translation
//! 3. Publish node
//! 4. Search and retrieve
//! 5. Verify events are emitted

use rustok_test_utils::*;
use rustok_content::dto::{CreateNodeInput, TranslationInput, BodyFormat};
use rustok_content::entities::NodeStatus;
use std::time::Duration;

/// Test the complete node lifecycle from creation to publication
#[tokio::test]
async fn test_complete_node_lifecycle() {
    let app = spawn_test_app().await;
    
    // 1. Create a node
    let node = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "Test Article".to_string(),
            slug: Some("test-article".to_string()),
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await
        .expect("Failed to create node");
    
    assert_eq!(node.kind, "article");
    assert_eq!(node.title, "Test Article");
    assert_eq!(node.status, NodeStatus::Draft.to_string());
    assert!(node.published_at.is_none());
    
    // 2. Add a Russian translation
    let node = app
        .add_translation(
            node.id,
            "ru",
            TranslationInput {
                title: "Тестовая статья".to_string(),
                slug: Some("testovaya-statya".to_string()),
                body: "# Тестовая статья\n\nЭто тестовый контент.".to_string(),
            },
        )
        .await
        .expect("Failed to add translation");
    
    assert!(node.translations.iter().any(|t| t.locale == "ru"));
    
    // 3. Publish the node
    let published_node = app
        .publish_node(node.id)
        .await
        .expect("Failed to publish node");
    
    assert_eq!(
        published_node.status,
        NodeStatus::Published.to_string(),
        "Node should be published"
    );
    assert!(
        published_node.published_at.is_some(),
        "Published node should have published_at timestamp"
    );
    
    // 4. Verify events were emitted
    let events = app.get_events_for_node(node.id).await;
    
    assert!(
        events.iter().any(|e| matches!(e, rustok_core::events::types::DomainEvent::NodeCreated { .. })),
        "NodeCreated event should be emitted"
    );
    
    assert!(
        events.iter().any(|e| matches!(e, rustok_core::events::types::DomainEvent::NodePublished { .. })),
        "NodePublished event should be emitted"
    );
    
    // 5. Search for the node
    wait_for_events(100).await; // Wait for search index to update
    
    let results = app
        .search_nodes("article")
        .await
        .expect("Failed to search nodes");
    
    assert!(
        results.iter().any(|n| n.id == node.id),
        "Node should be found in search results"
    );
}

/// Test node creation with different content types
#[tokio::test]
async fn test_node_with_different_content_types() {
    let app = spawn_test_app().await;
    
    // Create an article node
    let article = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "Test Article".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await
        .expect("Failed to create article");
    
    assert_eq!(article.kind, "article");
    
    // Create a page node
    let page = app
        .create_node(CreateNodeInput {
            kind: "page".to_string(),
            title: "Test Page".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await
        .expect("Failed to create page");
    
    assert_eq!(page.kind, "page");
    
    // Create a blog post node
    let blog = app
        .create_node(CreateNodeInput {
            kind: "blog_post".to_string(),
            title: "Test Blog Post".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await
        .expect("Failed to create blog post");
    
    assert_eq!(blog.kind, "blog_post");
}

/// Test node translation management
#[tokio::test]
async fn test_node_translations() {
    let app = spawn_test_app().await;
    
    // Create node in English (default)
    let node = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "English Article".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await
        .expect("Failed to create node");
    
    assert_eq!(node.translations.len(), 1); // English translation
    
    // Add Russian translation
    let node = app
        .add_translation(
            node.id,
            "ru",
            TranslationInput {
                title: "Русская статья".to_string(),
                slug: Some("russkaya-statya".to_string()),
                body: "Русский контент".to_string(),
            },
        )
        .await
        .expect("Failed to add Russian translation");
    
    assert_eq!(node.translations.len(), 2);
    assert!(node.translations.iter().any(|t| t.locale == "ru"));
    
    // Add Spanish translation
    let node = app
        .add_translation(
            node.id,
            "es",
            TranslationInput {
                title: "Artículo en español".to_string(),
                slug: Some("articulo-espanol".to_string()),
                body: "Contenido en español".to_string(),
            },
        )
        .await
        .expect("Failed to add Spanish translation");
    
    assert_eq!(node.translations.len(), 3);
    assert!(node.translations.iter().any(|t| t.locale == "es"));
    
    // Verify we can retrieve the node in all languages
    let retrieved = app
        .get_node(node.id)
        .await
        .expect("Failed to retrieve node");
    
    assert_eq!(retrieved.translations.len(), 3);
}

/// Test node search functionality
#[tokio::test]
async fn test_node_search() {
    let app = spawn_test_app().await;
    
    // Create multiple nodes
    let _node1 = app
        .create_node(test_node_input_with_title("Rust Programming Guide"))
        .await
        .expect("Failed to create node 1");
    
    let _node2 = app
        .create_node(test_node_input_with_title("Advanced Rust Tips"))
        .await
        .expect("Failed to create node 2");
    
    let _node3 = app
        .create_node(test_node_input_with_title("Python Tutorial"))
        .await
        .expect("Failed to create node 3");
    
    // Wait for search index to update
    wait_for_events(200).await;
    
    // Search for "Rust"
    let rust_results = app
        .search_nodes("Rust")
        .await
        .expect("Failed to search for 'Rust'");
    
    assert!(
        rust_results.len() >= 2,
        "Should find at least 2 nodes with 'Rust'"
    );
    
    // Search for "Python"
    let python_results = app
        .search_nodes("Python")
        .await
        .expect("Failed to search for 'Python'");
    
    assert!(
        python_results.len() >= 1,
        "Should find at least 1 node with 'Python'"
    );
    
    // Search for non-existent term
    let empty_results = app
        .search_nodes("NonExistentContentXYZ123")
        .await
        .expect("Failed to search");
    
    assert!(
        empty_results.is_empty(),
        "Should find no nodes for non-existent term"
    );
}

/// Test node validation (invalid kind, empty title, etc.)
#[tokio::test]
async fn test_node_validation() {
    let app = spawn_test_app().await;
    
    // Test with empty title
    let result = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "".to_string(), // Invalid
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await;
    
    assert!(result.is_err(), "Node with empty title should fail");
    
    // Test with invalid kind
    let result = app
        .create_node(CreateNodeInput {
            kind: "".to_string(), // Invalid
            title: "Test".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await;
    
    assert!(result.is_err(), "Node with empty kind should fail");
    
    // Test with overly long title
    let long_title = "A".repeat(256); // Too long
    let result = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: long_title,
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await;
    
    assert!(result.is_err(), "Node with overly long title should fail");
}

/// Test node state transitions
#[tokio::test]
async fn test_node_state_transitions() {
    let app = spawn_test_app().await;
    
    // Create node (initial state: Draft)
    let node = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    assert_eq!(node.status, NodeStatus::Draft.to_string());
    
    // Publish node: Draft -> Published
    let published = app
        .publish_node(node.id)
        .await
        .expect("Failed to publish node");
    
    assert_eq!(published.status, NodeStatus::Published.to_string());
    assert!(published.published_at.is_some());
    
    // Verify events
    let events = app.get_events_for_node(node.id).await;
    
    assert!(
        events.iter().any(|e| matches!(e, rustok_core::events::types::DomainEvent::NodeCreated { .. })),
        "NodeCreated event should be present"
    );
    
    assert!(
        events.iter().any(|e| matches!(e, rustok_core::events::types::DomainEvent::NodePublished { .. })),
        "NodePublished event should be present"
    );
}

/// Test node retrieval
#[tokio::test]
async fn test_node_retrieval() {
    let app = spawn_test_app().await;
    
    // Create node
    let created = app
        .create_node(test_node_input())
        .await
        .expect("Failed to create node");
    
    // Retrieve by ID
    let retrieved = app
        .get_node(created.id)
        .await
        .expect("Failed to retrieve node");
    
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.title, created.title);
    assert_eq!(retrieved.kind, created.kind);
    
    // Test non-existent node
    let result = app
        .get_node(test_uuid())
        .await;
    
    assert!(result.is_err(), "Retrieving non-existent node should fail");
}

/// Test node slug uniqueness
#[tokio::test]
async fn test_node_slug_uniqueness() {
    let app = spawn_test_app().await;
    
    // Create first node with specific slug
    let node1 = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "Test Article".to_string(),
            slug: Some("unique-slug".to_string()),
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await
        .expect("Failed to create first node");
    
    // Try to create second node with same slug
    let result = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "Another Article".to_string(),
            slug: Some("unique-slug".to_string()), // Duplicate!
            status: None,
            published_at: None,
            body: Some(rustok_test_utils::test_body_input()),
        })
        .await;
    
    assert!(
        result.is_err(),
        "Creating node with duplicate slug should fail"
    );
    
    // Verify first node still exists
    let retrieved = app
        .get_node(node1.id)
        .await
        .expect("Failed to retrieve first node");
    
    assert_eq!(retrieved.slug, Some("unique-slug".to_string()));
}

/// Test node with different body formats
#[tokio::test]
async fn test_node_with_different_body_formats() {
    let app = spawn_test_app().await;
    
    // Create node with Markdown body
    let markdown_node = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "Markdown Article".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_content::dto::BodyInput {
                format: BodyFormat::Markdown,
                content: "# Markdown Content\n\n**Bold** and *italic*.".to_string(),
            }),
        })
        .await
        .expect("Failed to create markdown node");
    
    assert_eq!(
        markdown_node.body.as_ref().unwrap().format,
        "markdown"
    );
    
    // Create node with HTML body
    let html_node = app
        .create_node(CreateNodeInput {
            kind: "article".to_string(),
            title: "HTML Article".to_string(),
            slug: None,
            status: None,
            published_at: None,
            body: Some(rustok_content::dto::BodyInput {
                format: BodyFormat::Html,
                content: "<h1>HTML Content</h1>\n<p><strong>Bold</strong> text.</p>".to_string(),
            }),
        })
        .await
        .expect("Failed to create HTML node");
    
    assert_eq!(
        html_node.body.as_ref().unwrap().format,
        "html"
    );
}

/// Helper function to wait for events/indices to propagate
async fn wait_for_events(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}
