use rustok_core::events::EventEnvelope;
use rustok_core::{DomainEvent, EventBus, SecurityContext};
use rustok_pages::dto::{CreatePageInput, PageBodyInput, PageTranslationInput};
use rustok_pages::services::PageService;
use tokio::sync::broadcast;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct TestContext {
    service: PageService,
    events: broadcast::Receiver<EventEnvelope>,
    tenant_id: Uuid,
}

#[tokio::test]
#[ignore = "Integration test requires database/migrations + indexer wiring"]
async fn test_page_lifecycle() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let input = CreatePageInput {
        template: None,
        publish: true,
        translations: vec![PageTranslationInput {
            locale: "en".to_string(),
            title: "Test Page".to_string(),
            slug: None,
            meta_title: None,
            meta_description: None,
        }],
        body: Some(PageBodyInput {
            locale: "en".to_string(),
            content: "Hello, Pages!".to_string(),
            format: Some("markdown".to_string()),
        }),
        blocks: None,
    };

    let page = ctx
        .service
        .create(ctx.tenant_id, SecurityContext::system(), input)
        .await?;

    let created_event = next_event(&mut ctx.events).await?;
    assert!(matches!(
        created_event.event,
        DomainEvent::NodeCreated { node_id, .. } if node_id == page.id
    ));

    let indexed = wait_for_index(&ctx, page.id).await?;
    assert_eq!(indexed.title, "Test Page");

    Ok(())
}

async fn test_context() -> TestResult<TestContext> {
    let event_bus = EventBus::new();
    let events = event_bus.subscribe();
    let tenant_id = Uuid::nil();
    let db = todo!("create test database connection and apply migrations");

    Ok(TestContext {
        service: PageService::new(db, event_bus),
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

struct IndexedPage {
    title: String,
}

async fn wait_for_index(_ctx: &TestContext, _page_id: Uuid) -> TestResult<IndexedPage> {
    todo!("wire index module or test double for read model lookup")
}
