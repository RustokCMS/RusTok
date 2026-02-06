use rustok_iggy_connector::{
    ConnectorConfig, ConnectorMode, EmbeddedConnectorConfig, RemoteConnectorConfig,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct IggyConfig {
    #[serde(default)]
    pub mode: IggyMode,
    #[serde(default)]
    pub serialization: SerializationFormat,
    #[serde(default)]
    pub embedded: EmbeddedConfig,
    #[serde(default)]
    pub remote: RemoteConfig,
    #[serde(default)]
    pub topology: TopologyConfig,
    #[serde(default)]
    pub retention: RetentionConfig,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IggyMode {
    #[default]
    Embedded,
    Remote,
}

#[derive(Debug, Deserialize, Clone, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SerializationFormat {
    #[default]
    Json,
    Bincode,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmbeddedConfig {
    pub data_dir: String,
    pub use_binary_fallback: bool,
    pub tcp_port: u16,
    pub http_port: u16,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        Self {
            data_dir: "./data/iggy".to_string(),
            use_binary_fallback: true,
            tcp_port: 8090,
            http_port: 3000,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RemoteConfig {
    pub addresses: Vec<String>,
    pub protocol: String,
    pub username: String,
    pub password: String,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            addresses: vec!["127.0.0.1:8090".to_string()],
            protocol: "tcp".to_string(),
            username: "rustok".to_string(),
            password: "${IGGY_PASSWORD}".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct TopologyConfig {
    pub stream_name: String,
    pub domain_partitions: u32,
    pub replication_factor: u8,
}

impl Default for TopologyConfig {
    fn default() -> Self {
        Self {
            stream_name: "rustok".to_string(),
            domain_partitions: 8,
            replication_factor: 1,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetentionConfig {
    pub domain_max_age_days: u32,
    pub domain_max_size_gb: u32,
    pub system_max_age_days: u32,
    pub dlq_max_age_days: u32,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            domain_max_age_days: 30,
            domain_max_size_gb: 10,
            system_max_age_days: 7,
            dlq_max_age_days: 365,
        }
    }
}

impl From<&IggyConfig> for ConnectorConfig {
    fn from(config: &IggyConfig) -> Self {
        let mode = match config.mode {
            IggyMode::Embedded => ConnectorMode::Embedded,
            IggyMode::Remote => ConnectorMode::Remote,
        };

        let embedded = EmbeddedConnectorConfig {
            data_dir: config.embedded.data_dir.clone(),
            tcp_port: config.embedded.tcp_port,
            http_port: config.embedded.http_port,
        };

        let remote = RemoteConnectorConfig {
            addresses: config.remote.addresses.clone(),
            protocol: config.remote.protocol.clone(),
            username: config.remote.username.clone(),
            password: config.remote.password.clone(),
        };

        ConnectorConfig {
            mode,
            embedded,
            remote,
        }
    }
}
