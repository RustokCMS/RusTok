use rust_decimal::Decimal;
use rustok_commerce::dto::{CreateProductInput, CreateVariantInput, PriceInput, ProductTranslationInput};
use rustok_commerce::services::CatalogService;
use rustok_core::events::EventEnvelope;
use rustok_core::{DomainEvent, EventBus};
use tokio::sync::broadcast;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct TestContext {
    service: CatalogService,
    events: broadcast::Receiver<EventEnvelope>,
    tenant_id: Uuid,
    actor_id: Uuid,
}

#[tokio::test]
#[ignore = "Integration test requires database/migrations + indexer wiring"]
async fn test_product_lifecycle() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let input = CreateProductInput {
        translations: vec![ProductTranslationInput {
            locale: "en".to_string(),
            title: "Test Product".to_string(),
            handle: None,
            description: Some("Sample description".to_string()),
            meta_title: None,
            meta_description: None,
        }],
        options: vec![],
        variants: vec![CreateVariantInput {
            sku: Some("SKU-001".to_string()),
            barcode: None,
            option1: None,
            option2: None,
            option3: None,
            prices: vec![PriceInput {
                currency_code: "USD".to_string(),
                amount: Decimal::new(1999, 2),
                compare_at_amount: None,
            }],
            inventory_quantity: 10,
            inventory_policy: "deny".to_string(),
            weight: None,
            weight_unit: None,
        }],
        vendor: Some("RusToK".to_string()),
        product_type: Some("demo".to_string()),
        metadata: serde_json::json!({}),
        publish: true,
    };

    let product = ctx
        .service
        .create_product(ctx.tenant_id, ctx.actor_id, input)
        .await?;

    let created_event = next_event(&mut ctx.events).await?;
    assert!(matches!(
        created_event.event,
        DomainEvent::ProductCreated { product_id } if product_id == product.id
    ));

    let indexed = wait_for_index(&ctx, product.id).await?;
    assert_eq!(indexed.title, "Test Product");

    Ok(())
}

async fn test_context() -> TestResult<TestContext> {
    let event_bus = EventBus::new();
    let events = event_bus.subscribe();
    let tenant_id = Uuid::nil();
    let actor_id = Uuid::nil();
    let db = todo!("create test database connection and apply migrations");

    Ok(TestContext {
        service: CatalogService::new(db, event_bus),
        events,
        tenant_id,
        actor_id,
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

struct IndexedProduct {
    title: String,
}

async fn wait_for_index(_ctx: &TestContext, _product_id: Uuid) -> TestResult<IndexedProduct> {
    todo!("wire index module or test double for read model lookup")
}
