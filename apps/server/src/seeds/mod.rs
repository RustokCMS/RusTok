//! # RusToK Database Seeds
//!
//! Seed data for development and testing.
//! Run with: `cargo loco db seed`

use loco_rs::{app::AppContext, Result};
use std::path::Path;

/// Seed the database with initial data
pub async fn seed(ctx: &AppContext, path: &Path) -> Result<()> {
    let seed_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("default");

    tracing::info!(seed = %seed_name, "Running database seed...");

    match seed_name {
        "default" | "dev" => seed_development(ctx).await?,
        "test" => seed_test(ctx).await?,
        "minimal" => seed_minimal(ctx).await?,
        _ => {
            tracing::warn!(seed = %seed_name, "Unknown seed file, using default");
            seed_development(ctx).await?;
        }
    }

    tracing::info!("Database seed complete");
    Ok(())
}

/// Development seed data
async fn seed_development(_ctx: &AppContext) -> Result<()> {
    tracing::info!("Seeding development data...");

    // TODO: Add development seed data
    // - Demo tenant
    // - Test users
    // - Sample content
    // - Sample products

    Ok(())
}

/// Test seed data
async fn seed_test(_ctx: &AppContext) -> Result<()> {
    tracing::info!("Seeding test data...");

    // Minimal data for tests

    Ok(())
}

/// Minimal seed data (just essentials)
async fn seed_minimal(_ctx: &AppContext) -> Result<()> {
    tracing::info!("Seeding minimal data...");

    // Only essential data

    Ok(())
}
