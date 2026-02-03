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

    fn serialize(&self, envelope: &EventEnvelope) -> Result<Vec<u8>> {
        let payload = bincode::serialize(envelope).map_err(|error| {
            let io_error = std::io::Error::other(error);
            rustok_core::Error::Serialization(serde_json::Error::io(io_error))
        })?;
        Ok(payload)
    }
}
