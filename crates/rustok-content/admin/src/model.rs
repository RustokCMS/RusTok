use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentTenant {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeList {
    pub items: Vec<NodeListItem>,
    pub total: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeListItem {
    pub id: String,
    pub kind: String,
    pub status: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeTranslation {
    pub locale: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeBody {
    pub locale: String,
    pub body: Option<String>,
    pub format: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeDetail {
    pub id: String,
    pub kind: String,
    pub status: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: Option<String>,
    pub translation: Option<NodeTranslation>,
    pub body: Option<NodeBody>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NodeDraft {
    pub locale: String,
    pub kind: String,
    pub title: String,
    pub slug: String,
    pub excerpt: String,
    pub body: String,
    pub body_format: String,
}
