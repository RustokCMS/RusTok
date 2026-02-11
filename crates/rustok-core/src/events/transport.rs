use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use uuid::Uuid;

use crate::Result;

use super::EventEnvelope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReliabilityLevel {
    InMemory,
    Outbox,
    Streaming,
}

#[async_trait]
pub trait EventTransport: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<()>;

    async fn publish_in_tx(
        &self,
        _txn: &DatabaseTransaction,
        envelope: EventEnvelope,
    ) -> Result<()> {
        self.publish(envelope).await
    }

    async fn publish_batch(&self, events: Vec<EventEnvelope>) -> Result<()> {
        for envelope in events {
            self.publish(envelope).await?;
        }
        Ok(())
    }

    async fn acknowledge(&self, _event_id: Uuid) -> Result<()> {
        Ok(())
    }

    fn reliability_level(&self) -> ReliabilityLevel;
}
