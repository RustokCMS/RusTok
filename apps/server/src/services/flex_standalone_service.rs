//! SeaORM-backed adapter implementation of `flex::FlexStandaloneService`.

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use uuid::Uuid;

use rustok_core::{
    build_locale_candidates, field_schema::FlexError, locale_tags_match, normalize_locale_tag,
    PLATFORM_FALLBACK_LOCALE,
};

use crate::models::{flex_entries, flex_schema_translations, flex_schemas, tenants};
use crate::services::flex_standalone_validation_service::FlexStandaloneValidationService;

pub struct FlexStandaloneSeaOrmService {
    db: DatabaseConnection,
}

impl FlexStandaloneSeaOrmService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn schema_to_view(
        model: flex_schemas::Model,
        translation: Option<&flex_schema_translations::Model>,
    ) -> flex::FlexSchemaView {
        let slug_fallback = model.slug.clone();
        let fields_config = model.parse_field_definitions().unwrap_or_default();

        flex::FlexSchemaView {
            id: model.id,
            slug: model.slug,
            name: translation
                .map(|row| row.name.clone())
                .unwrap_or(slug_fallback),
            description: translation.and_then(|row| row.description.clone()),
            fields_config,
            settings: model.settings,
            is_active: model.is_active,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }

    fn entry_to_view(model: flex_entries::Model) -> flex::FlexEntryView {
        flex::FlexEntryView {
            id: model.id,
            schema_id: model.schema_id,
            entity_type: model.entity_type,
            entity_id: model.entity_id,
            data: model.data,
            status: model.status,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }

    async fn get_schema_or_not_found(
        &self,
        tenant_id: Uuid,
        schema_id: Uuid,
    ) -> Result<flex_schemas::Model, FlexError> {
        flex_schemas::Entity::find_by_id(schema_id)
            .filter(flex_schemas::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?
            .ok_or(FlexError::NotFound(schema_id))
    }

    async fn tenant_default_locale(&self, tenant_id: Uuid) -> Result<String, FlexError> {
        let tenant = tenants::Entity::find_by_id(&self.db, tenant_id)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(tenant
            .and_then(|row| normalize_locale_tag(&row.default_locale))
            .unwrap_or_else(|| PLATFORM_FALLBACK_LOCALE.to_string()))
    }

    fn select_schema_translation<'a>(
        translations: &'a [flex_schema_translations::Model],
        preferred_locale: &str,
    ) -> Option<&'a flex_schema_translations::Model> {
        let candidates = build_locale_candidates(
            [Some(preferred_locale), Some(PLATFORM_FALLBACK_LOCALE)],
            true,
        );

        for candidate in candidates {
            if let Some(row) = translations
                .iter()
                .find(|translation| locale_tags_match(&translation.locale, &candidate))
            {
                return Some(row);
            }
        }

        translations.first()
    }

    async fn load_schema_translation_map(
        &self,
        schema_ids: &[Uuid],
    ) -> Result<HashMap<Uuid, Vec<flex_schema_translations::Model>>, FlexError> {
        if schema_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let rows = flex_schema_translations::Entity::find()
            .filter(flex_schema_translations::Column::SchemaId.is_in(schema_ids.iter().copied()))
            .all(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        let mut by_schema_id: HashMap<Uuid, Vec<flex_schema_translations::Model>> = HashMap::new();
        for row in rows {
            by_schema_id.entry(row.schema_id).or_default().push(row);
        }

        Ok(by_schema_id)
    }

    async fn upsert_schema_translation(
        &self,
        schema_id: Uuid,
        locale: &str,
        slug_fallback: &str,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<flex_schema_translations::Model, FlexError> {
        let existing = flex_schema_translations::Entity::find()
            .filter(flex_schema_translations::Column::SchemaId.eq(schema_id))
            .filter(flex_schema_translations::Column::Locale.eq(locale))
            .one(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        match existing {
            Some(row) => {
                let mut model: flex_schema_translations::ActiveModel = row.into();
                if let Some(name) = name {
                    model.name = Set(name);
                }
                if let Some(description) = description {
                    model.description = Set(Some(description));
                }

                model
                    .update(&self.db)
                    .await
                    .map_err(|e| FlexError::Database(e.to_string()))
            }
            None => flex_schema_translations::ActiveModel {
                schema_id: Set(schema_id),
                locale: Set(locale.to_string()),
                name: Set(name.unwrap_or_else(|| slug_fallback.to_string())),
                description: Set(description),
                created_at: sea_orm::ActiveValue::NotSet,
                updated_at: sea_orm::ActiveValue::NotSet,
            }
            .insert(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string())),
        }
    }

    async fn normalize_payload(
        &self,
        schema: &flex_schemas::Model,
        data: JsonValue,
    ) -> Result<JsonValue, FlexError> {
        FlexStandaloneValidationService::validate_entry_against_schema(schema, data)
    }
}

#[async_trait]
impl flex::FlexStandaloneService for FlexStandaloneSeaOrmService {
    async fn list_schemas(&self, tenant_id: Uuid) -> Result<Vec<flex::FlexSchemaView>, FlexError> {
        let preferred_locale = self.tenant_default_locale(tenant_id).await?;
        let rows = flex_schemas::Entity::find()
            .filter(flex_schemas::Column::TenantId.eq(tenant_id))
            .order_by_asc(flex_schemas::Column::Slug)
            .all(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        let schema_ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
        let translations = self.load_schema_translation_map(&schema_ids).await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let translation = translations
                    .get(&row.id)
                    .and_then(|items| Self::select_schema_translation(items, &preferred_locale));
                Self::schema_to_view(row, translation)
            })
            .collect())
    }

    async fn find_schema(
        &self,
        tenant_id: Uuid,
        schema_id: Uuid,
    ) -> Result<Option<flex::FlexSchemaView>, FlexError> {
        let preferred_locale = self.tenant_default_locale(tenant_id).await?;
        let row = flex_schemas::Entity::find_by_id(schema_id)
            .filter(flex_schemas::Column::TenantId.eq(tenant_id))
            .one(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let translations = self.load_schema_translation_map(&[row.id]).await?;
        let translation = translations
            .get(&row.id)
            .and_then(|items| Self::select_schema_translation(items, &preferred_locale));

        Ok(Some(Self::schema_to_view(row, translation)))
    }

    async fn create_schema(
        &self,
        tenant_id: Uuid,
        _actor_id: Option<Uuid>,
        input: flex::CreateFlexSchemaCommand,
    ) -> Result<flex::FlexSchemaView, FlexError> {
        let locale = self.tenant_default_locale(tenant_id).await?;
        let row = flex_schemas::ActiveModel {
            id: Set(rustok_core::generate_id()),
            tenant_id: Set(tenant_id),
            slug: Set(input.slug),
            fields_config: Set(serde_json::to_value(input.fields_config).unwrap_or_default()),
            settings: Set(input.settings.unwrap_or_else(|| serde_json::json!({}))),
            is_active: Set(input.is_active.unwrap_or(true)),
            created_at: sea_orm::ActiveValue::NotSet,
            updated_at: sea_orm::ActiveValue::NotSet,
        }
        .insert(&self.db)
        .await
        .map_err(|e| FlexError::Database(e.to_string()))?;

        let translation = self
            .upsert_schema_translation(
                row.id,
                &locale,
                &row.slug,
                Some(input.name),
                input.description,
            )
            .await?;

        Ok(Self::schema_to_view(row, Some(&translation)))
    }

    async fn update_schema(
        &self,
        tenant_id: Uuid,
        _actor_id: Option<Uuid>,
        schema_id: Uuid,
        input: flex::UpdateFlexSchemaCommand,
    ) -> Result<flex::FlexSchemaView, FlexError> {
        let locale = self.tenant_default_locale(tenant_id).await?;
        let row = self.get_schema_or_not_found(tenant_id, schema_id).await?;
        let mut model: flex_schemas::ActiveModel = row.into();

        if let Some(fields_config) = input.fields_config {
            model.fields_config = Set(serde_json::to_value(fields_config).unwrap_or_default());
        }
        if let Some(settings) = input.settings {
            model.settings = Set(settings);
        }
        if let Some(is_active) = input.is_active {
            model.is_active = Set(is_active);
        }

        let updated = model
            .update(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        let translation = if input.name.is_some() || input.description.is_some() {
            Some(
                self.upsert_schema_translation(
                    updated.id,
                    &locale,
                    &updated.slug,
                    input.name,
                    input.description,
                )
                .await?,
            )
        } else {
            let mut translations = self.load_schema_translation_map(&[updated.id]).await?;
            translations
                .remove(&updated.id)
                .and_then(|items| Self::select_schema_translation(&items, &locale).cloned())
        };

        Ok(Self::schema_to_view(updated, translation.as_ref()))
    }

    async fn delete_schema(
        &self,
        tenant_id: Uuid,
        _actor_id: Option<Uuid>,
        schema_id: Uuid,
    ) -> Result<(), FlexError> {
        let row = self.get_schema_or_not_found(tenant_id, schema_id).await?;

        flex_schemas::Entity::delete_by_id(row.id)
            .exec(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_entries(
        &self,
        tenant_id: Uuid,
        schema_id: Uuid,
    ) -> Result<Vec<flex::FlexEntryView>, FlexError> {
        self.get_schema_or_not_found(tenant_id, schema_id).await?;

        let rows = flex_entries::Entity::find()
            .filter(flex_entries::Column::TenantId.eq(tenant_id))
            .filter(flex_entries::Column::SchemaId.eq(schema_id))
            .order_by_asc(flex_entries::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Self::entry_to_view).collect())
    }

    async fn find_entry(
        &self,
        tenant_id: Uuid,
        schema_id: Uuid,
        entry_id: Uuid,
    ) -> Result<Option<flex::FlexEntryView>, FlexError> {
        let row = flex_entries::Entity::find_by_id(entry_id)
            .filter(flex_entries::Column::TenantId.eq(tenant_id))
            .filter(flex_entries::Column::SchemaId.eq(schema_id))
            .one(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(row.map(Self::entry_to_view))
    }

    async fn create_entry(
        &self,
        tenant_id: Uuid,
        _actor_id: Option<Uuid>,
        input: flex::CreateFlexEntryCommand,
    ) -> Result<flex::FlexEntryView, FlexError> {
        let schema = self
            .get_schema_or_not_found(tenant_id, input.schema_id)
            .await?;
        let normalized = self.normalize_payload(&schema, input.data).await?;

        let row = flex_entries::ActiveModel {
            id: Set(rustok_core::generate_id()),
            tenant_id: Set(tenant_id),
            schema_id: Set(input.schema_id),
            entity_type: Set(input.entity_type),
            entity_id: Set(input.entity_id),
            data: Set(normalized),
            status: Set(input.status.unwrap_or_else(|| "draft".to_string())),
            created_at: sea_orm::ActiveValue::NotSet,
            updated_at: sea_orm::ActiveValue::NotSet,
        }
        .insert(&self.db)
        .await
        .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(Self::entry_to_view(row))
    }

    async fn update_entry(
        &self,
        tenant_id: Uuid,
        _actor_id: Option<Uuid>,
        schema_id: Uuid,
        entry_id: Uuid,
        input: flex::UpdateFlexEntryCommand,
    ) -> Result<flex::FlexEntryView, FlexError> {
        let schema = self.get_schema_or_not_found(tenant_id, schema_id).await?;
        let row = flex_entries::Entity::find_by_id(entry_id)
            .filter(flex_entries::Column::TenantId.eq(tenant_id))
            .filter(flex_entries::Column::SchemaId.eq(schema_id))
            .one(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?
            .ok_or(FlexError::NotFound(entry_id))?;

        let mut model: flex_entries::ActiveModel = row.into();

        if let Some(data) = input.data {
            let normalized = self.normalize_payload(&schema, data).await?;
            model.data = Set(normalized);
        }

        if let Some(status) = input.status {
            model.status = Set(status);
        }

        let updated = model
            .update(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(Self::entry_to_view(updated))
    }

    async fn delete_entry(
        &self,
        tenant_id: Uuid,
        _actor_id: Option<Uuid>,
        schema_id: Uuid,
        entry_id: Uuid,
    ) -> Result<(), FlexError> {
        let row = flex_entries::Entity::find_by_id(entry_id)
            .filter(flex_entries::Column::TenantId.eq(tenant_id))
            .filter(flex_entries::Column::SchemaId.eq(schema_id))
            .one(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?
            .ok_or(FlexError::NotFound(entry_id))?;

        flex_entries::Entity::delete_by_id(row.id)
            .exec(&self.db)
            .await
            .map_err(|e| FlexError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FlexStandaloneSeaOrmService;
    use crate::models::flex_schema_translations;
    use chrono::Utc;
    use uuid::Uuid;

    fn translation(locale: &str, name: &str) -> flex_schema_translations::Model {
        let now = Utc::now().fixed_offset();
        flex_schema_translations::Model {
            schema_id: Uuid::new_v4(),
            locale: locale.to_string(),
            name: name.to_string(),
            description: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn select_schema_translation_prefers_exact_match_then_language_fallback() {
        let translations = vec![
            translation("pt", "Portuguese"),
            translation("en", "English"),
            translation("pt-BR", "Portuguese Brazil"),
        ];

        let selected =
            FlexStandaloneSeaOrmService::select_schema_translation(&translations, "pt-BR")
                .expect("translation must be selected");
        assert_eq!(selected.locale, "pt-BR");

        let selected =
            FlexStandaloneSeaOrmService::select_schema_translation(&translations, "pt-PT")
                .expect("translation must be selected");
        assert_eq!(selected.locale, "pt");
    }

    #[test]
    fn select_schema_translation_falls_back_to_en_then_first_available() {
        let translations = vec![translation("ru", "Russian"), translation("en", "English")];

        let selected =
            FlexStandaloneSeaOrmService::select_schema_translation(&translations, "de-DE")
                .expect("translation must be selected");
        assert_eq!(selected.locale, "en");

        let translations = vec![translation("ru", "Russian")];
        let selected =
            FlexStandaloneSeaOrmService::select_schema_translation(&translations, "de-DE")
                .expect("translation must be selected");
        assert_eq!(selected.locale, "ru");
    }
}
