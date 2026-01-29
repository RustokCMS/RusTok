use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, warn};
use uuid::Uuid;

use super::types::{DomainEvent, EventEnvelope};

#[derive(Debug, Default)]
pub struct EventBusStats {
    pub events_published: AtomicU64,
    pub events_dropped: AtomicU64,
}

#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
    stats: Arc<EventBusStats>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            stats: Arc::new(EventBusStats::default()),
        }
    }

    pub fn default_capacity() -> Self {
        Self::new(1024)
    }

    pub fn publish(&self, tenant_id: Uuid, actor_id: Option<Uuid>, event: DomainEvent) {
        let envelope = EventEnvelope::new(tenant_id, actor_id, event);
        self.publish_envelope(envelope);
    }

    pub fn publish_envelope(&self, envelope: EventEnvelope) {
        let event_type = envelope.event.event_type();
        let subscriber_count = self.sender.receiver_count();

        if subscriber_count == 0 {
            debug!(
                event_type = event_type,
                tenant_id = %envelope.tenant_id,
                "No subscribers for event"
            );
        }

        match self.sender.send(envelope) {
            Ok(_) => {
                self.stats
                    .events_published
                    .fetch_add(1, Ordering::Relaxed);
                debug!(
                    event_type = event_type,
                    subscribers = subscriber_count,
                    "Event published"
                );
            }
            Err(_) => {
                self.stats.events_dropped.fetch_add(1, Ordering::Relaxed);
                warn!(event_type = event_type, "Event dropped (no receivers)");
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        self.sender.subscribe()
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }

    pub fn stats(&self) -> &EventBusStats {
        &self.stats
    }

    pub fn events_published(&self) -> u64 {
        self.stats.events_published.load(Ordering::Relaxed)
    }

    pub fn events_dropped(&self) -> u64 {
        self.stats.events_dropped.load(Ordering::Relaxed)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::default_capacity()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("subscribers", &self.sender.receiver_count())
            .field("published", &self.events_published())
            .field("dropped", &self.events_dropped())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new(16);
        let mut receiver = bus.subscribe();

        let tenant_id = Uuid::new_v4();
        bus.publish(
            tenant_id,
            None,
            DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: "post".to_string(),
                author_id: None,
            },
        );

        let envelope = receiver.recv().await.unwrap();
        assert_eq!(envelope.tenant_id, tenant_id);
        assert!(matches!(envelope.event, DomainEvent::NodeCreated { .. }));
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        bus.publish(
            Uuid::new_v4(),
            None,
            DomainEvent::UserRegistered {
                user_id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
            },
        );

        assert!(rx1.recv().await.is_ok());
        assert!(rx2.recv().await.is_ok());
    }
}
