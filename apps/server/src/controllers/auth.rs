use axum::{
    extract::ConnectInfo,
    http::header::USER_AGENT,
    routing::{get, post},
    Json,
};
use chrono::{Duration, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use utoipa::ToSchema;

use crate::auth::{
    encode_access_token, generate_refresh_token, hash_password, hash_refresh_token,
    verify_password, AuthConfig,
};
use crate::extractors::{auth::CurrentUser, tenant::CurrentTenant};
use crate::models::{
    sessions,
    users::{self, ActiveModel as UserActiveModel, Entity as Users},
};
use crate::services::auth::AuthService;

// --- DTOs ---

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
struct UserInfo {
    id: uuid::Uuid,
    email: String,
    name: Option<String>,
    role: rustok_core::UserRole,
    status: rustok_core::UserStatus,
}

#[derive(Debug, Serialize, ToSchema)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    token_type: &'static str,
    expires_in: u64,
    user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
struct LogoutResponse {
    status: &'static str,
}

// --- Handlers ---

/// Register a new user
#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "auth",
    request_body = RegisterParams,
    responses(
        (status = 200, description = "Registration successful", body = AuthResponse),
        (status = 400, description = "Email already exists")
    )
)]
async fn register(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<RegisterParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;

    // 1. Проверяем существование
    if Users::find_by_email(&ctx.db, tenant.id, &params.email)
        .await?
        .is_some()
    {
        return Err(Error::BadRequest("Email already exists".into()));
    }

    // 2. Хешируем пароль
    let password_hash = hash_password(&params.password)?;

    // 3. Создаем юзера
    let mut user = UserActiveModel::new(tenant.id, &params.email, &password_hash);
    user.name = Set(params.name);

    let user = user.insert(&ctx.db).await?;

    let user_role = user.role.clone();
    AuthService::assign_role_permissions(&ctx.db, &user.id, &tenant.id, user_role.clone()).await?;

    // 4. Создаем сессию и токены
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

    let response = AuthResponse {
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
    };

    format::json(response)
}

/// Login with email and password
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "auth",
    request_body = LoginParams,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Unauthorized")
    )
)]
async fn login(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(params): Json<LoginParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;

    // 1. Ищем юзера
    let user = Users::find_by_email(&ctx.db, tenant.id, &params.email)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid credentials".into()))?;

    // 2. Проверяем пароль
    if !verify_password(&params.password, &user.password_hash)? {
        return Err(Error::Unauthorized("Invalid credentials".into()));
    }

    // 3. Создаем сессию и токены
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

    let user_role = user.role.clone();
    let access_token =
        encode_access_token(&config, user.id, tenant.id, user_role.clone(), session.id)?;

    let response = AuthResponse {
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
    };

    format::json(response)
}

    format::json(response)
}

/// Refresh access token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token")
    )
)]
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

    let response = AuthResponse {
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
    };

    format::json(response)
}

    format::json(response)
}

/// Revoke refresh token (Logout)
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "auth",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse),
        (status = 401, description = "Invalid refresh token")
    )
)]
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

    format::json(LogoutResponse { status: "ok" })
}

/// Get current user info
/// Requires Bearer token
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "auth",
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Current user info", body = UserResponse),
        (status = 401, description = "Unauthorized")
    )
)]
async fn me(CurrentUser { user, .. }: CurrentUser) -> Result<Response> {
    format::json(UserResponse::from(user))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/auth")
        .add("/register", post(register))
        .add("/login", post(login))
        .add("/refresh", post(refresh))
        .add("/logout", post(logout))
        .add("/me", get(me))
}
