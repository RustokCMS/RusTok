use async_graphql::{Context, Object, Result};
use rustok_api::graphql::require_module_enabled;
use rustok_core::Permission;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::NodeService;

use super::{require_content_permission, types::*, MODULE_SLUG};

#[derive(Default)]
pub struct ContentMutation;

#[Object]
impl ContentMutation {
    async fn create_node(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        author_id: Option<Uuid>,
        input: CreateNodeInput,
    ) -> Result<GqlNode> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_CREATE],
            "Permission denied: nodes:create required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let domain_input = crate::dto::CreateNodeInput {
            parent_id: input.parent_id,
            author_id: input.author_id.or(author_id),
            kind: input.kind,
            category_id: input.category_id,
            status: input.status.map(Into::into),
            position: input.position,
            depth: input.depth,
            reply_count: input.reply_count,
            metadata: input
                .metadata
                .unwrap_or_else(|| serde_json::Value::Object(Default::default())),
            translations: input
                .translations
                .into_iter()
                .map(|translation| crate::dto::NodeTranslationInput {
                    locale: translation.locale,
                    title: translation.title,
                    slug: translation.slug,
                    excerpt: translation.excerpt,
                })
                .collect(),
            bodies: input
                .bodies
                .into_iter()
                .map(|body| crate::dto::BodyInput {
                    locale: body.locale,
                    body: body.body,
                    format: body.format,
                })
                .collect(),
        };

        let node: crate::dto::NodeResponse = service
            .create_node(tenant_id, auth.security_context(), domain_input)
            .await?;

        Ok(node.into())
    }

    async fn update_node(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        tenant_id: Uuid,
        author_id: Option<Uuid>,
        input: UpdateNodeInput,
    ) -> Result<GqlNode> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_UPDATE],
            "Permission denied: nodes:update required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let resolved_author_id = input.author_id.or(author_id);
        let domain_input = crate::dto::UpdateNodeInput {
            parent_id: input.parent_id.map(Some),
            author_id: resolved_author_id.map(Some),
            category_id: input.category_id.map(Some),
            status: input.status.map(Into::into),
            position: input.position,
            depth: input.depth,
            reply_count: input.reply_count,
            metadata: None,
            translations: input.translations.map(|translations| {
                translations
                    .into_iter()
                    .map(|translation| crate::dto::NodeTranslationInput {
                        locale: translation.locale,
                        title: translation.title,
                        slug: translation.slug,
                        excerpt: translation.excerpt,
                    })
                    .collect()
            }),
            bodies: input.bodies.map(|bodies| {
                bodies
                    .into_iter()
                    .map(|body| crate::dto::BodyInput {
                        locale: body.locale,
                        body: body.body,
                        format: body.format,
                    })
                    .collect()
            }),
            expected_version: input.expected_version,
        };

        let node: crate::dto::NodeResponse = service
            .update_node(tenant_id, id, auth.security_context(), domain_input)
            .await?;

        Ok(node.into())
    }

    async fn delete_node(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        tenant_id: Uuid,
        _author_id: Option<Uuid>,
    ) -> Result<bool> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_DELETE],
            "Permission denied: nodes:delete required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        service
            .delete_node(tenant_id, id, auth.security_context())
            .await?;

        Ok(true)
    }

    async fn publish_node(&self, ctx: &Context<'_>, id: Uuid, tenant_id: Uuid) -> Result<GqlNode> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_UPDATE],
            "Permission denied: nodes:update required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let node = service
            .publish_node(tenant_id, id, auth.security_context())
            .await?;

        Ok(node.into())
    }

    async fn unpublish_node(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        tenant_id: Uuid,
    ) -> Result<GqlNode> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_UPDATE],
            "Permission denied: nodes:update required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let node = service
            .unpublish_node(tenant_id, id, auth.security_context())
            .await?;

        Ok(node.into())
    }

    async fn archive_node(&self, ctx: &Context<'_>, id: Uuid, tenant_id: Uuid) -> Result<GqlNode> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_UPDATE],
            "Permission denied: nodes:update required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let node = service
            .archive_node(tenant_id, id, auth.security_context())
            .await?;

        Ok(node.into())
    }

    async fn restore_node(&self, ctx: &Context<'_>, id: Uuid, tenant_id: Uuid) -> Result<GqlNode> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<rustok_outbox::TransactionalEventBus>()?;
        let auth = require_content_permission(
            ctx,
            &[Permission::NODES_UPDATE],
            "Permission denied: nodes:update required",
        )?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let node = service
            .restore_node(tenant_id, id, auth.security_context())
            .await?;

        Ok(node.into())
    }
}
