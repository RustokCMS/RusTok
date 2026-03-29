use sea_orm::{DatabaseConnection, TransactionTrait};
use tracing::instrument;
use uuid::Uuid;

use rustok_core::SecurityContext;
use rustok_events::DomainEvent;
use rustok_outbox::TransactionalEventBus;

use crate::error::ForumResult;
use crate::services::{ReplyService, TopicService};
use crate::state_machine::{ReplyStatus, TopicStatus};

pub struct ModerationService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
}

impl ModerationService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self { db, event_bus }
    }

    #[instrument(skip(self, security))]
    pub async fn approve_reply(
        &self,
        tenant_id: Uuid,
        reply_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_reply_status(
            tenant_id,
            reply_id,
            topic_id,
            security,
            ReplyStatus::Approved,
        )
        .await
    }

    #[instrument(skip(self, security))]
    pub async fn reject_reply(
        &self,
        tenant_id: Uuid,
        reply_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_reply_status(
            tenant_id,
            reply_id,
            topic_id,
            security,
            ReplyStatus::Rejected,
        )
        .await
    }

    #[instrument(skip(self, security))]
    pub async fn hide_reply(
        &self,
        tenant_id: Uuid,
        reply_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_reply_status(tenant_id, reply_id, topic_id, security, ReplyStatus::Hidden)
            .await
    }

    #[instrument(skip(self, security))]
    pub async fn pin_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        let txn = self.db.begin().await?;
        TopicService::set_pinned_in_tx(&txn, tenant_id, topic_id, true).await?;
        self.event_bus
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::ForumTopicPinned {
                    topic_id,
                    is_pinned: true,
                    moderator_id: security.user_id,
                },
            )
            .await?;
        txn.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn unpin_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        let txn = self.db.begin().await?;
        TopicService::set_pinned_in_tx(&txn, tenant_id, topic_id, false).await?;
        self.event_bus
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::ForumTopicPinned {
                    topic_id,
                    is_pinned: false,
                    moderator_id: security.user_id,
                },
            )
            .await?;
        txn.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn lock_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        let _ = security;
        let txn = self.db.begin().await?;
        TopicService::set_locked_in_tx(&txn, tenant_id, topic_id, true).await?;
        txn.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn unlock_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        let _ = security;
        let txn = self.db.begin().await?;
        TopicService::set_locked_in_tx(&txn, tenant_id, topic_id, false).await?;
        txn.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, security))]
    pub async fn close_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_topic_status(tenant_id, topic_id, security, TopicStatus::Closed)
            .await
    }

    #[instrument(skip(self, security))]
    pub async fn reopen_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_topic_status(tenant_id, topic_id, security, TopicStatus::Open)
            .await
    }

    #[instrument(skip(self, security))]
    pub async fn archive_topic(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
    ) -> ForumResult<()> {
        self.update_topic_status(tenant_id, topic_id, security, TopicStatus::Archived)
            .await
    }

    async fn update_reply_status(
        &self,
        tenant_id: Uuid,
        reply_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
        target: ReplyStatus,
    ) -> ForumResult<()> {
        let txn = self.db.begin().await?;
        let reply = ReplyService::find_reply_in_tx(&txn, tenant_id, reply_id).await?;
        let current = ReplyStatus::from_str_value(&reply.status).ok_or_else(|| {
            crate::error::ForumError::Validation(format!("Unknown reply status: {}", reply.status))
        })?;
        current.validate_transition(&target)?;

        let old_status = current.as_str().to_string();
        let new_status = target.as_str().to_string();

        ReplyService::set_status_in_tx(&txn, tenant_id, reply_id, &new_status).await?;

        self.event_bus
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::ForumReplyStatusChanged {
                    reply_id,
                    topic_id,
                    old_status,
                    new_status,
                    moderator_id: security.user_id,
                },
            )
            .await?;

        txn.commit().await?;
        Ok(())
    }

    async fn update_topic_status(
        &self,
        tenant_id: Uuid,
        topic_id: Uuid,
        security: SecurityContext,
        target: TopicStatus,
    ) -> ForumResult<()> {
        let topic_service = TopicService::new(self.db.clone(), self.event_bus.clone());
        let topic = topic_service.find_topic(tenant_id, topic_id).await?;
        let current = TopicStatus::from_str_value(&topic.status).unwrap_or(TopicStatus::Open);
        current.validate_transition(&target)?;

        let old_status = current.as_str().to_string();
        let new_status = target.as_str().to_string();

        let txn = self.db.begin().await?;
        TopicService::set_status_in_tx(&txn, tenant_id, topic_id, &new_status).await?;
        self.event_bus
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::ForumTopicStatusChanged {
                    topic_id,
                    old_status,
                    new_status,
                    moderator_id: security.user_id,
                },
            )
            .await?;
        txn.commit().await?;
        Ok(())
    }
}
