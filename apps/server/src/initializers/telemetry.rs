//! Telemetry Initializer
//!
//! Initializes OpenTelemetry and tracing infrastructure.

use async_trait::async_trait;
use loco_rs::{app::AppContext, initializer::Initializer, Result};

/// Initializes telemetry systems
pub struct TelemetryInitializer;

#[async_trait]
impl Initializer for TelemetryInitializer {
    fn name(&self) -> String {
        "telemetry".to_string()
    }

    async fn before_run(&self, _ctx: &AppContext) -> Result<()> {
        tracing::info!("Initializing telemetry systems...");

        // Telemetry is already initialized in main.rs via rustok_telemetry
        // This initializer serves as a hook for any additional setup

        tracing::info!("Telemetry initialization complete");
        Ok(())
    }
}
