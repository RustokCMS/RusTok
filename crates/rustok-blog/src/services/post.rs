use rustok_content::{
    BodyInput, ContentResult, CreateNodeInput, NodeService, NodeTranslationInput, UpdateNodeInput,
};
use rustok_core::EventBus;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePostInput {
    pub locale: String,
    pub title: String,
    pub body: String,
    pub excerpt: Option<String>,
    pub slug: Option<String>,
    pub publish: bool,
    pub tags: Vec<String>,
    pub metadata: Option<Value>,
}

pub struct PostService {
    node_service: NodeService,
}

impl PostService {
    pub fn new(db: DatabaseConnection, event_bus: EventBus) -> Self {
        Self {
            node_service: NodeService::new(db, event_bus),
        }
    }

    pub async fn create_post(
        &self,
        tenant_id: Uuid,
        author_id: Option<Uuid>,
        actor_id: Option<Uuid>,
        input: CreatePostInput,
    ) -> ContentResult<Uuid> {
        let mut metadata = input.metadata.unwrap_or_else(|| serde_json::json!({}));
        if let Value::Object(map) = &mut metadata {
            map.insert("tags".to_string(), serde_json::json!(input.tags));
        } else {
            metadata = serde_json::json!({
                "tags": input.tags,
                "meta": metadata,
            });
        }

        let locale = input.locale.clone();
        let node = self
            .node_service
            .create_node(
                tenant_id,
                actor_id,
                CreateNodeInput {
                    kind: "post".to_string(),
                    status: Some(if input.publish {
                        rustok_content::entities::node::ContentStatus::Published
                    } else {
                        rustok_content::entities::node::ContentStatus::Draft
                    }),
                    parent_id: None,
                    author_id,
                    category_id: None,
                    position: None,
                    depth: None,
                    reply_count: None,
                    metadata,
                    translations: vec![NodeTranslationInput {
                        locale: locale.clone(),
                        title: Some(input.title),
                        slug: input.slug,
                        excerpt: input.excerpt,
                    }],
                    bodies: vec![BodyInput {
                        locale,
                        body: Some(input.body),
                        format: None,
                    }],
                },
            )
            .await?;

        Ok(node.id)
    }

    pub async fn update_post(
        &self,
        post_id: Uuid,
        actor_id: Option<Uuid>,
        update: UpdateNodeInput,
    ) -> ContentResult<()> {
        self.node_service
            .update_node(post_id, actor_id, update)
            .await?;
        Ok(())
    }

    pub async fn publish_post(&self, post_id: Uuid, actor_id: Option<Uuid>) -> ContentResult<()> {
        self.node_service.publish_node(post_id, actor_id).await?;
        Ok(())
    }

    pub async fn unpublish_post(&self, post_id: Uuid, actor_id: Option<Uuid>) -> ContentResult<()> {
        self.node_service.unpublish_node(post_id, actor_id).await?;
        Ok(())
    }

    pub async fn delete_post(&self, post_id: Uuid, actor_id: Option<Uuid>) -> ContentResult<()> {
        self.node_service.delete_node(post_id, actor_id).await?;
        Ok(())
    }
}
