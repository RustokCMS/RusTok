use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorefrontForumData {
    pub categories: ForumCategoryConnection,
    pub topics: ForumTopicConnection,
    pub selected_category_id: Option<String>,
    pub selected_topic_id: Option<String>,
    pub selected_topic: Option<ForumTopicDetail>,
    pub replies: ForumReplyConnection,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumCategoryConnection {
    pub items: Vec<ForumCategoryListItem>,
    pub total: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumTopicConnection {
    pub items: Vec<ForumTopicListItem>,
    pub total: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumReplyConnection {
    pub items: Vec<ForumReplyDetail>,
    pub total: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumCategoryListItem {
    pub id: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "topicCount")]
    pub topic_count: i32,
    #[serde(rename = "replyCount")]
    pub reply_count: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumTopicListItem {
    pub id: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: String,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub title: String,
    pub slug: String,
    pub status: String,
    #[serde(rename = "isPinned")]
    pub is_pinned: bool,
    #[serde(rename = "isLocked")]
    pub is_locked: bool,
    #[serde(rename = "replyCount")]
    pub reply_count: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumTopicDetail {
    pub id: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: String,
    #[serde(rename = "availableLocales")]
    pub available_locales: Vec<String>,
    #[serde(rename = "categoryId")]
    pub category_id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    #[serde(rename = "bodyFormat")]
    pub body_format: String,
    pub status: String,
    pub tags: Vec<String>,
    #[serde(rename = "isPinned")]
    pub is_pinned: bool,
    #[serde(rename = "isLocked")]
    pub is_locked: bool,
    #[serde(rename = "replyCount")]
    pub reply_count: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ForumReplyDetail {
    pub id: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: String,
    #[serde(rename = "topicId")]
    pub topic_id: String,
    pub content: String,
    #[serde(rename = "contentFormat")]
    pub content_format: String,
    pub status: String,
    #[serde(rename = "parentReplyId")]
    pub parent_reply_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
