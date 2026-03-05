use crate::{has_effective_permission_in_set, ShadowCheck};
use rustok_core::Permission;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CasbinShadowComparison {
    pub decision: CasbinShadowDecision,
    pub checked_permissions: Vec<Permission>,
}

impl CasbinShadowComparison {
    pub fn checked_permissions_total(&self) -> usize {
        self.checked_permissions.len()
    }
}

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

pub fn evaluate_casbin_shadow_comparison(
    tenant_id: &uuid::Uuid,
    resolved_permissions: &[Permission],
    shadow_check: ShadowCheck<'_>,
    relation_allowed: bool,
) -> CasbinShadowComparison {
    let decision = compare_casbin_shadow_decision(
        tenant_id,
        resolved_permissions,
        shadow_check,
        relation_allowed,
    );
    let checked_permissions = permissions_for_shadow_check(shadow_check);

    CasbinShadowComparison {
        decision,
        checked_permissions,
    }
}

pub fn permissions_for_shadow_check(shadow_check: ShadowCheck<'_>) -> Vec<Permission> {
    let mut permissions = Vec::new();
    shadow_check.for_each_permission(|permission| permissions.push(permission));
    permissions
}

#[cfg(test)]
mod tests {
    use super::{
        compare_casbin_shadow_decision, evaluate_casbin_shadow, evaluate_casbin_shadow_comparison,
        permissions_for_shadow_check,
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

    #[test]
    fn comparison_includes_decision_and_checked_permissions() {
        let tenant_id = uuid::Uuid::new_v4();
        let comparison = evaluate_casbin_shadow_comparison(
            &tenant_id,
            &[Permission::USERS_READ],
            ShadowCheck::Any(&[Permission::USERS_READ, Permission::USERS_UPDATE]),
            true,
        );

        assert!(!comparison.decision.mismatch());
        assert_eq!(
            comparison.checked_permissions,
            vec![Permission::USERS_READ, Permission::USERS_UPDATE]
        );
        assert_eq!(comparison.checked_permissions_total(), 2);
    }
}
