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
use serde_json::json;
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

                if let Some(path) = vars.cli.get("output") {
                    write_rbac_report_file(path, stats)?;
                    tracing::info!(report_file = %path, "RBAC consistency report file saved");
                }
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

                if let Some(path) = vars.cli.get("report_file") {
                    write_backfill_report_file(
                        path,
                        BackfillExecutionReport {
                            dry_run,
                            candidates_total: users_without_tenant_roles.len(),
                            fixed_users,
                            failed_users,
                            users_without_roles_total_before: before.users_without_roles_total,
                            orphan_user_roles_total_before: before.orphan_user_roles_total,
                            orphan_role_permissions_total_before: before
                                .orphan_role_permissions_total,
                            users_without_roles_total_after: after.users_without_roles_total,
                            orphan_user_roles_total_after: after.orphan_user_roles_total,
                            orphan_role_permissions_total_after: after
                                .orphan_role_permissions_total,
                        },
                    )?;
                    tracing::info!(report_file = %path, "RBAC backfill report file saved");
                }

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
                let before = load_rbac_consistency_stats(ctx).await?;

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

                    let result = crate::services::auth::AuthService::remove_user_role_assignment(
                        &ctx.db,
                        &entry.user_id,
                        &entry.tenant_id,
                        entry.role.clone(),
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

                if let Some(path) = vars.cli.get("report_file") {
                    write_backfill_rollback_report_file(
                        path,
                        BackfillRollbackExecutionReport {
                            dry_run,
                            entries_total: entries.len(),
                            reverted,
                            failed,
                            users_without_roles_total_before: before.users_without_roles_total,
                            orphan_user_roles_total_before: before.orphan_user_roles_total,
                            orphan_role_permissions_total_before: before
                                .orphan_role_permissions_total,
                            users_without_roles_total_after: after.users_without_roles_total,
                            orphan_user_roles_total_after: after.orphan_user_roles_total,
                            orphan_role_permissions_total_after: after
                                .orphan_role_permissions_total,
                        },
                    )?;
                    tracing::info!(report_file = %path, "RBAC rollback report file saved");
                }

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
struct BackfillExecutionReport {
    dry_run: bool,
    candidates_total: usize,
    fixed_users: usize,
    failed_users: usize,
    users_without_roles_total_before: i64,
    orphan_user_roles_total_before: i64,
    orphan_role_permissions_total_before: i64,
    users_without_roles_total_after: i64,
    orphan_user_roles_total_after: i64,
    orphan_role_permissions_total_after: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackfillRollbackExecutionReport {
    dry_run: bool,
    entries_total: usize,
    reverted: usize,
    failed: usize,
    users_without_roles_total_before: i64,
    orphan_user_roles_total_before: i64,
    orphan_role_permissions_total_before: i64,
    users_without_roles_total_after: i64,
    orphan_user_roles_total_after: i64,
    orphan_role_permissions_total_after: i64,
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

fn write_rbac_report_file(
    path: &str,
    stats: crate::services::rbac_consistency::RbacConsistencyStats,
) -> Result<()> {
    let payload = serde_json::to_vec_pretty(&json!({
        "users_without_roles_total": stats.users_without_roles_total,
        "orphan_user_roles_total": stats.orphan_user_roles_total,
        "orphan_role_permissions_total": stats.orphan_role_permissions_total,
    }))
    .map_err(|error| {
        loco_rs::Error::string(format!("rbac report serialization failed: {error}"))
    })?;
    std::fs::write(path, payload)
        .map_err(|error| loco_rs::Error::string(format!("rbac report write failed: {error}")))?;
    Ok(())
}

fn write_backfill_report_file(path: &str, report: BackfillExecutionReport) -> Result<()> {
    let payload = serde_json::to_vec_pretty(&report).map_err(|error| {
        loco_rs::Error::string(format!(
            "rbac backfill report serialization failed: {error}"
        ))
    })?;
    std::fs::write(path, payload).map_err(|error| {
        loco_rs::Error::string(format!("rbac backfill report write failed: {error}"))
    })?;
    Ok(())
}

fn write_backfill_rollback_report_file(
    path: &str,
    report: BackfillRollbackExecutionReport,
) -> Result<()> {
    let payload = serde_json::to_vec_pretty(&report).map_err(|error| {
        loco_rs::Error::string(format!(
            "rbac backfill rollback report serialization failed: {error}"
        ))
    })?;
    std::fs::write(path, payload).map_err(|error| {
        loco_rs::Error::string(format!(
            "rbac backfill rollback report write failed: {error}"
        ))
    })?;
    Ok(())
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
        write_backfill_report_file, write_backfill_rollback_report_file, write_rbac_report_file,
        write_rollback_file, BackfillExecutionReport, BackfillRollbackEntry,
        BackfillRollbackExecutionReport,
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
    fn rbac_report_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rbac_report.json");
        let path_str = path.to_string_lossy().to_string();

        write_rbac_report_file(
            &path_str,
            crate::services::rbac_consistency::RbacConsistencyStats {
                users_without_roles_total: 3,
                orphan_user_roles_total: 2,
                orphan_role_permissions_total: 1,
            },
        )
        .unwrap();

        let payload: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(payload["users_without_roles_total"], 3);
        assert_eq!(payload["orphan_user_roles_total"], 2);
        assert_eq!(payload["orphan_role_permissions_total"], 1);
    }

    #[test]
    fn backfill_report_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rbac_backfill_report.json");
        let path_str = path.to_string_lossy().to_string();

        write_backfill_report_file(
            &path_str,
            BackfillExecutionReport {
                dry_run: true,
                candidates_total: 4,
                fixed_users: 0,
                failed_users: 0,
                users_without_roles_total_before: 4,
                orphan_user_roles_total_before: 2,
                orphan_role_permissions_total_before: 1,
                users_without_roles_total_after: 4,
                orphan_user_roles_total_after: 2,
                orphan_role_permissions_total_after: 1,
            },
        )
        .unwrap();

        let payload: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(payload["dry_run"], true);
        assert_eq!(payload["candidates_total"], 4);
        assert_eq!(payload["users_without_roles_total_before"], 4);
        assert_eq!(payload["users_without_roles_total_after"], 4);
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

    #[test]
    fn backfill_rollback_report_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("rbac_backfill_rollback_report.json");
        let path_str = path.to_string_lossy().to_string();

        write_backfill_rollback_report_file(
            &path_str,
            BackfillRollbackExecutionReport {
                dry_run: false,
                entries_total: 3,
                reverted: 2,
                failed: 1,
                users_without_roles_total_before: 8,
                orphan_user_roles_total_before: 3,
                orphan_role_permissions_total_before: 2,
                users_without_roles_total_after: 6,
                orphan_user_roles_total_after: 2,
                orphan_role_permissions_total_after: 2,
            },
        )
        .unwrap();

        let payload: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(payload["entries_total"], 3);
        assert_eq!(payload["reverted"], 2);
        assert_eq!(payload["failed"], 1);
        assert_eq!(payload["users_without_roles_total_before"], 8);
        assert_eq!(payload["users_without_roles_total_after"], 6);
    }
}
