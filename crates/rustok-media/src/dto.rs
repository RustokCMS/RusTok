use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Input for uploading a new media asset.
pub struct UploadInput {
    pub tenant_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    /// Original filename as provided by the client.
    pub original_name: String,
    pub content_type: String,
    pub data: bytes::Bytes,
}

/// Represents a stored media asset returned by the service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_path: String,
    pub storage_driver: String,
    pub public_url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Input for updating alt-text / title for a locale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertTranslationInput {
    pub locale: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaTranslationItem {
    pub id: Uuid,
    pub media_id: Uuid,
    pub locale: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
}

/// Allowed MIME type groups.
pub const ALLOWED_MIME_PREFIXES: &[&str] = &["image/", "video/", "audio/", "application/pdf"];

/// Default maximum upload size: 50 MiB.
pub const DEFAULT_MAX_SIZE: u64 = 50 * 1024 * 1024;
