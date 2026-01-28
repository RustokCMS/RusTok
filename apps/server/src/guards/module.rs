use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::context::TenantContextExt;
use crate::models::_entities::tenant_modules::{self, Entity as TenantModules};
use loco_rs::app::AppContext;

pub struct RequireModule<const SLUG: &'static str>;

#[async_trait]
impl<S, const SLUG: &'static str> FromRequestParts<S> for RequireModule<SLUG>
where
    S: Send + Sync,
    AppContext: FromRef<S>,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let tenant_id = parts
            .tenant_context()
            .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Tenant context missing"))?
            .id;
        let ctx = AppContext::from_ref(state);

        let is_enabled = TenantModules::find()
            .filter(tenant_modules::Column::TenantId.eq(tenant_id))
            .filter(tenant_modules::Column::ModuleSlug.eq(SLUG))
            .filter(tenant_modules::Column::Enabled.eq(true))
            .one(&ctx.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error"))?
            .is_some();

        if is_enabled {
            Ok(Self)
        } else {
            Err((StatusCode::NOT_FOUND, "Module is disabled or not found"))
        }
    }
}
