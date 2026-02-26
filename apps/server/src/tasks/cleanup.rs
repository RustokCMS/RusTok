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
use rustok_core::UserRole;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
                let excluded_user_ids = parse_uuid_set(&vars.cli, "exclude_user_ids")?;
                let excluded_roles = parse_role_set(&vars.cli, "exclude_roles")?;
                let rollback_file = vars.cli.get("rollback_file").cloned();

                let before = load_rbac_consistency_stats(ctx).await?;
                tracing::info!(
                    users_without_roles_total = before.users_without_roles_total,
                    orphan_user_roles_total = before.orphan_user_roles_total,
                    orphan_role_permissions_total = before.orphan_role_permissions_total,
                    "RBAC consistency before backfill"
                );

                let mut users_without_tenant_roles = load_users_without_tenant_roles(ctx).await?;
                users_without_tenant_roles.retain(|candidate| {
                    !excluded_user_ids.contains(&candidate.id)
                        && !excluded_roles.contains(&candidate.role)
                });

                if let Some(limit) = limit {
                    users_without_tenant_roles.truncate(limit);
                }

                let mut fixed_users = 0usize;
                let mut failed_users = 0usize;
                let mut applied_entries = Vec::new();

                if dry_run {
                    tracing::info!(
                        candidates = users_without_tenant_roles.len(),
                        excluded_user_ids = excluded_user_ids.len(),
                        excluded_roles = ?excluded_roles,
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
                                applied_entries.push(BackfillRollbackEntry {
                                    user_id: user.id,
                                    tenant_id: user.tenant_id,
                                    role: user.role.clone(),
                                });
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

                    if let Some(path) = rollback_file.as_ref() {
                        write_rollback_file(path, &applied_entries)?;
                        tracing::info!(
                            rollback_file = %path,
                            entries = applied_entries.len(),
                            "RBAC backfill rollback snapshot saved"
                        );
                    }
                }

                let after = load_rbac_consistency_stats(ctx).await?;
                tracing::info!(
                    fixed_users,
                    failed_users,
                    candidates = users_without_tenant_roles.len(),
                    dry_run,
                    continue_on_error,
                    excluded_user_ids = excluded_user_ids.len(),
                    excluded_roles = ?excluded_roles,
                    rollback_file = ?rollback_file,
                    users_without_roles_total = after.users_without_roles_total,
                    orphan_user_roles_total = after.orphan_user_roles_total,
                    orphan_role_permissions_total = after.orphan_role_permissions_total,
                    "RBAC backfill complete"
                );
            }
            "rbac-backfill-rollback" => {
                let source = vars
                    .cli
                    .get("source")
                    .ok_or_else(|| loco_rs::Error::string("missing source=<rollback_file>"))?;
                let entries = read_rollback_file(source)?;
                let dry_run = is_flag_enabled(&vars.cli, "dry_run");
                let mut reverted = 0usize;
                let mut failed = 0usize;
                let continue_on_error = is_flag_enabled(&vars.cli, "continue_on_error");

                if dry_run {
                    tracing::info!(
                        source = %source,
                        entries = entries.len(),
                        "RBAC backfill rollback dry-run: no relation changes applied"
                    );
                }

                for entry in &entries {
                    if dry_run {
                        continue;
                    }

                    let result =
                        crate::services::auth::AuthService::remove_tenant_role_assignments(
                            &ctx.db,
                            &entry.user_id,
                            &entry.tenant_id,
                        )
                        .await;

                    match result {
                        Ok(()) => reverted += 1,
                        Err(error) => {
                            failed += 1;
                            tracing::error!(
                                user_id = %entry.user_id,
                                tenant_id = %entry.tenant_id,
                                error = %error,
                                "RBAC backfill rollback failed for user"
                            );
                            if !continue_on_error {
                                return Err(error);
                            }
                        }
                    }
                }

                let after = load_rbac_consistency_stats(ctx).await?;
                tracing::info!(
                    source = %source,
                    reverted,
                    failed,
                    dry_run,
                    users_without_roles_total = after.users_without_roles_total,
                    orphan_user_roles_total = after.orphan_user_roles_total,
                    orphan_role_permissions_total = after.orphan_role_permissions_total,
                    "RBAC backfill rollback complete"
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
                    "Available targets: sessions, cache, rbac-report, rbac-backfill, rbac-backfill-rollback, or empty for full"
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackfillRollbackEntry {
    user_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    role: UserRole,
}

fn write_rollback_file(path: &str, entries: &[BackfillRollbackEntry]) -> Result<()> {
    let payload = serde_json::to_vec_pretty(entries).map_err(|error| {
        loco_rs::Error::string(format!("rollback file serialization failed: {error}"))
    })?;
    std::fs::write(path, payload)
        .map_err(|error| loco_rs::Error::string(format!("rollback file write failed: {error}")))?;
    Ok(())
}

fn read_rollback_file(path: &str) -> Result<Vec<BackfillRollbackEntry>> {
    let payload = std::fs::read(path)
        .map_err(|error| loco_rs::Error::string(format!("rollback file read failed: {error}")))?;
    serde_json::from_slice(&payload)
        .map_err(|error| loco_rs::Error::string(format!("rollback file parse failed: {error}")))
}

fn is_flag_enabled(cli: &HashMap<String, String>, key: &str) -> bool {
    matches!(
        cli.get(key).map(String::as_str),
        Some("1") | Some("true") | Some("yes")
    )
}

fn parse_limit(cli: &HashMap<String, String>) -> Result<Option<usize>> {
    match cli.get("limit") {
        None => Ok(None),
        Some(raw) => raw
            .parse::<usize>()
            .map(Some)
            .map_err(|_| loco_rs::Error::string(format!("invalid limit value: {raw}"))),
    }
}

fn parse_uuid_set(cli: &HashMap<String, String>, key: &str) -> Result<HashSet<uuid::Uuid>> {
    let Some(raw) = cli.get(key) else {
        return Ok(HashSet::new());
    };

    let mut result = HashSet::new();
    for value in raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let parsed = value.parse::<uuid::Uuid>().map_err(|error| {
            loco_rs::Error::string(format!("invalid UUID in {key}: {value} ({error})"))
        })?;
        result.insert(parsed);
    }

    Ok(result)
}

fn parse_role_set(cli: &HashMap<String, String>, key: &str) -> Result<HashSet<UserRole>> {
    let Some(raw) = cli.get(key) else {
        return Ok(HashSet::new());
    };

    let mut result = HashSet::new();
    for value in raw
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let parsed = value.parse::<UserRole>().map_err(|error| {
            loco_rs::Error::string(format!("invalid role in {key}: {value} ({error})"))
        })?;
        result.insert(parsed);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{
        is_flag_enabled, parse_limit, parse_role_set, parse_uuid_set, read_rollback_file,
        write_rollback_file, BackfillRollbackEntry,
    };
    use rustok_core::UserRole;
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

    #[test]
    fn parse_uuid_set_supports_csv() {
        let a = uuid::Uuid::new_v4();
        let b = uuid::Uuid::new_v4();
        let cli = HashMap::from([(String::from("exclude_user_ids"), format!("{a}, {b}"))]);

        let parsed = parse_uuid_set(&cli, "exclude_user_ids").unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains(&a));
        assert!(parsed.contains(&b));
    }

    #[test]
    fn parse_role_set_supports_csv() {
        let cli = HashMap::from([(
            String::from("exclude_roles"),
            String::from("admin,customer"),
        )]);

        let parsed = parse_role_set(&cli, "exclude_roles").unwrap();
        assert_eq!(parsed.len(), 2);
        assert!(parsed.contains(&UserRole::Admin));
        assert!(parsed.contains(&UserRole::Customer));
    }

    #[test]
    fn rollback_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rbac_rollback.json");
        let path_str = path.to_string_lossy().to_string();
        let payload = vec![BackfillRollbackEntry {
            user_id: uuid::Uuid::new_v4(),
            tenant_id: uuid::Uuid::new_v4(),
            role: UserRole::Manager,
        }];

        write_rollback_file(&path_str, &payload).unwrap();
        let loaded = read_rollback_file(&path_str).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].user_id, payload[0].user_id);
        assert_eq!(loaded[0].tenant_id, payload[0].tenant_id);
        assert_eq!(loaded[0].role, UserRole::Manager);
    }
}
