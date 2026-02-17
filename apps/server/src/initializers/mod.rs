//! # RusToK Server Initializers
//!
//! Third-party service initialization and setup.
//! Run during application startup before routes are mounted.

use loco_rs::{app::AppContext, initializer::Initializer, Result};
use std::vec::Vec;

pub mod telemetry;

/// Create and return all initializers
pub async fn create(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
    let initializers: Vec<Box<dyn Initializer>> = vec![Box::new(telemetry::TelemetryInitializer)];

    Ok(initializers)
}
