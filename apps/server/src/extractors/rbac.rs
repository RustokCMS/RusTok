use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use rustok_core::{Permission, Rbac};

use crate::extractors::auth::CurrentUser;

/// Extractor that enforces a specific permission
///
/// This extractor will:
/// 1. Extract the CurrentUser (which handles authentication)
/// 2. Check if the user has the required permission
/// 3. Return 403 Forbidden if permission is denied
///
/// # Example
/// ```rust,ignore
/// async fn create_product(
///     State(ctx): State<AppContext>,
///     RequirePermission(user, Permission::PRODUCTS_CREATE): RequirePermission,
///     Json(input): Json<CreateProductInput>,
/// ) -> Result<Json<ProductResponse>> {
///     // User is guaranteed to have PRODUCTS_CREATE permission
///     // ...
/// }
/// ```
pub struct RequirePermission<const P: Permission>(pub CurrentUser);

impl<S, const P: Permission> FromRequestParts<S> for RequirePermission<P>
where
    S: Send + Sync,
    loco_rs::prelude::AppContext: axum::extract::FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // First, authenticate the user
        let user = CurrentUser::from_request_parts(parts, state)
            .await
            .map_err(|(code, msg)| (code, msg.to_string()))?;

        // Check permission
        if !Rbac::has_permission(&user.user.role, &P) {
            return Err((
                StatusCode::FORBIDDEN,
                format!("Insufficient permissions. Required: {}", P),
            ));
        }

        Ok(RequirePermission(user))
    }
}

/// Helper macro to create permission-checking extractors
///
/// Since Rust doesn't support const generics for complex types yet,
/// we need to create wrapper types for each permission check.
///
/// # Example
/// ```rust,ignore
/// // In controller
/// async fn create_product(
///     State(ctx): State<AppContext>,
///     RequireProductCreate(user): RequireProductCreate,
///     Json(input): Json<CreateProductInput>,
/// ) -> Result<Json<ProductResponse>> {
///     // User has PRODUCTS_CREATE permission
/// }
/// ```
#[macro_export]
macro_rules! define_permission_extractor {
    ($name:ident, $permission:expr) => {
        pub struct $name(pub $crate::extractors::auth::CurrentUser);

        impl<S> axum::extract::FromRequestParts<S> for $name
        where
            S: Send + Sync,
            loco_rs::prelude::AppContext: axum::extract::FromRef<S>,
        {
            type Rejection = (axum::http::StatusCode, String);

            async fn from_request_parts(
                parts: &mut axum::http::request::Parts,
                state: &S,
            ) -> Result<Self, Self::Rejection> {
                use rustok_core::Rbac;

                let user = $crate::extractors::auth::CurrentUser::from_request_parts(parts, state)
                    .await
                    .map_err(|(code, msg)| (code, msg.to_string()))?;

                if !Rbac::has_permission(&user.user.role, &$permission) {
                    return Err((
                        axum::http::StatusCode::FORBIDDEN,
                        format!("Insufficient permissions. Required: {}", $permission),
                    ));
                }

                Ok($name(user))
            }
        }
    };
}

// Define common permission extractors
define_permission_extractor!(RequirePostsCreate, rustok_core::Permission::POSTS_CREATE);
define_permission_extractor!(RequirePostsRead, rustok_core::Permission::POSTS_READ);
define_permission_extractor!(RequirePostsUpdate, rustok_core::Permission::POSTS_UPDATE);
define_permission_extractor!(RequirePostsDelete, rustok_core::Permission::POSTS_DELETE);
define_permission_extractor!(RequirePostsList, rustok_core::Permission::POSTS_LIST);

define_permission_extractor!(RequireNodesCreate, rustok_core::Permission::NODES_CREATE);
define_permission_extractor!(RequireNodesRead, rustok_core::Permission::NODES_READ);
define_permission_extractor!(RequireNodesUpdate, rustok_core::Permission::NODES_UPDATE);
define_permission_extractor!(RequireNodesDelete, rustok_core::Permission::NODES_DELETE);
define_permission_extractor!(RequireNodesList, rustok_core::Permission::NODES_LIST);

define_permission_extractor!(
    RequireProductsCreate,
    rustok_core::Permission::PRODUCTS_CREATE
);
define_permission_extractor!(RequireProductsRead, rustok_core::Permission::PRODUCTS_READ);
define_permission_extractor!(
    RequireProductsUpdate,
    rustok_core::Permission::PRODUCTS_UPDATE
);
define_permission_extractor!(
    RequireProductsDelete,
    rustok_core::Permission::PRODUCTS_DELETE
);
define_permission_extractor!(RequireProductsList, rustok_core::Permission::PRODUCTS_LIST);

define_permission_extractor!(RequireOrdersCreate, rustok_core::Permission::ORDERS_CREATE);
define_permission_extractor!(RequireOrdersRead, rustok_core::Permission::ORDERS_READ);
define_permission_extractor!(RequireOrdersUpdate, rustok_core::Permission::ORDERS_UPDATE);
define_permission_extractor!(RequireOrdersDelete, rustok_core::Permission::ORDERS_DELETE);
define_permission_extractor!(RequireOrdersList, rustok_core::Permission::ORDERS_LIST);

define_permission_extractor!(RequireUsersCreate, rustok_core::Permission::USERS_CREATE);
define_permission_extractor!(RequireUsersRead, rustok_core::Permission::USERS_READ);
define_permission_extractor!(RequireUsersUpdate, rustok_core::Permission::USERS_UPDATE);
define_permission_extractor!(RequireUsersDelete, rustok_core::Permission::USERS_DELETE);
define_permission_extractor!(RequireUsersList, rustok_core::Permission::USERS_LIST);

define_permission_extractor!(RequireSettingsRead, rustok_core::Permission::SETTINGS_READ);
define_permission_extractor!(
    RequireSettingsUpdate,
    rustok_core::Permission::SETTINGS_UPDATE
);

define_permission_extractor!(
    RequireAnalyticsRead,
    rustok_core::Permission::ANALYTICS_READ
);
define_permission_extractor!(
    RequireAnalyticsExport,
    rustok_core::Permission::ANALYTICS_EXPORT
);

/// Helper to check permission inline without extractor
///
/// # Example
/// ```rust,ignore
/// pub async fn some_handler(
///     user: CurrentUser,
///     // ...
/// ) -> Result<Response> {
///     check_permission(&user, Permission::PRODUCTS_UPDATE)?;
///     // ...
/// }
/// ```
pub fn check_permission(
    user: &CurrentUser,
    permission: Permission,
) -> Result<(), (StatusCode, String)> {
    if !Rbac::has_permission(&user.user.role, &permission) {
        return Err((
            StatusCode::FORBIDDEN,
            format!("Insufficient permissions. Required: {}", permission),
        ));
    }
    Ok(())
}

/// Helper to check if user has any of the provided permissions
pub fn check_any_permission(
    user: &CurrentUser,
    permissions: &[Permission],
) -> Result<(), (StatusCode, String)> {
    if !Rbac::has_any_permission(&user.user.role, permissions) {
        let perms_str = permissions
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err((
            StatusCode::FORBIDDEN,
            format!("Insufficient permissions. Required any of: {}", perms_str),
        ));
    }
    Ok(())
}

/// Helper to check if user has all of the provided permissions
pub fn check_all_permissions(
    user: &CurrentUser,
    permissions: &[Permission],
) -> Result<(), (StatusCode, String)> {
    if !Rbac::has_all_permissions(&user.user.role, permissions) {
        let perms_str = permissions
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Err((
            StatusCode::FORBIDDEN,
            format!("Insufficient permissions. Required all of: {}", perms_str),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode},
        routing::get,
        Json, Router,
    };
    use loco_rs::prelude::AppContext;
    use rustok_core::{Permission, UserRole};
    use tower::ServiceExt;

    fn create_test_user(role: UserRole) -> CurrentUser {
        use crate::models::users;
        use uuid::Uuid;

        let user = users::Model {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            role,
            is_active: true,
            email_verified: true,
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
        };

        CurrentUser {
            user,
            session_id: Uuid::new_v4(),
            permissions: vec![],
        }
    }

    #[test]
    fn test_check_permission_allows_authorized() {
        let admin_user = create_test_user(UserRole::Admin);
        let result = check_permission(&admin_user, Permission::POSTS_CREATE);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_permission_denies_unauthorized() {
        let customer_user = create_test_user(UserRole::Customer);
        let result = check_permission(&customer_user, Permission::PRODUCTS_DELETE);
        assert!(result.is_err());
        if let Err((code, _msg)) = result {
            assert_eq!(code, StatusCode::FORBIDDEN);
        }
    }

    #[test]
    fn test_check_any_permission_success() {
        let customer_user = create_test_user(UserRole::Customer);
        let result = check_any_permission(
            &customer_user,
            &[Permission::POSTS_READ, Permission::PRODUCTS_CREATE],
        );
        assert!(result.is_ok()); // Customer has POSTS_READ
    }

    #[test]
    fn test_check_any_permission_failure() {
        let customer_user = create_test_user(UserRole::Customer);
        let result = check_any_permission(
            &customer_user,
            &[Permission::PRODUCTS_DELETE, Permission::USERS_DELETE],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_check_all_permissions_success() {
        let manager_user = create_test_user(UserRole::Manager);
        let result = check_all_permissions(
            &manager_user,
            &[Permission::PRODUCTS_READ, Permission::PRODUCTS_CREATE],
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_all_permissions_failure() {
        let customer_user = create_test_user(UserRole::Customer);
        let result = check_all_permissions(
            &customer_user,
            &[Permission::PRODUCTS_READ, Permission::PRODUCTS_DELETE],
        );
        assert!(result.is_err()); // Customer doesn't have DELETE
    }
}
