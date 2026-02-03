use once_cell::sync::OnceCell;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};

static METRICS_HANDLE: OnceCell<MetricsHandle> = OnceCell::new();

#[derive(Clone, Debug)]
pub struct MetricsHandle;

impl MetricsHandle {
    pub fn render(&self) -> String {
        String::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub log_format: LogFormat,
    pub metrics: bool,
}

#[derive(Clone)]
pub struct TelemetryHandles {
    pub metrics: Option<MetricsHandle>,
}

impl std::fmt::Debug for TelemetryHandles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelemetryHandles")
            .field("metrics", &self.metrics.is_some())
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("failed to set global tracing subscriber")]
    SubscriberAlreadySet,
}

pub fn init(config: TelemetryConfig) -> Result<TelemetryHandles, TelemetryError> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer: Box<dyn Layer<_> + Send + Sync> = match config.log_format {
        LogFormat::Json => fmt::layer()
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .json()
            .boxed(),
        LogFormat::Pretty => fmt::layer()
            .with_span_events(fmt::format::FmtSpan::CLOSE)
            .pretty()
            .boxed(),
    };

    let subscriber = Registry::default().with(env_filter).with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| TelemetryError::SubscriberAlreadySet)?;

    let metrics_handle = if config.metrics {
        let handle = MetricsHandle;
        let _ = METRICS_HANDLE.set(handle.clone());
        Some(handle)
    } else {
        None
    };

    Ok(TelemetryHandles {
        metrics: metrics_handle,
    })
}

pub fn metrics_handle() -> Option<MetricsHandle> {
    METRICS_HANDLE.get().cloned()
}

pub fn current_trace_id() -> Option<String> {
    let span = tracing::Span::current();
    span.id().map(|id| id.into_u64().to_string())
}
