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

#[derive(Deserialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Deserialize)]
pub struct RegisterParams {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

#[derive(Serialize)]
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

#[derive(Debug, Serialize)]
struct UserInfo {
    id: uuid::Uuid,
    email: String,
    name: Option<String>,
    role: rustok_core::UserRole,
    status: rustok_core::UserStatus,
}

#[derive(Debug, Serialize)]
struct LogoutResponse {
    status: &'static str,
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

// --- Handlers ---

/// POST /api/auth/register
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

    AuthService::assign_role_permissions(&ctx.db, &user.id, &tenant.id, user.role).await?;

    // 4. Создаем сессию и токены
    let now = Utc::now();
    let refresh_token = generate_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expires_at = now + Duration::seconds(config.refresh_expiration as i64);

    let session =
        sessions::ActiveModel::new(tenant.id, user.id, token_hash, expires_at, None, None)
            .insert(&ctx.db)
            .await?;

    let access_token = encode_access_token(&config, user.id, tenant.id, user.role, session.id)?;

    let response = AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in: config.access_expiration,
        user: UserInfo {
            id: user.id,
            email: user.email,
            name: user.name,
            role: user.role,
            status: user.status,
        },
    };

    format::json(response)
}

/// POST /api/auth/login
async fn login(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<LoginParams>,
) -> Result<Response> {
    let config = AuthConfig::from_ctx(&ctx)?;

    // 1. Ищем юзера
    let user = Users::find_by_email(&ctx.db, tenant.id, &payload.email)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid credentials".into()))?;

    // 2. Проверяем пароль
    if !verify_password(&payload.password, &user.password_hash)? {
        return Err(Error::Unauthorized("Invalid credentials".into()));
    }

    // 3. Создаем юзера
    let mut user = UserActiveModel::new(tenant.id, &params.email, &password_hash);
    user.name = Set(params.name);

    let user = user.insert(&ctx.db).await?;

    // 4. Генерируем токен
    let jwt_config = jwt_config_from_ctx(&ctx)?;
    let token = jwt::encode_token(&user.id, &tenant.id, &user.role.to_string(), &jwt_config)
        .map_err(|_| Error::InternalServerError)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// POST /api/auth/login
pub async fn login(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<LoginParams>,
) -> Result<Json<AuthResponse>> {
    // 1. Ищем юзера
    let user = Users::find_by_email(&ctx.db, tenant.id, &params.email)
        .await?
        .ok_or_else(|| Error::Unauthorized("Invalid credentials".into()))?;

    // 2. Проверяем пароль
    if !verify_password(&params.password, &user.password_hash)? {
        return Err(Error::Unauthorized("Invalid credentials".into()));
    }

    // 3. Генерируем токен
    let jwt_config = jwt_config_from_ctx(&ctx)?;
    let token = jwt::encode_token(&user.id, &tenant.id, &user.role.to_string(), &jwt_config)
        .map_err(|_| Error::InternalServerError)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// GET /api/auth/me
/// Требует авторизации через заголовок
pub async fn me(CurrentUser { user, .. }: CurrentUser) -> Result<Json<UserResponse>> {
    Ok(Json(user.into()))
}

/// GET /api/auth/me
/// Требует авторизации через заголовок
async fn me(CurrentUser { user, .. }: CurrentUser) -> Result<Json<UserResponse>> {
    Ok(Json(user.into()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/auth")
        .add("/register", post(register))
        .add("/login", post(login))
        .add("/register", post(register))
        .add("/refresh", post(refresh))
        .add("/logout", post(logout))
        .add("/me", get(me))
}
