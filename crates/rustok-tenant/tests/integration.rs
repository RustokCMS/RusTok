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
#[ignore = "Integration test requires tenant models + database wiring"]
async fn test_tenant_event_flow() -> TestResult<()> {
    let mut ctx = test_context().await?;

    let event = DomainEvent::TenantCreated {
        tenant_id: ctx.tenant_id,
    };

    let bus = ctx.bus.clone();
    bus.publish(ctx.tenant_id, None, event)?;

    let envelope = next_event(&mut ctx.events).await?;
    assert!(matches!(
        envelope.event,
        DomainEvent::TenantCreated { tenant_id } if tenant_id == ctx.tenant_id
    ));

    let resolved = wait_for_tenant(&ctx, ctx.tenant_id).await?;
    assert_eq!(resolved, ctx.tenant_id);

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

async fn wait_for_tenant(_ctx: &TestContext, tenant_id: Uuid) -> TestResult<Uuid> {
    todo!("wire tenant lookup for integration tests")
}
