use chrono::{DateTime, Utc};
use uuid::Uuid;

mod bus;

pub use bus::EventBus;

#[derive(Debug, Clone)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub event: DomainEvent,
}

#[derive(Debug, Clone)]
pub enum DomainEvent {
    ModuleEnabled {
        tenant_id: Uuid,
        module_slug: String,
    },
    ModuleDisabled {
        tenant_id: Uuid,
        module_slug: String,
    },
}

pub trait EventHandler: Send + Sync {
    fn handles(&self, event: &DomainEvent) -> bool;
    fn name(&self) -> &'static str;
    fn handle(&self, envelope: &EventEnvelope) -> crate::Result<()>;
}
