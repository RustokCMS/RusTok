use std::sync::Arc;

use rustok_core::Result;
use rustok_iggy_connector::{IggyConnector, PublishRequest};
use tokio::sync::RwLock;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DlqEntry {
    pub event_id: Uuid,
    pub original_topic: String,
    pub payload: Vec<u8>,
    pub error: String,
    pub retry_count: u32,
}

#[derive(Debug)]
pub struct DlqManager {
    stream: Arc<RwLock<String>>,
    topic: Arc<RwLock<String>>,
    max_retries: Arc<RwLock<u32>>,
}

impl Default for DlqManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DlqManager {
    pub fn new() -> Self {
        Self {
            stream: Arc::new(RwLock::new("rustok".to_string())),
            topic: Arc::new(RwLock::new("dlq".to_string())),
            max_retries: Arc::new(RwLock::new(3)),
        }
    }

    pub fn with_stream(self, stream: String) -> Self {
        *self.stream.blocking_write() = stream;
        self
    }

    pub fn with_topic(self, topic: String) -> Self {
        *self.topic.blocking_write() = topic;
        self
    }

    pub fn with_max_retries(self, max_retries: u32) -> Self {
        *self.max_retries.blocking_write() = max_retries;
        self
    }

    pub async fn move_to_dlq(&self, connector: &dyn IggyConnector, entry: DlqEntry) -> Result<()> {
        let stream = self.stream.read().await.clone();
        let topic = self.topic.read().await.clone();

        warn!(
            event_id = %entry.event_id,
            original_topic = %entry.original_topic,
            error = %entry.error,
            retry_count = entry.retry_count,
            dlq_stream = %stream,
            dlq_topic = %topic,
            "Moving event to dead letter queue"
        );

        let request = PublishRequest::new(
            stream,
            topic,
            entry.event_id.to_string(),
            entry.payload,
            format!("dlq-{}", entry.event_id),
        );

        connector.publish(request).await.map_err(|e| {
            error!(error = %e, "Failed to publish to DLQ");
            rustok_core::Error::External(e.to_string())
        })?;

        Ok(())
    }

    pub async fn retry_from_dlq(
        &self,
        _connector: &dyn IggyConnector,
        _event_id: Uuid,
        _target_topic: String,
    ) -> Result<()> {
        let max_retries = *self.max_retries.read().await;

        info!(
            max_retries = max_retries,
            "DLQ retry requested - implementation pending message consumption"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dlq_manager_defaults() {
        let manager = DlqManager::new();
        assert_eq!(*manager.stream.blocking_read(), "rustok");
        assert_eq!(*manager.topic.blocking_read(), "dlq");
        assert_eq!(*manager.max_retries.blocking_read(), 3);
    }

    #[test]
    fn dlq_entry_creation() {
        let entry = DlqEntry {
            event_id: Uuid::new_v4(),
            original_topic: "domain".to_string(),
            payload: vec![1, 2, 3],
            error: "Processing failed".to_string(),
            retry_count: 2,
        };

        assert!(!entry.payload.is_empty());
        assert_eq!(entry.retry_count, 2);
    }
}
