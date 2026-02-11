use async_graphql::{Context, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::context::AuthContext;
use rustok_content::NodeService;
use rustok_core::{EventBus, SecurityContext, UserRole};

use super::types::*;

#[derive(Default)]
pub struct ContentQuery;

#[Object]
impl ContentQuery {
    async fn node(&self, ctx: &Context<'_>, _tenant_id: Uuid, id: Uuid) -> Result<Option<GqlNode>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        match service.get_node(id).await {
            Ok(node) => Ok(Some(node.into())), // GqlNode implements From<NodeResponse>
            Err(rustok_content::ContentError::NodeNotFound(_)) => Ok(None),
            Err(err) => Err(async_graphql::Error::new(err.to_string())),
        }
    }

    async fn nodes(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        filter: Option<NodesFilter>,
    ) -> Result<GqlNodeList> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;

        let service = NodeService::new(db.clone(), event_bus.clone());
        let filter = filter.unwrap_or(NodesFilter {
            kind: None,
            status: None,
            parent_id: None,
            author_id: None,
            locale: None,
            page: Some(1),
            per_page: Some(20),
        });

        let domain_filter = rustok_content::dto::ListNodesFilter {
            kind: filter.kind,
            status: filter.status.map(Into::into),
            parent_id: filter.parent_id,
            author_id: filter.author_id,
            locale: filter.locale,
            page: filter.page.unwrap_or(1),
            per_page: filter.per_page.unwrap_or(20),
        };

        let security = ctx
            .data::<AuthContext>()
            .map(|auth| SecurityContext::new(auth.role.clone(), Some(auth.user_id)))
            .unwrap_or_else(|_| SecurityContext::new(UserRole::Customer, None));

        let (items, total): (Vec<rustok_content::dto::NodeListItem>, u64) = service
            .list_nodes(tenant_id, security, domain_filter)
            .await?;

        Ok(GqlNodeList {
            items: items.into_iter().map(Into::into).collect(),
            total,
        })
    }
}
