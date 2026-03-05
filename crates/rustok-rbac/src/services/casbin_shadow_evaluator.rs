use crate::{has_effective_permission_in_set, ShadowCheck};
use rustok_core::Permission;

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

#[cfg(test)]
mod tests {
    use super::evaluate_casbin_shadow;
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
}
