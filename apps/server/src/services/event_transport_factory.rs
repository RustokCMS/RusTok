use std::sync::Arc;
use std::time::Duration;

use loco_rs::app::AppContext;
use loco_rs::Result;
use rustok_core::events::{EventTransport, MemoryTransport};
use rustok_iggy::{IggyConfig, IggyTransport};
use rustok_outbox::{OutboxRelay, OutboxTransport, RelayConfig};
use tokio::task::JoinHandle;

use crate::common::settings::{EventTransportKind, RelayTargetKind, RustokSettings};

#[derive(Clone)]
pub struct EventRuntime {
    pub transport: Arc<dyn EventTransport>,
    pub relay_config: Option<RelayRuntimeConfig>,
}

#[derive(Clone)]
pub struct RelayRuntimeConfig {
    pub interval: Duration,
    pub relay: OutboxRelay,
}

pub async fn build_event_runtime(ctx: &AppContext) -> Result<EventRuntime> {
    let settings = RustokSettings::from_settings(&ctx.config.settings)
        .map_err(|error| loco_rs::Error::BadRequest(format!("Invalid rustok settings: {error}")))?;

    match settings.events.transport {
        EventTransportKind::Memory => Ok(EventRuntime {
            transport: Arc::new(MemoryTransport::new()),
            relay_config: None,
        }),
        EventTransportKind::Outbox => {
            let outbox_transport = Arc::new(OutboxTransport::new(ctx.db.clone()));
            let relay_target = resolve_relay_target(&settings).await;
            let relay_policy = &settings.events.relay_retry_policy;
            let max_attempts = if settings.events.dlq.enabled {
                settings.events.dlq.max_attempts
            } else {
                relay_policy.max_attempts
            };
            let relay_config = RelayRuntimeConfig {
                interval: Duration::from_millis(settings.events.relay_interval_ms),
                relay: OutboxRelay::new(ctx.db.clone(), relay_target).with_config(RelayConfig {
                    max_attempts,
                    backoff_base: Duration::from_millis(relay_policy.base_backoff_ms),
                    backoff_max: Duration::from_millis(relay_policy.max_backoff_ms),
                    ..RelayConfig::default()
                }),
            };

            Ok(EventRuntime {
                transport: outbox_transport,
                relay_config: Some(relay_config),
            })
        }
        EventTransportKind::Iggy => {
            let transport = IggyTransport::new(resolve_iggy_config(&settings))
                .await
                .map_err(|error| {
                    loco_rs::Error::BadRequest(format!(
                        "Failed to initialize iggy transport: {error}"
                    ))
                })?;
            Ok(EventRuntime {
                transport: Arc::new(transport),
                relay_config: None,
            })
        }
    }
}

pub fn spawn_outbox_relay_worker(config: RelayRuntimeConfig) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Err(error) = config.relay.process_pending_once().await {
                tracing::error!("Outbox relay iteration failed: {error}");
            }
            tokio::time::sleep(config.interval).await;
        }
    })
}

fn resolve_iggy_config(settings: &RustokSettings) -> IggyConfig {
    settings.events.iggy.clone()
}

async fn resolve_relay_target(settings: &RustokSettings) -> Arc<dyn EventTransport> {
    match settings.events.relay_target {
        RelayTargetKind::Memory => Arc::new(MemoryTransport::new()),
        RelayTargetKind::Iggy => match IggyTransport::new(resolve_iggy_config(settings)).await {
            Ok(transport) => Arc::new(transport),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "Failed to initialize relay_target=iggy, fallback to memory"
                );
                Arc::new(MemoryTransport::new())
            }
        },
    }
}
