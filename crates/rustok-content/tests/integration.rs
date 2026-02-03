use rustok_content::dto::{BodyInput, CreateNodeInput, NodeTranslationInput};
use rustok_content::services::NodeService;
use rustok_core::events::EventEnvelope;
use rustok_core::{DomainEvent, SecurityContext};
use tokio::sync::broadcast;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct TestContext {
    service: NodeService,
    events: broadcast::Receiver<EventEnvelope>,
    tenant_id: Uuid,
}

#[tokio::test]
#[ignore = "Integration test requires database/migrations + indexer wiring"]
async fn test_node_lifecycle() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let input = CreateNodeInput {
        kind: "post".to_string(),
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
        translations: vec![NodeTranslationInput {
            locale: "en".to_string(),
            title: Some("Test Post".to_string()),
            slug: None,
            excerpt: Some("Test excerpt".to_string()),
        }],
        bodies: vec![BodyInput {
            locale: "en".to_string(),
            body: Some("Hello, RusToK!".to_string()),
            format: Some("markdown".to_string()),
        }],
    };

    let node = ctx
        .service
        .create_node(ctx.tenant_id, SecurityContext::system(), input)
        .await?;

    let created_event = next_event(&mut ctx.events).await?;
    assert!(matches!(
        created_event.event,
        DomainEvent::NodeCreated { node_id, .. } if node_id == node.id
    ));

    let indexed = wait_for_index(&ctx, node.id).await?;
    assert_eq!(indexed.title, "Test Post");

    Ok(())
}

async fn test_context() -> TestResult<TestContext> {
    Err("create test database connection and apply migrations".into())
}

async fn next_event(
    receiver: &mut broadcast::Receiver<EventEnvelope>,
) -> TestResult<EventEnvelope> {
    let envelope = tokio::time::timeout(std::time::Duration::from_secs(5), receiver.recv())
        .await
        .map_err(|_| "timed out waiting for event")??;
    Ok(envelope)
}

struct IndexedNode {
    title: String,
}

async fn wait_for_index(_ctx: &TestContext, _node_id: Uuid) -> TestResult<IndexedNode> {
    Err("wire index module or test double for read model lookup".into())
}
