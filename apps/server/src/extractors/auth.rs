use crate::auth::{decode_access_token, AuthConfig};
use crate::context::TenantContextExt;
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
use rustok_core::Permission;

use crate::services::auth::AuthService;

pub struct CurrentUser {
    pub user: users::Model,
    pub session_id: uuid::Uuid,
    pub permissions: Vec<Permission>,
}

impl CurrentUser {
    pub fn security_context(&self) -> rustok_core::SecurityContext {
        rustok_core::SecurityContext::new(self.user.role.clone(), Some(self.user.id))
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

    let auth_config = AuthConfig::from_ctx(&ctx).map_err(|_| {
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

    let session = Sessions::find_by_id(claims.session_id)
        .one(&ctx.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "Session not found"))?;

    if session.tenant_id != tenant_id || !session.is_active() {
        return Err((StatusCode::UNAUTHORIZED, "Session expired"));
    }

    let user = Users::find_by_id(claims.sub)
        .one(&ctx.db)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
        .ok_or((StatusCode::UNAUTHORIZED, "User not found"))?;

    if !user.is_active() {
        return Err((StatusCode::FORBIDDEN, "User is inactive"));
    }

    let permissions = AuthService::get_user_permissions(&ctx.db, &tenant_id, &user.id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?;

    Ok(CurrentUser {
        user,
        session_id: claims.session_id,
        permissions,
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
