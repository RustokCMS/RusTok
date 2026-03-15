use crate::auth::{auth_config_from_ctx, decode_access_token};
use crate::context::{infer_user_role_from_permissions, TenantContextExt};
use crate::models::{
    sessions::Entity as Sessions,
    users::{self, Entity as Users},
};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use loco_rs::prelude::*;
use rustok_core::{Permission, UserRole};
use tracing::warn;

use crate::services::rbac_service::RbacService;

pub struct CurrentUser {
    pub user: users::Model,
    pub session_id: uuid::Uuid,
    pub permissions: Vec<Permission>,
    pub inferred_role: UserRole,
    // OAuth2 fields from JWT claims
    pub client_id: Option<uuid::Uuid>,
    pub scopes: Vec<String>,
    pub grant_type: String,
}

impl CurrentUser {
    pub fn security_context(&self) -> rustok_core::SecurityContext {
        rustok_core::SecurityContext::new(self.inferred_role.clone(), Some(self.user.id))
    }
}

async fn resolve_current_user<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<CurrentUser, (StatusCode, &'static str)>
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    let ctx = AppContext::from_ref(state);

    let tenant_id = parts
        .tenant_context()
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Tenant context missing"))?
        .id;

    let TypedHeader(Authorization(bearer)) =
        TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing or invalid token"))?;

    let auth_config = auth_config_from_ctx(&ctx).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "JWT secret not configured",
        )
    })?;

    let claims = decode_access_token(&auth_config, bearer.token())
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token signature"))?;

    if claims.tenant_id != tenant_id {
        return Err((StatusCode::FORBIDDEN, "Token belongs to another tenant"));
    }

    // For OAuth2 client_credentials tokens (session_id is nil), skip session check
    let is_oauth_service_token =
        claims.client_id.is_some() && claims.session_id == uuid::Uuid::nil();

    if !is_oauth_service_token {
        let session = Sessions::find_by_id(claims.session_id)
            .one(&ctx.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Session not found"))?;

        if session.tenant_id != tenant_id || !session.is_active() {
            return Err((StatusCode::UNAUTHORIZED, "Session expired"));
        }
    }

    let user = Users::find_by_id(claims.sub)
        .one(&ctx.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    // For OAuth2 service tokens, user may not exist (sub = app_id)
    let (user, permissions, inferred_role) = if let Some(user) = user {
        if !user.is_active() {
            return Err((StatusCode::FORBIDDEN, "User is inactive"));
        }

        let permissions = RbacService::get_user_permissions(&ctx.db, &tenant_id, &user.id)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

        let inferred_role = infer_user_role_from_permissions(&permissions);
        if claims.role != inferred_role {
            RbacService::record_claim_role_mismatch();
            warn!(
                user_id = %user.id,
                tenant_id = %tenant_id,
                claimed_role = %claims.role,
                inferred_role = %inferred_role,
                "rbac_claim_role_mismatch"
            );
        }
        (user, permissions, inferred_role)
    } else if is_oauth_service_token {
        // Service token without a real user — create a minimal model
        // The sub field is the app_id, not a user_id
        return Ok(CurrentUser {
            user: users::Model::default_service_user(claims.sub, tenant_id),
            session_id: uuid::Uuid::nil(),
            permissions: Vec::new(),
            inferred_role: claims.role,
            client_id: claims.client_id,
            scopes: claims.scopes,
            grant_type: claims.grant_type,
        });
    } else {
        return Err((StatusCode::UNAUTHORIZED, "User not found"));
    };

    Ok(CurrentUser {
        user,
        session_id: claims.session_id,
        permissions,
        inferred_role,
        client_id: claims.client_id,
        scopes: claims.scopes,
        grant_type: claims.grant_type,
    })
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        resolve_current_user(parts, state).await
    }
}

pub struct OptionalCurrentUser(pub Option<CurrentUser>);

impl<S> FromRequestParts<S> for OptionalCurrentUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .is_none()
        {
            return Ok(Self(None));
        }

        let current_user = resolve_current_user(parts, state).await?;
        Ok(Self(Some(current_user)))
    }
}
