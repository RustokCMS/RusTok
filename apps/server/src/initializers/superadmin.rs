//! Default SuperAdmin Initializer
//!
//! Automatically ensures a default SuperAdmin user exists on every startup.
//! Runs before the server accepts requests — safe for all environments.
//!
//! ## Security: how to supply the password
//!
//! Passwords should **never** live in plain text files that end up in source
//! control or on disk longer than needed.  Three safe patterns are supported:
//!
//! ### 1. Docker / Kubernetes secrets (recommended for production)
//! Mount the secret as a file and point to it with `_FILE` suffix:
//! ```
//! SUPERADMIN_PASSWORD_FILE=/run/secrets/superadmin_password
//! ```
//! Docker Swarm: `docker secret create superadmin_password <(echo -n "s3cr3t")`
//! Kubernetes:   use a `Secret` mounted at the path above.
//!
//! ### 2. Runtime env injection (CI/CD, cloud providers)
//! Pass the variable directly at runtime — never store it in a committed file:
//! ```
//! SUPERADMIN_EMAIL=admin@example.com SUPERADMIN_PASSWORD=s3cr3t ./server start
//! ```
//! GitHub Actions: use `secrets.*` context.
//! AWS ECS / GCP Cloud Run / Fly.io: inject via their secret/env UI.
//!
//! ### 3. Plain env var (dev only)
//! Acceptable for local development, **not** for production.
//! ```
//! SUPERADMIN_PASSWORD=dev-only-password
//! ```
//!
//! ## Variable resolution order (each variable)
//!
//! 1. `SUPERADMIN_*_FILE`  — read contents of the pointed file
//! 2. `SUPERADMIN_*`       — direct env value
//! 3. `SEED_ADMIN_*_FILE`  — legacy file fallback
//! 4. `SEED_ADMIN_*`       — legacy direct fallback
//!
//! If `SUPERADMIN_EMAIL` cannot be resolved, the initializer skips silently.
//!
//! ## After first boot
//! Once the superadmin row exists the initializer does nothing (idempotent).
//! You can safely unset `SUPERADMIN_PASSWORD` / `SUPERADMIN_PASSWORD_FILE`
//! after the first successful start.

use async_trait::async_trait;
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;

use crate::auth::hash_password;
use crate::models::{tenants, users};
use crate::services::auth::AuthService;

pub struct SuperAdminInitializer;

/// Read a secret: check `<key>_FILE` first (Docker secrets), then `<key>` directly.
fn read_secret(key: &str) -> Option<String> {
    let file_key = format!("{key}_FILE");
    if let Ok(path) = std::env::var(&file_key) {
        let path = path.trim().to_string();
        if !path.is_empty() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    let secret = contents.trim().to_string();
                    if !secret.is_empty() {
                        return Some(secret);
                    }
                    tracing::warn!(file = %path, "{file_key} points to an empty file");
                }
                Err(err) => {
                    tracing::warn!(file = %path, error = %err, "{file_key} could not be read");
                }
            }
        }
    }
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Try primary key pair, then legacy fallback pair.
fn resolve_secret(primary: &str, legacy: &str) -> Option<String> {
    read_secret(primary).or_else(|| read_secret(legacy))
}

#[async_trait]
impl Initializer for SuperAdminInitializer {
    fn name(&self) -> String {
        "superadmin".to_string()
    }

    async fn before_run(&self, ctx: &AppContext) -> Result<()> {
        let Some(email) = resolve_secret("SUPERADMIN_EMAIL", "SEED_ADMIN_EMAIL") else {
            tracing::debug!("SUPERADMIN_EMAIL not set — skipping default superadmin setup");
            return Ok(());
        };

        let Some(password) = resolve_secret("SUPERADMIN_PASSWORD", "SEED_ADMIN_PASSWORD") else {
            tracing::warn!(
                "SUPERADMIN_EMAIL is set but SUPERADMIN_PASSWORD (or SUPERADMIN_PASSWORD_FILE) \
                 is missing — skipping superadmin setup"
            );
            return Ok(());
        };

        let tenant_slug = resolve_secret("SUPERADMIN_TENANT_SLUG", "SEED_TENANT_SLUG")
            .unwrap_or_else(|| "default".to_string());

        let tenant_name = resolve_secret("SUPERADMIN_TENANT_NAME", "SEED_TENANT_NAME")
            .unwrap_or_else(|| "Default".to_string());

        let tenant =
            tenants::Entity::find_or_create(&ctx.db, &tenant_name, &tenant_slug, None).await?;

        if users::Entity::find_by_email(&ctx.db, tenant.id, &email)
            .await?
            .is_some()
        {
            tracing::debug!(
                email = %email,
                tenant = %tenant_slug,
                "Default superadmin already exists — skipping"
            );
            return Ok(());
        }

        let password_hash = hash_password(&password)?;
        // Drop the plain-text password from memory as soon as we have the hash.
        drop(password);

        let mut user = users::ActiveModel::new(tenant.id, &email, &password_hash);
        user.role = Set(rustok_core::UserRole::SuperAdmin);
        user.name = Set(Some("Super Admin".to_string()));
        let user = user.insert(&ctx.db).await?;

        AuthService::assign_role_permissions(
            &ctx.db,
            &user.id,
            &tenant.id,
            rustok_core::UserRole::SuperAdmin,
        )
        .await?;

        tracing::info!(
            email = %email,
            tenant = %tenant_slug,
            user_id = %user.id,
            "Default superadmin created"
        );

        // Remind operators to clean up credentials from the environment.
        if std::env::var("SUPERADMIN_PASSWORD").is_ok()
            || std::env::var("SEED_ADMIN_PASSWORD").is_ok()
        {
            tracing::warn!(
                "SUPERADMIN_PASSWORD is set as a plain env var. \
                 Consider switching to SUPERADMIN_PASSWORD_FILE (Docker/K8s secrets) \
                 or removing the variable now that the superadmin has been created."
            );
        }

        Ok(())
    }
}
