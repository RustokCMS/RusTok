use rustok_core::{Action, Permission};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeniedReasonKind {
    NoPermissionsResolved,
    MissingPermissions,
    Unknown,
}

pub fn has_effective_permission_in_set(
    user_permissions: &[Permission],
    required_permission: &Permission,
) -> bool {
    user_permissions.contains(required_permission)
        || user_permissions.contains(&Permission::new(
            required_permission.resource,
            Action::Manage,
        ))
}

pub fn missing_permissions(
    user_permissions: &[Permission],
    required_permissions: &[Permission],
) -> Vec<Permission> {
    required_permissions
        .iter()
        .copied()
        .filter(|permission| !has_effective_permission_in_set(user_permissions, permission))
        .collect()
}

pub fn denied_reason_for_denial(
    user_permissions: &[Permission],
    missing_permissions: &[Permission],
) -> (DeniedReasonKind, String) {
    if user_permissions.is_empty() {
        return (
            DeniedReasonKind::NoPermissionsResolved,
            "no_permissions_resolved".to_string(),
        );
    }

    if missing_permissions.is_empty() {
        return (DeniedReasonKind::Unknown, "unknown".to_string());
    }

    let mut reason = String::from("missing_permissions:");
    for (index, permission) in missing_permissions.iter().enumerate() {
        if index > 0 {
            reason.push(',');
        }
        let _ = write!(&mut reason, "{}", permission);
    }

    (DeniedReasonKind::MissingPermissions, reason)
}

#[cfg(test)]
mod tests {
    use super::{
        denied_reason_for_denial, has_effective_permission_in_set, missing_permissions,
        DeniedReasonKind,
    };
    use rustok_core::{Action, Permission, Resource};

    #[test]
    fn effective_permission_supports_manage_wildcard() {
        let permissions = vec![Permission::new(Resource::Users, Action::Manage)];

        assert!(has_effective_permission_in_set(
            &permissions,
            &Permission::USERS_UPDATE,
        ));
    }

    #[test]
    fn missing_permissions_respects_manage_wildcard() {
        let permissions = vec![Permission::new(Resource::Users, Action::Manage)];
        let required = vec![Permission::USERS_READ, Permission::USERS_UPDATE];

        assert!(missing_permissions(&permissions, &required).is_empty());
    }

    #[test]
    fn denied_reason_reports_no_permissions() {
        let (reason_kind, reason) = denied_reason_for_denial(&[], &[Permission::USERS_READ]);

        assert_eq!(reason_kind, DeniedReasonKind::NoPermissionsResolved);
        assert_eq!(reason, "no_permissions_resolved");
    }

    #[test]
    fn denied_reason_reports_missing_permissions() {
        let (reason_kind, reason) =
            denied_reason_for_denial(&[Permission::USERS_READ], &[Permission::USERS_UPDATE]);

        assert_eq!(reason_kind, DeniedReasonKind::MissingPermissions);
        assert!(reason.starts_with("missing_permissions:"));
    }
}
