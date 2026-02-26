//! Cleanup Task
//!
//! Removes old sessions and temporary data.
//! Run with: `cargo loco task --name cleanup --args "sessions"`

use async_trait::async_trait;
use chrono::Utc;
use loco_rs::{
    app::AppContext,
    task::{Task, TaskInfo, Vars},
    Result,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::models::sessions;
use crate::services::rbac_consistency::{
    load_rbac_consistency_stats, load_users_without_tenant_roles,
};

/// Cleanup task for maintenance operations
pub struct CleanupTask;

#[async_trait]
impl Task for CleanupTask {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "cleanup".to_string(),
            detail: "Remove old sessions and temporary data".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let target = vars.cli.get("target").map_or("", String::as_str);

        match target {
            "sessions" => {
                tracing::info!("Cleaning up expired sessions...");
                let now = Utc::now();
                let result = sessions::Entity::delete_many()
                    .filter(sessions::Column::ExpiresAt.lt(now))
                    .exec(&ctx.db)
                    .await?;

                tracing::info!(deleted = result.rows_affected, "Session cleanup complete");
            }
            "cache" => {
                tracing::info!("Clearing temporary cache entries...");
                // Cache cleanup would go here
                tracing::info!("Cache cleanup complete");
            }
            "rbac-report" => {
                let stats = load_rbac_consistency_stats(ctx).await?;
                tracing::info!(
                    users_without_roles_total = stats.users_without_roles_total,
                    orphan_user_roles_total = stats.orphan_user_roles_total,
                    orphan_role_permissions_total = stats.orphan_role_permissions_total,
                    "RBAC consistency report"
                );
            }
            "rbac-backfill" => {
                let dry_run = is_flag_enabled(&vars.cli, "dry_run");
                let continue_on_error = is_flag_enabled(&vars.cli, "continue_on_error");
                let limit = parse_limit(&vars.cli)?;

                let before = load_rbac_consistency_stats(ctx).await?;
                tracing::info!(
                    users_without_roles_total = before.users_without_roles_total,
                    orphan_user_roles_total = before.orphan_user_roles_total,
                    orphan_role_permissions_total = before.orphan_role_permissions_total,
                    "RBAC consistency before backfill"
                );

                let mut users_without_tenant_roles = load_users_without_tenant_roles(ctx).await?;
                if let Some(limit) = limit {
                    users_without_tenant_roles.truncate(limit);
                }

                let mut fixed_users = 0usize;
                let mut failed_users = 0usize;

                if dry_run {
                    tracing::info!(
                        candidates = users_without_tenant_roles.len(),
                        limit = limit,
                        "RBAC backfill dry-run: no relation changes applied"
                    );
                } else {
                    for user in &users_without_tenant_roles {
                        let assign_result =
                            crate::services::auth::AuthService::assign_role_permissions(
                                &ctx.db,
                                &user.id,
                                &user.tenant_id,
                                user.role.clone(),
                            )
                            .await;

                        match assign_result {
                            Ok(()) => {
                                fixed_users += 1;
                            }
                            Err(error) => {
                                failed_users += 1;
                                tracing::error!(
                                    user_id = %user.id,
                                    tenant_id = %user.tenant_id,
                                    error = %error,
                                    "RBAC backfill failed for user"
                                );

                                if !continue_on_error {
                                    return Err(error);
                                }
                            }
                        }
                    }
                }

                let after = load_rbac_consistency_stats(ctx).await?;
                tracing::info!(
                    fixed_users,
                    failed_users,
                    candidates = users_without_tenant_roles.len(),
                    dry_run,
                    continue_on_error,
                    users_without_roles_total = after.users_without_roles_total,
                    orphan_user_roles_total = after.orphan_user_roles_total,
                    orphan_role_permissions_total = after.orphan_role_permissions_total,
                    "RBAC backfill complete"
                );
            }
            "" => {
                tracing::info!("Running full cleanup...");
                let now = Utc::now();
                let result = sessions::Entity::delete_many()
                    .filter(sessions::Column::ExpiresAt.lt(now))
                    .exec(&ctx.db)
                    .await?;

                tracing::info!(deleted = result.rows_affected, "Full cleanup complete");
            }
            _ => {
                tracing::warn!("Unknown cleanup target: {}", target);
                tracing::info!(
                    "Available targets: sessions, cache, rbac-report, rbac-backfill, or empty for full"
                );
            }
        }

        Ok(())
    }
}

fn is_flag_enabled(cli: &std::collections::HashMap<String, String>, key: &str) -> bool {
    matches!(
        cli.get(key).map(String::as_str),
        Some("1") | Some("true") | Some("yes")
    )
}

fn parse_limit(cli: &std::collections::HashMap<String, String>) -> Result<Option<usize>> {
    match cli.get("limit") {
        None => Ok(None),
        Some(raw) => raw
            .parse::<usize>()
            .map(Some)
            .map_err(|_| loco_rs::Error::string(format!("invalid limit value: {raw}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::{is_flag_enabled, parse_limit};
    use std::collections::HashMap;

    #[test]
    fn parse_limit_returns_value() {
        let cli = HashMap::from([(String::from("limit"), String::from("42"))]);

        assert_eq!(parse_limit(&cli).unwrap(), Some(42));
    }

    #[test]
    fn parse_limit_rejects_invalid_input() {
        let cli = HashMap::from([(String::from("limit"), String::from("oops"))]);

        assert!(parse_limit(&cli).is_err());
    }

    #[test]
    fn dry_run_flag_accepts_true_aliases() {
        let cli = HashMap::from([(String::from("dry_run"), String::from("yes"))]);

        assert!(is_flag_enabled(&cli, "dry_run"));
    }

    #[test]
    fn continue_on_error_flag_defaults_to_false() {
        let cli = HashMap::new();

        assert!(!is_flag_enabled(&cli, "continue_on_error"));
    }
}
