use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    CreateNodeInput, ListNodesFilter, NodeService, NodeTranslationInput, UpdateNodeInput,
};
use rustok_core::SecurityContext;
use rustok_outbox::TransactionalEventBus;

use crate::constants::KIND_CATEGORY;
use crate::dto::{CategoryListItem, CategoryResponse, CreateCategoryInput, UpdateCategoryInput};
use crate::error::{ForumError, ForumResult};
use crate::locale::{available_locales, resolve_translation};

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
        if input.name.trim().is_empty() {
            return Err(ForumError::Validation(
                "Category name cannot be empty".to_string(),
            ));
        }
        if input.slug.trim().is_empty() {
            return Err(ForumError::Validation(
                "Category slug cannot be empty".to_string(),
            ));
        }

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

        let _ = tenant_id;
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
                    include_deleted: false,
                    category_id: None,
                },
            )
            .await?;

        let list = items
            .into_iter()
            .map(|item| {
                let metadata = &item.metadata;
                CategoryListItem {
                    id: item.id,
                    locale: locale.to_string(),
                    effective_locale: locale.to_string(),
                    name: item.title.unwrap_or_default(),
                    slug: item.slug.unwrap_or_default(),
                    description: item.excerpt,
                    icon: metadata
                        .get("icon")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    color: metadata
                        .get("color")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string()),
                    topic_count: metadata
                        .get("topic_count")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                    reply_count: metadata
                        .get("reply_count")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0) as i32,
                }
            })
            .collect();

        Ok(list)
    }

    fn node_to_category(node: rustok_content::NodeResponse, locale: &str) -> CategoryResponse {
        let resolved = resolve_translation(&node.translations, locale);
        let locales = available_locales(&node.translations);
        let metadata = node.metadata;

        CategoryResponse {
            id: node.id,
            locale: locale.to_string(),
            effective_locale: resolved.effective_locale,
            available_locales: locales,
            name: resolved
                .translation
                .and_then(|t| t.title.clone())
                .unwrap_or_default(),
            slug: resolved
                .translation
                .and_then(|t| t.slug.clone())
                .unwrap_or_default(),
            description: resolved.translation.and_then(|t| t.excerpt.clone()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::KIND_CATEGORY;
    use rustok_content::dto::{NodeResponse, NodeTranslationResponse};
    use rustok_content::entities::node::ContentStatus;

    fn make_category_node(
        metadata: serde_json::Value,
        translations: Vec<NodeTranslationResponse>,
        parent_id: Option<Uuid>,
        position: i32,
    ) -> NodeResponse {
        NodeResponse {
            id: Uuid::nil(),
            tenant_id: Uuid::nil(),
            kind: KIND_CATEGORY.to_string(),
            status: ContentStatus::Published,
            parent_id,
            author_id: None,
            category_id: None,
            position,
            depth: 0,
            reply_count: 0,
            metadata,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
            published_at: None,
            deleted_at: None,
            version: 1,
            translations,
            bodies: vec![],
        }
    }

    fn tr(locale: &str, name: &str, slug: &str) -> NodeTranslationResponse {
        NodeTranslationResponse {
            locale: locale.to_string(),
            title: Some(name.to_string()),
            slug: Some(slug.to_string()),
            excerpt: None,
        }
    }

    #[test]
    fn node_to_category_maps_metadata_fields() {
        let metadata = serde_json::json!({
            "icon": "chat",
            "color": "#ff0000",
            "moderated": true,
            "topic_count": 10,
            "reply_count": 50
        });
        let node = make_category_node(metadata, vec![tr("en", "General", "general")], None, 3);

        let result = CategoryService::node_to_category(node, "en");

        assert_eq!(result.name, "General");
        assert_eq!(result.slug, "general");
        assert_eq!(result.icon, Some("chat".to_string()));
        assert_eq!(result.color, Some("#ff0000".to_string()));
        assert!(result.moderated);
        assert_eq!(result.topic_count, 10);
        assert_eq!(result.reply_count, 50);
        assert_eq!(result.position, 3);
        assert_eq!(result.effective_locale, "en");
        assert_eq!(result.available_locales, vec!["en"]);
    }

    #[test]
    fn node_to_category_defaults_on_empty_metadata() {
        let node = make_category_node(serde_json::json!({}), vec![tr("en", "", "")], None, 0);

        let result = CategoryService::node_to_category(node, "en");

        assert_eq!(result.icon, None);
        assert_eq!(result.color, None);
        assert!(!result.moderated);
        assert_eq!(result.topic_count, 0);
        assert_eq!(result.reply_count, 0);
    }

    #[test]
    fn node_to_category_propagates_parent_id() {
        let parent_id = Uuid::new_v4();
        let node = make_category_node(
            serde_json::json!({}),
            vec![tr("en", "Sub", "sub")],
            Some(parent_id),
            0,
        );

        let result = CategoryService::node_to_category(node, "en");
        assert_eq!(result.parent_id, Some(parent_id));
    }

    #[test]
    fn node_to_category_fallback_to_en() {
        let node = make_category_node(
            serde_json::json!({}),
            vec![
                tr("en", "General", "general"),
                tr("ru", "Общее", "obshchee"),
            ],
            None,
            0,
        );
        let result = CategoryService::node_to_category(node, "de");
        assert_eq!(result.effective_locale, "en");
        assert_eq!(result.name, "General");
    }

    #[test]
    fn node_to_category_fallback_to_first_when_no_en() {
        let node = make_category_node(
            serde_json::json!({}),
            vec![
                tr("de", "Allgemein", "allgemein"),
                tr("fr", "Général", "general"),
            ],
            None,
            0,
        );
        let result = CategoryService::node_to_category(node, "ru");
        assert_eq!(result.effective_locale, "de");
    }

    #[test]
    fn node_to_category_available_locales() {
        let node = make_category_node(
            serde_json::json!({}),
            vec![
                tr("en", "A", "a"),
                tr("ru", "А", "a-ru"),
                tr("de", "A", "a-de"),
            ],
            None,
            0,
        );
        let result = CategoryService::node_to_category(node, "en");
        assert_eq!(result.available_locales, vec!["en", "ru", "de"]);
    }
}
