use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    CreateNodeInput, ListNodesFilter, NodeService, NodeTranslationInput, UpdateNodeInput,
};
use rustok_outbox::TransactionalEventBus;
use rustok_core::SecurityContext;

use crate::constants::KIND_CATEGORY;
use crate::dto::{CategoryListItem, CategoryResponse, CreateCategoryInput, UpdateCategoryInput};
use crate::error::{ForumError, ForumResult};

pub struct CategoryService {
    nodes: NodeService,
}

impl CategoryService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    #[instrument(skip(self, input, security))]
    pub async fn create(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateCategoryInput,
    ) -> ForumResult<CategoryResponse> {
        let metadata = serde_json::json!({
            "icon": input.icon,
            "color": input.color,
            "moderated": input.moderated,
            "topic_count": 0,
            "reply_count": 0
        });

        let node = self
            .nodes
            .create_node(
                tenant_id,
                security,
                CreateNodeInput {
                    kind: KIND_CATEGORY.to_string(),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    parent_id: input.parent_id,
                    author_id: None,
                    category_id: None,
                    position: input.position,
                    depth: None,
                    reply_count: None,
                    metadata,
                    translations: vec![NodeTranslationInput {
                        locale: input.locale.clone(),
                        title: Some(input.name),
                        slug: Some(input.slug),
                        excerpt: input.description,
                    }],
                    bodies: Vec::new(),
                },
            )
            .await?;

        Ok(Self::node_to_category(node, &input.locale))
    }

    #[instrument(skip(self))]
    pub async fn get(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        locale: &str,
    ) -> ForumResult<CategoryResponse> {
        let node = self.nodes.get_node(category_id).await?;

        if node.kind != KIND_CATEGORY {
            return Err(ForumError::CategoryNotFound(category_id));
        }

        Ok(Self::node_to_category(node, locale))
    }

    #[instrument(skip(self, security, input))]
    pub async fn update(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        security: SecurityContext,
        input: UpdateCategoryInput,
    ) -> ForumResult<CategoryResponse> {
        let existing = self.get(tenant_id, category_id, &input.locale).await?;
        let metadata = serde_json::json!({
            "icon": input.icon.or(existing.icon),
            "color": input.color.or(existing.color),
            "moderated": input.moderated.unwrap_or(existing.moderated),
            "topic_count": existing.topic_count,
            "reply_count": existing.reply_count
        });

        let translations =
            if input.name.is_some() || input.slug.is_some() || input.description.is_some() {
                Some(vec![NodeTranslationInput {
                    locale: input.locale.clone(),
                    title: Some(input.name.unwrap_or(existing.name)),
                    slug: Some(input.slug.unwrap_or(existing.slug)),
                    excerpt: input.description.or(existing.description),
                }])
            } else {
                None
            };

        let node = self
            .nodes
            .update_node(
                category_id,
                security,
                UpdateNodeInput {
                    position: input.position,
                    metadata: Some(metadata),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    translations,
                    ..UpdateNodeInput::default()
                },
            )
            .await?;

        Ok(Self::node_to_category(node, &input.locale))
    }

    #[instrument(skip(self, security))]
    pub async fn delete(&self, category_id: Uuid, security: SecurityContext) -> ForumResult<()> {
        self.nodes.delete_node(category_id, security).await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn list(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        locale: &str,
    ) -> ForumResult<Vec<CategoryListItem>> {
        let (items, _) = self
            .nodes
            .list_nodes(
                tenant_id,
                security,
                ListNodesFilter {
                    kind: Some(KIND_CATEGORY.to_string()),
                    status: None,
                    parent_id: None,
                    author_id: None,
                    locale: Some(locale.to_string()),
                    page: 1,
                    per_page: 1000,
                },
            )
            .await?;

        let list = items
            .into_iter()
            .map(|item| CategoryListItem {
                id: item.id,
                locale: locale.to_string(),
                name: item.title.unwrap_or_default(),
                slug: item.slug.unwrap_or_default(),
                description: item.excerpt,
                icon: None,
                color: None,
                topic_count: 0,
                reply_count: 0,
            })
            .collect();

        Ok(list)
    }

    fn node_to_category(node: rustok_content::NodeResponse, locale: &str) -> CategoryResponse {
        let translation = node
            .translations
            .iter()
            .find(|t| t.locale == locale)
            .or_else(|| node.translations.first());
        let metadata = node.metadata;

        CategoryResponse {
            id: node.id,
            locale: locale.to_string(),
            name: translation
                .and_then(|t| t.title.clone())
                .unwrap_or_default(),
            slug: translation.and_then(|t| t.slug.clone()).unwrap_or_default(),
            description: translation.and_then(|t| t.excerpt.clone()),
            icon: metadata
                .get("icon")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            color: metadata
                .get("color")
                .and_then(|v| v.as_str())
                .map(|v| v.to_string()),
            parent_id: node.parent_id,
            position: node.position,
            topic_count: metadata
                .get("topic_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            reply_count: metadata
                .get("reply_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            moderated: metadata
                .get("moderated")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        }
    }
}
