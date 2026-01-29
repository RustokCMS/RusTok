use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use tokio::sync::broadcast;
use uuid::Uuid;

use super::{DomainEvent, EventEnvelope};

const DEFAULT_CHANNEL_CAPACITY: usize = 128;

#[derive(Debug)]
pub struct EventBus {
    sender: broadcast::Sender<EventEnvelope>,
    stats: Arc<EventBusStats>,
}

#[derive(Debug, Default)]
pub struct EventBusStats {
    events_published: AtomicUsize,
    events_dropped: AtomicUsize,
    subscribers: AtomicUsize,
}

impl EventBusStats {
    pub fn events_published(&self) -> usize {
        self.events_published.load(Ordering::Relaxed)
    }

    pub fn events_dropped(&self) -> usize {
        self.events_dropped.load(Ordering::Relaxed)
    }

    pub fn subscribers(&self) -> usize {
        self.subscribers.load(Ordering::Relaxed)
    }
}

impl EventBus {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CHANNEL_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            stats: Arc::new(EventBusStats::default()),
        }
    }

    pub fn stats(&self) -> Arc<EventBusStats> {
        Arc::clone(&self.stats)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventEnvelope> {
        let receiver = self.sender.subscribe();
        self.stats
            .subscribers
            .store(self.sender.receiver_count(), Ordering::Relaxed);
        receiver
    }

    pub fn publish(
        &self,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> crate::Result<()> {
        let envelope = EventEnvelope::new(tenant_id, actor_id, event);

        if self.sender.receiver_count() == 0 {
            tracing::debug!(event = ?envelope.event, "Event published without subscribers");
        }

        match self.sender.send(envelope) {
            Ok(_) => {
                self.stats.events_published.fetch_add(1, Ordering::Relaxed);
            }
            Err(error) => {
                self.stats.events_dropped.fetch_add(1, Ordering::Relaxed);
                tracing::debug!(?error, "Event dropped while publishing");
            }
        }

        Ok(())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            stats: Arc::clone(&self.stats),
        }
    }
}
