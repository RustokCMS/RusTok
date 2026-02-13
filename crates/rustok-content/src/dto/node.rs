use crate::entities::node::ContentStatus;
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use utoipa::ToSchema;

use super::validation::{
    validate_body_format, validate_kind, validate_locale, validate_slug,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct CreateNodeInput {
    #[validate(length(min = 1, max = 64, message = "Kind must be 1-64 characters"))]
    #[validate(custom(function = "validate_kind", message = "Invalid kind type"))]
    pub kind: String,
    
    pub status: Option<ContentStatus>,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    
    pub position: Option<i32>,
    pub depth: Option<i32>,
    pub reply_count: Option<i32>,
    
    #[serde(default)]
    pub metadata: Value,
    
    #[validate(length(min = 1, message = "At least one translation required"))]
    #[validate(nested)]
    pub translations: Vec<NodeTranslationInput>,
    
    #[validate(nested)]
    pub bodies: Vec<BodyInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct NodeTranslationInput {
    #[validate(custom(function = "validate_locale", message = "Invalid locale format"))]
    pub locale: String,
    
    #[validate(length(min = 1, max = 255, message = "Title must be 1-255 characters"))]
    pub title: Option<String>,
    
    #[validate(custom(function = "validate_slug", message = "Invalid slug format"))]
    pub slug: Option<String>,
    
    #[validate(length(max = 1000, message = "Excerpt must be max 1000 characters"))]
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Validate)]
pub struct BodyInput {
    #[validate(custom(function = "validate_locale", message = "Invalid locale format"))]
    pub locale: String,
    
    #[validate(length(max = 1_000_000, message = "Body too large (max 1MB)"))]
    pub body: Option<String>,
    
    #[validate(custom(function = "validate_body_format", message = "Invalid body format"))]
    pub format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema)]
pub struct UpdateNodeInput {
    pub status: Option<ContentStatus>,
    pub parent_id: Option<Option<Uuid>>,
    pub author_id: Option<Option<Uuid>>,
    pub category_id: Option<Option<Uuid>>,
    pub position: Option<i32>,
    pub depth: Option<i32>,
    pub reply_count: Option<i32>,
    pub metadata: Option<Value>,
    pub translations: Option<Vec<NodeTranslationInput>>,
    pub bodies: Option<Vec<BodyInput>>,
    #[schema(value_type = String, format = DateTime)]
    pub published_at: Option<Option<DateTimeWithTimeZone>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, ToSchema, utoipa::IntoParams)]
pub struct ListNodesFilter {
    pub kind: Option<String>,
    pub status: Option<ContentStatus>,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
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
pub struct NodeResponse {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub kind: String,
    pub status: ContentStatus,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub position: i32,
    pub depth: i32,
    pub reply_count: i32,
    pub metadata: Value,
    pub created_at: String,
    pub updated_at: String,
    pub published_at: Option<String>,
    pub translations: Vec<NodeTranslationResponse>,
    pub bodies: Vec<BodyResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeTranslationResponse {
    pub locale: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct BodyResponse {
    pub locale: String,
    pub body: Option<String>,
    pub format: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodeListItem {
    pub id: Uuid,
    pub kind: String,
    pub status: ContentStatus,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub author_id: Option<Uuid>,
    pub created_at: String,
    pub published_at: Option<String>,
}
