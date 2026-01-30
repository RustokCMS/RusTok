use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePostInput {
    pub locale: String,
    pub title: String,
    pub body: String,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub publish: bool,
    pub tags: Vec<String>,
    pub metadata: Option<Value>,
}
