use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    CreateNodeInput, ListNodesFilter, NodeService, NodeTranslationInput, UpdateNodeInput,
    PLATFORM_FALLBACK_LOCALE,
};
use rustok_core::SecurityContext;
use rustok_outbox::TransactionalEventBus;

use crate::dto::tag::{
    CreateTagInput, ListTagsFilter, TagListItem, TagResponse, UpdateTagInput,
};
use crate::error::{BlogError, BlogResult};
use crate::services::category::slugify;

const KIND_TAG: &str = "tag";

pub struct TagService {
    nodes: NodeService,
}

impl TagService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    #[instrument(skip(self, security, input))]
    pub async fn create_tag(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateTagInput,
    ) -> BlogResult<Uuid> {
        if input.name.trim().is_empty() {
            return Err(BlogError::validation("Tag name cannot be empty"));
        }
        if input.name.len() > 100 {
            return Err(BlogError::validation(
                "Tag name cannot exceed 100 characters",
            ));
        }

        let slug = input
            .slug
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| slugify(&input.name));

        let node = self
            .nodes
            .create_node(
                tenant_id,
                security,
                CreateNodeInput {
                    kind: KIND_TAG.to_string(),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    parent_id: None,
                    author_id: None,
                    category_id: None,
                    position: None,
                    depth: None,
                    reply_count: None,
                    metadata: serde_json::json!({}),
                    translations: vec![NodeTranslationInput {
                        locale: input.locale.clone(),
                        title: Some(input.name),
                        slug: Some(slug),
                        excerpt: None,
                    }],
                    bodies: vec![],
                },
            )
            .await
            .map_err(BlogError::from)?;

        Ok(node.id)
    }

    #[instrument(skip(self))]
    pub async fn get_tag(
        &self,
        tenant_id: Uuid,
        tag_id: Uuid,
        locale: &str,
    ) -> BlogResult<TagResponse> {
        let node = self
            .nodes
            .get_node(tenant_id, tag_id)
            .await
            .map_err(BlogError::from)?;

        if node.kind != KIND_TAG {
            return Err(BlogError::tag_not_found(tag_id));
        }

        Ok(node_to_tag(node, locale))
    }

    #[instrument(skip(self, security, input))]
    pub async fn update_tag(
        &self,
        tenant_id: Uuid,
        tag_id: Uuid,
        security: SecurityContext,
        input: UpdateTagInput,
    ) -> BlogResult<TagResponse> {
        let existing = self
            .nodes
            .get_node(tenant_id, tag_id)
            .await
            .map_err(BlogError::from)?;
        if existing.kind != KIND_TAG {
            return Err(BlogError::tag_not_found(tag_id));
        }

        let locale = &input.locale;
        let mut update = UpdateNodeInput::default();

        if input.name.is_some() || input.slug.is_some() {
            let slug = input.slug.clone().or_else(|| input.name.as_deref().map(slugify));
            update.translations = Some(vec![NodeTranslationInput {
                locale: locale.clone(),
                title: input.name.clone(),
                slug,
                excerpt: None,
            }]);
        }

        let node = self
            .nodes
            .update_node(tenant_id, tag_id, security, update)
            .await
            .map_err(BlogError::from)?;

        Ok(node_to_tag(node, locale))
    }

    #[instrument(skip(self, security))]
    pub async fn delete_tag(
        &self,
        tenant_id: Uuid,
        tag_id: Uuid,
        security: SecurityContext,
    ) -> BlogResult<()> {
        let node = self
            .nodes
            .get_node(tenant_id, tag_id)
            .await
            .map_err(BlogError::from)?;
        if node.kind != KIND_TAG {
            return Err(BlogError::tag_not_found(tag_id));
        }

        self.nodes
            .delete_node(tenant_id, tag_id, security)
            .await
            .map_err(BlogError::from)?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn list_tags(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        filter: ListTagsFilter,
    ) -> BlogResult<(Vec<TagListItem>, u64)> {
        let locale = filter
            .locale
            .unwrap_or_else(|| PLATFORM_FALLBACK_LOCALE.to_string());

        let (items, total) = self
            .nodes
            .list_nodes_with_locale_fallback(
                tenant_id,
                security,
                ListNodesFilter {
                    kind: Some(KIND_TAG.to_string()),
                    status: None,
                    locale: Some(locale.clone()),
                    page: filter.page,
                    per_page: filter.per_page,
                    include_deleted: false,
                    ..Default::default()
                },
                None,
            )
            .await
            .map_err(BlogError::from)?;

        let list = items
            .into_iter()
            .map(|item| TagListItem {
                id: item.id,
                locale: locale.clone(),
                name: item.title.unwrap_or_default(),
                slug: item.slug.unwrap_or_default(),
                created_at: item
                    .created_at
                    .parse()
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
            .collect();

        Ok((list, total))
    }
}

fn node_to_tag(node: rustok_content::NodeResponse, locale: &str) -> TagResponse {
    let tr = node
        .translations
        .iter()
        .find(|t| t.locale == locale)
        .or_else(|| node.translations.iter().find(|t| t.locale == "en"))
        .or_else(|| node.translations.first());

    TagResponse {
        id: node.id,
        tenant_id: node.tenant_id,
        locale: locale.to_string(),
        name: tr.and_then(|t| t.title.clone()).unwrap_or_default(),
        slug: tr.and_then(|t| t.slug.clone()).unwrap_or_default(),
        created_at: node
            .created_at
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now()),
        updated_at: node
            .updated_at
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now()),
    }
}
