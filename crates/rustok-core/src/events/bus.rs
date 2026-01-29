use super::DomainEvent;

#[derive(Clone, Default)]
pub struct EventBus;

impl EventBus {
    pub fn new() -> Self {
        Self
    }

    pub fn subscribe(&self) {
        tracing::debug!("EventBus subscribe (stub)");
    }

    pub fn publish(&self, event: DomainEvent) -> crate::Result<()> {
        tracing::debug!(?event, "Event published (stub)");
        Ok(())
    }
}
