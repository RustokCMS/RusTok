use async_trait::async_trait;
use std::any::Any;
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

    fn as_any(&self) -> &dyn Any;
}
