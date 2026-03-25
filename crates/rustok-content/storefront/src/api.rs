use leptos_graphql::{execute as execute_graphql, GraphqlHttpError, GraphqlRequest};
use serde::{Deserialize, Serialize};

use crate::model::{CurrentTenant, NodeDetail, NodeList, StorefrontContentData};

pub type ApiError = GraphqlHttpError;

const CURRENT_TENANT_QUERY: &str = "query ContentCurrentTenant { currentTenant { id slug name } }";
const PUBLISHED_NODES_QUERY: &str = "query StorefrontContentNodes($tenantId: UUID!, $filter: NodesFilter) { nodes(tenantId: $tenantId, filter: $filter) { total items { id kind status effectiveLocale title slug excerpt createdAt publishedAt } } }";
const NODE_QUERY: &str = "query StorefrontContentNode($tenantId: UUID!, $id: UUID!, $locale: String) { node(tenantId: $tenantId, id: $id, locale: $locale) { id kind status effectiveLocale translation { locale title slug excerpt } body { locale body format updatedAt } publishedAt } }";

#[derive(Debug, Deserialize)]
struct CurrentTenantResponse {
    #[serde(rename = "currentTenant")]
    current_tenant: CurrentTenant,
}

#[derive(Debug, Deserialize)]
struct NodesResponse {
    nodes: NodeList,
}

#[derive(Debug, Deserialize)]
struct NodeResponse {
    node: Option<NodeDetail>,
}

#[derive(Debug, Serialize)]
struct TenantScopedVariables<T> {
    #[serde(rename = "tenantId")]
    tenant_id: String,
    #[serde(flatten)]
    extra: T,
}

#[derive(Debug, Serialize)]
struct NodesVariables {
    filter: NodesFilter,
}

#[derive(Debug, Serialize)]
struct NodeVariables {
    id: String,
    locale: Option<String>,
}

#[derive(Debug, Serialize)]
struct NodesFilter {
    kind: Option<String>,
    status: Option<String>,
    locale: Option<String>,
    page: Option<u64>,
    #[serde(rename = "perPage")]
    per_page: Option<u64>,
}

fn configured_tenant_slug() -> Option<String> {
    [
        "RUSTOK_TENANT_SLUG",
        "NEXT_PUBLIC_TENANT_SLUG",
        "NEXT_PUBLIC_DEFAULT_TENANT_SLUG",
    ]
    .into_iter()
    .find_map(|key| {
        std::env::var(key).ok().and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
    })
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

async fn request<V, T>(query: &str, variables: Option<V>) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        &graphql_url(),
        GraphqlRequest::new(query, variables),
        None,
        configured_tenant_slug(),
        None,
    )
    .await
}

async fn fetch_current_tenant() -> Result<CurrentTenant, ApiError> {
    let response: CurrentTenantResponse =
        request::<serde_json::Value, CurrentTenantResponse>(CURRENT_TENANT_QUERY, None).await?;
    Ok(response.current_tenant)
}

fn tenant_scoped<T>(tenant_id: String, extra: T) -> TenantScopedVariables<T> {
    TenantScopedVariables { tenant_id, extra }
}

pub async fn fetch_storefront_content(
    selected_id: Option<String>,
    kind: Option<String>,
    locale: Option<String>,
) -> Result<StorefrontContentData, ApiError> {
    let tenant = fetch_current_tenant().await?;
    let response: NodesResponse = request(
        PUBLISHED_NODES_QUERY,
        Some(tenant_scoped(
            tenant.id.clone(),
            NodesVariables {
                filter: NodesFilter {
                    kind: kind.clone(),
                    status: Some("PUBLISHED".to_string()),
                    locale: locale.clone(),
                    page: Some(1),
                    per_page: Some(8),
                },
            },
        )),
    )
    .await?;

    let resolved_selected_id =
        selected_id.or_else(|| response.nodes.items.first().map(|item| item.id.clone()));
    let selected_node = if let Some(node_id) = resolved_selected_id.clone() {
        let response: NodeResponse = request(
            NODE_QUERY,
            Some(tenant_scoped(
                tenant.id,
                NodeVariables {
                    id: node_id,
                    locale,
                },
            )),
        )
        .await?;
        response.node
    } else {
        None
    };

    Ok(StorefrontContentData {
        selected_node,
        nodes: response.nodes,
        selected_id: resolved_selected_id,
        selected_kind: kind,
    })
}
