use async_trait::async_trait;
use rustok_core::{Permission, UserRole};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionResolution {
    pub permissions: Vec<Permission>,
    pub cache_hit: bool,
}

#[async_trait]
pub trait PermissionResolver {
    type Error;

    async fn resolve_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<PermissionResolution, Self::Error>;

    async fn has_permission(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permission: &Permission,
    ) -> Result<bool, Self::Error>;

    async fn has_any_permission(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permissions: &[Permission],
    ) -> Result<bool, Self::Error>;

    async fn has_all_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        required_permissions: &[Permission],
    ) -> Result<bool, Self::Error>;

    async fn assign_role_permissions(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error>;

    async fn replace_user_role(
        &self,
        tenant_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        role: UserRole,
    ) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::PermissionResolution;
    use rustok_core::Permission;

    #[test]
    fn resolution_keeps_permissions_payload() {
        let resolved = PermissionResolution {
            permissions: vec![Permission::USERS_READ],
            cache_hit: true,
        };

        assert_eq!(resolved.permissions, vec![Permission::USERS_READ]);
        assert!(resolved.cache_hit);
    }
}
