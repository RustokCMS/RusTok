use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};
use serde_json::Value;
use uuid::Uuid;

use rustok_core::{Action, DomainEvent, PermissionScope, Resource, SecurityContext};

use crate::dto::{BodyInput, CreateNodeInput, NodeTranslationInput};
use crate::entities::{body, node, node_translation};
use crate::error::{ContentError, ContentResult};
use crate::services::orchestration_mapping::{
    map_post_to_topic_input, map_topic_to_post_input, stamp_audit_metadata, AuditStamp, KIND_POST,
    KIND_TOPIC,
};
use crate::NodeService;

const COMMENT_KINDS: [&str; 2] = ["forum_reply", "comment"];

#[derive(Debug, Clone)]
pub struct PromoteTopicToPostInput {
    pub topic_id: Uuid,
    pub locale: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DemotePostToTopicInput {
    pub post_id: Uuid,
    pub locale: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SplitTopicInput {
    pub topic_id: Uuid,
    pub locale: String,
    pub reply_ids: Vec<Uuid>,
    pub new_title: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MergeTopicsInput {
    pub target_topic_id: Uuid,
    pub source_topic_ids: Vec<Uuid>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    pub source_id: Uuid,
    pub target_id: Uuid,
    pub moved_comments: u64,
}

pub struct ContentOrchestrationService {
    node_service: NodeService,
}

impl ContentOrchestrationService {
    pub fn new(node_service: NodeService) -> Self {
        Self { node_service }
    }

    pub async fn promote_topic_to_post(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: PromoteTopicToPostInput,
    ) -> ContentResult<OrchestrationResult> {
        self.ensure_scope(security.clone(), Resource::ForumTopics, Action::Moderate)?;
        self.ensure_scope(security.clone(), Resource::BlogPosts, Action::Create)?;

        let txn = self.node_service.db().begin().await?;

        let topic = self
            .find_node_in_tx(&txn, tenant_id, input.topic_id, KIND_TOPIC)
            .await?;
        let (translations, bodies) = self
            .extract_locale_payload_in_tx(&txn, topic.id, &input.locale)
            .await?;

        let create_input = map_topic_to_post_input(&topic, translations, bodies);
        let post_id = self
            .node_service
            .create_node_in_tx(&txn, tenant_id, security.clone(), create_input)
            .await?;

        let moved_comments = self
            .rebind_children_in_tx(&txn, tenant_id, topic.id, post_id, None)
            .await?;

        self.mark_source_metadata(
            &txn,
            topic,
            "topic.promoted_to_post",
            post_id,
            security.user_id,
            input.reason.as_deref(),
        )
        .await?;

        self.node_service
            .event_bus()
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::TopicPromotedToPost {
                    topic_id: input.topic_id,
                    post_id,
                    moved_comments,
                    locale: input.locale,
                    reason: input.reason,
                },
            )
            .await?;

        txn.commit().await?;

        Ok(OrchestrationResult {
            source_id: input.topic_id,
            target_id: post_id,
            moved_comments,
        })
    }

    pub async fn demote_post_to_topic(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: DemotePostToTopicInput,
    ) -> ContentResult<OrchestrationResult> {
        self.ensure_scope(security.clone(), Resource::BlogPosts, Action::Moderate)?;
        self.ensure_scope(security.clone(), Resource::ForumTopics, Action::Create)?;

        let txn = self.node_service.db().begin().await?;

        let post = self
            .find_node_in_tx(&txn, tenant_id, input.post_id, KIND_POST)
            .await?;
        let (translations, bodies) = self
            .extract_locale_payload_in_tx(&txn, post.id, &input.locale)
            .await?;

        let create_input = map_post_to_topic_input(&post, translations, bodies);
        let topic_id = self
            .node_service
            .create_node_in_tx(&txn, tenant_id, security.clone(), create_input)
            .await?;

        let moved_comments = self
            .rebind_children_in_tx(&txn, tenant_id, post.id, topic_id, None)
            .await?;

        self.mark_source_metadata(
            &txn,
            post,
            "post.demoted_to_topic",
            topic_id,
            security.user_id,
            input.reason.as_deref(),
        )
        .await?;

        self.node_service
            .event_bus()
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::PostDemotedToTopic {
                    post_id: input.post_id,
                    topic_id,
                    moved_comments,
                    locale: input.locale,
                    reason: input.reason,
                },
            )
            .await?;

        txn.commit().await?;

        Ok(OrchestrationResult {
            source_id: input.post_id,
            target_id: topic_id,
            moved_comments,
        })
    }

    pub async fn split_topic(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: SplitTopicInput,
    ) -> ContentResult<OrchestrationResult> {
        self.ensure_scope(security.clone(), Resource::ForumTopics, Action::Moderate)?;
        if input.reply_ids.is_empty() {
            return Err(ContentError::Validation(
                "split_topic requires at least one reply/comment id".to_string(),
            ));
        }

        let txn = self.node_service.db().begin().await?;
        let topic = self
            .find_node_in_tx(&txn, tenant_id, input.topic_id, KIND_TOPIC)
            .await?;

        let (mut translations, bodies) = self
            .extract_locale_payload_in_tx(&txn, topic.id, &input.locale)
            .await?;

        if let Some(primary) = translations.iter_mut().find(|tr| tr.locale == input.locale) {
            primary.title = Some(input.new_title.clone());
            primary.slug = Some(slug::slugify(&input.new_title));
        }

        let new_topic_id = self
            .node_service
            .create_node_in_tx(
                &txn,
                tenant_id,
                security.clone(),
                CreateNodeInput {
                    kind: KIND_TOPIC.to_string(),
                    status: Some(topic.status.clone()),
                    parent_id: topic.parent_id,
                    author_id: topic.author_id,
                    category_id: topic.category_id,
                    position: Some(topic.position + 1),
                    depth: Some(topic.depth),
                    reply_count: Some(0),
                    metadata: topic.metadata.clone(),
                    translations,
                    bodies,
                },
            )
            .await?;

        let moved_comments = self
            .rebind_children_in_tx(
                &txn,
                tenant_id,
                topic.id,
                new_topic_id,
                Some(input.reply_ids.clone()),
            )
            .await?;

        self.node_service
            .event_bus()
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::TopicSplit {
                    source_topic_id: topic.id,
                    target_topic_id: new_topic_id,
                    moved_comment_ids: input.reply_ids,
                    moved_comments,
                    reason: input.reason,
                },
            )
            .await?;

        txn.commit().await?;

        Ok(OrchestrationResult {
            source_id: topic.id,
            target_id: new_topic_id,
            moved_comments,
        })
    }

    pub async fn merge_topics(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: MergeTopicsInput,
    ) -> ContentResult<OrchestrationResult> {
        self.ensure_scope(security.clone(), Resource::ForumTopics, Action::Moderate)?;
        if input.source_topic_ids.is_empty() {
            return Err(ContentError::Validation(
                "merge_topics requires at least one source topic".to_string(),
            ));
        }

        let txn = self.node_service.db().begin().await?;
        let target = self
            .find_node_in_tx(&txn, tenant_id, input.target_topic_id, KIND_TOPIC)
            .await?;

        let mut moved_comments: u64 = 0;

        for source_topic_id in input.source_topic_ids {
            let source = self
                .find_node_in_tx(&txn, tenant_id, source_topic_id, KIND_TOPIC)
                .await?;

            moved_comments += self
                .rebind_children_in_tx(&txn, tenant_id, source.id, target.id, None)
                .await?;

            self.mark_source_metadata(
                &txn,
                source,
                "topic.merged_into",
                target.id,
                security.user_id,
                input.reason.as_deref(),
            )
            .await?;
        }

        self.node_service
            .event_bus()
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::TopicsMerged {
                    target_topic_id: target.id,
                    moved_comments,
                    reason: input.reason,
                },
            )
            .await?;

        txn.commit().await?;

        Ok(OrchestrationResult {
            source_id: target.id,
            target_id: target.id,
            moved_comments,
        })
    }

    fn ensure_scope(
        &self,
        security: SecurityContext,
        resource: Resource,
        action: Action,
    ) -> ContentResult<()> {
        match security.get_scope(resource, action) {
            PermissionScope::All => Ok(()),
            PermissionScope::Own => {
                if security.user_id.is_some() {
                    Ok(())
                } else {
                    Err(ContentError::Forbidden("Permission denied".to_string()))
                }
            }
            PermissionScope::None => Err(ContentError::Forbidden("Permission denied".to_string())),
        }
    }

    async fn extract_locale_payload_in_tx(
        &self,
        txn: &DatabaseTransaction,
        node_id: Uuid,
        locale: &str,
    ) -> ContentResult<(Vec<NodeTranslationInput>, Vec<BodyInput>)> {
        let translations = node_translation::Entity::find()
            .filter(node_translation::Column::NodeId.eq(node_id))
            .all(txn)
            .await?;

        if !translations.iter().any(|tr| tr.locale == locale) {
            return Err(ContentError::Validation(format!(
                "Locale '{locale}' is not available for node {node_id}"
            )));
        }

        let bodies = body::Entity::find()
            .filter(body::Column::NodeId.eq(node_id))
            .all(txn)
            .await?;

        let translations = translations
            .into_iter()
            .map(|tr| NodeTranslationInput {
                locale: tr.locale,
                title: tr.title,
                slug: tr.slug,
                excerpt: tr.excerpt,
            })
            .collect();

        let bodies = bodies
            .into_iter()
            .map(|b| BodyInput {
                locale: b.locale,
                body: b.body,
                format: Some(b.format),
            })
            .collect();

        Ok((translations, bodies))
    }

    async fn find_node_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        node_id: Uuid,
        expected_kind: &str,
    ) -> ContentResult<node::Model> {
        let model = node::Entity::find_by_id(node_id)
            .filter(node::Column::TenantId.eq(tenant_id))
            .filter(node::Column::DeletedAt.is_null())
            .one(txn)
            .await?
            .ok_or(ContentError::NodeNotFound(node_id))?;

        if model.kind != expected_kind {
            return Err(ContentError::Validation(format!(
                "Node {node_id} must have kind '{expected_kind}', got '{}'",
                model.kind
            )));
        }

        Ok(model)
    }

    async fn rebind_children_in_tx(
        &self,
        txn: &DatabaseTransaction,
        tenant_id: Uuid,
        from_parent_id: Uuid,
        to_parent_id: Uuid,
        limit_ids: Option<Vec<Uuid>>,
    ) -> ContentResult<u64> {
        let mut query = node::Entity::find()
            .filter(node::Column::TenantId.eq(tenant_id))
            .filter(node::Column::ParentId.eq(from_parent_id))
            .filter(node::Column::Kind.is_in(COMMENT_KINDS));

        if let Some(limit_ids) = &limit_ids {
            query = query.filter(node::Column::Id.is_in(limit_ids.clone()));
        }

        let children = query.all(txn).await?;
        let moved = children.len() as u64;

        for child in children {
            let previous_version = child.version;
            let mut active: node::ActiveModel = child.into();
            active.parent_id = Set(Some(to_parent_id));
            active.updated_at = Set(Utc::now().into());
            active.version = Set(previous_version + 1);
            active.update(txn).await?;
        }

        Ok(moved)
    }

    async fn mark_source_metadata(
        &self,
        txn: &DatabaseTransaction,
        source: node::Model,
        op: &str,
        linked_id: Uuid,
        actor_id: Option<Uuid>,
        reason: Option<&str>,
    ) -> ContentResult<()> {
        let mut active: node::ActiveModel = source.clone().into();
        let mut metadata: Value = source.metadata;
        stamp_audit_metadata(
            &mut metadata,
            AuditStamp {
                operation: op,
                actor_id,
                reason,
            },
        );
        let mut obj = metadata.as_object().cloned().unwrap_or_default();
        obj.insert(
            "linked_node_id".to_string(),
            Value::String(linked_id.to_string()),
        );

        active.metadata = Set(Value::Object(obj));
        active.updated_at = Set(Utc::now().into());
        active.version = Set(source.version + 1);
        active.update(txn).await?;
        Ok(())
    }
}
