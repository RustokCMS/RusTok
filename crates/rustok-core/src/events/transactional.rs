use crate::events::{DomainEvent, EventEnvelope, EventTransport};
use crate::Result;
use sea_orm::DatabaseTransaction;
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

    pub async fn publish_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<()> {
        let envelope = EventEnvelope::new(tenant_id, actor_id, event);

        self.transport.publish_in_tx(txn, envelope).await?;

        Ok(())
    }

    pub async fn publish(
        &self,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<()> {
        let envelope = EventEnvelope::new(tenant_id, actor_id, event);
        self.transport.publish(envelope).await
    }
}
