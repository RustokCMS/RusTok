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
        use bincode::serde::Compat;
        bincode::encode_to_vec(Compat(envelope), bincode::config::standard())
            .map_err(|err| rustok_core::Error::External(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustok_core::events::{DomainEvent, EventEnvelope};
    use uuid::Uuid;

    fn create_test_envelope() -> EventEnvelope {
        EventEnvelope::new(
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: "test".to_string(),
                author_id: None,
            },
        )
    }

    #[test]
    fn json_serializer_format() {
        let serializer = JsonSerializer;
        assert_eq!(serializer.format(), SerializationFormat::Json);
    }

    #[test]
    fn bincode_serializer_format() {
        let serializer = BincodeSerializer;
        assert_eq!(serializer.format(), SerializationFormat::Bincode);
    }

    #[test]
    fn json_serialize_event() {
        let serializer = JsonSerializer;
        let envelope = create_test_envelope();

        let result = serializer.serialize(&envelope);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty());

        let json_str = String::from_utf8(bytes).unwrap();
        assert!(json_str.contains("node.created"));
    }

    #[test]
    fn bincode_serialize_event() {
        let serializer = BincodeSerializer;
        let envelope = create_test_envelope();

        let result = serializer.serialize(&envelope);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty());
    }

    #[test]
    fn json_roundtrip() {
        let serializer = JsonSerializer;
        let envelope = create_test_envelope();

        let bytes = serializer.serialize(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(envelope.id, deserialized.id);
        assert_eq!(envelope.tenant_id, deserialized.tenant_id);
    }
}
