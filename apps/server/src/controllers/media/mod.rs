use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use loco_rs::app::AppContext;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rustok_media::{
    dto::{MediaItem, MediaTranslationItem, UpsertTranslationInput},
    MediaError, MediaService,
};
use rustok_storage::StorageService;

use crate::context::TenantContext;
use crate::error::{Error, Result};
use crate::extractors::auth::CurrentUser;

// ── Storage service accessor ──────────────────────────────────────────────────

fn storage_from_ctx(ctx: &AppContext) -> Result<StorageService> {
    ctx.shared_store
        .get::<StorageService>()
        .ok_or(Error::InternalServerError)
}

fn media_error(e: MediaError) -> Error {
    match e {
        MediaError::NotFound(_) => Error::NotFound,
        MediaError::Forbidden => Error::Unauthorized("Access denied".to_string()),
        MediaError::UnsupportedMimeType(t) => {
            Error::BadRequest(format!("Unsupported media type: {t}"))
        }
        MediaError::FileTooLarge { size, max } => Error::BadRequest(format!(
            "File too large: {size} bytes (max {max} bytes)"
        )),
        MediaError::Storage(e) => Error::Message(e.to_string()),
        MediaError::Db(e) => Error::Message(e.to_string()),
    }
}

// ── Query params ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListParams {
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub offset: u64,
}

fn default_limit() -> u64 {
    20
}

#[derive(Serialize)]
pub struct MediaListResponse {
    pub items: Vec<MediaItem>,
    pub total: u64,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// Upload a media file.
///
/// Accepts `multipart/form-data` with a `file` field.
pub async fn upload(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    current_user: CurrentUser,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<MediaItem>)> {
    let storage = storage_from_ctx(&ctx)?;
    let service = MediaService::new(ctx.db.clone(), storage);

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| Error::BadRequest(format!("Multipart error: {e}")))?
    {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name != "file" {
            continue;
        }

        let file_name = field.file_name().unwrap_or("upload.bin").to_string();
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let data = field
            .bytes()
            .await
            .map_err(|e| Error::BadRequest(format!("Failed to read upload: {e}")))?;

        let input = rustok_media::UploadInput {
            tenant_id: tenant.id,
            uploaded_by: Some(current_user.user.id),
            original_name: file_name,
            content_type,
            data,
        };

        let item = service.upload(input).await.map_err(media_error)?;
        return Ok((StatusCode::CREATED, Json(item)));
    }

    Err(Error::BadRequest(
        "No `file` field found in multipart body".to_string(),
    ))
}

/// List media assets for the current tenant.
pub async fn list(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Query(params): Query<ListParams>,
) -> Result<Json<MediaListResponse>> {
    let storage = storage_from_ctx(&ctx)?;
    let service = MediaService::new(ctx.db.clone(), storage);
    let limit = params.limit.clamp(1, 100);
    let (items, total) = service
        .list(tenant.id, limit, params.offset)
        .await
        .map_err(media_error)?;
    Ok(Json(MediaListResponse { items, total }))
}

/// Get a single media asset by ID.
pub async fn get_media(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<MediaItem>> {
    let storage = storage_from_ctx(&ctx)?;
    let service = MediaService::new(ctx.db.clone(), storage);
    let item = service.get(tenant.id, id).await.map_err(media_error)?;
    Ok(Json(item))
}

/// Delete a media asset.
pub async fn delete_media(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    let storage = storage_from_ctx(&ctx)?;
    let service = MediaService::new(ctx.db.clone(), storage);
    service.delete(tenant.id, id).await.map_err(media_error)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Upsert alt-text / title / caption for a locale.
pub async fn upsert_translation(
    State(ctx): State<AppContext>,
    tenant: TenantContext,
    _user: CurrentUser,
    Path((id, locale)): Path<(Uuid, String)>,
    Json(body): Json<UpsertTranslationInput>,
) -> Result<Json<MediaTranslationItem>> {
    let storage = storage_from_ctx(&ctx)?;
    let service = MediaService::new(ctx.db.clone(), storage);
    let result = service
        .upsert_translation(tenant.id, id, UpsertTranslationInput { locale, ..body })
        .await
        .map_err(media_error)?;
    Ok(Json(result))
}

// ── Route registration ────────────────────────────────────────────────────────

pub fn routes() -> loco_rs::controller::Routes {
    use axum::routing::{delete, get, post, put};
    loco_rs::controller::Routes::new()
        .prefix("api/media")
        .add("/", get(list).post(upload))
        .add("/{id}", get(get_media).delete(delete_media))
        .add("/{id}/translations/{locale}", put(upsert_translation))
}
