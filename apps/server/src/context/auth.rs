use rustok_core::{Permission, Rbac, UserRole};
use uuid::Uuid;

fn role_matches_permissions(role: UserRole, permissions: &[Permission]) -> bool {
    let required_permissions = Rbac::permissions_for_role(&role);
    required_permissions
        .iter()
        .all(|permission| permissions.contains(permission))
}

pub fn infer_user_role_from_permissions(permissions: &[Permission]) -> UserRole {
    if role_matches_permissions(UserRole::SuperAdmin, permissions) {
        return UserRole::SuperAdmin;
    }

    if role_matches_permissions(UserRole::Admin, permissions) {
        return UserRole::Admin;
    }

    if role_matches_permissions(UserRole::Manager, permissions) {
        return UserRole::Manager;
    }

    UserRole::Customer
}

#[derive(Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub tenant_id: Uuid,
    pub permissions: Vec<Permission>,
    // OAuth2 fields
    pub client_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub grant_type: String,
}

impl AuthContext {
    pub fn security_context(&self) -> rustok_core::SecurityContext {
        let inferred_role = infer_user_role_from_permissions(&self.permissions);
        rustok_core::SecurityContext::new(inferred_role, Some(self.user_id))
    }

    /// Check if the current context has the required scope.
    /// For direct grants (embedded/user login), scopes are empty and access is allowed.
    /// For OAuth2 tokens, scopes must include the required scope (with wildcard support).
    pub fn require_scope(&self, required: &str) -> Result<(), async_graphql::Error> {
        // Direct grant (no client_id) — no scope restriction
        if self.client_id.is_none() {
            return Ok(());
        }

        // Check scopes with wildcard matching
        if crate::services::oauth_app::scope_matches(&self.scopes, required) {
            return Ok(());
        }

        Err(async_graphql::Error::new(format!(
            "Insufficient scope: required '{}', granted: {:?}",
            required, self.scopes
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infer_role_admin_permissions() {
        let permissions = Rbac::permissions_for_role(&UserRole::Admin)
            .iter()
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(
            infer_user_role_from_permissions(&permissions),
            UserRole::Admin
        );
    }

    #[test]
    fn infer_role_customer_permissions() {
        let permissions = Rbac::permissions_for_role(&UserRole::Customer)
            .iter()
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(
            infer_user_role_from_permissions(&permissions),
            UserRole::Customer
        );
    }
}
