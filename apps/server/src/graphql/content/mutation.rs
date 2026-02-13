use async_graphql::{Context, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::context::AuthContext;
use rustok_content::NodeService;
use rustok_core::{EventBus, SecurityContext, TransactionalEventBus};

use super::types::*;

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
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let auth = ctx.data::<AuthContext>().map_err(|_| "Unauthorized")?;
        let security = SecurityContext::new(auth.role.clone(), Some(auth.user_id));

        let service = NodeService::new(db.clone(), event_bus.clone());
        let domain_input = rustok_content::dto::CreateNodeInput {
            parent_id: input.parent_id,
            author_id: input.author_id.or(author_id),
            kind: input.kind,
            category_id: input.category_id,
            status: input.status.map(Into::into),
            position: input.position,
            depth: input.depth,
            reply_count: input.reply_count,
            metadata: serde_json::Value::Object(Default::default()),
            translations: input
                .translations
                .into_iter()
                .map(|t| rustok_content::dto::NodeTranslationInput {
                    locale: t.locale,
                    title: t.title,
                    slug: t.slug,
                    excerpt: t.excerpt,
                })
                .collect(),
            bodies: input
                .bodies
                .into_iter()
                .map(|b| rustok_content::dto::BodyInput {
                    locale: b.locale,
                    body: b.body,
                    format: b.format,
                })
                .collect(),
        };

        let node: rustok_content::dto::NodeResponse = service
            .create_node(tenant_id, security, domain_input)
            .await?;

        Ok(node.into())
    }

    async fn update_node(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        author_id: Option<Uuid>,
        input: UpdateNodeInput,
    ) -> Result<GqlNode> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let auth = ctx.data::<AuthContext>().map_err(|_| "Unauthorized")?;
        let security = SecurityContext::new(auth.role.clone(), Some(auth.user_id));

        let service = NodeService::new(db.clone(), event_bus.clone());
        let resolved_author_id = input.author_id.or(author_id);
        let domain_input = rustok_content::dto::UpdateNodeInput {
            parent_id: input.parent_id.map(Some),
            author_id: resolved_author_id.map(Some),
            category_id: input.category_id.map(Some),
            status: input.status.map(Into::into),
            position: input.position,
            depth: input.depth,
            reply_count: input.reply_count,
            metadata: None,
            published_at: None, // Need parsing logic if string provided, or fix DTO to accept string
            translations: input.translations.map(|ts| {
                ts.into_iter()
                    .map(|t| rustok_content::dto::NodeTranslationInput {
                        locale: t.locale,
                        title: t.title,
                        slug: t.slug,
                        excerpt: t.excerpt,
                    })
                    .collect()
            }),
            bodies: input.bodies.map(|bs| {
                bs.into_iter()
                    .map(|b| rustok_content::dto::BodyInput {
                        locale: b.locale,
                        body: b.body,
                        format: b.format,
                    })
                    .collect()
            }),
        };

        let node: rustok_content::dto::NodeResponse =
            service.update_node(id, security, domain_input).await?;

        Ok(node.into())
    }

    async fn delete_node(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        _author_id: Option<Uuid>,
    ) -> Result<bool> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let auth = ctx.data::<AuthContext>().map_err(|_| "Unauthorized")?;
        let security = SecurityContext::new(auth.role.clone(), Some(auth.user_id));

        let service = NodeService::new(db.clone(), event_bus.clone());
        service.delete_node(id, security).await?;

        Ok(true)
    }
}
