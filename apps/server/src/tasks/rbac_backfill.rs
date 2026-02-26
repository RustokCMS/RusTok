//! RBAC Backfill Task
//!
//! Finds users without `user_roles` entries and assigns them relation RBAC
//! data based on their `users.role` field.
//!
//! Run with: `cargo loco task --name rbac_backfill`
//! Dry-run:  `cargo loco task --name rbac_backfill --args "dry_run=true"`

use async_trait::async_trait;
use loco_rs::{
    app::AppContext,
    task::{Task, TaskInfo, Vars},
    Result,
};

use crate::services::auth::AuthService;

pub struct RbacBackfillTask;

#[async_trait]
impl Task for RbacBackfillTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "rbac_backfill".to_string(),
            detail: "Backfill relation RBAC data for users missing user_roles entries".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let dry_run = vars
            .cli
            .get("dry_run")
            .map(|v| v == "true")
            .unwrap_or(false);

        if dry_run {
            tracing::info!("rbac_backfill: running in dry-run mode (no changes will be made)");
        } else {
            tracing::info!("rbac_backfill: running backfill");
        }

        let users_without_roles = AuthService::count_users_without_roles(&ctx.db).await?;
        tracing::info!(
            users_without_roles,
            dry_run,
            "rbac_backfill: users without roles before backfill"
        );

        let backfilled = AuthService::backfill_missing_role_permissions(&ctx.db, dry_run).await?;

        if dry_run {
            tracing::info!(
                backfilled,
                "rbac_backfill: dry-run complete, {} user(s) would be backfilled",
                backfilled
            );
        } else {
            tracing::info!(
                backfilled,
                "rbac_backfill: complete, {} user(s) backfilled",
                backfilled
            );
        }

        Ok(())
    }
}
