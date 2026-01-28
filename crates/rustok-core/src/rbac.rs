use std::collections::HashSet;

use crate::permissions::{Action, Permission, Resource};
use crate::types::UserRole;

pub struct Rbac;

impl Rbac {
    pub fn permissions_for_role(role: &UserRole) -> HashSet<Permission> {
        match role {
            UserRole::SuperAdmin => Self::super_admin_permissions(),
            UserRole::Admin => Self::admin_permissions(),
            UserRole::Manager => Self::manager_permissions(),
            UserRole::Customer => Self::customer_permissions(),
        }
    }

    pub fn has_permission(role: &UserRole, permission: &Permission) -> bool {
        let permissions = Self::permissions_for_role(role);

        if permissions.contains(permission) {
            return true;
        }

        let manage_permission = Permission::new(permission.resource, Action::Manage);
        permissions.contains(&manage_permission)
    }

    pub fn has_any_permission(role: &UserRole, permissions: &[Permission]) -> bool {
        permissions.iter().any(|permission| Self::has_permission(role, permission))
    }

    pub fn has_all_permissions(role: &UserRole, permissions: &[Permission]) -> bool {
        permissions.iter().all(|permission| Self::has_permission(role, permission))
    }

    fn super_admin_permissions() -> HashSet<Permission> {
        [
            Resource::Users,
            Resource::Tenants,
            Resource::Modules,
            Resource::Settings,
            Resource::Products,
            Resource::Categories,
            Resource::Orders,
            Resource::Customers,
            Resource::Inventory,
            Resource::Discounts,
            Resource::Posts,
            Resource::Pages,
            Resource::Media,
            Resource::Comments,
            Resource::Analytics,
            Resource::Logs,
            Resource::Webhooks,
        ]
        .into_iter()
        .map(|resource| Permission::new(resource, Action::Manage))
        .collect()
    }

    fn admin_permissions() -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        let managed_resources = [
            Resource::Users,
            Resource::Settings,
            Resource::Products,
            Resource::Categories,
            Resource::Orders,
            Resource::Customers,
            Resource::Inventory,
            Resource::Discounts,
            Resource::Posts,
            Resource::Pages,
            Resource::Media,
            Resource::Comments,
            Resource::Analytics,
            Resource::Webhooks,
        ];

        for resource in managed_resources {
            permissions.insert(Permission::new(resource, Action::Manage));
        }

        permissions.insert(Permission::new(Resource::Modules, Action::Read));
        permissions.insert(Permission::new(Resource::Modules, Action::List));

        permissions
    }

    fn manager_permissions() -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        permissions.insert(Permission::PRODUCTS_CREATE);
        permissions.insert(Permission::PRODUCTS_READ);
        permissions.insert(Permission::PRODUCTS_UPDATE);
        permissions.insert(Permission::PRODUCTS_DELETE);
        permissions.insert(Permission::PRODUCTS_LIST);

        for action in [Action::Create, Action::Read, Action::Update, Action::Delete, Action::List] {
            permissions.insert(Permission::new(Resource::Categories, action));
        }

        permissions.insert(Permission::ORDERS_READ);
        permissions.insert(Permission::ORDERS_UPDATE);
        permissions.insert(Permission::ORDERS_LIST);

        permissions.insert(Permission::new(Resource::Customers, Action::Read));
        permissions.insert(Permission::new(Resource::Customers, Action::List));

        for action in [Action::Create, Action::Read, Action::Update, Action::List] {
            permissions.insert(Permission::new(Resource::Inventory, action));
        }

        permissions.insert(Permission::POSTS_CREATE);
        permissions.insert(Permission::POSTS_READ);
        permissions.insert(Permission::POSTS_UPDATE);
        permissions.insert(Permission::POSTS_DELETE);
        permissions.insert(Permission::POSTS_LIST);

        for action in [Action::Create, Action::Read, Action::Update, Action::Delete, Action::List] {
            permissions.insert(Permission::new(Resource::Media, action));
        }

        permissions.insert(Permission::ANALYTICS_READ);

        permissions
    }

    fn customer_permissions() -> HashSet<Permission> {
        let mut permissions = HashSet::new();

        permissions.insert(Permission::PRODUCTS_READ);
        permissions.insert(Permission::PRODUCTS_LIST);

        permissions.insert(Permission::new(Resource::Categories, Action::Read));
        permissions.insert(Permission::new(Resource::Categories, Action::List));

        permissions.insert(Permission::ORDERS_READ);
        permissions.insert(Permission::ORDERS_LIST);
        permissions.insert(Permission::ORDERS_CREATE);

        permissions.insert(Permission::POSTS_READ);
        permissions.insert(Permission::POSTS_LIST);

        permissions.insert(Permission::new(Resource::Comments, Action::Create));
        permissions.insert(Permission::new(Resource::Comments, Action::Read));
        permissions.insert(Permission::new(Resource::Comments, Action::List));

        permissions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionScope {
    All,
    Own,
    None,
}

impl Rbac {
    pub fn get_scope(role: &UserRole, permission: &Permission) -> PermissionScope {
        if matches!(role, UserRole::SuperAdmin | UserRole::Admin | UserRole::Manager)
            && Self::has_permission(role, permission)
        {
            return PermissionScope::All;
        }

        if matches!(role, UserRole::Customer) {
            if permission.resource == Resource::Orders && Self::has_permission(role, permission) {
                return PermissionScope::Own;
            }

            if permission.resource == Resource::Comments {
                if matches!(permission.action, Action::Update | Action::Delete) {
                    return PermissionScope::Own;
                }
                if Self::has_permission(role, permission) {
                    return PermissionScope::All;
                }
            }

            if Self::has_permission(role, permission) {
                return PermissionScope::All;
            }
        }

        PermissionScope::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manage_implies_specific_action() {
        let permission = Permission::new(Resource::Products, Action::Read);
        assert!(Rbac::has_permission(&UserRole::Admin, &permission));
    }
}
