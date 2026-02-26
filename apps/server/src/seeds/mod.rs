//! # RusToK Database Seeds
//!
//! Seed data for development and testing.
//! Run with: `cargo loco db seed`

use loco_rs::{app::AppContext, Result};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};
use std::path::Path;

use crate::auth::hash_password;
use crate::models::{tenants, users};
use crate::services::auth::AuthService;

const DEFAULT_DEV_SEED_PASSWORD: &str = "dev-password-123";

/// Seed the database with initial data
pub async fn seed(ctx: &AppContext, path: &Path) -> Result<()> {
    let seed_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("default");

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
async fn seed_development(ctx: &AppContext) -> Result<()> {
    tracing::info!("Seeding development data...");

    let demo_tenant =
        tenants::Entity::find_or_create(&ctx.db, "Demo Tenant", "demo", Some("demo.localhost"))
            .await?;

    seed_user(
        ctx,
        demo_tenant.id,
        "admin@demo.local",
        "Demo Admin",
        rustok_core::UserRole::Admin,
    )
    .await?;

    seed_user(
        ctx,
        demo_tenant.id,
        "customer@demo.local",
        "Demo Customer",
        rustok_core::UserRole::Customer,
    )
    .await?;

    for module in ["content", "commerce", "pages", "blog", "forum", "index"] {
        crate::models::tenant_modules::toggle(&ctx.db, demo_tenant.id, module, true).await?;
    }

    tracing::info!(tenant_id = %demo_tenant.id, "Development seed data ensured");

    Ok(())
}

async fn seed_user(
    ctx: &AppContext,
    tenant_id: uuid::Uuid,
    email: &str,
    name: &str,
    role: rustok_core::UserRole,
) -> Result<()> {
    if users::Entity::find_by_email(&ctx.db, tenant_id, email)
        .await?
        .is_some()
    {
        return Ok(());
    }

    let seed_password = std::env::var("RUSTOK_DEV_SEED_PASSWORD")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_DEV_SEED_PASSWORD.to_string());

    let password_hash = hash_password(&seed_password)?;
    let mut user = users::ActiveModel::new(tenant_id, email, &password_hash);
    user.name = Set(Some(name.to_string()));
    user.role = Set(role.clone());
    let user = user.insert(&ctx.db).await?;

    AuthService::assign_role_permissions(&ctx.db, &user.id, &tenant_id, role).await?;

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
