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

#[derive(Debug, Clone, Copy)]
pub enum ShadowCheck<'a> {
    Single(&'a Permission),
    Any(&'a [Permission]),
    All(&'a [Permission]),
}

impl ShadowCheck<'_> {
    pub fn as_str(self) -> &'static str {
        match self {
            ShadowCheck::Single(_) => "single",
            ShadowCheck::Any(_) => "any",
            ShadowCheck::All(_) => "all",
        }
    }

    pub fn for_each_permission(self, mut visitor: impl FnMut(Permission)) {
        match self {
            ShadowCheck::Single(permission) => visitor(*permission),
            ShadowCheck::Any(permissions) | ShadowCheck::All(permissions) => {
                for permission in permissions {
                    visitor(*permission);
                }
            }
        }
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

pub fn compare_shadow_decision(
    legacy_role: &UserRole,
    check: ShadowCheck<'_>,
    relation_allowed: bool,
) -> ShadowDecision {
    match check {
        ShadowCheck::Single(permission) => {
            compare_single_permission(legacy_role, permission, relation_allowed)
        }
        ShadowCheck::Any(permissions) => {
            compare_any_permissions(legacy_role, permissions, relation_allowed)
        }
        ShadowCheck::All(permissions) => {
            compare_all_permissions(legacy_role, permissions, relation_allowed)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compare_any_permissions, compare_shadow_decision, compare_single_permission, ShadowCheck,
    };
    use rustok_core::{Action, Permission, Resource, UserRole};

    fn permission(resource: Resource, action: Action) -> Permission {
        Permission::new(resource, action)
    }

    #[test]
    fn detects_single_permission_mismatch() {
        let required = permission(Resource::Users, Action::Delete);
        let decision = compare_single_permission(&UserRole::Manager, &required, true);

        assert!(!decision.legacy_allowed);
        assert!(decision.mismatch());
    }

    #[test]
    fn any_permissions_match_when_legacy_allows_one() {
        let required = vec![
            permission(Resource::BlogPosts, Action::Read),
            permission(Resource::Users, Action::Delete),
        ];

        let decision = compare_any_permissions(&UserRole::Manager, &required, true);

        assert!(decision.legacy_allowed);
        assert!(!decision.mismatch());
    }

    #[test]
    fn unified_shadow_check_supports_all_modes() {
        let required = vec![
            permission(Resource::BlogPosts, Action::Read),
            permission(Resource::Users, Action::Delete),
        ];

        let single =
            compare_shadow_decision(&UserRole::Manager, ShadowCheck::Single(&required[0]), true);
        let any = compare_shadow_decision(&UserRole::Manager, ShadowCheck::Any(&required), true);
        let all = compare_shadow_decision(&UserRole::Manager, ShadowCheck::All(&required), true);

        assert!(!single.mismatch());
        assert!(!any.mismatch());
        assert!(all.mismatch());
    }

    #[test]
    fn shadow_check_string_representation_is_stable() {
        let required = permission(Resource::Users, Action::Read);
        let required_set = [required];

        assert_eq!(ShadowCheck::Single(&required).as_str(), "single");
        assert_eq!(ShadowCheck::Any(&required_set).as_str(), "any");
        assert_eq!(ShadowCheck::All(&required_set).as_str(), "all");
    }

    #[test]
    fn shadow_check_for_each_permission_visits_all_variants() {
        let single_required = permission(Resource::Users, Action::Read);
        let required_set = [
            permission(Resource::Users, Action::Read),
            permission(Resource::Users, Action::Update),
        ];

        let mut single = Vec::new();
        ShadowCheck::Single(&single_required)
            .for_each_permission(|permission| single.push(permission));

        let mut many = Vec::new();
        ShadowCheck::All(&required_set).for_each_permission(|permission| many.push(permission));

        assert_eq!(single, vec![single_required]);
        assert_eq!(many, required_set.to_vec());
    }
}
