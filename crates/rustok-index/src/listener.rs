use async_trait::async_trait;
use std::sync::Arc;

use rustok_core::events::{DomainEvent, EventEnvelope, EventHandler, HandlerResult};
use rustok_core::Error;
use sea_orm::DatabaseConnection;

use crate::engine::SearchEngine;

pub struct IndexListener {
    engine: Arc<dyn SearchEngine>,
    db: DatabaseConnection,
}

impl IndexListener {
    pub fn new(engine: Arc<dyn SearchEngine>, db: DatabaseConnection) -> Self {
        Self { engine, db }
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn engine(&self) -> Arc<dyn SearchEngine> {
        Arc::clone(&self.engine)
    }

    async fn reindex_node(&self, _node_id: uuid::Uuid) -> Result<(), Error> {
        Ok(())
    }
}

#[async_trait]
impl EventHandler for IndexListener {
    fn name(&self) -> &'static str {
        "index_listener"
    }

    fn handles(&self, event: &DomainEvent) -> bool {
        event.affects_index()
    }

    async fn handle(&self, envelope: &EventEnvelope) -> HandlerResult {
        match &envelope.event {
            DomainEvent::NodeCreated { node_id, .. }
            | DomainEvent::NodeUpdated { node_id, .. }
            | DomainEvent::NodePublished { node_id, .. } => {
                self.reindex_node(*node_id).await?;
            }
            DomainEvent::NodeDeleted { node_id, .. } => {
                self.engine.delete(*node_id, None).await?;
            }
            _ => {}
        }
        Ok(())
    }
}
