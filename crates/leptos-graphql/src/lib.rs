use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const GRAPHQL_ENDPOINT: &str = "/api/graphql";
pub const TENANT_HEADER: &str = "X-Tenant-Slug";
pub const AUTH_HEADER: &str = "Authorization";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GraphqlRequest<V = Value> {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<V>,
}

impl<V> GraphqlRequest<V> {
    pub fn new(query: impl Into<String>, variables: Option<V>) -> Self {
        Self {
            query: query.into(),
            variables,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GraphqlResponse<T> {
    pub data: Option<T>,
    pub errors: Option<Vec<GraphqlError>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphqlError {
    pub message: String,
}
