//! GraphQL mutations for Flex field definitions.
//!
//! RBAC is enforced with explicit `flex_schemas:*` permissions resolved by the
//! server RBAC runtime.

use async_graphql::{Context, FieldError, Object, Result};
use uuid::Uuid;

use rustok_core::{field_schema::FieldType, Permission};
use rustok_events::EventEnvelope;

use crate::context::{AuthContext, TenantContext};
use crate::graphql::errors::GraphQLError;
use crate::services::event_bus::event_bus_from_context;
use crate::services::field_definition_cache::FieldDefinitionCache;
use flex::{CreateFieldDefinitionCommand, FieldDefRegistry, UpdateFieldDefinitionCommand};

use super::{
    bad_user_input, map_flex_error, resolve_entity_type,
    types::{
        CreateFieldDefinitionInput, DeleteFieldDefinitionPayload, FieldDefinitionObject,
        UpdateFieldDefinitionInput,
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
}

fn auth_has_permission(auth: &AuthContext, permission: Permission) -> bool {
    rustok_rbac::has_effective_permission_in_set(&auth.permissions, &permission)
}

fn require_permission<'a>(ctx: &'a Context<'a>, permission: Permission) -> Result<&'a AuthContext> {
    let auth = ctx
        .data::<AuthContext>()
        .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

    if !auth_has_permission(auth, permission) {
        return Err(<FieldError as GraphQLError>::permission_denied(&format!(
            "{permission} required"
        )));
    }

    Ok(auth)
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
    use super::auth_has_permission;
    use crate::context::AuthContext;
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
