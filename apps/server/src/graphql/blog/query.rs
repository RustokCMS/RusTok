use async_graphql::{Context, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use rustok_content::NodeService;
use rustok_core::{EventBus, TransactionalEventBus};

use super::types::*;

#[derive(Default)]
pub struct BlogQuery;

#[Object]
impl BlogQuery {
    async fn post(&self, ctx: &Context<'_>, _tenant_id: Uuid, id: Uuid) -> Result<Option<GqlPost>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let node = match service.get_node(id).await {
            Ok(node) => node,
            Err(rustok_content::ContentError::NodeNotFound(_)) => return Ok(None),
            Err(err) => return Err(async_graphql::Error::new(err.to_string())),
        };

        if node.kind != "post" {
            return Ok(None);
        }

        Ok(Some(node.into()))
    }

    async fn posts(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        filter: Option<PostsFilter>,
    ) -> Result<GqlPostList> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let filter = filter.unwrap_or(PostsFilter {
            status: None,
            author_id: None,
            locale: None,
            page: Some(1),
            per_page: Some(20),
        });

        let domain_filter = rustok_content::dto::ListNodesFilter {
            kind: Some("post".to_string()),
            status: filter.status.map(Into::into),
            parent_id: None,
            author_id: filter.author_id,
            locale: filter.locale,
            page: filter.page.unwrap_or(1),
            per_page: filter.per_page.unwrap_or(20),
        };

        let security = ctx
            .data::<crate::context::AuthContext>()
            .map(|a| a.security_context())
            .unwrap_or_else(|_| rustok_core::SecurityContext::system());

        let (items, total): (Vec<rustok_content::dto::NodeListItem>, u64) = service
            .list_nodes(tenant_id, security, domain_filter)
            .await?;

        Ok(GqlPostList {
            items: items.into_iter().map(Into::into).collect(),
            total,
        })
    }
}
