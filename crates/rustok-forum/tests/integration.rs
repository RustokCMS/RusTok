use rustok_core::events::EventEnvelope;
use rustok_core::{DomainEvent, EventBus, SecurityContext};
use rustok_forum::dto::CreateTopicInput;
use rustok_forum::services::TopicService;
use tokio::sync::broadcast;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct TestContext {
    service: TopicService,
    events: broadcast::Receiver<EventEnvelope>,
    tenant_id: Uuid,
}

#[tokio::test]
#[ignore = "Integration test requires database/migrations + indexer wiring"]
async fn test_topic_lifecycle() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let input = CreateTopicInput {
        locale: "en".to_string(),
        category_id: Uuid::nil(),
        title: "Test Topic".to_string(),
        body: "Hello, Forum!".to_string(),
        tags: vec!["rust".to_string()],
    };

    let topic = ctx
        .service
        .create(ctx.tenant_id, SecurityContext::system(), input)
        .await?;

    let created_event = next_event(&mut ctx.events).await?;
    assert!(matches!(
        created_event.event,
        DomainEvent::NodeCreated { node_id, .. } if node_id == topic.id
    ));

    let indexed = wait_for_index(&ctx, topic.id).await?;
    assert_eq!(indexed.title, "Test Topic");

    Ok(())
}

async fn test_context() -> TestResult<TestContext> {
    let event_bus = EventBus::new();
    let events = event_bus.subscribe();
    let tenant_id = Uuid::nil();
    let db = todo!("create test database connection and apply migrations");

    Ok(TestContext {
        service: TopicService::new(db, event_bus),
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

struct IndexedTopic {
    title: String,
}

async fn wait_for_index(_ctx: &TestContext, _topic_id: Uuid) -> TestResult<IndexedTopic> {
    todo!("wire index module or test double for read model lookup")
}
