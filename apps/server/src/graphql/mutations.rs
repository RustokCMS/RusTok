use async_graphql::{Context, FieldError, Object, Result};

use crate::context::{AuthContext, TenantContext};
use crate::graphql::errors::GraphQLError;
use crate::graphql::types::TenantModule;
use rustok_core::{ModuleContext, ModuleRegistry};
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
        let registry = ctx.data::<ModuleRegistry>()?;

        if !registry.contains(&module_slug) {
            return Err(FieldError::new("Unknown module"));
        }
        let module = TenantModulesEntity::toggle(&app_ctx.db, tenant.id, &module_slug, enabled)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        if let Some(module_impl) = registry.get(&module_slug) {
            let module_ctx = ModuleContext {
                db: &app_ctx.db,
                tenant_id: tenant.id,
                config: &module.settings,
            };

            let hook_result = if enabled {
                module_impl.on_enable(module_ctx).await
            } else {
                module_impl.on_disable(module_ctx).await
            };

            if let Err(err) = hook_result {
                tracing::error!(
                    "Module hook failed for {} (enabled={}): {}",
                    module_slug,
                    enabled,
                    err
                );
            }
        }

        Ok(TenantModule {
            module_slug: module.module_slug,
            enabled: module.enabled,
            settings: module.settings.to_string(),
        })
    }
}
