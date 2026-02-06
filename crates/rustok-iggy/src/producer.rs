use rustok_core::events::EventEnvelope;
use rustok_core::Result;
use rustok_iggy_connector::PublishRequest;

use crate::config::IggyConfig;
use crate::serialization::EventSerializer;

pub fn build_publish_request(
    config: &IggyConfig,
    serializer: &dyn EventSerializer,
    envelope: EventEnvelope,
) -> Result<PublishRequest> {
    let topic = match envelope.event.event_type() {
        event_type if event_type.starts_with("system.") => "system",
        _ => "domain",
    };
    let partition_key = envelope.tenant_id.to_string();

    let payload = serializer.serialize(&envelope)?;

    Ok(PublishRequest {
        stream: config.topology.stream_name.clone(),
        topic: topic.to_string(),
        partition_key,
        payload,
        event_id: envelope.id.to_string(),
    })
}
