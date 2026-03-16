use async_graphql::{Context, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use rustok_media::{dto::UpsertTranslationInput, MediaService};
use rustok_storage::StorageService;

use crate::graphql::common::require_module_enabled;
use crate::graphql::schema::module_slug;

use super::types::*;

#[derive(Default)]
pub struct MediaMutation;

#[Object]
impl MediaMutation {
    /// Delete a media asset and remove it from storage.
    async fn delete_media(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        id: Uuid,
    ) -> Result<bool> {
        require_module_enabled(ctx, module_slug::MEDIA).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let storage = ctx.data::<StorageService>()?;

        let service = MediaService::new(db.clone(), storage.clone());
        service
            .delete(tenant_id, id)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(true)
    }

    /// Upsert alt-text / title / caption for a given locale.
    async fn upsert_media_translation(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        media_id: Uuid,
        input: UpsertMediaTranslationInput,
    ) -> Result<GqlMediaTranslation> {
        require_module_enabled(ctx, module_slug::MEDIA).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let storage = ctx.data::<StorageService>()?;

        let service = MediaService::new(db.clone(), storage.clone());
        let result = service
            .upsert_translation(
                tenant_id,
                media_id,
                UpsertTranslationInput {
                    locale: input.locale,
                    title: input.title,
                    alt_text: input.alt_text,
                    caption: input.caption,
                },
            )
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?;
        Ok(result.into())
    }
}
