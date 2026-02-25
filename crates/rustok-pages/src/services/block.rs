use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::{CreateNodeInput, ListNodesFilter, NodeService, UpdateNodeInput};
use rustok_core::SecurityContext;
use rustok_outbox::TransactionalEventBus;

use crate::dto::*;
use crate::error::{PagesError, PagesResult};

const BLOCK_KIND: &str = "block";

pub struct BlockService {
    nodes: NodeService,
}

impl BlockService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    #[instrument(skip(self, input))]
    pub async fn create(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        page_id: Uuid,
        input: CreateBlockInput,
    ) -> PagesResult<BlockResponse> {
        let metadata = serde_json::json!({
            "block_type": input.block_type,
            "data": input.data,
            "translations": input.translations,
        });

        let node = self
            .nodes
            .create_node(
                tenant_id,
                security,
                CreateNodeInput {
                    kind: BLOCK_KIND.to_string(),
                    status: Some(rustok_content::entities::node::ContentStatus::Published),
                    parent_id: Some(page_id),
                    author_id: None,
                    category_id: None,
                    position: Some(input.position),
                    depth: None,
                    reply_count: None,
                    metadata,
                    translations: vec![],
                    bodies: vec![],
                },
            )
            .await?;

        Ok(node_to_block(node))
    }

    #[instrument(skip(self))]
    pub async fn list_for_page(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        page_id: Uuid,
    ) -> PagesResult<Vec<BlockResponse>> {
        let (items, _) = self
            .nodes
            .list_nodes(
                tenant_id,
                security.clone(),
                ListNodesFilter {
                    kind: Some(BLOCK_KIND.to_string()),
                    status: None,
                    parent_id: Some(page_id),
                    author_id: None,
                    category_id: None,
                    locale: None,
                    page: 1,
                    per_page: 100,
                    include_deleted: false,
                },
            )
            .await?;

        let mut blocks = Vec::with_capacity(items.len());
        for item in items {
            let node = self.nodes.get_node(item.id).await?;
            blocks.push(node_to_block(node));
        }

        Ok(blocks)
    }

    #[instrument(skip(self, input))]
    pub async fn update(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        block_id: Uuid,
        input: UpdateBlockInput,
    ) -> PagesResult<BlockResponse> {
        let existing = self.nodes.get_node(block_id).await?;
        if existing.kind != BLOCK_KIND {
            return Err(PagesError::BlockNotFound(block_id));
        }

        let mut metadata = if existing.metadata.is_object() {
            existing.metadata
        } else {
            serde_json::json!({})
        };
        if let Some(data) = input.data {
            metadata["data"] = data;
        }
        if let Some(translations) = input.translations {
            metadata["translations"] = serde_json::json!(translations);
        }

        self.nodes
            .update_node(
                block_id,
                security.clone(),
                UpdateNodeInput {
                    status: None,
                    parent_id: None,
                    author_id: None,
                    category_id: None,
                    position: input.position,
                    depth: None,
                    reply_count: None,
                    metadata: Some(metadata),
                    translations: None,
                    bodies: None,
                    expected_version: None,
                },
            )
            .await?;

        let node = self.nodes.get_node(block_id).await?;
        Ok(node_to_block(node))
    }

    #[instrument(skip(self))]
    pub async fn reorder(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        page_id: Uuid,
        block_order: Vec<Uuid>,
    ) -> PagesResult<()> {
        let _ = page_id;
        for (position, block_id) in block_order.into_iter().enumerate() {
            self.nodes
                .update_node(
                    block_id,
                    security.clone(),
                    UpdateNodeInput {
                        status: None,
                        parent_id: None,
                        author_id: None,
                        category_id: None,
                        position: Some(position as i32),
                        depth: None,
                        reply_count: None,
                        metadata: None,
                        translations: None,
                        bodies: None,
                        expected_version: None,
                    },
                )
                .await?;
        }
        Ok(())
    }

    pub async fn delete(
        &self,
        _tenant_id: Uuid,
        security: SecurityContext,
        block_id: Uuid,
    ) -> PagesResult<()> {
        self.nodes.delete_node(block_id, security).await?;
        Ok(())
    }

    pub async fn delete_all_for_page(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        page_id: Uuid,
    ) -> PagesResult<()> {
        let blocks = self
            .list_for_page(tenant_id, security.clone(), page_id)
            .await?;
        for block in blocks {
            self.nodes.delete_node(block.id, security.clone()).await?;
        }
        Ok(())
    }
}

fn node_to_block(node: rustok_content::dto::NodeResponse) -> BlockResponse {
    let block_type: BlockType = node
        .metadata
        .get("block_type")
        .and_then(|value| serde_json::from_value(value.clone()).ok())
        .unwrap_or_default();

    let data = node
        .metadata
        .get("data")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    let translations = node
        .metadata
        .get("translations")
        .and_then(|value| serde_json::from_value(value.clone()).ok());

    BlockResponse {
        id: node.id,
        block_type,
        position: node.position,
        data,
        translations,
    }
}
