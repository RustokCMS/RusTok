use async_graphql::{Context, FieldError, Object, Result};

use crate::context::{AuthContext, TenantContext};
use crate::graphql::errors::GraphQLError;
use crate::graphql::types::TenantModule;
use crate::models::_entities::tenant_modules::Entity as TenantModulesEntity;

#[derive(Default)]
pub struct RootMutation;

#[Object]
impl RootMutation {
    async fn toggle_module(
        &self,
        ctx: &Context<'_>,
        module_slug: String,
        enabled: bool,
    ) -> Result<TenantModule> {
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

        if !matches!(
            auth.role,
            rustok_core::UserRole::Admin | rustok_core::UserRole::SuperAdmin
        ) {
            return Err(<FieldError as GraphQLError>::permission_denied("Forbidden"));
        }

        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let module = TenantModulesEntity::toggle(&app_ctx.db, tenant.id, &module_slug, enabled)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(TenantModule {
            module_slug: module.module_slug,
            enabled: module.enabled,
            settings: module.settings.to_string(),
        })
    }
}
