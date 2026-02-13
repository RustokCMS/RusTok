/// OpenTelemetry Integration
///
/// Provides distributed tracing через OpenTelemetry и OTLP exporter.
///
/// # Features
/// - OTLP gRPC export (Jaeger, Tempo, etc.)
/// - Context propagation
/// - Span creation и management
/// - Resource attributes (service info)
/// - Batch span processor

use opentelemetry::{
    global,
    sdk::{
        trace::{self, BatchConfig, RandomIdGenerator, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing::{span, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// OpenTelemetry configuration
#[derive(Debug, Clone)]
pub struct OtelConfig {
    /// Service name (e.g., "rustok-server")
    pub service_name: String,
    
    /// Service version (e.g., "0.1.0")
    pub service_version: String,
    
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub otlp_endpoint: String,
    
    /// Environment (e.g., "development", "production")
    pub environment: String,
    
    /// Sampling rate (0.0-1.0, where 1.0 = 100%)
    pub sampling_rate: f64,
    
    /// Enable tracing export
    pub enabled: bool,
}

impl Default for OtelConfig {
    fn default() -> Self {
        Self {
            service_name: "rustok-server".to_string(),
            service_version: "0.1.0".to_string(),
            otlp_endpoint: "http://localhost:4317".to_string(),
            environment: "development".to_string(),
            sampling_rate: 1.0, // 100% sampling by default
            enabled: true,
        }
    }
}

impl OtelConfig {
    /// Load from environment variables
    pub fn from_env() -> Self {
        Self {
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "rustok-server".to_string()),
            service_version: std::env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| "0.1.0".to_string()),
            otlp_endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
            environment: std::env::var("RUST_ENV")
                .or_else(|_| std::env::var("ENVIRONMENT"))
                .unwrap_or_else(|_| "development".to_string()),
            sampling_rate: std::env::var("OTEL_SAMPLING_RATE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1.0),
            enabled: std::env::var("OTEL_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }
}

/// Initialize OpenTelemetry tracing
///
/// # Example
/// ```no_run
/// use rustok_telemetry::otel::{OtelConfig, init_tracing};
///
/// # tokio_test::block_on(async {
/// let config = OtelConfig {
///     service_name: "my-service".to_string(),
///     otlp_endpoint: "http://localhost:4317".to_string(),
///     ..Default::default()
/// };
///
/// init_tracing(config).await.expect("Failed to initialize tracing");
/// # })
/// ```
pub async fn init_tracing(config: OtelConfig) -> Result<(), OtelError> {
    if !config.enabled {
        tracing::info!("OpenTelemetry tracing disabled");
        return Ok(());
    }

    // Create resource with service information
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", config.service_version.clone()),
        KeyValue::new("deployment.environment", config.environment.clone()),
    ]);

    // Configure batch span processor
    let batch_config = BatchConfig::default()
        .with_max_queue_size(2048)
        .with_max_export_batch_size(512)
        .with_scheduled_delay(std::time::Duration::from_secs(5));

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint);

    // Build tracer provider
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(config.sampling_rate))
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(32)
                .with_resource(resource),
        )
        .with_batch_config(batch_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| OtelError::InitFailed(e.to_string()))?;

    // Set global tracer provider
    global::set_tracer_provider(tracer.tracer_provider().clone());

    tracing::info!(
        service_name = %config.service_name,
        otlp_endpoint = %config.otlp_endpoint,
        environment = %config.environment,
        sampling_rate = %config.sampling_rate,
        "OpenTelemetry tracing initialized"
    );

    Ok(())
}

/// Initialize OpenTelemetry layer for tracing-subscriber
///
/// Это альтернативный подход, который работает с existing tracing subscriber.
///
/// # Example
/// ```no_run
/// use rustok_telemetry::otel::{OtelConfig, init_otel_layer};
/// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
///
/// # tokio_test::block_on(async {
/// let config = OtelConfig::default();
/// let otel_layer = init_otel_layer(config).await.expect("Failed to create OTel layer");
///
/// tracing_subscriber::registry()
///     .with(EnvFilter::new("info"))
///     .with(otel_layer)
///     .init();
/// # })
/// ```
pub async fn init_otel_layer(
    config: OtelConfig,
) -> Result<tracing_opentelemetry::OpenTelemetryLayer<tracing_subscriber::Registry, opentelemetry_sdk::trace::Tracer>, OtelError> {
    if !config.enabled {
        return Err(OtelError::Disabled);
    }

    // Create resource
    let resource = Resource::new(vec![
        KeyValue::new("service.name", config.service_name.clone()),
        KeyValue::new("service.version", config.service_version.clone()),
        KeyValue::new("deployment.environment", config.environment.clone()),
    ]);

    // Configure batch span processor
    let batch_config = BatchConfig::default()
        .with_max_queue_size(2048)
        .with_max_export_batch_size(512)
        .with_scheduled_delay(std::time::Duration::from_secs(5));

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.otlp_endpoint);

    // Build tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::TraceIdRatioBased(config.sampling_rate))
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(32)
                .with_resource(resource),
        )
        .with_batch_config(batch_config)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map_err(|e| OtelError::InitFailed(e.to_string()))?;

    // Create tracing layer
    let layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Ok(layer)
}

/// Shutdown OpenTelemetry gracefully
///
/// Flushes all pending spans before shutdown.
pub async fn shutdown() {
    tracing::info!("Shutting down OpenTelemetry");
    global::shutdown_tracer_provider();
}

/// OpenTelemetry errors
#[derive(Debug, thiserror::Error)]
pub enum OtelError {
    #[error("OpenTelemetry initialization failed: {0}")]
    InitFailed(String),
    
    #[error("OpenTelemetry is disabled")]
    Disabled,
    
    #[error("Failed to export spans: {0}")]
    ExportFailed(String),
}

/// Helper macro to create a traced span with common attributes
///
/// # Example
/// ```no_run
/// use rustok_telemetry::traced_span;
///
/// # async fn example() {
/// let tenant_id = uuid::Uuid::new_v4();
/// let user_id = uuid::Uuid::new_v4();
///
/// traced_span!("fetch_user", tenant_id = %tenant_id, user_id = %user_id, {
///     // Your code here
///     println!("Fetching user...");
/// }).await;
/// # }
/// ```
#[macro_export]
macro_rules! traced_span {
    ($name:expr, $($key:ident = $value:expr),* $(,)?, $body:block) => {{
        let span = tracing::info_span!($name, $($key = $value),*);
        let _guard = span.enter();
        $body
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otel_config_default() {
        let config = OtelConfig::default();
        assert_eq!(config.service_name, "rustok-server");
        assert_eq!(config.otlp_endpoint, "http://localhost:4317");
        assert_eq!(config.sampling_rate, 1.0);
        assert!(config.enabled);
    }

    #[test]
    fn test_otel_config_from_env() {
        std::env::set_var("OTEL_SERVICE_NAME", "test-service");
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://test:4317");
        std::env::set_var("OTEL_SAMPLING_RATE", "0.5");
        std::env::set_var("RUST_ENV", "test");

        let config = OtelConfig::from_env();
        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.otlp_endpoint, "http://test:4317");
        assert_eq!(config.sampling_rate, 0.5);
        assert_eq!(config.environment, "test");

        // Cleanup
        std::env::remove_var("OTEL_SERVICE_NAME");
        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        std::env::remove_var("OTEL_SAMPLING_RATE");
        std::env::remove_var("RUST_ENV");
    }

    #[test]
    fn test_otel_config_disabled() {
        std::env::set_var("OTEL_ENABLED", "false");
        let config = OtelConfig::from_env();
        assert!(!config.enabled);
        std::env::remove_var("OTEL_ENABLED");
    }

    #[tokio::test]
    async fn test_init_tracing_disabled() {
        let config = OtelConfig {
            enabled: false,
            ..Default::default()
        };

        let result = init_tracing(config).await;
        assert!(result.is_ok());
    }

    // Note: Full integration tests require a running OTLP collector
    // See tests/otel_integration_test.rs for integration tests
}
