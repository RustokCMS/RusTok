use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    BodyInput, CreateNodeInput, ListNodesFilter, NodeService, NodeTranslationInput, UpdateNodeInput,
};
use rustok_outbox::TransactionalEventBus;
use rustok_core::SecurityContext;

use crate::constants::{topic_status, KIND_TOPIC};
use crate::dto::{
    CreateTopicInput, ListTopicsFilter, TopicListItem, TopicResponse, UpdateTopicInput,
};
use crate::error::{ForumError, ForumResult};

pub struct TopicService {
    nodes: NodeService,
}

impl TopicService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    #[instrument(skip(self, security, input))]
    pub async fn create(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateTopicInput,
    ) -> ForumResult<TopicResponse> {
        let metadata = serde_json::json!({
            "tags": input.tags,
            "is_pinned": false,
            "is_locked": false,
            "reply_count": 0,
            "forum_status": topic_status::OPEN
        });

        let node = self
            .nodes
            .create_node(
                tenant_id,
                security,
                CreateNodeInput {
                    kind: KIND_TOPIC.to_string(),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    parent_id: Some(input.category_id),
                    author_id: None,
                    category_id: Some(input.category_id),
                    position: None,
                    depth: None,
                    reply_count: Some(0),
                    metadata,
                    translations: vec![NodeTranslationInput {
                        locale: input.locale.clone(),
                        title: Some(input.title.clone()),
                        slug: None,
                        excerpt: None,
                    }],
                    bodies: vec![BodyInput {
                        locale: input.locale.clone(),
                        body: Some(input.body),
                        format: Some("markdown".to_string()),
                    }],
                },
            )
            .await?;

        Ok(Self::node_to_topic(node, &input.locale))
    }

    #[instrument(skip(self))]
    pub async fn get(&self, topic_id: Uuid, locale: &str) -> ForumResult<TopicResponse> {
        let node = self.nodes.get_node(topic_id).await?;

        if node.kind != KIND_TOPIC {
            return Err(ForumError::TopicNotFound(topic_id));
        }

        Ok(Self::node_to_topic(node, locale))
    }

    #[instrument(skip(self, security, input))]
    pub async fn update(
        &self,
        topic_id: Uuid,
        security: SecurityContext,
        input: UpdateTopicInput,
    ) -> ForumResult<TopicResponse> {
        let existing = self.get(topic_id, &input.locale).await?;
        let metadata = serde_json::json!({
            "tags": input.tags.unwrap_or(existing.tags.clone()),
            "is_pinned": existing.is_pinned,
            "is_locked": existing.is_locked,
            "reply_count": existing.reply_count,
            "forum_status": existing.status
        });

        let translations = if input.title.is_some() {
            Some(vec![NodeTranslationInput {
                locale: input.locale.clone(),
                title: Some(input.title.unwrap_or(existing.title.clone())),
                slug: None,
                excerpt: None,
            }])
        } else {
            None
        };

        let bodies = input.body.map(|body| {
            vec![BodyInput {
                locale: input.locale.clone(),
                body: Some(body),
                format: Some("markdown".to_string()),
            }]
        });

        let node = self
            .nodes
            .update_node(
                topic_id,
                security,
                UpdateNodeInput {
                    metadata: Some(metadata),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    translations,
                    bodies,
                    ..UpdateNodeInput::default()
                },
            )
            .await?;

        Ok(Self::node_to_topic(node, &input.locale))
    }

    #[instrument(skip(self, security))]
    pub async fn delete(&self, topic_id: Uuid, security: SecurityContext) -> ForumResult<()> {
        self.nodes.delete_node(topic_id, security).await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn list(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        filter: ListTopicsFilter,
    ) -> ForumResult<(Vec<TopicListItem>, u64)> {
        let locale = filter.locale.clone().unwrap_or_else(|| "en".to_string());
        let (items, total) = self
            .nodes
            .list_nodes(
                tenant_id,
                security,
                ListNodesFilter {
                    kind: Some(KIND_TOPIC.to_string()),
                    status: None,
                    parent_id: filter.category_id,
                    author_id: None,
                    locale: Some(locale.clone()),
                    page: filter.page,
                    per_page: filter.per_page,
                },
            )
            .await?;

        let list = items
            .into_iter()
            .map(|item| TopicListItem {
                id: item.id,
                locale: locale.clone(),
                category_id: Uuid::nil(),
                title: item.title.unwrap_or_default(),
                status: topic_status::OPEN.to_string(),
                is_pinned: false,
                is_locked: false,
                reply_count: 0,
                created_at: item.created_at,
            })
            .collect();

        Ok((list, total))
    }

    fn node_to_topic(node: rustok_content::NodeResponse, locale: &str) -> TopicResponse {
        let translation = node
            .translations
            .iter()
            .find(|t| t.locale == locale)
            .or_else(|| node.translations.first());
        let body = node
            .bodies
            .iter()
            .find(|b| b.locale == locale)
            .or_else(|| node.bodies.first());
        let metadata = node.metadata;

        TopicResponse {
            id: node.id,
            locale: locale.to_string(),
            category_id: node.category_id.unwrap_or(Uuid::nil()),
            title: translation
                .and_then(|t| t.title.clone())
                .unwrap_or_default(),
            body: body.and_then(|b| b.body.clone()).unwrap_or_default(),
            status: metadata
                .get("forum_status")
                .and_then(|v| v.as_str())
                .unwrap_or(topic_status::OPEN)
                .to_string(),
            tags: metadata
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            is_pinned: metadata
                .get("is_pinned")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            is_locked: metadata
                .get("is_locked")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            reply_count: metadata
                .get("reply_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            created_at: node.created_at,
            updated_at: node.updated_at,
        }
    }
}
