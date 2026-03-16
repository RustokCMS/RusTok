use async_graphql::{InputObject, SimpleObject};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use rustok_media::dto::{MediaItem, MediaTranslationItem};

// ── Output types ──────────────────────────────────────────────────────────────

#[derive(SimpleObject, Clone, Debug)]
pub struct GqlMediaItem {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub uploaded_by: Option<Uuid>,
    pub filename: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub storage_driver: String,
    pub public_url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<MediaItem> for GqlMediaItem {
    fn from(m: MediaItem) -> Self {
        Self {
            id: m.id,
            tenant_id: m.tenant_id,
            uploaded_by: m.uploaded_by,
            filename: m.filename,
            original_name: m.original_name,
            mime_type: m.mime_type,
            size: m.size,
            storage_driver: m.storage_driver,
            public_url: m.public_url,
            width: m.width,
            height: m.height,
            created_at: m.created_at,
        }
    }
}

#[derive(SimpleObject, Clone, Debug)]
pub struct GqlMediaList {
    pub items: Vec<GqlMediaItem>,
    pub total: i64,
}

#[derive(SimpleObject, Clone, Debug)]
pub struct GqlMediaTranslation {
    pub id: Uuid,
    pub media_id: Uuid,
    pub locale: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
}

impl From<MediaTranslationItem> for GqlMediaTranslation {
    fn from(t: MediaTranslationItem) -> Self {
        Self {
            id: t.id,
            media_id: t.media_id,
            locale: t.locale,
            title: t.title,
            alt_text: t.alt_text,
            caption: t.caption,
        }
    }
}

// ── Input types ───────────────────────────────────────────────────────────────

#[derive(InputObject, Clone, Debug)]
pub struct UpsertMediaTranslationInput {
    pub locale: String,
    pub title: Option<String>,
    pub alt_text: Option<String>,
    pub caption: Option<String>,
}
