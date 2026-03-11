use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateReplyInput {
    pub locale: String,
    pub content: String,
    pub content_format: Option<String>,
    pub content_json: Option<serde_json::Value>,
    pub parent_reply_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateReplyInput {
    pub locale: String,
    pub content: Option<String>,
    pub content_format: Option<String>,
    pub content_json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, IntoParams)]
pub struct ListRepliesFilter {
    pub locale: Option<String>,
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReplyResponse {
    pub id: Uuid,
    pub locale: String,
    pub effective_locale: String,
    pub topic_id: Uuid,
    pub author_id: Option<Uuid>,
    pub content: String,
    pub status: String,
    pub parent_reply_id: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ReplyListItem {
    pub id: Uuid,
    pub locale: String,
    pub effective_locale: String,
    pub topic_id: Uuid,
    pub author_id: Option<Uuid>,
    pub content_preview: String,
    pub status: String,
    pub parent_reply_id: Option<Uuid>,
    pub created_at: String,
}
