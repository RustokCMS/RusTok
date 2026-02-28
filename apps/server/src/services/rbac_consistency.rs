use loco_rs::{app::AppContext, Error, Result};
use rustok_core::UserRole;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RbacConsistencyStats {
    pub users_without_roles_total: i64,
    pub orphan_user_roles_total: i64,
    pub orphan_role_permissions_total: i64,
}

#[derive(Debug, Clone)]
pub struct UserBackfillCandidate {
    pub id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub role: UserRole,
}

pub async fn load_rbac_consistency_stats(ctx: &AppContext) -> Result<RbacConsistencyStats> {
    let row = ctx
        .db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT
                 (SELECT COUNT(*)
                  FROM users u
                  LEFT JOIN user_roles ur ON ur.user_id = u.id
                  WHERE ur.id IS NULL) AS users_without_roles_total,
                 (SELECT COUNT(*)
                  FROM user_roles ur
                  LEFT JOIN roles r ON r.id = ur.role_id
                  WHERE r.id IS NULL) AS orphan_user_roles_total,
                 (SELECT COUNT(*)
                  FROM role_permissions rp
                  LEFT JOIN permissions p ON p.id = rp.permission_id
                  WHERE p.id IS NULL) AS orphan_role_permissions_total"
                .to_string(),
        ))
        .await?
        .ok_or_else(|| Error::string("RBAC consistency stats query returned no rows"))?;

    Ok(RbacConsistencyStats {
        users_without_roles_total: read_metric(&row, "users_without_roles_total")?,
        orphan_user_roles_total: read_metric(&row, "orphan_user_roles_total")?,
        orphan_role_permissions_total: read_metric(&row, "orphan_role_permissions_total")?,
    })
}

pub async fn load_users_without_tenant_roles(
    ctx: &AppContext,
) -> Result<Vec<UserBackfillCandidate>> {
    let rows = ctx
        .db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT u.id, u.tenant_id, u.role
             FROM users u
             WHERE NOT EXISTS (
                 SELECT 1
                 FROM user_roles ur
                 JOIN roles r ON r.id = ur.role_id
                 WHERE ur.user_id = u.id AND r.tenant_id = u.tenant_id
             )"
            .to_string(),
        ))
        .await?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let role_raw = row.try_get::<String>("", "role")?;
        let role = role_raw.parse::<UserRole>().map_err(|error| {
            Error::string(&format!(
                "invalid user role '{role_raw}' for backfill: {error}"
            ))
        })?;

        result.push(UserBackfillCandidate {
            id: row.try_get("", "id")?,
            tenant_id: row.try_get("", "tenant_id")?,
            role,
        });
    }

    Ok(result)
}

fn read_metric(row: &sea_orm::QueryResult, column: &str) -> Result<i64> {
    row.try_get::<i64>("", column).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::RbacConsistencyStats;

    #[test]
    fn stats_default_is_zeroed() {
        let stats = RbacConsistencyStats::default();
        assert_eq!(stats.users_without_roles_total, 0);
        assert_eq!(stats.orphan_user_roles_total, 0);
        assert_eq!(stats.orphan_role_permissions_total, 0);
    }
}
