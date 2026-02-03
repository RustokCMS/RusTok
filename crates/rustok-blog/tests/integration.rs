use rustok_blog::dto::CreatePostInput;
use rustok_blog::services::PostService;
use rustok_core::events::EventEnvelope;
use rustok_core::{DomainEvent, EventBus, SecurityContext};
use tokio::sync::broadcast;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct TestContext {
    service: PostService,
    events: broadcast::Receiver<EventEnvelope>,
    tenant_id: Uuid,
}

#[tokio::test]
#[ignore = "Integration test requires database/migrations + indexer wiring"]
async fn test_post_lifecycle() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let input = CreatePostInput {
        locale: "en".to_string(),
        title: "Test Post".to_string(),
        body: "Hello, Blog!".to_string(),
        excerpt: Some("Short excerpt".to_string()),
        slug: None,
        publish: true,
        tags: vec!["rust".to_string()],
        metadata: None,
    };

    let post_id = ctx
        .service
        .create_post(ctx.tenant_id, SecurityContext::system(), input)
        .await?;

    let created_event = next_event(&mut ctx.events).await?;
    assert!(matches!(
        created_event.event,
        DomainEvent::NodeCreated { node_id, .. } if node_id == post_id
    ));

    let indexed = wait_for_index(&ctx, post_id).await?;
    assert_eq!(indexed.title, "Test Post");

    Ok(())
}

async fn test_context() -> TestResult<TestContext> {
    let event_bus = EventBus::new();
    let events = event_bus.subscribe();
    let tenant_id = Uuid::nil();
    let db = todo!("create test database connection and apply migrations");

    Ok(TestContext {
        service: PostService::new(db, event_bus),
        events,
        tenant_id,
    })
}

async fn next_event(
    receiver: &mut broadcast::Receiver<EventEnvelope>,
) -> TestResult<EventEnvelope> {
    let envelope = tokio::time::timeout(std::time::Duration::from_secs(5), receiver.recv())
        .await
        .map_err(|_| "timed out waiting for event")??;
    Ok(envelope)
}

struct IndexedPost {
    title: String,
}

async fn wait_for_index(_ctx: &TestContext, _post_id: Uuid) -> TestResult<IndexedPost> {
    todo!("wire index module or test double for read model lookup")
}
