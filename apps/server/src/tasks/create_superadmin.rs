//! Create SuperAdmin Task
//!
//! One-shot CLI command to create (or update) a SuperAdmin user.
//! Equivalent to `bin/magento admin:user:create` — run it once during
//! deployment or initial server setup instead of storing credentials
//! in environment files.
//!
//! # Usage
//!
//! ```sh
//! cargo loco task --name create_superadmin \
//!     --args "email=admin@example.com password=secret tenant_slug=default"
//! ```
//!
//! # Arguments
//!
//! | Argument      | Required | Description                                    |
//! |---------------|----------|------------------------------------------------|
//! | `email`       | yes      | SuperAdmin email address                       |
//! | `password`    | yes      | Plain-text password (hashed before storage)    |
//! | `tenant_slug` | no       | Tenant slug (default: `"default"`)             |
//! | `tenant_name` | no       | Tenant display name (default: `"Default"`)     |
//! | `update`      | no       | Set `update=true` to reset the password if the |
//! |               |          | user already exists                            |
//!
//! The password is **never** persisted; only its Argon2 hash is stored.

use async_trait::async_trait;
use loco_rs::{
    app::AppContext,
    task::{Task, TaskInfo, Vars},
    Result,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set};

use crate::auth::hash_password;
use crate::models::{tenants, users};
use crate::services::auth::AuthService;

pub struct CreateSuperAdminTask;

#[async_trait]
impl Task for CreateSuperAdminTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "create_superadmin".to_string(),
            detail: "Create or update the default SuperAdmin user (email=... password=...)"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let email = vars
            .cli
            .get("email")
            .map(String::as_str)
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| loco_rs::Error::string("missing required argument: email=<address>"))?;

        let password = vars
            .cli
            .get("password")
            .map(String::as_str)
            .filter(|v| !v.trim().is_empty())
            .ok_or_else(|| {
                loco_rs::Error::string("missing required argument: password=<secret>")
            })?;

        let tenant_slug = vars
            .cli
            .get("tenant_slug")
            .map(String::as_str)
            .unwrap_or("default");

        let tenant_name = vars
            .cli
            .get("tenant_name")
            .map(String::as_str)
            .unwrap_or("Default");

        let update_existing = matches!(
            vars.cli.get("update").map(String::as_str),
            Some("1") | Some("true") | Some("yes")
        );

        let tenant =
            tenants::Entity::find_or_create(&ctx.db, tenant_name, tenant_slug, None).await?;

        let password_hash = hash_password(password)?;
        // Clear the plain-text reference as early as possible.
        drop(password);

        if let Some(existing) = users::Entity::find_by_email(&ctx.db, tenant.id, email).await? {
            if !update_existing {
                tracing::info!(
                    email = %email,
                    user_id = %existing.id,
                    "SuperAdmin already exists. Pass update=true to reset the password."
                );
                return Ok(());
            }

            // Update password and promote to SuperAdmin if needed.
            let mut active: users::ActiveModel = existing.into();
            active.password_hash = Set(password_hash);
            active.role = Set(rustok_core::UserRole::SuperAdmin);
            let updated = active.update(&ctx.db).await?;

            AuthService::assign_role_permissions(
                &ctx.db,
                &updated.id,
                &tenant.id,
                rustok_core::UserRole::SuperAdmin,
            )
            .await?;

            tracing::info!(
                email = %email,
                user_id = %updated.id,
                "SuperAdmin password updated"
            );
            return Ok(());
        }

        let mut user = users::ActiveModel::new(tenant.id, email, &password_hash);
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
            "SuperAdmin created successfully"
        );

        Ok(())
    }
}
