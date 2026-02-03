use loco_rs::cli;
use migration::Migrator;
use rustok_server::app::App;
use rustok_telemetry::{LogFormat, TelemetryConfig};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _telemetry = rustok_telemetry::init(telemetry_config())?;
    Ok(cli::main::<App, Migrator>().await?)
}

fn telemetry_config() -> TelemetryConfig {
    let log_format = match std::env::var("RUSTOK_LOG_FORMAT").as_deref() {
        Ok("json") => LogFormat::Json,
        _ => LogFormat::Pretty,
    };
    let metrics = std::env::var("RUSTOK_METRICS")
        .map(|value| value != "0")
        .unwrap_or(true);

    TelemetryConfig {
        service_name: "rustok-server".to_string(),
        log_format,
        metrics,
    }
}
