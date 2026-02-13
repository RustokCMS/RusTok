use rustok_core::events::EventEnvelope;
use rustok_core::Result;

use crate::config::SerializationFormat;

pub trait EventSerializer: Send + Sync {
    fn format(&self) -> SerializationFormat;
    fn serialize(&self, envelope: &EventEnvelope) -> Result<Vec<u8>>;
}

#[derive(Debug, Default)]
pub struct JsonSerializer;

impl EventSerializer for JsonSerializer {
    fn format(&self) -> SerializationFormat {
        SerializationFormat::Json
    }

    fn serialize(&self, envelope: &EventEnvelope) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(envelope)?)
    }
}

#[derive(Debug, Default)]
pub struct BincodeSerializer;

impl EventSerializer for BincodeSerializer {
    fn format(&self) -> SerializationFormat {
        SerializationFormat::Bincode
    }

    fn serialize(&self, _envelope: &EventEnvelope) -> Result<Vec<u8>> {
        Err(rustok_core::Error::Serialization(serde_json::Error::io(
            std::io::Error::other("bincode serialization is temporarily unavailable"),
        )))
    }
}
