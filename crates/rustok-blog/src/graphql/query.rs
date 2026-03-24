use async_graphql::{Context, Object, Result};
use rustok_api::{
    graphql::{require_module_enabled, resolve_graphql_locale},
    AuthContext, TenantContext,
};
use rustok_content::NodeService;
use rustok_core::SecurityContext;
use rustok_outbox::TransactionalEventBus;
use rustok_telemetry::metrics;
use sea_orm::DatabaseConnection;
use std::time::Instant;
use uuid::Uuid;

use crate::{BlogError, PostService};

use super::types::*;

const MODULE_SLUG: &str = "blog";

#[derive(Default)]
pub struct BlogQuery;

#[Object]
impl BlogQuery {
    async fn post(
        &self,
        ctx: &Context<'_>,
        id: Uuid,
        locale: Option<String>,
        tenant_id: Option<Uuid>,
    ) -> Result<Option<GqlPost>> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let tenant_id = tenant_id.unwrap_or(tenant.id);
        let locale = resolve_graphql_locale(ctx, locale.as_deref());

        let service = PostService::new(db.clone(), event_bus.clone());
        let post = match service
            .get_post_with_locale_fallback(
                tenant_id,
                id,
                &locale,
                Some(tenant.default_locale.as_str()),
            )
            .await
        {
            Ok(post) => post,
            Err(BlogError::PostNotFound(_))
            | Err(BlogError::Content(rustok_content::ContentError::NodeNotFound(_))) => {
                return Ok(None);
            }
            Err(err) => return Err(async_graphql::Error::new(err.to_string())),
        };

        Ok(Some(post.into()))
    }

    async fn post_by_slug(
        &self,
        ctx: &Context<'_>,
        slug: String,
        locale: Option<String>,
        tenant_id: Option<Uuid>,
    ) -> Result<Option<GqlPost>> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let tenant_id = tenant_id.unwrap_or(tenant.id);
        let locale = resolve_graphql_locale(ctx, locale.as_deref());

        let service = PostService::new(db.clone(), event_bus.clone());
        let post = service
            .get_post_by_slug_with_locale_fallback(
                tenant_id,
                &locale,
                &slug,
                Some(tenant.default_locale.as_str()),
            )
            .await
            .map_err(|err| async_graphql::Error::new(err.to_string()))?;

        Ok(post.map(Into::into))
    }

    async fn posts(
        &self,
        ctx: &Context<'_>,
        filter: Option<PostsFilter>,
        tenant_id: Option<Uuid>,
    ) -> Result<GqlPostList> {
        require_module_enabled(ctx, MODULE_SLUG).await?;
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let tenant = ctx.data::<TenantContext>()?;
        let tenant_id = tenant_id.unwrap_or(tenant.id);

        let filter = filter.unwrap_or(PostsFilter {
            status: None,
            author_id: None,
            locale: None,
            page: Some(1),
            per_page: Some(20),
        });
        let requested_limit = filter.per_page;
        let locale = resolve_graphql_locale(ctx, filter.locale.as_deref());
        let effective_limit = filter.per_page.unwrap_or(20).clamp(1, 100);

        let domain_filter = rustok_content::dto::ListNodesFilter {
            kind: Some("post".to_string()),
            status: filter.status.map(Into::into),
            parent_id: None,
            author_id: filter.author_id,
            locale: Some(locale),
            category_id: None,
            page: filter.page.unwrap_or(1),
            per_page: filter.per_page.unwrap_or(20),
            include_deleted: false,
        };

        let service = NodeService::new(db.clone(), event_bus.clone());
        let list_started_at = Instant::now();
        let (items, total): (Vec<rustok_content::dto::NodeListItem>, u64) = service
            .list_nodes_with_locale_fallback(
                tenant_id,
                auth_context_to_security(ctx),
                domain_filter,
                Some(tenant.default_locale.as_str()),
            )
            .await?;
        metrics::record_read_path_query(
            "graphql",
            "blog.posts",
            "service_list",
            list_started_at.elapsed().as_secs_f64(),
            total,
        );

        let items = items.into_iter().map(Into::into).collect::<Vec<_>>();

        metrics::record_read_path_budget(
            "graphql",
            "blog.posts",
            requested_limit,
            effective_limit,
            items.len(),
        );

        Ok(GqlPostList { items, total })
    }
}

fn auth_context_to_security(ctx: &Context<'_>) -> SecurityContext {
    ctx.data::<AuthContext>()
        .map(|auth| auth.security_context())
        .unwrap_or_else(|_| SecurityContext::system())
}
