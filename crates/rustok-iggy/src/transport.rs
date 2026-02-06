use std::sync::Arc;

use async_trait::async_trait;

use rustok_core::events::{EventEnvelope, EventTransport, ReliabilityLevel};
use rustok_core::Result;
use rustok_iggy_connector::{ConnectorConfig, EmbeddedConnector, IggyConnector, RemoteConnector};
use crate::config::{IggyConfig, IggyMode};
use crate::consumer::ConsumerGroupManager;
use crate::serialization::{BincodeSerializer, EventSerializer, JsonSerializer};
use crate::topology::TopologyManager;
use crate::{producer, topology};

pub struct IggyTransport {
    config: IggyConfig,
    connector: Arc<dyn IggyConnector>,
    topology: TopologyManager,
    consumers: ConsumerGroupManager,
    serializer: Arc<dyn EventSerializer>,
}

impl IggyTransport {
    pub async fn new(config: IggyConfig) -> Result<Self> {
        let connector: Arc<dyn IggyConnector> = match config.mode {
            IggyMode::Remote => Arc::new(RemoteConnector::default()),
            IggyMode::Embedded => Arc::new(EmbeddedConnector::default()),
        };

        let connector_config = ConnectorConfig::from(&config);
        connector.connect(&connector_config).await?;
        topology::ensure_topology(&config).await?;

        let serializer: Arc<dyn EventSerializer> = match config.serialization {
            crate::config::SerializationFormat::Json => Arc::new(JsonSerializer),
            crate::config::SerializationFormat::Bincode => Arc::new(BincodeSerializer),
        };

        Ok(Self {
            config,
            connector,
            topology: TopologyManager,
            consumers: ConsumerGroupManager,
            serializer,
        })
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.connector
            .shutdown()
            .await
            .map_err(|error| rustok_core::Error::External(error.to_string()))
    }

    pub async fn subscribe_as_group(&self, _group: &str) -> Result<()> {
        let _ = (&self.topology, &self.consumers);
        Ok(())
    }

    pub async fn replay(&self) -> Result<()> {
        let _ = (&self.topology, &self.consumers);
        Ok(())
    }
}

#[async_trait]
impl EventTransport for IggyTransport {
    async fn publish(&self, envelope: EventEnvelope) -> Result<()> {
        let request = producer::build_publish_request(&self.config, &*self.serializer, envelope)?;
        self.connector
            .publish(request)
            .await
            .map_err(|error| rustok_core::Error::External(error.to_string()))
    }

    fn reliability_level(&self) -> ReliabilityLevel {
        ReliabilityLevel::Streaming
    }
}
