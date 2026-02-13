use sea_orm::DatabaseConnection;
use tracing::instrument;
use uuid::Uuid;

use rustok_content::NodeService;
use rustok_outbox::TransactionalEventBus;
use rustok_core::SecurityContext;

use crate::constants::reply_status;
use crate::error::ForumResult;

pub struct ModerationService {
    nodes: NodeService,
}

impl ModerationService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self {
            nodes: NodeService::new(db, event_bus),
        }
    }

    #[instrument(skip(self, security))]
    pub async fn approve_reply(
        &self,
        reply_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_reply_status(reply_id, security, reply_status::APPROVED)
            .await
    }

    #[instrument(skip(self, security))]
    pub async fn reject_reply(&self, reply_id: Uuid, security: SecurityContext) -> ForumResult<()> {
        self.update_reply_status(reply_id, security, reply_status::REJECTED)
            .await
    }

    #[instrument(skip(self, security))]
    pub async fn hide_reply(&self, reply_id: Uuid, security: SecurityContext) -> ForumResult<()> {
        self.update_reply_status(reply_id, security, reply_status::HIDDEN)
            .await
    }

    async fn update_reply_status(
        &self,
        reply_id: Uuid,
        security: SecurityContext,
        status: &str,
    ) -> ForumResult<()> {
        let node = self.nodes.get_node(reply_id).await?;
        let mut metadata = node.metadata;
        metadata["reply_status"] = serde_json::json!(status);

        self.nodes
            .update_node(
                reply_id,
                security,
                rustok_content::UpdateNodeInput {
                    metadata: Some(metadata),
                    ..Default::default()
                },
            )
            .await?;
        Ok(())
    }
}
