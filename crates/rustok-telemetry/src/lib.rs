use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use once_cell::sync::OnceCell;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

static PROMETHEUS_HANDLE: OnceCell<PrometheusHandle> = OnceCell::new();

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

#[derive(Debug, Clone)]
pub struct TelemetryHandles {
    pub metrics: Option<PrometheusHandle>,
}

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("failed to set global tracing subscriber")]
    SubscriberAlreadySet,
    #[error("failed to install prometheus recorder: {0}")]
    MetricsRecorder(String),
}

pub fn init(config: TelemetryConfig) -> Result<TelemetryHandles, TelemetryError> {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let fmt_layer = match config.log_format {
        LogFormat::Json => fmt::layer()
            .json()
            .with_current_span(true)
            .with_span_events(fmt::format::FmtSpan::CLOSE),
        LogFormat::Pretty => fmt::layer()
            .pretty()
            .with_current_span(true)
            .with_span_events(fmt::format::FmtSpan::CLOSE),
    };

    let subscriber = Registry::default().with(env_filter).with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|_| TelemetryError::SubscriberAlreadySet)?;

    let metrics_handle = if config.metrics {
        let handle = PrometheusBuilder::new()
            .with_prefix(config.service_name)
            .install_recorder()
            .map_err(|error| TelemetryError::MetricsRecorder(error.to_string()))?;
        let _ = PROMETHEUS_HANDLE.set(handle.clone());
        Some(handle)
    } else {
        None
    };

    Ok(TelemetryHandles {
        metrics: metrics_handle,
    })
}

pub fn metrics_handle() -> Option<PrometheusHandle> {
    PROMETHEUS_HANDLE.get().cloned()
}

pub fn current_trace_id() -> Option<String> {
    let span = tracing::Span::current();
    let context = span.context();
    let span_context = context.span().span_context();
    if span_context.is_valid() {
        Some(span_context.trace_id().to_string())
    } else {
        span.id().map(|id| id.into_u64().to_string())
    }
}
