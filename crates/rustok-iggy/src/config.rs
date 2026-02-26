use rustok_iggy_connector::{
    ConnectorConfig, ConnectorMode, EmbeddedConnectorConfig, RemoteConnectorConfig,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IggyMode {
    #[default]
    Embedded,
    Remote,
}

impl std::fmt::Display for IggyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IggyMode::Embedded => write!(f, "embedded"),
            IggyMode::Remote => write!(f, "remote"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SerializationFormat {
    #[default]
    Json,
    Bincode,
}

impl std::fmt::Display for SerializationFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializationFormat::Json => write!(f, "json"),
            SerializationFormat::Bincode => write!(f, "bincode"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RemoteConfig {
    pub addresses: Vec<String>,
    pub protocol: String,
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub tls_enabled: bool,
}

impl Default for RemoteConfig {
    fn default() -> Self {
        Self {
            addresses: vec!["127.0.0.1:8090".to_string()],
            protocol: "tcp".to_string(),
            username: "iggy".to_string(),
            password: "iggy".to_string(),
            tls_enabled: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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
            persistent: config.embedded.use_binary_fallback,
        };

        let remote = RemoteConnectorConfig {
            addresses: config.remote.addresses.clone(),
            protocol: config.remote.protocol.clone(),
            username: config.remote.username.clone(),
            password: config.remote.password.clone(),
            tls_enabled: config.remote.tls_enabled,
        };

        ConnectorConfig {
            mode,
            embedded,
            remote,
            stream_name: config.topology.stream_name.clone(),
            topic_name: "domain".to_string(),
            partitions: config.topology.domain_partitions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iggy_config_defaults() {
        let config = IggyConfig::default();

        assert_eq!(config.mode, IggyMode::Embedded);
        assert_eq!(config.serialization, SerializationFormat::Json);
        assert_eq!(config.topology.stream_name, "rustok");
        assert_eq!(config.topology.domain_partitions, 8);
    }

    #[test]
    fn iggy_mode_display() {
        assert_eq!(IggyMode::Embedded.to_string(), "embedded");
        assert_eq!(IggyMode::Remote.to_string(), "remote");
    }

    #[test]
    fn serialization_format_display() {
        assert_eq!(SerializationFormat::Json.to_string(), "json");
        assert_eq!(SerializationFormat::Bincode.to_string(), "bincode");
    }

    #[test]
    fn embedded_config_defaults() {
        let config = EmbeddedConfig::default();

        assert_eq!(config.data_dir, "./data/iggy");
        assert!(config.use_binary_fallback);
        assert_eq!(config.tcp_port, 8090);
        assert_eq!(config.http_port, 3000);
    }

    #[test]
    fn remote_config_defaults() {
        let config = RemoteConfig::default();

        assert_eq!(config.addresses, vec!["127.0.0.1:8090"]);
        assert_eq!(config.protocol, "tcp");
        assert_eq!(config.username, "iggy");
        assert!(!config.tls_enabled);
    }

    #[test]
    fn config_to_connector_config_embedded() {
        let iggy_config = IggyConfig {
            mode: IggyMode::Embedded,
            ..Default::default()
        };

        let connector_config: ConnectorConfig = (&iggy_config).into();

        assert_eq!(connector_config.mode, ConnectorMode::Embedded);
        assert_eq!(connector_config.stream_name, "rustok");
        assert_eq!(connector_config.partitions, 8);
    }

    #[test]
    fn config_to_connector_config_remote() {
        let iggy_config = IggyConfig {
            mode: IggyMode::Remote,
            remote: RemoteConfig {
                addresses: vec!["192.168.1.1:8090".to_string()],
                username: "admin".to_string(),
                password: "secret".to_string(),
                tls_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let connector_config: ConnectorConfig = (&iggy_config).into();

        assert_eq!(connector_config.mode, ConnectorMode::Remote);
        assert_eq!(connector_config.remote.addresses, vec!["192.168.1.1:8090"]);
        assert!(connector_config.remote.tls_enabled);
    }

    #[test]
    fn config_serialization_roundtrip() {
        let config = IggyConfig {
            mode: IggyMode::Remote,
            serialization: SerializationFormat::Bincode,
            topology: TopologyConfig {
                stream_name: "custom-stream".to_string(),
                domain_partitions: 16,
                replication_factor: 3,
            },
            ..Default::default()
        };

        let json = serde_json::to_string(&config).unwrap();
        let parsed: IggyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.mode, IggyMode::Remote);
        assert_eq!(parsed.serialization, SerializationFormat::Bincode);
        assert_eq!(parsed.topology.stream_name, "custom-stream");
        assert_eq!(parsed.topology.domain_partitions, 16);
    }
}
