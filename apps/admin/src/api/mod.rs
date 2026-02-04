use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const API_URL: &str = "http://localhost:3000/api/graphql";
pub const REST_API_URL: &str = "http://localhost:3000";

#[derive(Error, Debug, Clone)]
pub enum ApiError {
    #[error("Network error")]
    Network,
    #[error("GraphQL error: {0}")]
    Graphql(String),
    #[error("Http error: {0}")]
    Http(String),
    #[error("Unauthorized")]
    Unauthorized,
}

#[derive(Serialize)]
struct GraphqlRequest<V> {
    query: String,
    variables: V,
}

#[derive(Deserialize)]
struct GraphqlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphqlErrorObj>>,
}

#[derive(Deserialize)]
struct GraphqlErrorObj {
    message: String,
}

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
    let client = reqwest::Client::new();
    let mut req = client.post(API_URL).json(&GraphqlRequest {
        query: query.to_string(),
        variables,
    });

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

    let body: GraphqlResponse<T> = res.json().await.map_err(|_| ApiError::Network)?;

    if let Some(errors) = body.errors {
        if let Some(err) = errors.first() {
            return Err(ApiError::Graphql(err.message.clone()));
        }
    }

    body.data
        .ok_or_else(|| ApiError::Graphql("No data".to_string()))
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
