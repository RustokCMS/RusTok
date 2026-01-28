use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use loco_rs::prelude::*;
use crate::context::TenantContextExt;
use crate::models::{
    sessions::Entity as Sessions,
    users::{self, Entity as Users},
};

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
        let tenant_id = parts
            .tenant_context()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Tenant context missing"))?
            .id
            .to_string();

        // 3. Достаем Bearer token
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| (StatusCode::UNAUTHORIZED, "Missing or invalid token"))?;

        // 4. Берем секрет из конфига Loco
        let auth_config = AuthConfig::from_ctx(&ctx)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not configured"))?;

        // 5. Валидируем токен
        let claims = decode_access_token(&auth_config, bearer.token())
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token signature"))?;

        // 6. ПРОВЕРКА МУЛЬТИТЕНАНТНОСТИ
        // Если токен выдан для магазина А, а запрос пришел в магазин Б - отлуп.
        if claims.tenant != tenant_id {
            return Err((StatusCode::FORBIDDEN, "Token belongs to another tenant"));
        }

        // 7. Проверяем сессию
        let session = Sessions::find_by_id(claims.session_id)
            .one(&ctx.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "Session not found"))?;

        if session.tenant_id != tenant_ctx.id || !session.is_active() {
            return Err((StatusCode::UNAUTHORIZED, "Session expired"));
        }

        // 8. Достаем юзера из БД
        let user = Users::find_by_id(claims.sub)
            .one(&ctx.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "User not found"))?;

        // 9. Проверяем статус (не забанен ли)
        if !user.is_active() {
            return Err((StatusCode::FORBIDDEN, "User is inactive"));
        }

        Ok(CurrentUser { user })
    }
}
