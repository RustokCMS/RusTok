use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub locale: String,
}

impl RequestContext {
    pub fn require_user(&self) -> Result<Uuid, (StatusCode, &'static str)> {
        self.user_id
            .ok_or((StatusCode::UNAUTHORIZED, "Authentication required"))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for RequestContext
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let tenant_id = parts
            .headers
            .get("X-Tenant-ID")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Uuid::parse_str(value).ok())
            .ok_or((StatusCode::BAD_REQUEST, "X-Tenant-ID header required"))?;

        let user_id = parts
            .headers
            .get("X-User-ID")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| Uuid::parse_str(value).ok());

        let locale = parts
            .headers
            .get("Accept-Language")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .unwrap_or("en")
            .to_string();

        Ok(RequestContext {
            tenant_id,
            user_id,
            locale,
        })
    }
}
