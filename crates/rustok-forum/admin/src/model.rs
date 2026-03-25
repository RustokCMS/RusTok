use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryDetail {
    pub id: String,
    pub requested_locale: String,
    pub locale: String,
    pub effective_locale: String,
    pub available_locales: Vec<String>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub position: i32,
    pub topic_count: i32,
    pub reply_count: i32,
    pub moderated: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CategoryListItem {
    pub id: String,
    pub locale: String,
    pub effective_locale: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub topic_count: i32,
    pub reply_count: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopicDetail {
    pub id: String,
    pub requested_locale: String,
    pub locale: String,
    pub effective_locale: String,
    pub available_locales: Vec<String>,
    pub category_id: String,
    pub author_id: Option<String>,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub body_format: String,
    pub content_json: Option<Value>,
    pub status: String,
    pub tags: Vec<String>,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub reply_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TopicListItem {
    pub id: String,
    pub locale: String,
    pub effective_locale: String,
    pub category_id: String,
    pub author_id: Option<String>,
    pub title: String,
    pub slug: String,
    pub status: String,
    pub is_pinned: bool,
    pub is_locked: bool,
    pub reply_count: i32,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReplyListItem {
    pub id: String,
    pub locale: String,
    pub effective_locale: String,
    pub topic_id: String,
    pub author_id: Option<String>,
    pub content_preview: String,
    pub status: String,
    pub parent_reply_id: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug)]
pub struct CategoryDraft {
    pub locale: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub position: i32,
    pub moderated: bool,
}

#[derive(Clone, Debug)]
pub struct TopicDraft {
    pub locale: String,
    pub category_id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub body_format: String,
    pub tags: Vec<String>,
}
