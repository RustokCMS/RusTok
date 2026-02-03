use std::sync::Arc;

use async_trait::async_trait;

use rustok_core::events::{EventEnvelope, EventTransport, ReliabilityLevel};
use rustok_core::Result;

use crate::backend::embedded::EmbeddedBackend;
use crate::backend::remote::RemoteBackend;
use crate::backend::IggyBackend;
use crate::config::{IggyConfig, IggyMode};
use crate::consumer::ConsumerGroupManager;
use crate::serialization::{BincodeSerializer, EventSerializer, JsonSerializer};
use crate::topology::TopologyManager;
use crate::{producer, topology};

pub struct IggyTransport {
    config: IggyConfig,
    backend: Arc<dyn IggyBackend>,
    topology: TopologyManager,
    consumers: ConsumerGroupManager,
    serializer: Arc<dyn EventSerializer>,
}

impl IggyTransport {
    pub async fn new(config: IggyConfig) -> Result<Self> {
        let backend: Arc<dyn IggyBackend> = match config.mode {
            IggyMode::Remote => Arc::new(RemoteBackend),
            IggyMode::Embedded => Arc::new(EmbeddedBackend),
        };

        backend.connect(&config).await?;
        topology::ensure_topology(&config).await?;

        let serializer: Arc<dyn EventSerializer> = match config.serialization {
            crate::config::SerializationFormat::Json => Arc::new(JsonSerializer),
            crate::config::SerializationFormat::Bincode => Arc::new(BincodeSerializer),
        };

        Ok(Self {
            config,
            backend,
            topology: TopologyManager,
            consumers: ConsumerGroupManager,
            serializer,
        })
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.backend.shutdown().await
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
        producer::publish(&self.config, &*self.serializer, envelope).await
    }

    fn reliability_level(&self) -> ReliabilityLevel {
        ReliabilityLevel::Streaming
    }
}
