use std::sync::Arc;

use rustok_core::Result;
use rustok_iggy_connector::IggyConnector;
use tokio::sync::RwLock;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReplayConfig {
    pub stream: String,
    pub topic: String,
    pub from_offset: Option<u64>,
    pub to_offset: Option<u64>,
    pub consumer_group: Option<String>,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            stream: "rustok".to_string(),
            topic: "domain".to_string(),
            from_offset: None,
            to_offset: None,
            consumer_group: None,
        }
    }
}

impl ReplayConfig {
    pub fn new(stream: String, topic: String) -> Self {
        Self {
            stream,
            topic,
            ..Default::default()
        }
    }

    pub fn from_offset(mut self, offset: u64) -> Self {
        self.from_offset = Some(offset);
        self
    }

    pub fn to_offset(mut self, offset: u64) -> Self {
        self.to_offset = Some(offset);
        self
    }

    pub fn consumer_group(mut self, group: String) -> Self {
        self.consumer_group = Some(group);
        self
    }
}

#[derive(Debug)]
pub struct ReplayManager {
    active_replays: Arc<RwLock<Vec<ActiveReplay>>>,
}

#[derive(Debug, Clone)]
pub struct ActiveReplay {
    pub id: Uuid,
    pub config: ReplayConfig,
    pub status: ReplayStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for ReplayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplayManager {
    pub fn new() -> Self {
        Self {
            active_replays: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start_replay(
        &self,
        _connector: &dyn IggyConnector,
        config: ReplayConfig,
    ) -> Result<Uuid> {
        let replay_id = Uuid::new_v4();

        info!(
            replay_id = %replay_id,
            stream = %config.stream,
            topic = %config.topic,
            from_offset = ?config.from_offset,
            to_offset = ?config.to_offset,
            consumer_group = ?config.consumer_group,
            "Starting event replay"
        );

        let replay = ActiveReplay {
            id: replay_id,
            config,
            status: ReplayStatus::Running,
        };

        self.active_replays.write().await.push(replay);

        Ok(replay_id)
    }

    pub async fn get_replay_status(&self, replay_id: Uuid) -> Option<ReplayStatus> {
        self.active_replays
            .read()
            .await
            .iter()
            .find(|r| r.id == replay_id)
            .map(|r| r.status)
    }

    pub async fn cancel_replay(&self, replay_id: Uuid) -> bool {
        let mut replays = self.active_replays.write().await;
        if let Some(replay) = replays.iter_mut().find(|r| r.id == replay_id) {
            replay.status = ReplayStatus::Failed;
            info!(replay_id = %replay_id, "Replay cancelled");
            return true;
        }
        false
    }
}

pub async fn replay_events(connector: &dyn IggyConnector, config: ReplayConfig) -> Result<Uuid> {
    let manager = ReplayManager::new();
    manager.start_replay(connector, config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_config_defaults() {
        let config = ReplayConfig::default();
        assert_eq!(config.stream, "rustok");
        assert_eq!(config.topic, "domain");
        assert!(config.from_offset.is_none());
        assert!(config.to_offset.is_none());
    }

    #[test]
    fn replay_config_builder() {
        let config = ReplayConfig::new("stream1".to_string(), "topic1".to_string())
            .from_offset(100)
            .to_offset(200)
            .consumer_group("replayer".to_string());

        assert_eq!(config.stream, "stream1");
        assert_eq!(config.topic, "topic1");
        assert_eq!(config.from_offset, Some(100));
        assert_eq!(config.to_offset, Some(200));
        assert_eq!(config.consumer_group, Some("replayer".to_string()));
    }

    #[tokio::test]
    async fn replay_manager_starts_empty() {
        let manager = ReplayManager::new();
        assert!(manager.active_replays.read().await.is_empty());
    }
}
