use crate::transport::OutboxTransport;
use rustok_core::events::{DomainEvent, EventEnvelope, EventTransport, ValidateEvent};
use rustok_core::Result;
use sea_orm::ConnectionTrait;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TransactionalEventBus {
    transport: Arc<dyn EventTransport>,
}

impl TransactionalEventBus {
    pub fn new(transport: Arc<dyn EventTransport>) -> Self {
        Self { transport }
    }

    pub async fn publish_in_tx<C>(
        &self,
        txn: &C,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<()>
    where
        C: ConnectionTrait,
    {
        // Validate event before publishing
        event.validate().map_err(|e| {
            tracing::error!(
                event_type = event.event_type(),
                error = %e,
                "Event validation failed"
            );
            rustok_core::Error::Validation(format!("Event validation failed: {}", e))
        })?;

        let envelope = EventEnvelope::new(tenant_id, actor_id, event);

        if let Some(outbox) = self.transport.as_any().downcast_ref::<OutboxTransport>() {
            outbox.write_to_outbox(txn, envelope).await?;
        } else {
            tracing::warn!(
                "EventTransport doesn't support transactional writes. \
                 Event may be lost if transaction fails."
            );
            self.transport.publish(envelope).await?;
        }

        Ok(())
    }

    pub async fn publish(
        &self,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<()> {
        // Validate event before publishing
        event.validate().map_err(|e| {
            tracing::error!(
                event_type = event.event_type(),
                error = %e,
                "Event validation failed"
            );
            rustok_core::Error::Validation(format!("Event validation failed: {}", e))
        })?;

        let envelope = EventEnvelope::new(tenant_id, actor_id, event);
        self.transport.publish(envelope).await
    }
}
