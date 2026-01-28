use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use loco_rs::prelude::*;
use sea_orm::EntityTrait;

use crate::context::TenantContextExt;
use crate::models::users::{self, Entity as Users};
use rustok_core::auth::jwt::{self, JwtConfig};

// Структура, которую мы будем просить в контроллерах
pub struct CurrentUser {
    pub user: users::Model,
}

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Достаем AppContext (для доступа к БД и конфигам)
        let ctx = AppContext::from_ref(state);

        // 2. Достаем TenantContext (он ОБЯЗАН быть, так как Auth идет ПОСЛЕ TenantMiddleware)
        let tenant_ctx = parts
            .tenant_context()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Tenant context missing"))?;

        // 3. Достаем Bearer token
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing or invalid token"))?;

        // 4. Берем секрет из конфига Loco
        let jwt_settings = ctx
            .config
            .auth
            .as_ref()
            .and_then(|auth| auth.jwt.as_ref())
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not configured"))?;

        let jwt_config = JwtConfig::new(jwt_settings.secret.clone(), jwt_settings.expiration as i64);

        // 5. Валидируем токен
        let claims = jwt::decode_token(bearer.token(), &jwt_config)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token signature"))?;

        // 6. ПРОВЕРКА МУЛЬТИТЕНАНТНОСТИ
        // Если токен выдан для магазина А, а запрос пришел в магазин Б - отлуп.
        if claims.tenant != tenant_ctx.id.to_string() {
            return Err((StatusCode::FORBIDDEN, "Token belongs to another tenant"));
        }

        // 7. Достаем юзера из БД
        let user_id = uuid::Uuid::parse_str(&claims.sub)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID in token"))?;

        let user = Users::find_by_id(user_id)
            .one(&ctx.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "User not found"))?;

        // 8. Проверяем статус (не забанен ли)
        if !user.is_active() {
            return Err((StatusCode::FORBIDDEN, "User is inactive"));
        }

        Ok(CurrentUser { user })
    }
}
