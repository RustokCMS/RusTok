//! Connector layer for Iggy transports.
//!
//! Status: initial implementation with logging-only stubs. Replace `RemoteConnector`
//! and `EmbeddedConnector` with real SDK/embedded runtime wiring.

use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorMode {
    Embedded,
    Remote,
}

#[derive(Debug, Clone)]
pub struct EmbeddedConnectorConfig {
    pub data_dir: String,
    pub tcp_port: u16,
    pub http_port: u16,
}

impl Default for EmbeddedConnectorConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data/iggy".to_string(),
            tcp_port: 8090,
            http_port: 3000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RemoteConnectorConfig {
    pub addresses: Vec<String>,
    pub protocol: String,
    pub username: String,
    pub password: String,
}

impl Default for RemoteConnectorConfig {
    fn default() -> Self {
        Self {
            addresses: vec!["127.0.0.1:8090".to_string()],
            protocol: "tcp".to_string(),
            username: "rustok".to_string(),
            password: "${IGGY_PASSWORD}".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectorConfig {
    pub mode: ConnectorMode,
    pub embedded: EmbeddedConnectorConfig,
    pub remote: RemoteConnectorConfig,
}

impl Default for ConnectorConfig {
    fn default() -> Self {
        Self {
            mode: ConnectorMode::Embedded,
            embedded: EmbeddedConnectorConfig::default(),
            remote: RemoteConnectorConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PublishRequest {
    pub stream: String,
    pub topic: String,
    pub partition_key: String,
    pub payload: Vec<u8>,
    pub event_id: String,
}

#[async_trait]
pub trait IggyConnector: Send + Sync + 'static {
    async fn connect(&self, config: &ConnectorConfig) -> Result<(), ConnectorError>;
    async fn publish(&self, request: PublishRequest) -> Result<(), ConnectorError>;
    async fn shutdown(&self) -> Result<(), ConnectorError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectorError {
    #[error("connector error: {0}")]
    Message(String),
}

#[derive(Debug, Default)]
pub struct RemoteConnector;

#[async_trait]
impl IggyConnector for RemoteConnector {
    async fn connect(&self, config: &ConnectorConfig) -> Result<(), ConnectorError> {
        tracing::info!(
            mode = "remote",
            addresses = ?config.remote.addresses,
            protocol = %config.remote.protocol,
            "Iggy remote connector initialized"
        );
        Ok(())
    }

    async fn publish(&self, request: PublishRequest) -> Result<(), ConnectorError> {
        tracing::debug!(
            mode = "remote",
            stream = %request.stream,
            topic = %request.topic,
            partition_key = %request.partition_key,
            event_id = %request.event_id,
            payload_size = request.payload.len(),
            "Publishing event via remote connector"
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ConnectorError> {
        tracing::info!(mode = "remote", "Iggy remote connector shutdown");
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct EmbeddedConnector;

#[async_trait]
impl IggyConnector for EmbeddedConnector {
    async fn connect(&self, config: &ConnectorConfig) -> Result<(), ConnectorError> {
        tracing::info!(
            mode = "embedded",
            data_dir = %config.embedded.data_dir,
            tcp_port = config.embedded.tcp_port,
            http_port = config.embedded.http_port,
            "Iggy embedded connector initialized"
        );
        Ok(())
    }

    async fn publish(&self, request: PublishRequest) -> Result<(), ConnectorError> {
        tracing::debug!(
            mode = "embedded",
            stream = %request.stream,
            topic = %request.topic,
            partition_key = %request.partition_key,
            event_id = %request.event_id,
            payload_size = request.payload.len(),
            "Publishing event via embedded connector"
        );
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ConnectorError> {
        tracing::info!(mode = "embedded", "Iggy embedded connector shutdown");
        Ok(())
    }
}
