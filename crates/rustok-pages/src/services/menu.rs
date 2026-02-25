use sea_orm::DatabaseConnection;
use std::future::Future;
use std::pin::Pin;
use uuid::Uuid;

use rustok_content::{CreateNodeInput, ListNodesFilter, NodeService};
use rustok_core::SecurityContext;
use rustok_outbox::TransactionalEventBus;

use crate::dto::*;
use crate::error::{PagesError, PagesResult};

const MENU_KIND: &str = "menu";
const MENU_ITEM_KIND: &str = "menu_item";

pub struct MenuService {
    nodes: NodeService,
}

impl MenuService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    pub async fn create(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateMenuInput,
    ) -> PagesResult<MenuResponse> {
        let metadata = serde_json::json!({
            "location": input.location,
        });

        let menu_node = self
            .nodes
            .create_node(
                tenant_id,
                security.clone(),
                CreateNodeInput {
                    kind: MENU_KIND.to_string(),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    parent_id: None,
                    author_id: security.user_id,
                    category_id: None,
                    position: None,
                    depth: None,
                    reply_count: None,
                    metadata,
                    translations: vec![rustok_content::NodeTranslationInput {
                        locale: "default".to_string(),
                        title: Some(input.name.clone()),
                        slug: None,
                        excerpt: None,
                    }],
                    bodies: vec![],
                },
            )
            .await?;

        let menu_id = menu_node.id;

        for item in input.items {
            self.create_menu_item(tenant_id, security.clone(), menu_id, None, item)
                .await?;
        }

        self.get(tenant_id, security, menu_id).await
    }

    fn create_menu_item(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        menu_id: Uuid,
        parent_item_id: Option<Uuid>,
        input: MenuItemInput,
    ) -> Pin<Box<dyn Future<Output = PagesResult<Uuid>> + Send + '_>> {
        Box::pin(async move {
            let url = input.url.clone().unwrap_or_else(|| "/".to_string());

            let metadata = serde_json::json!({
                "menu_id": menu_id,
                "url": url,
                "icon": input.icon,
                "page_id": input.page_id,
            });

            let parent = parent_item_id.or(Some(menu_id));

            let node = self
                .nodes
                .create_node(
                    tenant_id,
                    security.clone(),
                    CreateNodeInput {
                        kind: MENU_ITEM_KIND.to_string(),
                        status: Some(rustok_content::entities::node::ContentStatus::Published),
                        parent_id: parent,
                        author_id: security.user_id,
                        category_id: None,
                        position: Some(input.position),
                        depth: None,
                        reply_count: None,
                        metadata,
                        translations: vec![rustok_content::NodeTranslationInput {
                            locale: "default".to_string(),
                            title: Some(input.title),
                            slug: None,
                            excerpt: None,
                        }],
                        bodies: vec![],
                    },
                )
                .await?;

            if let Some(children) = input.children {
                for child in children {
                    self.create_menu_item(
                        tenant_id,
                        security.clone(),
                        menu_id,
                        Some(node.id),
                        child,
                    )
                    .await?;
                }
            }

            Ok(node.id)
        })
    }

    pub async fn get(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        menu_id: Uuid,
    ) -> PagesResult<MenuResponse> {
        let menu = self.nodes.get_node(menu_id).await?;
        if menu.kind != MENU_KIND {
            return Err(PagesError::MenuNotFound(menu_id));
        }

        let location: MenuLocation = menu
            .metadata
            .get("location")
            .and_then(|value| serde_json::from_value(value.clone()).ok())
            .unwrap_or(MenuLocation::Header);

        let name = menu
            .translations
            .first()
            .and_then(|t| t.title.clone())
            .unwrap_or_default();

        let items = self.get_menu_items(tenant_id, security, menu_id).await?;

        Ok(MenuResponse {
            id: menu_id,
            name,
            location,
            items,
        })
    }

    fn get_menu_items(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        parent_id: Uuid,
    ) -> Pin<Box<dyn Future<Output = PagesResult<Vec<MenuItemResponse>>> + Send + '_>> {
        Box::pin(async move {
            let (items, _) = self
                .nodes
                .list_nodes(
                    tenant_id,
                    security.clone(),
                    ListNodesFilter {
                        kind: Some(MENU_ITEM_KIND.to_string()),
                        status: None,
                        parent_id: Some(parent_id),
                        author_id: None,
                        category_id: None,
                        locale: None,
                        page: 1,
                        per_page: 100,
                        include_deleted: false,
                    },
                )
                .await?;

            let mut responses = Vec::with_capacity(items.len());
            for item in items {
                let node = self.nodes.get_node(item.id).await?;
                let title = node.translations.first().and_then(|t| t.title.clone());

                let url = node
                    .metadata
                    .get("url")
                    .and_then(|value| value.as_str())
                    .unwrap_or("#")
                    .to_string();

                let icon = node
                    .metadata
                    .get("icon")
                    .and_then(|value| value.as_str())
                    .map(String::from);

                let children = self
                    .get_menu_items(tenant_id, security.clone(), node.id)
                    .await?;

                responses.push(MenuItemResponse {
                    id: node.id,
                    title,
                    url,
                    icon,
                    children,
                });
            }

            Ok(responses)
        })
    }
}
