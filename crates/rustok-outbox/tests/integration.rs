use rustok_core::events::EventEnvelope;
use rustok_core::{DomainEvent, EventBus};
use tokio::sync::broadcast;
use uuid::Uuid;

type TestResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct TestContext {
    bus: EventBus,
    events: broadcast::Receiver<EventEnvelope>,
    tenant_id: Uuid,
}

#[tokio::test]
#[ignore = "Integration test requires database outbox wiring"]
async fn test_outbox_persist_and_dispatch() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let event = DomainEvent::UserRegistered {
        user_id: Uuid::nil(),
        email: "test@example.com".to_string(),
    };

    let bus = ctx.bus.clone();
    bus.publish(ctx.tenant_id, None, event)?;

    let envelope = next_event(&mut ctx.events).await?;
    assert!(matches!(
        envelope.event,
        DomainEvent::UserRegistered { .. }
    ));

    let dispatched = wait_for_dispatch(&ctx, envelope.id).await?;
    assert!(dispatched);

    Ok(())
}

async fn test_context() -> TestResult<TestContext> {
    let bus = EventBus::new();
    let events = bus.subscribe();
    let tenant_id = Uuid::nil();

    Ok(TestContext {
        bus,
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

async fn wait_for_dispatch(_ctx: &TestContext, _event_id: Uuid) -> TestResult<bool> {
    todo!("wire outbox dispatch status lookup for integration tests")
}
