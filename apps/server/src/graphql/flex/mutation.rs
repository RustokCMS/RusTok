//! GraphQL mutations for Flex field definitions.
//!
//! RBAC is enforced with explicit `flex_schemas:*` permissions resolved by the
//! server RBAC runtime.

use async_graphql::{Context, Object, Result};
use uuid::Uuid;

use rustok_core::{field_schema::FieldType, Permission};
use rustok_events::EventEnvelope;

use crate::context::TenantContext;
use crate::services::event_bus::event_bus_from_context;
use crate::services::field_definition_cache::FieldDefinitionCache;
use crate::services::flex_standalone_service::FlexStandaloneSeaOrmService;
use flex::{
    CreateFieldDefinitionCommand, CreateFlexEntryCommand, CreateFlexSchemaCommand,
    FieldDefRegistry, UpdateFieldDefinitionCommand, UpdateFlexEntryCommand,
    UpdateFlexSchemaCommand,
};

use super::{
    bad_user_input, map_flex_error, require_permission, resolve_entity_type,
    types::{
        CreateFieldDefinitionInput, CreateFlexEntryInput, CreateFlexSchemaInput,
        DeleteFieldDefinitionPayload, DeleteFlexPayload, FieldDefinitionObject, FlexEntryObject,
        FlexSchemaObject, UpdateFieldDefinitionInput, UpdateFlexEntryInput, UpdateFlexSchemaInput,
    },
};

#[derive(Default)]
pub struct FlexMutation;

#[Object]
impl FlexMutation {
    /// Create a new custom field definition.
    ///
    /// Requires `flex_schemas:create`.
    async fn create_field_definition(
        &self,
        ctx: &Context<'_>,
        input: CreateFieldDefinitionInput,
    ) -> Result<FieldDefinitionObject> {
        let auth = require_permission(ctx, Permission::FLEX_SCHEMAS_CREATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let field_type: FieldType =
            serde_json::from_value(serde_json::Value::String(input.field_type.clone()))
                .map_err(|_| bad_user_input("Unknown field_type value"))?;

        let label = serde_json::from_value(input.label)
            .map_err(|_| bad_user_input("label must be a JSON object {\"en\": \"...\"}"))?;

        let description = input
            .description
            .map(|v| {
                serde_json::from_value(v)
                    .map_err(|_| bad_user_input("description must be a JSON object"))
            })
            .transpose()?;

        let validation = input
            .validation
            .map(|v| {
                serde_json::from_value(v)
                    .map_err(|_| bad_user_input("validation must be a valid ValidationRule JSON"))
            })
            .transpose()?;

        let entity_type = resolve_entity_type(input.entity_type)?;

        let registry = ctx.data::<FieldDefRegistry>()?;

        let service_input = CreateFieldDefinitionCommand {
            field_key: input.field_key,
            field_type,
            label,
            description,
            is_localized: input.is_localized,
            is_required: input.is_required,
            default_value: input.default_value,
            validation,
            position: input.position,
        };

        let (model, event) = flex::create_field_definition(
            registry,
            &app_ctx.db,
            tenant.id,
            &entity_type,
            Some(auth.user_id),
            service_input,
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        invalidate_field_def_cache(ctx, tenant.id, &entity_type).await;

        Ok(FieldDefinitionObject::from(model))
    }

    /// Update an existing field definition.
    ///
    /// Requires `flex_schemas:update`.
    async fn update_field_definition(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: UpdateFieldDefinitionInput,
    ) -> Result<FieldDefinitionObject> {
        let auth = require_permission(ctx, Permission::FLEX_SCHEMAS_UPDATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let label = input
            .label
            .map(|v| {
                serde_json::from_value(v).map_err(|_| bad_user_input("label must be a JSON object"))
            })
            .transpose()?;

        let description = input
            .description
            .map(|v| {
                serde_json::from_value(v)
                    .map_err(|_| bad_user_input("description must be a JSON object"))
            })
            .transpose()?;

        let validation = input
            .validation
            .map(|v| {
                serde_json::from_value(v)
                    .map_err(|_| bad_user_input("validation must be a valid ValidationRule JSON"))
            })
            .transpose()?;

        let entity_type = resolve_entity_type(input.entity_type)?;

        let registry = ctx.data::<FieldDefRegistry>()?;

        let service_input = UpdateFieldDefinitionCommand {
            label,
            description,
            is_localized: input.is_localized,
            is_required: input.is_required,
            default_value: input.default_value,
            validation,
            position: input.position,
            is_active: input.is_active,
        };

        let (model, event) = flex::update_field_definition(
            registry,
            &app_ctx.db,
            tenant.id,
            &entity_type,
            Some(auth.user_id),
            id,
            service_input,
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        invalidate_field_def_cache(ctx, tenant.id, &entity_type).await;

        Ok(FieldDefinitionObject::from(model))
    }

    /// Soft-delete a field definition (`is_active = false`).
    ///
    /// Requires `flex_schemas:delete`. Data in `users.metadata` is preserved.
    async fn delete_field_definition(
        &self,
        ctx: &Context<'_>,
        entity_type: Option<String>,
        id: Uuid,
    ) -> Result<DeleteFieldDefinitionPayload> {
        let auth = require_permission(ctx, Permission::FLEX_SCHEMAS_DELETE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let entity_type = resolve_entity_type(entity_type)?;

        let registry = ctx.data::<FieldDefRegistry>()?;

        let event = flex::deactivate_field_definition(
            registry,
            &app_ctx.db,
            tenant.id,
            &entity_type,
            Some(auth.user_id),
            id,
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        invalidate_field_def_cache(ctx, tenant.id, &entity_type).await;

        Ok(DeleteFieldDefinitionPayload { success: true })
    }

    /// Reorder field definitions by supplying an ordered list of ids.
    ///
    /// Requires `flex_schemas:update`.
    async fn reorder_field_definitions(
        &self,
        ctx: &Context<'_>,
        entity_type: Option<String>,
        ids: Vec<Uuid>,
    ) -> Result<Vec<FieldDefinitionObject>> {
        require_permission(ctx, Permission::FLEX_SCHEMAS_UPDATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;

        let entity_type = resolve_entity_type(entity_type)?;

        let registry = ctx.data::<FieldDefRegistry>()?;

        let rows =
            flex::reorder_field_definitions(registry, &app_ctx.db, tenant.id, &entity_type, &ids)
                .await
                .map_err(map_flex_error)?;

        invalidate_field_def_cache(ctx, tenant.id, &entity_type).await;

        Ok(rows.into_iter().map(FieldDefinitionObject::from).collect())
    }

    /// Create a standalone Flex schema.
    ///
    /// Requires `flex_schemas:create`.
    async fn create_flex_schema(
        &self,
        ctx: &Context<'_>,
        input: CreateFlexSchemaInput,
    ) -> Result<FlexSchemaObject> {
        let auth = require_permission(ctx, Permission::FLEX_SCHEMAS_CREATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let service = FlexStandaloneSeaOrmService::new(app_ctx.db.clone());

        let (view, event) = flex::create_schema_with_event(
            &service,
            tenant.id,
            Some(auth.user_id),
            CreateFlexSchemaCommand {
                slug: input.slug,
                name: input.name,
                description: input.description,
                fields_config: parse_fields_config(input.fields_config)?,
                settings: input.settings,
                is_active: input.is_active,
            },
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        Ok(FlexSchemaObject::from(view))
    }

    /// Update a standalone Flex schema.
    ///
    /// Requires `flex_schemas:update`.
    async fn update_flex_schema(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        input: UpdateFlexSchemaInput,
    ) -> Result<FlexSchemaObject> {
        let auth = require_permission(ctx, Permission::FLEX_SCHEMAS_UPDATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let service = FlexStandaloneSeaOrmService::new(app_ctx.db.clone());

        let (view, event) = flex::update_schema_with_event(
            &service,
            tenant.id,
            Some(auth.user_id),
            id,
            UpdateFlexSchemaCommand {
                name: input.name,
                description: input.description,
                fields_config: input.fields_config.map(parse_fields_config).transpose()?,
                settings: input.settings,
                is_active: input.is_active,
            },
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        Ok(FlexSchemaObject::from(view))
    }

    /// Delete a standalone Flex schema.
    ///
    /// Requires `flex_schemas:delete`.
    async fn delete_flex_schema(&self, ctx: &Context<'_>, id: Uuid) -> Result<DeleteFlexPayload> {
        let auth = require_permission(ctx, Permission::FLEX_SCHEMAS_DELETE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let service = FlexStandaloneSeaOrmService::new(app_ctx.db.clone());

        let event = flex::delete_schema_with_event(&service, tenant.id, Some(auth.user_id), id)
            .await
            .map_err(map_flex_error)?;

        publish_event(ctx, event);
        Ok(DeleteFlexPayload { success: true })
    }

    /// Create a standalone Flex entry.
    ///
    /// Requires `flex_entries:create`.
    async fn create_flex_entry(
        &self,
        ctx: &Context<'_>,
        input: CreateFlexEntryInput,
    ) -> Result<FlexEntryObject> {
        let auth = require_permission(ctx, Permission::FLEX_ENTRIES_CREATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let service = FlexStandaloneSeaOrmService::new(app_ctx.db.clone());

        let (view, event) = flex::create_entry_with_event(
            &service,
            tenant.id,
            Some(auth.user_id),
            CreateFlexEntryCommand {
                schema_id: input.schema_id,
                entity_type: input.entity_type,
                entity_id: input.entity_id,
                data: input.data,
                status: input.status,
            },
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        Ok(FlexEntryObject::from(view))
    }

    /// Update a standalone Flex entry.
    ///
    /// Requires `flex_entries:update`.
    async fn update_flex_entry(
        &self,
        ctx: &Context<'_>,
        schema_id: Uuid,
        id: Uuid,
        input: UpdateFlexEntryInput,
    ) -> Result<FlexEntryObject> {
        let auth = require_permission(ctx, Permission::FLEX_ENTRIES_UPDATE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let service = FlexStandaloneSeaOrmService::new(app_ctx.db.clone());

        let (view, event) = flex::update_entry_with_event(
            &service,
            tenant.id,
            Some(auth.user_id),
            schema_id,
            id,
            UpdateFlexEntryCommand {
                data: input.data,
                status: input.status,
            },
        )
        .await
        .map_err(map_flex_error)?;

        publish_event(ctx, event);
        Ok(FlexEntryObject::from(view))
    }

    /// Delete a standalone Flex entry.
    ///
    /// Requires `flex_entries:delete`.
    async fn delete_flex_entry(
        &self,
        ctx: &Context<'_>,
        schema_id: Uuid,
        id: Uuid,
    ) -> Result<DeleteFlexPayload> {
        let auth = require_permission(ctx, Permission::FLEX_ENTRIES_DELETE)?;
        let tenant = ctx.data::<TenantContext>()?;
        let app_ctx = ctx.data::<loco_rs::prelude::AppContext>()?;
        let service = FlexStandaloneSeaOrmService::new(app_ctx.db.clone());

        let event =
            flex::delete_entry_with_event(&service, tenant.id, Some(auth.user_id), schema_id, id)
                .await
                .map_err(map_flex_error)?;

        publish_event(ctx, event);
        Ok(DeleteFlexPayload { success: true })
    }
}

fn parse_fields_config(
    value: serde_json::Value,
) -> Result<Vec<rustok_core::field_schema::FieldDefinition>> {
    serde_json::from_value(value).map_err(|_| {
        bad_user_input(
            "fields_config must be a valid JSON array of FieldDefinition-compatible objects",
        )
    })
}

async fn invalidate_field_def_cache(ctx: &Context<'_>, tenant_id: Uuid, entity_type: &str) {
    if let Ok(cache) = ctx.data::<FieldDefinitionCache>() {
        flex::invalidate_field_definition_cache(cache, tenant_id, entity_type).await;
    }
}

/// Fire-and-forget event publishing: errors are logged but not propagated.
fn publish_event(ctx: &Context<'_>, event: EventEnvelope) {
    if let Ok(app_ctx) = ctx.data::<loco_rs::prelude::AppContext>() {
        let bus = event_bus_from_context(app_ctx);
        if let Err(e) = bus.publish_envelope(event) {
            tracing::warn!(error = %e, "Failed to publish flex event");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::context::AuthContext;
    use crate::graphql::flex::auth_has_permission;
    use rustok_core::Permission;
    use uuid::Uuid;

    fn auth_context(permissions: Vec<Permission>) -> AuthContext {
        AuthContext {
            user_id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            permissions,
            client_id: None,
            scopes: Vec::new(),
            grant_type: "direct".to_string(),
        }
    }

    #[test]
    fn explicit_flex_permission_is_accepted() {
        let auth = auth_context(vec![Permission::FLEX_SCHEMAS_UPDATE]);
        assert!(auth_has_permission(&auth, Permission::FLEX_SCHEMAS_UPDATE));
    }

    #[test]
    fn manage_permission_grants_delete() {
        let auth = auth_context(vec![Permission::FLEX_SCHEMAS_MANAGE]);
        assert!(auth_has_permission(&auth, Permission::FLEX_SCHEMAS_DELETE));
    }

    #[test]
    fn missing_permission_is_rejected() {
        let auth = auth_context(vec![Permission::FLEX_SCHEMAS_UPDATE]);
        assert!(!auth_has_permission(&auth, Permission::FLEX_SCHEMAS_DELETE));
    }
}
