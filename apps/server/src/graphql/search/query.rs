use async_graphql::{Context, FieldError, Object, Result};
use loco_rs::app::AppContext;
use uuid::Uuid;

use crate::context::{AuthContext, TenantContext};
use crate::graphql::errors::GraphQLError;
use crate::services::rbac_service::RbacService;
use rustok_search::{
    PgSearchEngine, SearchDiagnosticsService, SearchEngine, SearchModule, SearchQuery,
    SearchSettingsService,
};

use super::types::{
    LaggingSearchDocumentPayload, SearchDiagnosticsPayload, SearchEngineDescriptor,
    SearchPreviewInput, SearchPreviewPayload, SearchSettingsPayload,
};

#[derive(Default)]
pub struct SearchQueryRoot;

const MAX_SEARCH_QUERY_LEN: usize = 256;
const MAX_FILTER_VALUES: usize = 10;
const MAX_FILTER_VALUE_LEN: usize = 64;
const MAX_LOCALE_LEN: usize = 16;

#[Object]
impl SearchQueryRoot {
    /// Returns the list of search engines available in the current runtime.
    /// External engines appear only when their connector crates are installed.
    async fn available_search_engines(
        &self,
        ctx: &Context<'_>,
    ) -> Result<Vec<SearchEngineDescriptor>> {
        ensure_settings_read_permission(ctx).await?;

        let module = SearchModule;
        Ok(module
            .available_engines()
            .into_iter()
            .map(Into::into)
            .collect())
    }

    /// Returns the effective search settings for the current tenant.
    /// This is the first search GraphQL surface and is intentionally read-only.
    async fn search_settings_preview(
        &self,
        ctx: &Context<'_>,
        tenant_id: Option<Uuid>,
    ) -> Result<SearchSettingsPayload> {
        ensure_settings_read_permission(ctx).await?;

        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let tenant_id = Some(resolve_tenant_scope(tenant, tenant_id)?);

        let settings = SearchSettingsService::load_effective(&app_ctx.db, tenant_id)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(settings.into())
    }

    /// Returns diagnostics for the current tenant search storage and lag state.
    async fn search_diagnostics(
        &self,
        ctx: &Context<'_>,
        tenant_id: Option<Uuid>,
    ) -> Result<SearchDiagnosticsPayload> {
        ensure_settings_read_permission(ctx).await?;

        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let tenant_id = resolve_tenant_scope(tenant, tenant_id)?;

        let snapshot = SearchDiagnosticsService::snapshot(&app_ctx.db, tenant_id)
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(snapshot.into())
    }

    /// Returns the latest lagging search documents for diagnostics/debugging in admin.
    async fn search_lagging_documents(
        &self,
        ctx: &Context<'_>,
        tenant_id: Option<Uuid>,
        limit: Option<i32>,
    ) -> Result<Vec<LaggingSearchDocumentPayload>> {
        ensure_settings_read_permission(ctx).await?;

        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let tenant_id = resolve_tenant_scope(tenant, tenant_id)?;

        let rows = SearchDiagnosticsService::lagging_documents(
            &app_ctx.db,
            tenant_id,
            limit.unwrap_or(25).clamp(1, 100) as usize,
        )
        .await
        .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Executes a PostgreSQL-backed search preview over rustok-search owned search documents.
    async fn search_preview(
        &self,
        ctx: &Context<'_>,
        input: SearchPreviewInput,
    ) -> Result<SearchPreviewPayload> {
        ensure_settings_read_permission(ctx).await?;

        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let input = normalize_search_preview_input(input)?;
        let tenant_id = Some(resolve_tenant_scope(
            tenant,
            parse_optional_uuid(input.tenant_id.as_deref())?,
        )?);
        let engine = PgSearchEngine::new(app_ctx.db.clone());

        let result = engine
            .search(SearchQuery {
                tenant_id,
                locale: input.locale,
                query: input.query,
                limit: input.limit.unwrap_or(10).clamp(1, 50) as usize,
                offset: input.offset.unwrap_or(0).max(0) as usize,
                published_only: false,
                entity_types: input.entity_types.unwrap_or_default(),
                source_modules: input.source_modules.unwrap_or_default(),
                statuses: input.statuses.unwrap_or_default(),
            })
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(result.into())
    }

    /// Executes public storefront search over published content and published products only.
    async fn storefront_search(
        &self,
        ctx: &Context<'_>,
        input: SearchPreviewInput,
    ) -> Result<SearchPreviewPayload> {
        let app_ctx = ctx.data::<AppContext>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let input = normalize_search_preview_input(input)?;
        let engine = PgSearchEngine::new(app_ctx.db.clone());

        let result = engine
            .search(SearchQuery {
                tenant_id: Some(tenant.id),
                locale: input.locale,
                query: input.query,
                limit: input.limit.unwrap_or(12).clamp(1, 50) as usize,
                offset: input.offset.unwrap_or(0).max(0) as usize,
                published_only: true,
                entity_types: input.entity_types.unwrap_or_default(),
                source_modules: input.source_modules.unwrap_or_default(),
                statuses: input.statuses.unwrap_or_default(),
            })
            .await
            .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

        Ok(result.into())
    }
}

async fn ensure_settings_read_permission(ctx: &Context<'_>) -> Result<()> {
    let app_ctx = ctx.data::<AppContext>()?;
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;
    let tenant = ctx.data::<TenantContext>()?;

    let can_read = RbacService::has_permission(
        &app_ctx.db,
        &tenant.id,
        &auth.user_id,
        &rustok_core::Permission::SETTINGS_READ,
    )
    .await
    .map_err(|err| <FieldError as GraphQLError>::internal_error(&err.to_string()))?;

    if !can_read {
        return Err(<FieldError as GraphQLError>::permission_denied(
            "settings:read required",
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

fn normalize_search_preview_input(input: SearchPreviewInput) -> Result<SearchPreviewInput> {
    let query = normalize_query(&input.query)?;
    let locale = normalize_locale(input.locale.as_deref())?;
    let entity_types = normalize_filter_values("entity_types", input.entity_types)?;
    let source_modules = normalize_filter_values("source_modules", input.source_modules)?;
    let statuses = normalize_filter_values("statuses", input.statuses)?;

    Ok(SearchPreviewInput {
        query,
        locale,
        tenant_id: input.tenant_id,
        limit: input.limit,
        offset: input.offset,
        entity_types: Some(entity_types),
        source_modules: Some(source_modules),
        statuses: Some(statuses),
    })
}

fn normalize_query(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.len() > MAX_SEARCH_QUERY_LEN {
        return Err(FieldError::new(format!(
            "Search query exceeds the maximum length of {MAX_SEARCH_QUERY_LEN} characters"
        )));
    }

    if trimmed.chars().any(|ch| ch.is_control()) {
        return Err(FieldError::new(
            "Search query contains unsupported control characters",
        ));
    }

    Ok(trimmed.to_string())
}

fn normalize_locale(value: Option<&str>) -> Result<Option<String>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };

    if value.len() > MAX_LOCALE_LEN
        || !value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(FieldError::new("Invalid locale format"));
    }

    Ok(Some(value.to_ascii_lowercase()))
}

fn normalize_filter_values(field_name: &str, values: Option<Vec<String>>) -> Result<Vec<String>> {
    let values = values.unwrap_or_default();
    if values.len() > MAX_FILTER_VALUES {
        return Err(FieldError::new(format!(
            "{field_name} exceeds the maximum size of {MAX_FILTER_VALUES} values"
        )));
    }

    values
        .into_iter()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            if normalized.is_empty() {
                return Err(FieldError::new(format!(
                    "{field_name} contains an empty value"
                )));
            }
            if normalized.len() > MAX_FILTER_VALUE_LEN
                || !normalized
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == ':')
            {
                return Err(FieldError::new(format!(
                    "{field_name} contains an invalid value"
                )));
            }
            Ok(normalized)
        })
        .collect()
}
