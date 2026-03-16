use async_graphql::{Context, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use rustok_media::MediaService;
use rustok_storage::StorageService;

use crate::graphql::common::{require_module_enabled, PaginationInput};
use crate::graphql::schema::module_slug;

use super::types::*;

#[derive(Default)]
pub struct MediaQuery;

#[Object]
impl MediaQuery {
    /// List media assets for a tenant.
    async fn media(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        #[graphql(default)] pagination: PaginationInput,
    ) -> Result<GqlMediaList> {
        require_module_enabled(ctx, module_slug::MEDIA).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let storage = ctx.data::<StorageService>()?;

        let service = MediaService::new(db.clone(), storage.clone());
        let (offset, limit) = pagination.normalize()?;
        let (items, total) = service
            .list(tenant_id, limit as u64, offset as u64)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(GqlMediaList {
            items: items.into_iter().map(Into::into).collect(),
            total: total as i64,
        })
    }

    /// Get a single media asset by ID.
    async fn media_item(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<Option<GqlMediaItem>> {
        require_module_enabled(ctx, module_slug::MEDIA).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let storage = ctx.data::<StorageService>()?;

        let service = MediaService::new(db.clone(), storage.clone());
        match service.get(tenant_id, id).await {
            Ok(item) => Ok(Some(item.into())),
            Err(rustok_media::MediaError::NotFound(_)) => Ok(None),
            Err(e) => Err(async_graphql::Error::new(e.to_string())),
        }
    }

    /// Get all translations for a media asset.
    async fn media_translations(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        media_id: Uuid,
    ) -> Result<Vec<GqlMediaTranslation>> {
        require_module_enabled(ctx, module_slug::MEDIA).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let storage = ctx.data::<StorageService>()?;

        let service = MediaService::new(db.clone(), storage.clone());
        let translations = service
            .get_translations(tenant_id, media_id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;

        Ok(translations.into_iter().map(Into::into).collect())
    }
}
