use leptos_graphql::{
    execute as execute_graphql, persisted_query_extension, GraphqlHttpError, GraphqlRequest,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const API_URL: &str = "http://localhost:3000/api/graphql";
pub const REST_API_URL: &str = "http://localhost:3000";

pub type ApiError = GraphqlHttpError;

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
        API_URL,
        GraphqlRequest::new(query, Some(variables)),
        token,
        tenant_slug,
    )
    .await
}

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
        API_URL,
        GraphqlRequest::new(query, Some(variables))
            .with_extensions(persisted_query_extension(sha256_hash)),
        token,
        tenant_slug,
    )
    .await
}

pub async fn rest_get<T>(
    path: &str,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    T: for<'de> Deserialize<'de>,
{
    let client = reqwest::Client::new();
    let mut req = client.get(format!("{}{}", REST_API_URL, path));

    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    if let Some(slug) = tenant_slug {
        req = req.header("X-Tenant-Slug", slug);
    }

    let res = req.send().await.map_err(|_| ApiError::Network)?;

    if res.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !res.status().is_success() {
        return Err(ApiError::Http(res.status().to_string()));
    }

    res.json().await.map_err(|_| ApiError::Network)
}

pub async fn rest_post<B, T>(
    path: &str,
    body: &B,
    token: Option<String>,
    tenant_slug: Option<String>,
) -> Result<T, ApiError>
where
    B: Serialize,
    T: for<'de> Deserialize<'de>,
{
    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{}{}", REST_API_URL, path))
        .json(body)
        .header("Idempotency-Key", Uuid::new_v4().to_string());

    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    if let Some(slug) = tenant_slug {
        req = req.header("X-Tenant-Slug", slug);
    }

    let res = req.send().await.map_err(|_| ApiError::Network)?;

    if res.status() == 401 {
        return Err(ApiError::Unauthorized);
    }

    if !res.status().is_success() {
        return Err(ApiError::Http(res.status().to_string()));
    }

    res.json().await.map_err(|_| ApiError::Network)
}
