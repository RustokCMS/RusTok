use axum::{
    extract::ConnectInfo,
    http::header::USER_AGENT,
    routing::{get, post},
    Json,
};
use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use utoipa::ToSchema;

use crate::auth::{
    decode_invite_token, decode_password_reset_token, encode_access_token,
    encode_password_reset_token, generate_refresh_token, hash_password, hash_refresh_token,
    verify_password, AuthConfig,
};
use crate::extractors::{auth::CurrentUser, tenant::CurrentTenant};
use crate::models::{
    sessions,
    users::{self, ActiveModel as UserActiveModel, Entity as Users},
};
use crate::services::auth::AuthService;

const DEFAULT_RESET_TOKEN_TTL_SECS: u64 = 15 * 60;

#[derive(Deserialize, ToSchema)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize, ToSchema)]
pub struct RegisterParams {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct AcceptInviteParams {
    pub token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct InviteAcceptResponse {
    pub status: &'static str,
    pub email: String,
    pub role: rustok_core::UserRole,
}

#[derive(Deserialize, ToSchema)]
pub struct RequestResetParams {
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ConfirmResetParams {
    pub token: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ChangePasswordParams {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateProfileParams {
    pub name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ResetRequestResponse {
    pub status: &'static str,
    pub reset_token: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GenericStatusResponse {
    pub status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionItem {
    pub id: uuid::Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub current: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SessionsResponse {
    pub sessions: Vec<SessionItem>,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

impl From<users::Model> for UserResponse {
    fn from(m: users::Model) -> Self {
        Self {
            id: m.id,
            email: m.email,
            name: m.name,
            role: m.role.to_string(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: rustok_core::UserRole,
    pub status: rustok_core::UserStatus,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub status: &'static str,
}

#[utoipa::path(post, path = "/api/auth/register", tag = "auth", request_body = RegisterParams,
    responses((status = 200, description = "Registration successful", body = AuthResponse),(status = 400, description = "Email already exists")))]
async fn register(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<RegisterParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;

    if Users::find_by_email(&ctx.db, tenant.id, &params.email)
        .await?
        .is_some()
    {
        return Err(Error::BadRequest("Email already exists".into()));
    }

    let password_hash = hash_password(&params.password)?;
    let mut user = UserActiveModel::new(tenant.id, &params.email, &password_hash);
    user.name = Set(params.name);
    let user = user.insert(&ctx.db).await?;

    let user_role = user.role.clone();
    AuthService::assign_role_permissions(&ctx.db, &user.id, &tenant.id, user_role.clone()).await?;

    let now = Utc::now();
    let refresh_token = generate_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

    let session =
        sessions::ActiveModel::new(tenant.id, user.id, token_hash, expires_at, None, None)
            .insert(&ctx.db)
            .await?;

    let access_token =
        encode_access_token(&config, user.id, tenant.id, user_role.clone(), session.id)?;

    format::json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: config.access_expiration,
        user: UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user_role,
            status: user.status,
        },
    })
}

#[utoipa::path(post, path = "/api/auth/login", tag = "auth", request_body = LoginParams,
    responses((status = 200, description = "Login successful", body = AuthResponse),(status = 401, description = "Unauthorized")))]
async fn login(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;

    let user = Users::find_by_email(&ctx.db, tenant.id, &params.email)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid credentials".into()))?;

    if !verify_password(&params.password, &user.password_hash)? {
        return Err(Error::Unauthorized("Invalid credentials".into()));
    }

    let now = Utc::now();
    let refresh_token = generate_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expires_at = now + Duration::seconds(config.refresh_expiration as i64);
    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let session = sessions::ActiveModel::new(
        tenant.id,
        user.id,
        token_hash,
        expires_at,
        Some(addr.ip().to_string()),
        user_agent,
    )
    .insert(&ctx.db)
    .await?;

    let mut user_active: users::ActiveModel = user.clone().into();
    user_active.last_login_at = Set(Some(now.into()));
    let user = user_active.update(&ctx.db).await?;

    let user_role = user.role.clone();
    let access_token =
        encode_access_token(&config, user.id, tenant.id, user_role.clone(), session.id)?;

    format::json(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: config.access_expiration,
        user: UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user_role,
            status: user.status,
        },
    })
}

#[utoipa::path(post, path = "/api/auth/refresh", tag = "auth", request_body = RefreshRequest,
    responses((status = 200, description = "Token refreshed", body = AuthResponse),(status = 401, description = "Invalid or expired refresh token")))]
async fn refresh(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<RefreshRequest>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;
    let token_hash = hash_refresh_token(&params.refresh_token);

    let session = sessions::Entity::find_by_token_hash(&ctx.db, tenant.id, &token_hash)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid refresh token".into()))?;

    if !session.is_active() {
        return Err(Error::Unauthorized("Session expired".into()));
    }

    let user = Users::find_by_id(session.user_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::Unauthorized("User not found".into()))?;

    if !user.is_active() {
        return Err(Error::Unauthorized("User is inactive".into()));
    }

    let now = Utc::now();
    let new_refresh_token = generate_refresh_token();
    let new_token_hash = hash_refresh_token(&new_refresh_token);
    let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

    let session_id = session.id;
    let mut session_model: sessions::ActiveModel = session.into();
    session_model.token_hash = Set(new_token_hash);
    session_model.expires_at = Set(expires_at.into());
    session_model.last_used_at = Set(Some(now.into()));
    session_model.update(&ctx.db).await?;

    let user_role = user.role.clone();
    let access_token =
        encode_access_token(&config, user.id, tenant.id, user_role.clone(), session_id)?;

    format::json(AuthResponse {
        access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer",
        expires_in: config.access_expiration,
        user: UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user_role,
            status: user.status,
        },
    })
}

#[utoipa::path(post, path = "/api/auth/logout", tag = "auth", request_body = RefreshRequest,
    responses((status = 200, description = "Logout successful", body = LogoutResponse),(status = 401, description = "Invalid refresh token")))]
async fn logout(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<RefreshRequest>,
) -> Result<Response> {
    let token_hash = hash_refresh_token(&params.refresh_token);
    let session = sessions::Entity::find_by_token_hash(&ctx.db, tenant.id, &token_hash)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid refresh token".into()))?;

    if session.revoked_at.is_none() {
        let mut session_model: sessions::ActiveModel = session.into();
        session_model.revoked_at = Set(Some(Utc::now().into()));
        session_model.update(&ctx.db).await?;
    }

    format::json(LogoutResponse { status: "ok" })
}

#[utoipa::path(get, path = "/api/auth/me", tag = "auth", security(("bearer_auth" = [])),
    responses((status = 200, description = "Current user info", body = UserResponse),(status = 401, description = "Unauthorized")))]
async fn me(CurrentUser { user, .. }: CurrentUser) -> Result<Response> {
    format::json(UserResponse::from(user))
}

#[utoipa::path(post, path = "/api/auth/invite/accept", tag = "auth", request_body = AcceptInviteParams,
    responses((status = 200, description = "Invite accepted", body = InviteAcceptResponse),(status = 401, description = "Invalid or expired invite token")))]
async fn accept_invite(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<AcceptInviteParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;
    let claims = decode_invite_token(&config, &params.token)?;

    if claims.tenant_id != tenant.id {
        return Err(Error::Unauthorized("Invalid invite token".into()));
    }

    format::json(InviteAcceptResponse {
        status: "ok",
        email: claims.sub,
        role: claims.role,
    })
}

#[utoipa::path(post, path = "/api/auth/reset/request", tag = "auth", request_body = RequestResetParams,
    responses((status = 200, description = "Reset request accepted", body = ResetRequestResponse)))]
async fn request_reset(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<RequestResetParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;

    let user_exists = Users::find_by_email(&ctx.db, tenant.id, &params.email)
        .await?
        .is_some();

    let expose_token = std::env::var("RUSTOK_DEMO_MODE")
        .map(|value| value == "1")
        .unwrap_or(false);

    let reset_token = if user_exists {
        Some(encode_password_reset_token(
            &config,
            tenant.id,
            &params.email,
            DEFAULT_RESET_TOKEN_TTL_SECS,
        )?)
    } else {
        None
    };

    format::json(ResetRequestResponse {
        status: "ok",
        reset_token: if expose_token { reset_token } else { None },
    })
}

#[utoipa::path(post, path = "/api/auth/reset/confirm", tag = "auth", request_body = ConfirmResetParams,
    responses((status = 200, description = "Password updated", body = GenericStatusResponse),(status = 401, description = "Invalid token")))]
async fn confirm_reset(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<ConfirmResetParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;
    let claims = decode_password_reset_token(&config, &params.token)?;

    if claims.tenant_id != tenant.id {
        return Err(Error::Unauthorized("Invalid reset token".into()));
    }

    let user = Users::find_by_email(&ctx.db, tenant.id, &claims.sub)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid reset token".into()))?;

    let mut user_active: users::ActiveModel = user.into();
    user_active.password_hash = Set(hash_password(&params.password)?);
    user_active.update(&ctx.db).await?;

    format::json(GenericStatusResponse { status: "ok" })
}

#[utoipa::path(get, path = "/api/auth/sessions", tag = "auth", security(("bearer_auth" = [])),
    responses((status = 200, description = "Active sessions", body = SessionsResponse)))]
async fn list_sessions(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    current: CurrentUser,
) -> Result<Response> {
    let rows = sessions::Entity::find()
        .filter(sessions::Column::TenantId.eq(tenant.id))
        .filter(sessions::Column::UserId.eq(current.user.id))
        .filter(sessions::Column::RevokedAt.is_null())
        .filter(sessions::Column::ExpiresAt.gt(Utc::now()))
        .order_by_desc(sessions::Column::CreatedAt)
        .all(&ctx.db)
        .await?;

    let data = rows
        .into_iter()
        .map(|item| SessionItem {
            id: item.id,
            ip_address: item.ip_address,
            user_agent: item.user_agent,
            last_used_at: item.last_used_at.map(|value| value.into()),
            expires_at: item.expires_at.into(),
            created_at: item.created_at.into(),
            current: item.id == current.session_id,
        })
        .collect();

    format::json(SessionsResponse { sessions: data })
}

#[utoipa::path(post, path = "/api/auth/sessions/revoke-all", tag = "auth", security(("bearer_auth" = [])),
    responses((status = 200, description = "Sessions revoked", body = GenericStatusResponse)))]
async fn revoke_all_sessions(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    current: CurrentUser,
) -> Result<Response> {
    let now = Utc::now();

    sessions::Entity::update_many()
        .col_expr(sessions::Column::RevokedAt, Expr::value(now))
        .filter(sessions::Column::TenantId.eq(tenant.id))
        .filter(sessions::Column::UserId.eq(current.user.id))
        .filter(sessions::Column::RevokedAt.is_null())
        .filter(sessions::Column::Id.ne(current.session_id))
        .exec(&ctx.db)
        .await?;

    format::json(GenericStatusResponse { status: "ok" })
}

#[utoipa::path(post, path = "/api/auth/change-password", tag = "auth", security(("bearer_auth" = [])), request_body = ChangePasswordParams,
    responses((status = 200, description = "Password changed", body = GenericStatusResponse),(status = 401, description = "Invalid credentials")))]
async fn change_password(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    current: CurrentUser,
    Json(params): Json<ChangePasswordParams>,
) -> Result<Response> {
    if !verify_password(&params.current_password, &current.user.password_hash)? {
        return Err(Error::Unauthorized("Invalid credentials".into()));
    }

    let mut user_active: users::ActiveModel = current.user.clone().into();
    user_active.password_hash = Set(hash_password(&params.new_password)?);
    user_active.update(&ctx.db).await?;

    let now = Utc::now();
    sessions::Entity::update_many()
        .col_expr(sessions::Column::RevokedAt, Expr::value(now))
        .filter(sessions::Column::TenantId.eq(tenant.id))
        .filter(sessions::Column::UserId.eq(current.user.id))
        .filter(sessions::Column::RevokedAt.is_null())
        .filter(sessions::Column::Id.ne(current.session_id))
        .exec(&ctx.db)
        .await?;

    format::json(GenericStatusResponse { status: "ok" })
}

#[utoipa::path(post, path = "/api/auth/profile", tag = "auth", security(("bearer_auth" = [])), request_body = UpdateProfileParams,
    responses((status = 200, description = "Profile updated", body = UserResponse)))]
async fn update_profile(
    State(ctx): State<AppContext>,
    current: CurrentUser,
    Json(params): Json<UpdateProfileParams>,
) -> Result<Response> {
    let mut user_active: users::ActiveModel = current.user.into();
    user_active.name = Set(params.name);
    let user = user_active.update(&ctx.db).await?;

    format::json(UserResponse::from(user))
}

#[utoipa::path(get, path = "/api/auth/history", tag = "auth", security(("bearer_auth" = [])),
    responses((status = 200, description = "Login history", body = SessionsResponse)))]
async fn login_history(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    current: CurrentUser,
) -> Result<Response> {
    let rows = sessions::Entity::find()
        .filter(sessions::Column::TenantId.eq(tenant.id))
        .filter(sessions::Column::UserId.eq(current.user.id))
        .order_by_desc(sessions::Column::CreatedAt)
        .limit(50)
        .all(&ctx.db)
        .await?;

    let data = rows
        .into_iter()
        .map(|item| SessionItem {
            id: item.id,
            ip_address: item.ip_address,
            user_agent: item.user_agent,
            last_used_at: item.last_used_at.map(|value| value.into()),
            expires_at: item.expires_at.into(),
            created_at: item.created_at.into(),
            current: item.id == current.session_id,
        })
        .collect();

    format::json(SessionsResponse { sessions: data })
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/auth")
        .add("/register", post(register))
        .add("/login", post(login))
        .add("/refresh", post(refresh))
        .add("/logout", post(logout))
        .add("/me", get(me))
        .add("/invite/accept", post(accept_invite))
        .add("/reset/request", post(request_reset))
        .add("/reset/confirm", post(confirm_reset))
        .add("/sessions", get(list_sessions))
        .add("/sessions/revoke-all", post(revoke_all_sessions))
        .add("/change-password", post(change_password))
        .add("/profile", post(update_profile))
        .add("/history", get(login_history))
}
