use rustok_core::{Permission, Rbac, UserRole};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShadowDecision {
    pub legacy_allowed: bool,
    pub relation_allowed: bool,
}

impl ShadowDecision {
    pub fn mismatch(self) -> bool {
        self.legacy_allowed != self.relation_allowed
    }
}

pub fn compare_single_permission(
    legacy_role: &UserRole,
    required_permission: &Permission,
    relation_allowed: bool,
) -> ShadowDecision {
    ShadowDecision {
        legacy_allowed: Rbac::has_permission(legacy_role, required_permission),
        relation_allowed,
    }
}

pub fn compare_any_permissions(
    legacy_role: &UserRole,
    required_permissions: &[Permission],
    relation_allowed: bool,
) -> ShadowDecision {
    ShadowDecision {
        legacy_allowed: Rbac::has_any_permission(legacy_role, required_permissions),
        relation_allowed,
    }
}

pub fn compare_all_permissions(
    legacy_role: &UserRole,
    required_permissions: &[Permission],
    relation_allowed: bool,
) -> ShadowDecision {
    ShadowDecision {
        legacy_allowed: Rbac::has_all_permissions(legacy_role, required_permissions),
        relation_allowed,
    }
}

#[cfg(test)]
mod tests {
    use super::{compare_any_permissions, compare_single_permission};
    use rustok_core::{Action, Permission, Resource, UserRole};

    fn permission(resource: Resource, action: Action) -> Permission {
        Permission::new(resource, action)
    }

    #[test]
    fn detects_single_permission_mismatch() {
        let required = permission(Resource::User, Action::Delete);
        let decision = compare_single_permission(&UserRole::Editor, &required, true);

        assert!(!decision.legacy_allowed);
        assert!(decision.mismatch());
    }

    #[test]
    fn any_permissions_match_when_legacy_allows_one() {
        let required = vec![
            permission(Resource::BlogPost, Action::Read),
            permission(Resource::User, Action::Delete),
        ];

        let decision = compare_any_permissions(&UserRole::Editor, &required, true);

        assert!(decision.legacy_allowed);
        assert!(!decision.mismatch());
    }
}
