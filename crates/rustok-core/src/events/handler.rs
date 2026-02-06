use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::{broadcast, Semaphore};
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn, Instrument};

use super::bus::EventBus;
use super::types::{DomainEvent, EventEnvelope};
use crate::Error;

pub type HandlerResult = Result<(), Error>;

#[async_trait]
pub trait EventHandler: Send + Sync + 'static {
    fn name(&self) -> &'static str;

    fn handles(&self, event: &DomainEvent) -> bool;

    async fn handle(&self, envelope: &EventEnvelope) -> HandlerResult;

    async fn on_error(&self, envelope: &EventEnvelope, error: &Error) {
        error!(
            handler = self.name(),
            event_type = envelope.event.event_type(),
            event_id = %envelope.id,
            error = %error,
            "Event handler error"
        );
    }
}

#[derive(Clone, Debug)]
pub struct DispatcherConfig {
    pub fail_fast: bool,
    pub max_concurrent: usize,
    pub retry_count: usize,
    pub retry_delay_ms: u64,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            max_concurrent: 10,
            retry_count: 0,
            retry_delay_ms: 100,
        }
    }
}

pub struct EventDispatcher {
    bus: EventBus,
    handlers: Vec<Arc<dyn EventHandler>>,
    config: DispatcherConfig,
}

impl EventDispatcher {
    pub fn new(bus: EventBus) -> Self {
        Self {
            bus,
            handlers: Vec::new(),
            config: DispatcherConfig::default(),
        }
    }

    pub fn with_config(bus: EventBus, config: DispatcherConfig) -> Self {
        Self {
            bus,
            handlers: Vec::new(),
            config,
        }
    }

    pub fn register<H: EventHandler>(&mut self, handler: H) -> &mut Self {
        info!(handler = handler.name(), "Registering event handler");
        self.handlers.push(Arc::new(handler));
        self
    }

    pub fn register_boxed(&mut self, handler: Arc<dyn EventHandler>) -> &mut Self {
        info!(handler = handler.name(), "Registering event handler");
        self.handlers.push(handler);
        self
    }

    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    pub fn start(self) -> RunningDispatcher {
        let handlers = self.handlers;
        let config = self.config;
        let mut receiver = self.bus.subscribe();
        let bus = self.bus.clone();

        let handle = tokio::spawn(
            async move {
                info!(handlers = handlers.len(), "Event dispatcher started");
                let max_concurrent = config.max_concurrent.max(1);
                let semaphore = Arc::new(Semaphore::new(max_concurrent));

                loop {
                    match receiver.recv().await {
                        Ok(envelope) => {
                            let span = tracing::info_span!(
                                "event_dispatch",
                                event_type = envelope.event.event_type(),
                                event_id = %envelope.id,
                                tenant_id = %envelope.tenant_id
                            );

                            Self::dispatch_to_handlers(
                                &envelope,
                                &handlers,
                                &config,
                                Arc::clone(&semaphore),
                            )
                            .instrument(span)
                            .await;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!(skipped = skipped, "Event dispatcher lagged, skipped events");
                        }
                        Err(broadcast::error::RecvError::Closed) => {
                            info!("Event bus closed, stopping dispatcher");
                            break;
                        }
                    }
                }
            }
            .in_current_span(),
        );

        RunningDispatcher { handle, bus }
    }

    async fn dispatch_to_handlers(
        envelope: &EventEnvelope,
        handlers: &[Arc<dyn EventHandler>],
        config: &DispatcherConfig,
        semaphore: Arc<Semaphore>,
    ) {
        let matching_handlers: Vec<_> = handlers
            .iter()
            .filter(|handler| handler.handles(&envelope.event))
            .cloned()
            .collect();

        if matching_handlers.is_empty() {
            debug!(
                event_type = envelope.event.event_type(),
                "No handlers for event"
            );
            return;
        }

        debug!(
            event_type = envelope.event.event_type(),
            handler_count = matching_handlers.len(),
            "Dispatching to handlers"
        );

        if config.fail_fast {
            for handler in matching_handlers {
                let envelope = envelope.clone();
                let event_type = envelope.event.event_type().to_string();
                if let Err(error) = Self::handle_with_retry(handler, envelope, config).await {
                    error!(
                        event_type = event_type.as_str(),
                        error = %error,
                        "Fail fast enabled, stopping dispatch after handler error"
                    );
                    break;
                }
            }
            return;
        }

        for handler in matching_handlers {
            let envelope = envelope.clone();
            let config = config.clone();
            let permit = semaphore.clone().acquire_owned().await;

            tokio::spawn(async move {
                let _permit = permit;
                let _ = Self::handle_with_retry(handler, envelope, &config).await;
            });
        }
    }

    async fn handle_with_retry(
        handler: Arc<dyn EventHandler>,
        envelope: EventEnvelope,
        config: &DispatcherConfig,
    ) -> Result<(), Error> {
        let mut attempts = 0;
        let max_attempts = config.retry_count + 1;

        loop {
            attempts += 1;
            match handler.handle(&envelope).await {
                Ok(()) => {
                    debug!(
                        handler = handler.name(),
                        event_type = envelope.event.event_type(),
                        "Handler completed successfully"
                    );
                    return Ok(());
                }
                Err(error) => {
                    if attempts < max_attempts {
                        warn!(
                            handler = handler.name(),
                            attempt = attempts,
                            max_attempts = max_attempts,
                            error = %error,
                            "Handler failed, retrying"
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            config.retry_delay_ms,
                        ))
                        .await;
                    } else {
                        handler.on_error(&envelope, &error).await;
                        return Err(error);
                    }
                }
            }
        }
    }
}

pub struct RunningDispatcher {
    handle: JoinHandle<()>,
    bus: EventBus,
}

impl RunningDispatcher {
    pub fn bus(&self) -> &EventBus {
        &self.bus
    }

    pub fn stop(self) {
        self.handle.abort();
    }

    pub async fn join(self) -> Result<(), tokio::task::JoinError> {
        self.handle.await
    }
}

pub struct HandlerBuilder<F, P>
where
    F: Fn(&EventEnvelope) -> HandlerResult + Send + Sync + 'static,
    P: Fn(&DomainEvent) -> bool + Send + Sync + 'static,
{
    name: &'static str,
    predicate: P,
    handler: F,
}

impl<F, P> HandlerBuilder<F, P>
where
    F: Fn(&EventEnvelope) -> HandlerResult + Send + Sync + 'static,
    P: Fn(&DomainEvent) -> bool + Send + Sync + 'static,
{
    pub fn new(name: &'static str, predicate: P, handler: F) -> Self {
        Self {
            name,
            predicate,
            handler,
        }
    }
}

#[async_trait]
impl<F, P> EventHandler for HandlerBuilder<F, P>
where
    F: Fn(&EventEnvelope) -> HandlerResult + Send + Sync + 'static,
    P: Fn(&DomainEvent) -> bool + Send + Sync + 'static,
{
    fn name(&self) -> &'static str {
        self.name
    }

    fn handles(&self, event: &DomainEvent) -> bool {
        (self.predicate)(event)
    }

    async fn handle(&self, envelope: &EventEnvelope) -> HandlerResult {
        (self.handler)(envelope)
    }
}

#[macro_export]
macro_rules! event_handler {
    ($name:expr, $predicate:expr, $handler:expr) => {
        $crate::events::handler::HandlerBuilder::new($name, $predicate, $handler)
    };
}
