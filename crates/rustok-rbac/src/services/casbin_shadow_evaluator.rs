use crate::{has_effective_permission_in_set, ShadowCheck};
use rustok_core::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CasbinShadowDecision {
    pub relation_allowed: bool,
    pub casbin_allowed: bool,
}

impl CasbinShadowDecision {
    pub fn mismatch(self) -> bool {
        self.relation_allowed != self.casbin_allowed
    }
}

pub fn evaluate_casbin_shadow(
    _tenant_id: &uuid::Uuid,
    resolved_permissions: &[Permission],
    shadow_check: ShadowCheck<'_>,
) -> bool {
    match shadow_check {
        ShadowCheck::Single(permission) => {
            has_effective_permission_in_set(resolved_permissions, permission)
        }
        ShadowCheck::Any(required_permissions) => required_permissions
            .iter()
            .any(|permission| has_effective_permission_in_set(resolved_permissions, permission)),
        ShadowCheck::All(required_permissions) => required_permissions
            .iter()
            .all(|permission| has_effective_permission_in_set(resolved_permissions, permission)),
    }
}

pub fn compare_casbin_shadow_decision(
    tenant_id: &uuid::Uuid,
    resolved_permissions: &[Permission],
    shadow_check: ShadowCheck<'_>,
    relation_allowed: bool,
) -> CasbinShadowDecision {
    CasbinShadowDecision {
        relation_allowed,
        casbin_allowed: evaluate_casbin_shadow(tenant_id, resolved_permissions, shadow_check),
    }
}

pub fn permissions_for_shadow_check(shadow_check: ShadowCheck<'_>) -> Vec<Permission> {
    match shadow_check {
        ShadowCheck::Single(required_permission) => vec![*required_permission],
        ShadowCheck::Any(required_permissions) | ShadowCheck::All(required_permissions) => {
            required_permissions.to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_casbin_shadow_decision, evaluate_casbin_shadow, permissions_for_shadow_check,
    };
    use crate::ShadowCheck;
    use rustok_core::Permission;

    #[test]
    fn casbin_shadow_allows_single_matching_permission() {
        let result = evaluate_casbin_shadow(
            &uuid::Uuid::new_v4(),
            &[Permission::USERS_READ],
            ShadowCheck::Single(&Permission::USERS_READ),
        );

        assert!(result);
    }

    #[test]
    fn casbin_shadow_denies_missing_permission() {
        let result = evaluate_casbin_shadow(
            &uuid::Uuid::new_v4(),
            &[Permission::USERS_READ],
            ShadowCheck::Single(&Permission::USERS_UPDATE),
        );

        assert!(!result);
    }

    #[test]
    fn casbin_shadow_any_all_respect_manage_wildcard() {
        let tenant_id = uuid::Uuid::new_v4();
        let permissions = [Permission::USERS_MANAGE];

        let allows_any = evaluate_casbin_shadow(
            &tenant_id,
            &permissions,
            ShadowCheck::Any(&[Permission::USERS_READ, Permission::USERS_DELETE]),
        );
        let allows_all = evaluate_casbin_shadow(
            &tenant_id,
            &permissions,
            ShadowCheck::All(&[Permission::USERS_READ, Permission::USERS_UPDATE]),
        );

        assert!(allows_any);
        assert!(allows_all);
    }

    #[test]
    fn compare_reports_mismatch_state() {
        let tenant_id = uuid::Uuid::new_v4();
        let decision = compare_casbin_shadow_decision(
            &tenant_id,
            &[Permission::USERS_READ],
            ShadowCheck::Single(&Permission::USERS_READ),
            false,
        );

        assert!(decision.mismatch());
        assert!(decision.casbin_allowed);
        assert!(!decision.relation_allowed);
    }

    #[test]
    fn permissions_for_shadow_check_returns_flat_set() {
        let single = permissions_for_shadow_check(ShadowCheck::Single(&Permission::USERS_READ));
        let any = permissions_for_shadow_check(ShadowCheck::Any(&[
            Permission::USERS_READ,
            Permission::USERS_UPDATE,
        ]));

        assert_eq!(single, vec![Permission::USERS_READ]);
        assert_eq!(any, vec![Permission::USERS_READ, Permission::USERS_UPDATE]);
    }
}
