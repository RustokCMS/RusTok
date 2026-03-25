use async_graphql::{Context, FieldError, Object, Result};
use loco_rs::app::AppContext;
use uuid::Uuid;

use crate::context::{AuthContext, TenantContext};
use crate::graphql::errors::GraphQLError;
use crate::services::event_bus::transactional_event_bus_from_context;
use crate::services::rbac_service::RbacService;
use rustok_events::DomainEvent;
use rustok_search::{SearchEngineKind, SearchModule, SearchSettingsService};

use super::types::{
    SearchSettingsPayload, TriggerSearchRebuildInput, TriggerSearchRebuildPayload,
    UpdateSearchSettingsInput, UpdateSearchSettingsPayload,
};

#[derive(Default)]
pub struct SearchMutationRoot;

#[Object]
impl SearchMutationRoot {
    async fn update_search_settings(
        &self,
        ctx: &Context<'_>,
        input: UpdateSearchSettingsInput,
    ) -> Result<UpdateSearchSettingsPayload> {
        ensure_settings_manage_permission(ctx).await?;

        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let target_tenant_id = Some(resolve_tenant_scope(
            tenant,
            parse_optional_uuid(input.tenant_id.as_deref())?,
        )?);
        let active_engine = parse_requested_engine(&input.active_engine, "active_engine")?;
        let fallback_engine = input
            .fallback_engine
            .as_deref()
            .map(|value| parse_requested_engine(value, "fallback_engine"))
            .transpose()?
            .unwrap_or(SearchEngineKind::Postgres);
        ensure_engine_is_available(active_engine)?;
        ensure_engine_is_available(fallback_engine)?;
        let config: serde_json::Value = serde_json::from_str(&input.config)
            .map_err(|err| FieldError::new(format!("Invalid JSON in config: {err}")))?;

        let settings = SearchSettingsService::save(
            &app_ctx.db,
            target_tenant_id,
            active_engine,
            fallback_engine,
            config,
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(UpdateSearchSettingsPayload {
            success: true,
            settings: SearchSettingsPayload::from(settings),
        })
    }

    async fn trigger_search_rebuild(
        &self,
        ctx: &Context<'_>,
        input: TriggerSearchRebuildInput,
    ) -> Result<TriggerSearchRebuildPayload> {
        ensure_settings_manage_permission(ctx).await?;

        let app_ctx = ctx.data::<AppContext>()?;
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
        let tenant = ctx.data::<TenantContext>()?;
        let target_tenant_id =
            resolve_tenant_scope(tenant, parse_optional_uuid(input.tenant_id.as_deref())?)?;
        let target_type = input
            .target_type
            .unwrap_or_else(|| "search".to_string())
            .trim()
            .to_ascii_lowercase();
        let target_id = parse_optional_uuid(input.target_id.as_deref())?;

        if !matches!(target_type.as_str(), "search" | "content" | "product") {
            return Err(FieldError::new(
                "Invalid target_type. Expected one of: search, content, product",
            ));
        }

        let event_bus = transactional_event_bus_from_context(app_ctx);
        event_bus
            .publish(
                target_tenant_id,
                Some(auth.user_id),
                DomainEvent::ReindexRequested {
                    target_type: target_type.clone(),
                    target_id,
                },
            )
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(TriggerSearchRebuildPayload {
            success: true,
            queued: true,
            tenant_id: target_tenant_id.to_string(),
            target_type,
            target_id: target_id.map(|value| value.to_string()),
        })
    }
}

async fn ensure_settings_manage_permission(ctx: &Context<'_>) -> Result<()> {
    let app_ctx = ctx.data::<AppContext>()?;
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
    let tenant = ctx.data::<TenantContext>()?;

    let can_manage = RbacService::has_permission(
        &app_ctx.db,
        &tenant.id,
        &auth.user_id,
        &rustok_core::Permission::SETTINGS_MANAGE,
    )
    .await
    .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

    if !can_manage {
        return Err(<FieldError as GraphQLError>::permission_denied(
            "settings:manage required",
        ));
    }

    Ok(())
}

fn parse_optional_uuid(value: Option<&str>) -> Result<Option<Uuid>> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| Uuid::parse_str(value).map_err(|_| FieldError::new("Invalid UUID")))
        .transpose()
}

fn parse_requested_engine(value: &str, field_name: &str) -> Result<SearchEngineKind> {
    SearchEngineKind::try_from_str(value)
        .ok_or_else(|| FieldError::new(format!("Invalid {field_name}: unsupported engine")))
}

fn ensure_engine_is_available(engine: SearchEngineKind) -> Result<()> {
    let module = SearchModule;
    let is_available = module
        .available_engines()
        .into_iter()
        .any(|descriptor| descriptor.enabled && descriptor.kind == engine);

    if !is_available {
        return Err(FieldError::new(format!(
            "Engine '{}' is not installed in the current runtime",
            engine.as_str()
        )));
    }

    Ok(())
}

fn resolve_tenant_scope(tenant: &TenantContext, requested_tenant_id: Option<Uuid>) -> Result<Uuid> {
    match requested_tenant_id {
        Some(requested_tenant_id) if requested_tenant_id != tenant.id => {
            Err(<FieldError as GraphQLError>::permission_denied(
                "cross-tenant search access is not allowed",
            ))
        }
        _ => Ok(tenant.id),
    }
}
