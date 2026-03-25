use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CurrentTenant {
    pub id: String,
    pub slug: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorefrontContentData {
    pub selected_node: Option<NodeDetail>,
    pub nodes: NodeList,
    pub selected_id: Option<String>,
    pub selected_kind: Option<String>,
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
pub struct NodeDetail {
    pub id: String,
    pub kind: String,
    pub status: String,
    #[serde(rename = "effectiveLocale")]
    pub effective_locale: Option<String>,
    pub translation: Option<NodeTranslation>,
    pub body: Option<NodeBody>,
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
