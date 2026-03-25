use leptos_graphql::{execute as execute_graphql, GraphqlHttpError, GraphqlRequest};
use serde::{Deserialize, Serialize};

use crate::model::{
    LaggingSearchDocumentPayload, SearchAdminBootstrap, SearchPreviewFilters, SearchPreviewPayload,
    TriggerSearchRebuildPayload,
};

pub type ApiError = GraphqlHttpError;

const SEARCH_ADMIN_BOOTSTRAP_QUERY: &str = "query SearchAdminBootstrap { availableSearchEngines { kind label providedBy enabled defaultEngine } searchSettingsPreview { tenantId activeEngine fallbackEngine config updatedAt } searchDiagnostics { tenantId totalDocuments publicDocuments contentDocuments productDocuments staleDocuments newestIndexedAt oldestIndexedAt maxLagSeconds state } }";
const SEARCH_PREVIEW_QUERY: &str = "query SearchPreview($input: SearchPreviewInput!) { searchPreview(input: $input) { total tookMs engine items { id entityType sourceModule title snippet score locale payload } facets { name buckets { value count } } } }";
const SEARCH_LAGGING_DOCUMENTS_QUERY: &str = "query SearchLaggingDocuments($limit: Int) { searchLaggingDocuments(limit: $limit) { documentKey documentId sourceModule entityType locale status isPublic title updatedAt indexedAt lagSeconds } }";
const TRIGGER_SEARCH_REBUILD_MUTATION: &str = "mutation TriggerSearchRebuild($input: TriggerSearchRebuildInput!) { triggerSearchRebuild(input: $input) { success queued tenantId targetType targetId } }";

#[derive(Debug, Deserialize)]
struct SearchPreviewResponse {
    #[serde(rename = "searchPreview")]
    search_preview: SearchPreviewPayload,
}

#[derive(Debug, Deserialize)]
struct TriggerSearchRebuildResponse {
    #[serde(rename = "triggerSearchRebuild")]
    trigger_search_rebuild: TriggerSearchRebuildPayload,
}

#[derive(Debug, Deserialize)]
struct SearchLaggingDocumentsResponse {
    #[serde(rename = "searchLaggingDocuments")]
    search_lagging_documents: Vec<LaggingSearchDocumentPayload>,
}

#[derive(Debug, Serialize)]
struct SearchPreviewVariables {
    input: SearchPreviewInput,
}

#[derive(Debug, Serialize)]
struct TriggerSearchRebuildVariables {
    input: TriggerSearchRebuildInput,
}

#[derive(Debug, Serialize)]
struct SearchLaggingDocumentsVariables {
    limit: Option<i32>,
}

#[derive(Debug, Serialize)]
struct SearchPreviewInput {
    query: String,
    locale: Option<String>,
    #[serde(rename = "tenantId")]
    tenant_id: Option<String>,
    limit: Option<i32>,
    offset: Option<i32>,
    #[serde(rename = "entityTypes")]
    entity_types: Option<Vec<String>>,
    #[serde(rename = "sourceModules")]
    source_modules: Option<Vec<String>>,
    statuses: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct TriggerSearchRebuildInput {
    #[serde(rename = "tenantId")]
    tenant_id: Option<String>,
    #[serde(rename = "targetType")]
    target_type: Option<String>,
    #[serde(rename = "targetId")]
    target_id: Option<String>,
}

fn graphql_url() -> String {
    if let Some(url) = option_env!("RUSTOK_GRAPHQL_URL") {
        return url.to_string();
    }

    #[cfg(target_arch = "wasm32")]
    {
        let origin = web_sys::window()
            .and_then(|window| window.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:5150".to_string());
        format!("{origin}/api/graphql")
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base =
            std::env::var("RUSTOK_API_URL").unwrap_or_else(|_| "http://localhost:5150".to_string());
        format!("{base}/api/graphql")
    }
}

async fn request<V, T>(
    query: &str,
    variables: Option<V>,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        &graphql_url(),
        GraphqlRequest::new(query, variables),
        token,
        tenant_slug,
        None,
    )
    .await
}

pub async fn fetch_bootstrap(
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<SearchAdminBootstrap, ApiError> {
    request::<serde_json::Value, SearchAdminBootstrap>(
        SEARCH_ADMIN_BOOTSTRAP_QUERY,
        None,
        token,
        tenant_slug,
    )
    .await
}

pub async fn fetch_search_preview(
    token: Option<String>,
    tenant_slug: Option<String>,
    query: String,
    locale: Option<String>,
    filters: SearchPreviewFilters,
) -> Result<SearchPreviewPayload, ApiError> {
    let response: SearchPreviewResponse = request(
        SEARCH_PREVIEW_QUERY,
        Some(SearchPreviewVariables {
            input: SearchPreviewInput {
                query,
                locale,
                tenant_id: None,
                limit: Some(12),
                offset: Some(0),
                entity_types: (!filters.entity_types.is_empty()).then_some(filters.entity_types),
                source_modules: (!filters.source_modules.is_empty())
                    .then_some(filters.source_modules),
                statuses: (!filters.statuses.is_empty()).then_some(filters.statuses),
            },
        }),
        token,
        tenant_slug,
    )
    .await?;

    Ok(response.search_preview)
}

pub async fn trigger_search_rebuild(
    token: Option<String>,
    tenant_slug: Option<String>,
    target_type: Option<String>,
    target_id: Option<String>,
) -> Result<TriggerSearchRebuildPayload, ApiError> {
    let response: TriggerSearchRebuildResponse = request(
        TRIGGER_SEARCH_REBUILD_MUTATION,
        Some(TriggerSearchRebuildVariables {
            input: TriggerSearchRebuildInput {
                tenant_id: None,
                target_type,
                target_id,
            },
        }),
        token,
        tenant_slug,
    )
    .await?;

    Ok(response.trigger_search_rebuild)
}

pub async fn fetch_lagging_documents(
    token: Option<String>,
    tenant_slug: Option<String>,
    limit: Option<i32>,
) -> Result<Vec<LaggingSearchDocumentPayload>, ApiError> {
    let response: SearchLaggingDocumentsResponse = request(
        SEARCH_LAGGING_DOCUMENTS_QUERY,
        Some(SearchLaggingDocumentsVariables { limit }),
        token,
        tenant_slug,
    )
    .await?;

    Ok(response.search_lagging_documents)
}
