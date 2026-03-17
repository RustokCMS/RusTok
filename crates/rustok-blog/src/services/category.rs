use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{
    BodyInput, CreateNodeInput, ListNodesFilter, NodeService, NodeTranslationInput, UpdateNodeInput,
    PLATFORM_FALLBACK_LOCALE,
};
use rustok_core::SecurityContext;
use rustok_outbox::TransactionalEventBus;

use crate::dto::category::{
    CategoryListItem, CategoryResponse, CreateCategoryInput, ListCategoriesFilter,
    UpdateCategoryInput,
};
use crate::error::{BlogError, BlogResult};

const KIND_CATEGORY: &str = "category";

pub struct CategoryService {
    nodes: NodeService,
}

impl CategoryService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    #[instrument(skip(self, security, input))]
    pub async fn create_category(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateCategoryInput,
    ) -> BlogResult<Uuid> {
        if input.name.trim().is_empty() {
            return Err(BlogError::validation("Category name cannot be empty"));
        }
        if input.name.len() > 255 {
            return Err(BlogError::validation(
                "Category name cannot exceed 255 characters",
            ));
        }

        let slug = input
            .slug
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| slugify(&input.name));

        let locale = input.locale.clone();
        let bodies = if let Some(desc) = &input.description {
            vec![BodyInput {
                locale: locale.clone(),
                body: Some(desc.clone()),
                format: Some("markdown".to_string()),
            }]
        } else {
            vec![]
        };

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
                    position: None,
                    depth: None,
                    reply_count: None,
                    metadata: serde_json::json!({}),
                    translations: vec![NodeTranslationInput {
                        locale: locale.clone(),
                        title: Some(input.name),
                        slug: Some(slug),
                        excerpt: None,
                    }],
                    bodies,
                },
            )
            .await
            .map_err(BlogError::from)?;

        Ok(node.id)
    }

    #[instrument(skip(self))]
    pub async fn get_category(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        locale: &str,
    ) -> BlogResult<CategoryResponse> {
        let node = self
            .nodes
            .get_node(tenant_id, category_id)
            .await
            .map_err(BlogError::from)?;

        if node.kind != KIND_CATEGORY {
            return Err(BlogError::category_not_found(category_id));
        }

        Ok(node_to_category(node, locale))
    }

    #[instrument(skip(self, security, input))]
    pub async fn update_category(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        security: SecurityContext,
        input: UpdateCategoryInput,
    ) -> BlogResult<CategoryResponse> {
        let existing = self.nodes.get_node(tenant_id, category_id).await.map_err(BlogError::from)?;
        if existing.kind != KIND_CATEGORY {
            return Err(BlogError::category_not_found(category_id));
        }

        let locale = &input.locale;
        let mut update = UpdateNodeInput::default();

        if input.name.is_some() || input.slug.is_some() {
            let slug = input.slug.clone().or_else(|| {
                input.name.as_deref().map(slugify)
            });
            update.translations = Some(vec![NodeTranslationInput {
                locale: locale.clone(),
                title: input.name.clone(),
                slug,
                excerpt: None,
            }]);
        }

        if let Some(desc) = &input.description {
            update.bodies = Some(vec![BodyInput {
                locale: locale.clone(),
                body: Some(desc.clone()),
                format: Some("markdown".to_string()),
            }]);
        }

        let node = self
            .nodes
            .update_node(tenant_id, category_id, security, update)
            .await
            .map_err(BlogError::from)?;

        Ok(node_to_category(node, locale))
    }

    #[instrument(skip(self, security))]
    pub async fn delete_category(
        &self,
        tenant_id: Uuid,
        category_id: Uuid,
        security: SecurityContext,
    ) -> BlogResult<()> {
        let node = self
            .nodes
            .get_node(tenant_id, category_id)
            .await
            .map_err(BlogError::from)?;
        if node.kind != KIND_CATEGORY {
            return Err(BlogError::category_not_found(category_id));
        }

        self.nodes
            .delete_node(tenant_id, category_id, security)
            .await
            .map_err(BlogError::from)?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn list_categories(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        filter: ListCategoriesFilter,
    ) -> BlogResult<(Vec<CategoryListItem>, u64)> {
        let locale = filter
            .locale
            .unwrap_or_else(|| PLATFORM_FALLBACK_LOCALE.to_string());

        let (items, total) = self
            .nodes
            .list_nodes_with_locale_fallback(
                tenant_id,
                security,
                ListNodesFilter {
                    kind: Some(KIND_CATEGORY.to_string()),
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
            .map(|item| CategoryListItem {
                id: item.id,
                locale: locale.clone(),
                name: item.title.unwrap_or_default(),
                slug: item.slug.unwrap_or_default(),
                parent_id: item.category_id,
                created_at: item
                    .created_at
                    .parse()
                    .unwrap_or_else(|_| chrono::Utc::now()),
            })
            .collect();

        Ok((list, total))
    }
}

fn node_to_category(node: rustok_content::NodeResponse, locale: &str) -> CategoryResponse {
    let tr = node
        .translations
        .iter()
        .find(|t| t.locale == locale)
        .or_else(|| node.translations.iter().find(|t| t.locale == "en"))
        .or_else(|| node.translations.first());

    let description = node
        .bodies
        .iter()
        .find(|b| b.locale == locale)
        .or_else(|| node.bodies.iter().find(|b| b.locale == "en"))
        .or_else(|| node.bodies.first())
        .and_then(|b| b.body.clone())
        .filter(|s| !s.is_empty());

    CategoryResponse {
        id: node.id,
        tenant_id: node.tenant_id,
        locale: locale.to_string(),
        name: tr.and_then(|t| t.title.clone()).unwrap_or_default(),
        slug: tr.and_then(|t| t.slug.clone()).unwrap_or_default(),
        description,
        parent_id: node.parent_id,
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

/// Normalize a string into a URL-friendly slug.
/// Lowercases, replaces spaces/underscores with hyphens, strips non-alphanumeric chars.
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn slugify_special_chars() {
        assert_eq!(slugify("Rust & CMS!"), "rust-cms");
    }

    #[test]
    fn slugify_cyrillic() {
        assert_eq!(slugify("привет мир"), "привет-мир");
    }

    #[test]
    fn slugify_multiple_spaces() {
        assert_eq!(slugify("foo   bar"), "foo-bar");
    }

    #[test]
    fn slugify_already_slugified() {
        assert_eq!(slugify("my-post"), "my-post");
    }
}
