// GraphQL-only API layer for RusTok admin
// All data fetching goes through GraphQL — no REST endpoints

use leptos_graphql::{
    execute as execute_graphql, persisted_query_extension, GraphqlHttpError, GraphqlRequest,
};
use serde::{Deserialize, Serialize};

/// GraphQL endpoint URL — resolves from window.location in browser
fn get_graphql_url() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:5150".to_string());
        format!("{}/api/graphql", origin)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let base = std::env::var("RUSTOK_API_URL")
            .unwrap_or_else(|_| "http://localhost:5150".to_string());
        format!("{}/api/graphql", base)
    }
}

pub type ApiError = GraphqlHttpError;

/// Execute a GraphQL query/mutation
pub async fn request<V, T>(
    query: &str,
    variables: V,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        &get_graphql_url(),
        GraphqlRequest::new(query, Some(variables)),
        token,
        tenant_slug,
    )
    .await
}

/// Execute a GraphQL query with persisted query hash
pub async fn request_with_persisted<V, T>(
    query: &str,
    variables: V,
    sha256_hash: &str,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    V: Serialize,
    T: for<'de> Deserialize<'de>,
{
    execute_graphql(
        &get_graphql_url(),
        GraphqlRequest::new(query, Some(variables))
            .with_extensions(persisted_query_extension(sha256_hash)),
        token,
        tenant_slug,
    )
    .await
}
