use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ConnectionTrait, DatabaseConnection, EntityTrait, Set};
use std::any::Any;

use rustok_core::events::{EventEnvelope, EventTransport, ReliabilityLevel};
use rustok_core::Result;

use crate::entity;
use crate::entity::SysEventStatus;

#[derive(Clone, Debug)]
pub struct OutboxTransport {
    db: DatabaseConnection,
}

impl OutboxTransport {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn write_to_outbox<C>(&self, txn: &C, envelope: EventEnvelope) -> Result<()>
    where
        C: ConnectionTrait,
    {
        let payload = serde_json::to_value(&envelope)?;
        let model = entity::ActiveModel {
            id: Set(envelope.id),
            event_type: Set(envelope.event_type.clone()),
            schema_version: Set(envelope.schema_version as i16),
            payload: Set(payload),
            status: Set(SysEventStatus::Pending),
            retry_count: Set(0),
            next_attempt_at: Set(None),
            last_error: Set(None),
            claimed_by: Set(None),
            claimed_at: Set(None),
            created_at: Set(Utc::now()),
            dispatched_at: Set(None),
        };
        model.insert(txn).await?;
        Ok(())
    }
}

#[async_trait]
impl EventTransport for OutboxTransport {
    async fn publish(&self, envelope: EventEnvelope) -> Result<()> {
        let payload = serde_json::to_value(&envelope)?;
        let model = entity::ActiveModel {
            id: Set(envelope.id),
            event_type: Set(envelope.event_type.clone()),
            schema_version: Set(envelope.schema_version as i16),
            payload: Set(payload),
            status: Set(SysEventStatus::Pending),
            retry_count: Set(0),
            next_attempt_at: Set(None),
            last_error: Set(None),
            claimed_by: Set(None),
            claimed_at: Set(None),
            created_at: Set(Utc::now()),
            dispatched_at: Set(None),
        };
        model.insert(&self.db).await?;
        Ok(())
    }

    async fn acknowledge(&self, event_id: uuid::Uuid) -> Result<()> {
        let mut model: entity::ActiveModel = entity::Entity::find_by_id(event_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| rustok_core::Error::NotFound(format!("sys_event {event_id}")))?
            .into();
        model.status = Set(SysEventStatus::Dispatched);
        model.dispatched_at = Set(Some(Utc::now()));
        model.claimed_by = Set(None);
        model.claimed_at = Set(None);
        model.last_error = Set(None);
        model.update(&self.db).await?;
        Ok(())
    }

    fn reliability_level(&self) -> ReliabilityLevel {
        ReliabilityLevel::Outbox
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
