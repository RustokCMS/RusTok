use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateCategoryInput {
    /// Locale for the name and description
    pub locale: String,
    #[schema(max_length = 255)]
    pub name: String,
    #[schema(max_length = 255)]
    pub slug: Option<String>,
    #[schema(max_length = 1000)]
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateCategoryInput {
    pub locale: String,
    #[schema(max_length = 255)]
    pub name: Option<String>,
    #[schema(max_length = 255)]
    pub slug: Option<String>,
    #[schema(max_length = 1000)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub locale: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryListItem {
    pub id: Uuid,
    pub locale: String,
    pub name: String,
    pub slug: String,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, IntoParams)]
pub struct ListCategoriesFilter {
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
