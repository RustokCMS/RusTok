use async_graphql::{Context, FieldError, Object, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::context::AuthContext;
use crate::graphql::errors::GraphQLError;
use crate::services::auth::AuthService;
use rustok_core::Permission;
use rustok_forum::{CategoryService, ReplyService, TopicService};
use rustok_outbox::TransactionalEventBus;

use super::types::*;

#[derive(Default)]
pub struct ForumQuery;

#[Object]
impl ForumQuery {
    async fn forum_categories(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        locale: Option<String>,
    ) -> Result<Vec<GqlForumCategory>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

        let has_perm = AuthService::has_any_permission(
            db,
            &tenant_id,
            &auth.user_id,
            &[
                Permission::FORUM_CATEGORIES_LIST,
                Permission::FORUM_CATEGORIES_MANAGE,
            ],
        )
        .await
        .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        if !has_perm {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: forum_categories:list required",
            ));
        }

        let security = auth.security_context();
        let service = CategoryService::new(db.clone(), event_bus.clone());
        let locale = locale.as_deref().unwrap_or("en");

        let categories = service.list(tenant_id, security, locale).await?;

        Ok(categories
            .into_iter()
            .map(|c| GqlForumCategory {
                id: c.id,
                locale: c.locale,
                effective_locale: c.effective_locale,
                name: c.name,
                slug: c.slug,
                description: c.description,
                icon: c.icon,
                color: c.color,
                topic_count: c.topic_count,
                reply_count: c.reply_count,
            })
            .collect())
    }

    async fn forum_topics(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        category_id: Option<Uuid>,
        locale: Option<String>,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<GqlForumTopic>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

        let has_perm = AuthService::has_any_permission(
            db,
            &tenant_id,
            &auth.user_id,
            &[
                Permission::FORUM_TOPICS_LIST,
                Permission::FORUM_TOPICS_MANAGE,
            ],
        )
        .await
        .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        if !has_perm {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: forum_topics:list required",
            ));
        }

        let security = auth.security_context();
        let service = TopicService::new(db.clone(), event_bus.clone());
        let filter = rustok_forum::ListTopicsFilter {
            category_id,
            status: None,
            locale,
            page: page.unwrap_or(1),
            per_page: per_page.unwrap_or(20),
        };

        let (topics, _total) = service.list(tenant_id, security, filter).await?;

        Ok(topics
            .into_iter()
            .map(|t| GqlForumTopic {
                id: t.id,
                locale: t.locale,
                effective_locale: t.effective_locale,
                category_id: t.category_id,
                author_id: t.author_id,
                title: t.title,
                slug: t.slug,
                body: String::new(),
                status: t.status,
                tags: Vec::new(),
                is_pinned: t.is_pinned,
                is_locked: t.is_locked,
                reply_count: t.reply_count,
                created_at: t.created_at,
                updated_at: String::new(),
            })
            .collect())
    }

    async fn forum_replies(
        &self,
        ctx: &Context<'_>,
        tenant_id: Uuid,
        topic_id: Uuid,
        locale: Option<String>,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> Result<Vec<GqlForumReply>> {
        let db = ctx.data::<DatabaseConnection>()?;
        let event_bus = ctx.data::<TransactionalEventBus>()?;
        let auth = ctx
            .data::<AuthContext>()
            .map_err(|_| <FieldError as GraphQLError>::unauthenticated())?;

        let has_perm = AuthService::has_any_permission(
            db,
            &tenant_id,
            &auth.user_id,
            &[
                Permission::FORUM_REPLIES_READ,
                Permission::FORUM_REPLIES_MANAGE,
            ],
        )
        .await
        .map_err(|e| <FieldError as GraphQLError>::internal_error(&e.to_string()))?;

        if !has_perm {
            return Err(<FieldError as GraphQLError>::permission_denied(
                "Permission denied: forum_replies:read required",
            ));
        }

        let security = auth.security_context();
        let service = ReplyService::new(db.clone(), event_bus.clone());
        let filter = rustok_forum::ListRepliesFilter {
            locale,
            page: page.unwrap_or(1),
            per_page: per_page.unwrap_or(20),
        };

        let (replies, _total) = service.list_for_topic(tenant_id, security, topic_id, filter).await?;

        Ok(replies
            .into_iter()
            .map(|r| GqlForumReply {
                id: r.id,
                locale: r.locale,
                effective_locale: r.effective_locale,
                topic_id: r.topic_id,
                author_id: r.author_id,
                content: r.content_preview,
                status: r.status,
                parent_reply_id: r.parent_reply_id,
                created_at: r.created_at,
                updated_at: String::new(),
            })
            .collect())
    }
}
