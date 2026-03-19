use crate::{build_enforcer_for_permissions, resolved_permissions_subject};
use casbin::CoreApi;
use rustok_core::Permission;

use super::permission_check::PermissionCheck;

pub async fn evaluate_casbin_permissions(
    tenant_id: &uuid::Uuid,
    resolved_permissions: &[Permission],
    permission_check: PermissionCheck<'_>,
) -> bool {
    let enforcer = match build_enforcer_for_permissions(tenant_id, resolved_permissions).await {
        Ok(enforcer) => enforcer,
        Err(_) => return false,
    };
    let tenant_domain = tenant_id.to_string();

    match permission_check {
        PermissionCheck::Single(permission) => {
            enforce_permission(&enforcer, &tenant_domain, permission)
        }
        PermissionCheck::Any(required_permissions) => required_permissions
            .iter()
            .any(|permission| enforce_permission(&enforcer, &tenant_domain, permission)),
        PermissionCheck::All(required_permissions) => required_permissions
            .iter()
            .all(|permission| enforce_permission(&enforcer, &tenant_domain, permission)),
    }
}

fn enforce_permission(
    enforcer: &casbin::Enforcer,
    tenant_domain: &str,
    permission: &Permission,
) -> bool {
    enforcer
        .enforce((
            resolved_permissions_subject(),
            tenant_domain.to_string(),
            permission.resource.to_string(),
            permission.action.to_string(),
        ))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::evaluate_casbin_permissions;
    use crate::services::permission_check::PermissionCheck;
    use rustok_core::Permission;

    #[tokio::test]
    async fn casbin_evaluator_allows_single_matching_permission() {
        let result = evaluate_casbin_permissions(
            &uuid::Uuid::new_v4(),
            &[Permission::USERS_READ],
            PermissionCheck::Single(&Permission::USERS_READ),
        )
        .await;

        assert!(result);
    }

    #[tokio::test]
    async fn casbin_evaluator_denies_missing_permission() {
        let result = evaluate_casbin_permissions(
            &uuid::Uuid::new_v4(),
            &[Permission::USERS_READ],
            PermissionCheck::Single(&Permission::USERS_UPDATE),
        )
        .await;

        assert!(!result);
    }

    #[tokio::test]
    async fn casbin_evaluator_any_all_respect_manage_wildcard() {
        let tenant_id = uuid::Uuid::new_v4();
        let permissions = [Permission::USERS_MANAGE];

        let allows_any = evaluate_casbin_permissions(
            &tenant_id,
            &permissions,
            PermissionCheck::Any(&[Permission::USERS_READ, Permission::USERS_DELETE]),
        )
        .await;
        let allows_all = evaluate_casbin_permissions(
            &tenant_id,
            &permissions,
            PermissionCheck::All(&[Permission::USERS_READ, Permission::USERS_UPDATE]),
        )
        .await;

        assert!(allows_any);
        assert!(allows_all);
    }
}
