use loco_rs::prelude::*;
use moka::future::Cache;
use once_cell::sync::Lazy;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter,
};
use std::collections::HashSet;
use std::fmt::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use rustok_core::{Action, Permission, Rbac, Resource, UserRole};

use crate::models::_entities::{permissions, role_permissions, roles, user_roles};

pub struct AuthService;

static USER_PERMISSION_CACHE: Lazy<Cache<(uuid::Uuid, uuid::Uuid), Vec<Permission>>> =
    Lazy::new(|| {
        Cache::builder()
            .max_capacity(20_000)
            .time_to_live(Duration::from_secs(60))
            .build()
    });

#[derive(Debug, Clone, Copy)]
pub struct RbacResolverMetricsSnapshot {
    pub permission_cache_hits: u64,
    pub permission_cache_misses: u64,
    pub permission_checks_allowed: u64,
    pub permission_checks_denied: u64,
    pub permission_check_latency_ms_total: u64,
    pub permission_check_latency_samples: u64,
    pub permission_lookup_latency_ms_total: u64,
    pub permission_lookup_latency_samples: u64,
    pub denied_no_permissions_resolved: u64,
    pub denied_missing_permissions: u64,
    pub denied_unknown: u64,
}

static RBAC_PERMISSION_CACHE_HITS: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_CACHE_MISSES: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_CHECKS_ALLOWED: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_CHECKS_DENIED: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_CHECK_LATENCY_MS_TOTAL: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_CHECK_LATENCY_SAMPLES: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_LOOKUP_LATENCY_MS_TOTAL: AtomicU64 = AtomicU64::new(0);
static RBAC_PERMISSION_LOOKUP_LATENCY_SAMPLES: AtomicU64 = AtomicU64::new(0);
static RBAC_DENIED_NO_PERMISSIONS_RESOLVED: AtomicU64 = AtomicU64::new(0);
static RBAC_DENIED_MISSING_PERMISSIONS: AtomicU64 = AtomicU64::new(0);
static RBAC_DENIED_UNKNOWN: AtomicU64 = AtomicU64::new(0);

impl AuthService {
    fn has_effective_permission(
        user_permissions: &[Permission],
        required_permission: &Permission,
    ) -> bool {
        user_permissions.contains(required_permission)
            || user_permissions.contains(&Permission::new(
                required_permission.resource,
                Action::Manage,
            ))
    }

    fn missing_permissions(
        user_permissions: &[Permission],
        required_permissions: &[Permission],
    ) -> Vec<Permission> {
        required_permissions
            .iter()
            .copied()
            .filter(|permission| !Self::has_effective_permission(user_permissions, permission))
            .collect()
    }

    fn denied_reason(
        user_permissions: &[Permission],
        missing_permissions: &[Permission],
    ) -> String {
        if user_permissions.is_empty() {
            return "no_permissions_resolved".to_string();
        }

        if missing_permissions.is_empty() {
            return "unknown".to_string();
        }

        let mut reason = String::from("missing_permissions:");
        for (index, permission) in missing_permissions.iter().enumerate() {
            if index > 0 {
                reason.push(',');
            }
            let _ = write!(&mut reason, "{}", permission);
        }

        reason
    }

    fn denied_reason_bucket(denied_reason: &str) {
        if denied_reason == "no_permissions_resolved" {
            RBAC_DENIED_NO_PERMISSIONS_RESOLVED.fetch_add(1, Ordering::Relaxed);
            return;
        }

        if denied_reason.starts_with("missing_permissions:") {
            RBAC_DENIED_MISSING_PERMISSIONS.fetch_add(1, Ordering::Relaxed);
            return;
        }

        RBAC_DENIED_UNKNOWN.fetch_add(1, Ordering::Relaxed);
    }

    fn cache_key(tenant_id: &uuid::Uuid, user_id: &uuid::Uuid) -> (uuid::Uuid, uuid::Uuid) {
        (*tenant_id, *user_id)
    }

    pub async fn invalidate_user_permissions_cache(tenant_id: &uuid::Uuid, user_id: &uuid::Uuid) {
        USER_PERMISSION_CACHE
            .invalidate(&Self::cache_key(tenant_id, user_id))
            .await;
    }

    fn record_permission_check_result(allowed: bool) {
        if allowed {
            RBAC_PERMISSION_CHECKS_ALLOWED.fetch_add(1, Ordering::Relaxed);
        } else {
            RBAC_PERMISSION_CHECKS_DENIED.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn record_permission_check_latency(latency_ms: u64) {
        RBAC_PERMISSION_CHECK_LATENCY_MS_TOTAL.fetch_add(latency_ms, Ordering::Relaxed);
        RBAC_PERMISSION_CHECK_LATENCY_SAMPLES.fetch_add(1, Ordering::Relaxed);
    }

    fn record_permission_lookup_latency(latency_ms: u64) {
        RBAC_PERMISSION_LOOKUP_LATENCY_MS_TOTAL.fetch_add(latency_ms, Ordering::Relaxed);
        RBAC_PERMISSION_LOOKUP_LATENCY_SAMPLES.fetch_add(1, Ordering::Relaxed);
    }

    pub fn metrics_snapshot() -> RbacResolverMetricsSnapshot {
        RbacResolverMetricsSnapshot {
            permission_cache_hits: RBAC_PERMISSION_CACHE_HITS.load(Ordering::Relaxed),
            permission_cache_misses: RBAC_PERMISSION_CACHE_MISSES.load(Ordering::Relaxed),
            permission_checks_allowed: RBAC_PERMISSION_CHECKS_ALLOWED.load(Ordering::Relaxed),
            permission_checks_denied: RBAC_PERMISSION_CHECKS_DENIED.load(Ordering::Relaxed),
            permission_check_latency_ms_total: RBAC_PERMISSION_CHECK_LATENCY_MS_TOTAL
                .load(Ordering::Relaxed),
            permission_check_latency_samples: RBAC_PERMISSION_CHECK_LATENCY_SAMPLES
                .load(Ordering::Relaxed),
            permission_lookup_latency_ms_total: RBAC_PERMISSION_LOOKUP_LATENCY_MS_TOTAL
                .load(Ordering::Relaxed),
            permission_lookup_latency_samples: RBAC_PERMISSION_LOOKUP_LATENCY_SAMPLES
                .load(Ordering::Relaxed),
            denied_no_permissions_resolved: RBAC_DENIED_NO_PERMISSIONS_RESOLVED
                .load(Ordering::Relaxed),
            denied_missing_permissions: RBAC_DENIED_MISSING_PERMISSIONS.load(Ordering::Relaxed),
            denied_unknown: RBAC_DENIED_UNKNOWN.load(Ordering::Relaxed),
        }
    }

    pub async fn has_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permission: &Permission,
    ) -> Result<bool> {
        let started_at = Instant::now();
        let user_permissions = Self::get_user_permissions(db, tenant_id, user_id).await?;
        let allowed = Self::has_effective_permission(&user_permissions, required_permission);
        let missing_permissions = if allowed {
            Vec::new()
        } else {
            vec![*required_permission]
        };
        let denied_reason = Self::denied_reason(&user_permissions, &missing_permissions);
        let latency_ms = started_at.elapsed().as_millis() as u64;

        debug!(
            tenant_id = %tenant_id,
            user_id = %user_id,
            required_permission = %required_permission,
            permissions_count = user_permissions.len(),
            denied_reason = %denied_reason,
            allowed,
            latency_ms,
            "rbac resolver decision (single permission check)"
        );

        if !allowed {
            warn!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                required_permission = %required_permission,
                denied_reason = %denied_reason,
                "rbac deny: missing required permission"
            );
        }

        Self::record_permission_check_result(allowed);
        Self::record_permission_check_latency(latency_ms);
        if !allowed {
            Self::denied_reason_bucket(&denied_reason);
        }

        Ok(allowed)
    }

    pub async fn has_any_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permissions: &[Permission],
    ) -> Result<bool> {
        let started_at = Instant::now();

        if required_permissions.is_empty() {
            return Ok(true);
        }

        let user_permissions = Self::get_user_permissions(db, tenant_id, user_id).await?;
        let allowed = required_permissions
            .iter()
            .any(|permission| Self::has_effective_permission(&user_permissions, permission));
        let missing_permissions = if allowed {
            Vec::new()
        } else {
            required_permissions.to_vec()
        };
        let denied_reason = Self::denied_reason(&user_permissions, &missing_permissions);
        let latency_ms = started_at.elapsed().as_millis() as u64;

        debug!(
            tenant_id = %tenant_id,
            user_id = %user_id,
            required_permissions = ?required_permissions,
            permissions_count = user_permissions.len(),
            denied_reason = %denied_reason,
            allowed,
            latency_ms,
            "rbac resolver decision (any-permission check)"
        );

        if !allowed {
            warn!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                required_permissions = ?required_permissions,
                denied_reason = %denied_reason,
                "rbac deny: none of required permissions granted"
            );
        }

        Self::record_permission_check_result(allowed);
        Self::record_permission_check_latency(latency_ms);
        if !allowed {
            Self::denied_reason_bucket(&denied_reason);
        }

        Ok(allowed)
    }

    pub async fn has_all_permissions(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permissions: &[Permission],
    ) -> Result<bool> {
        let started_at = Instant::now();

        if required_permissions.is_empty() {
            return Ok(true);
        }

        let user_permissions = Self::get_user_permissions(db, tenant_id, user_id).await?;
        let missing_permissions =
            Self::missing_permissions(&user_permissions, required_permissions);
        let allowed = missing_permissions.is_empty();
        let denied_reason = Self::denied_reason(&user_permissions, &missing_permissions);
        let latency_ms = started_at.elapsed().as_millis() as u64;

        debug!(
            tenant_id = %tenant_id,
            user_id = %user_id,
            required_permissions = ?required_permissions,
            permissions_count = user_permissions.len(),
            denied_reason = %denied_reason,
            missing_permissions = ?missing_permissions,
            allowed,
            latency_ms,
            "rbac resolver decision (all-permissions check)"
        );

        if !allowed {
            warn!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                required_permissions = ?required_permissions,
                denied_reason = %denied_reason,
                missing_permissions = ?missing_permissions,
                "rbac deny: not all required permissions granted"
            );
        }

        Self::record_permission_check_result(allowed);
        Self::record_permission_check_latency(latency_ms);
        if !allowed {
            Self::denied_reason_bucket(&denied_reason);
        }

        Ok(allowed)
    }

    pub async fn get_user_permissions(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<Vec<Permission>> {
        let started_at = Instant::now();
        let cache_key = Self::cache_key(tenant_id, user_id);

        if let Some(cached_permissions) = USER_PERMISSION_CACHE.get(&cache_key).await {
            RBAC_PERMISSION_CACHE_HITS.fetch_add(1, Ordering::Relaxed);
            debug!(
                tenant_id = %tenant_id,
                user_id = %user_id,
                cache_hit = true,
                permissions_count = cached_permissions.len(),
                latency_ms = started_at.elapsed().as_millis(),
                "rbac resolver permission lookup"
            );

            Self::record_permission_lookup_latency(started_at.elapsed().as_millis() as u64);

            return Ok(cached_permissions);
        }

        RBAC_PERMISSION_CACHE_MISSES.fetch_add(1, Ordering::Relaxed);

        let resolved_permissions =
            Self::load_user_permissions_from_db(db, tenant_id, user_id).await?;
        USER_PERMISSION_CACHE
            .insert(cache_key, resolved_permissions.clone())
            .await;

        debug!(
            tenant_id = %tenant_id,
            user_id = %user_id,
            cache_hit = false,
            permissions_count = resolved_permissions.len(),
            latency_ms = started_at.elapsed().as_millis(),
            "rbac resolver permission lookup"
        );

        Self::record_permission_lookup_latency(started_at.elapsed().as_millis() as u64);

        Ok(resolved_permissions)
    }

    async fn load_user_permissions_from_db(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<Vec<Permission>> {
        let user_role_models = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(*user_id))
            .all(db)
            .await?;

        if user_role_models.is_empty() {
            return Ok(vec![]);
        }

        let role_ids: Vec<uuid::Uuid> = user_role_models
            .into_iter()
            .map(|user_role| user_role.role_id)
            .collect();

        let tenant_role_models = roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .filter(roles::Column::Id.is_in(role_ids))
            .all(db)
            .await?;

        if tenant_role_models.is_empty() {
            return Ok(vec![]);
        }

        let tenant_role_ids: Vec<uuid::Uuid> =
            tenant_role_models.into_iter().map(|role| role.id).collect();

        let role_permission_models = role_permissions::Entity::find()
            .filter(role_permissions::Column::RoleId.is_in(tenant_role_ids))
            .all(db)
            .await?;

        if role_permission_models.is_empty() {
            return Ok(vec![]);
        }

        let permission_ids: Vec<uuid::Uuid> = role_permission_models
            .into_iter()
            .map(|role_permission| role_permission.permission_id)
            .collect();

        let permission_models = permissions::Entity::find()
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Id.is_in(permission_ids))
            .all(db)
            .await?;

        let mut result = HashSet::new();
        for permission in permission_models {
            let resource = permission
                .resource
                .parse::<Resource>()
                .map_err(Error::BadRequest)?;
            let action = permission
                .action
                .parse::<Action>()
                .map_err(Error::BadRequest)?;
            result.insert(Permission::new(resource, action));
        }

        Ok(result.into_iter().collect())
    }

    pub async fn assign_role_permissions(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
        tenant_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<()> {
        let role_model = Self::get_or_create_role(db, tenant_id, &role).await?;

        user_roles::Entity::insert(user_roles::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            user_id: ActiveValue::Set(*user_id),
            role_id: ActiveValue::Set(role_model.id),
        })
        .on_conflict(
            OnConflict::columns([user_roles::Column::UserId, user_roles::Column::RoleId])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

        for permission in Rbac::permissions_for_role(&role).iter() {
            let permission_model =
                Self::get_or_create_permission(db, tenant_id, permission).await?;

            role_permissions::Entity::insert(role_permissions::ActiveModel {
                id: ActiveValue::Set(rustok_core::generate_id()),
                role_id: ActiveValue::Set(role_model.id),
                permission_id: ActiveValue::Set(permission_model.id),
            })
            .on_conflict(
                OnConflict::columns([
                    role_permissions::Column::RoleId,
                    role_permissions::Column::PermissionId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .exec(db)
            .await?;
        }

        Self::invalidate_user_permissions_cache(tenant_id, user_id).await;

        Ok(())
    }

    pub async fn replace_user_role(
        db: &DatabaseConnection,
        user_id: &uuid::Uuid,
        tenant_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<()> {
        let tenant_role_models = roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .all(db)
            .await?;

        let tenant_role_ids: Vec<uuid::Uuid> = tenant_role_models
            .into_iter()
            .map(|tenant_role| tenant_role.id)
            .collect();

        if !tenant_role_ids.is_empty() {
            user_roles::Entity::delete_many()
                .filter(user_roles::Column::UserId.eq(*user_id))
                .filter(user_roles::Column::RoleId.is_in(tenant_role_ids))
                .exec(db)
                .await?;
        }

        Self::invalidate_user_permissions_cache(tenant_id, user_id).await;

        Self::assign_role_permissions(db, user_id, tenant_id, role).await
    }

    async fn get_or_create_role(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        role: &UserRole,
    ) -> Result<roles::Model> {
        let role_slug = role.to_string();

        if let Some(existing) = roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .filter(roles::Column::Slug.eq(&role_slug))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        roles::Entity::insert(roles::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            tenant_id: ActiveValue::Set(*tenant_id),
            name: ActiveValue::Set(role_slug.clone()),
            slug: ActiveValue::Set(role_slug),
            description: ActiveValue::Set(None),
            is_system: ActiveValue::Set(true),
            created_at: ActiveValue::NotSet,
            updated_at: ActiveValue::NotSet,
        })
        .on_conflict(
            OnConflict::columns([roles::Column::TenantId, roles::Column::Slug])
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

        roles::Entity::find()
            .filter(roles::Column::TenantId.eq(*tenant_id))
            .filter(roles::Column::Slug.eq(role.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| Error::InternalServerError("role upsert failed".to_string()))
    }

    async fn get_or_create_permission(
        db: &DatabaseConnection,
        tenant_id: &uuid::Uuid,
        permission: &Permission,
    ) -> Result<permissions::Model> {
        let resource_str = permission.resource.to_string();
        let action_str = permission.action.to_string();

        if let Some(existing) = permissions::Entity::find()
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Resource.eq(&resource_str))
            .filter(permissions::Column::Action.eq(&action_str))
            .one(db)
            .await?
        {
            return Ok(existing);
        }

        permissions::Entity::insert(permissions::ActiveModel {
            id: ActiveValue::Set(rustok_core::generate_id()),
            tenant_id: ActiveValue::Set(*tenant_id),
            resource: ActiveValue::Set(resource_str.clone()),
            action: ActiveValue::Set(action_str.clone()),
            description: ActiveValue::Set(None),
            created_at: ActiveValue::NotSet,
        })
        .on_conflict(
            OnConflict::columns([
                permissions::Column::TenantId,
                permissions::Column::Resource,
                permissions::Column::Action,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec(db)
        .await?;

        permissions::Entity::find()
            .filter(permissions::Column::TenantId.eq(*tenant_id))
            .filter(permissions::Column::Resource.eq(resource_str))
            .filter(permissions::Column::Action.eq(action_str))
            .one(db)
            .await?
            .ok_or_else(|| Error::InternalServerError("permission upsert failed".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::AuthService;
    use rustok_core::{Action, Permission, Resource};

    #[test]
    fn denied_reason_reports_no_permissions_resolved() {
        let denied_reason = AuthService::denied_reason(&[], &[Permission::USERS_READ]);
        assert_eq!(denied_reason, "no_permissions_resolved");
    }

    #[test]
    fn missing_permissions_respects_manage_wildcard() {
        let user_permissions = vec![Permission::new(Resource::Users, Action::Manage)];
        let required_permissions = vec![Permission::USERS_READ, Permission::USERS_UPDATE];

        let missing_permissions =
            AuthService::missing_permissions(&user_permissions, &required_permissions);

        assert!(missing_permissions.is_empty());
    }

    #[test]
    fn denied_reason_bucket_classifies_missing_permissions() {
        let denied_reason =
            AuthService::denied_reason(&[Permission::USERS_READ], &[Permission::USERS_UPDATE]);
        assert!(denied_reason.starts_with("missing_permissions:"));
    }
}
