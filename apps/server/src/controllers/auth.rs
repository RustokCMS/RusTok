use axum::{routing::{get, post}, Json};
use loco_rs::prelude::*;
use sea_orm::{ActiveModelTrait, Set};
use serde::{Deserialize, Serialize};

use crate::auth::{hash_password, verify_password};
use crate::extractors::{auth::CurrentUser, tenant::CurrentTenant};
use crate::models::users::{self, ActiveModel as UserActiveModel, Entity as Users};
use rustok_core::auth::jwt::{self, JwtConfig};

// --- DTOs ---

#[derive(Deserialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterParams {
    pub email: String,
    pub password: String,
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
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

fn jwt_config_from_ctx(ctx: &AppContext) -> Result<JwtConfig> {
    let jwt_settings = ctx
        .config
        .auth
        .as_ref()
        .and_then(|auth| auth.jwt.as_ref())
        .ok_or(Error::InternalServerError)?;

    Ok(JwtConfig::new(
        jwt_settings.secret.clone(),
        jwt_settings.expiration as i64,
    ))
}

// --- Handlers ---

/// POST /api/auth/register
pub async fn register(
    State(ctx): State<AppContext>,
    CurrentTenant(tenant): CurrentTenant,
    Json(params): Json<RegisterParams>,
) -> Result<Json<AuthResponse>> {
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

    // 4. Генерируем токен
    let jwt_config = jwt_config_from_ctx(&ctx)?;
    let token = jwt::encode_token(&user.id, &tenant.id, &user.role.to_string(), &jwt_config)?;

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
    let token = jwt::encode_token(&user.id, &tenant.id, &user.role.to_string(), &jwt_config)?;

    Ok(Json(AuthResponse {
        token,
        user: user.into(),
    }))
}

/// GET /api/auth/me
/// Требует авторизации через заголовок
pub async fn me(CurrentUser { user }: CurrentUser) -> Result<Json<UserResponse>> {
    Ok(Json(user.into()))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/auth")
        .add("/register", post(register))
        .add("/login", post(login))
        .add("/me", get(me))
}
