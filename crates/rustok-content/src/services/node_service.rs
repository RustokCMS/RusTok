use chrono::Utc;
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, Set, TransactionTrait,
};
use serde_json::Value;
use uuid::Uuid;

use rustok_core::{DomainEvent, EventBus};

use crate::entities::{body, node, node_translation};
use crate::error::{ContentError, ContentResult};

#[derive(Debug, Clone)]
pub struct NodeTranslationInput {
    pub locale: String,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub excerpt: Option<String>,
    pub body: Option<String>,
    pub body_format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NodeBodyInput {
    pub locale: String,
    pub body: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateNodeInput {
    pub kind: String,
    pub status: Option<String>,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    pub position: Option<i32>,
    pub depth: Option<i32>,
    pub reply_count: Option<i32>,
    pub metadata: Option<Value>,
    pub translations: Vec<NodeTranslationInput>,
    pub bodies: Vec<NodeBodyInput>,
}

#[derive(Debug, Clone, Default)]
pub struct NodeUpdate {
    pub parent_id: Option<Option<Uuid>>,
    pub author_id: Option<Option<Uuid>>,
    pub category_id: Option<Option<Uuid>>,
    pub status: Option<String>,
    pub position: Option<i32>,
    pub depth: Option<i32>,
    pub reply_count: Option<i32>,
    pub metadata: Option<Value>,
    pub published_at: Option<Option<DateTimeWithTimeZone>>,
}

pub struct NodeService {
    db: DatabaseConnection,
    event_bus: EventBus,
}

impl NodeService {
    pub fn new(db: DatabaseConnection, event_bus: EventBus) -> Self {
        Self { db, event_bus }
    }

    pub async fn create_node(
        &self,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        input: CreateNodeInput,
    ) -> ContentResult<node::Model> {
        let now = Utc::now().into();
        let node_id = rustok_core::generate_id();
        let status = input.status.unwrap_or_else(|| "draft".to_string());
        let metadata = input
            .metadata
            .unwrap_or_else(|| serde_json::json!({}));

        let txn = self.db.begin().await?;

        let node_model = node::ActiveModel {
            id: Set(node_id),
            tenant_id: Set(tenant_id),
            parent_id: Set(input.parent_id),
            author_id: Set(input.author_id),
            kind: Set(input.kind.clone()),
            category_id: Set(input.category_id),
            status: Set(status.clone()),
            position: Set(input.position.unwrap_or(0)),
            depth: Set(input.depth.unwrap_or(0)),
            reply_count: Set(input.reply_count.unwrap_or(0)),
            metadata: Set(metadata.into()),
            created_at: Set(now),
            updated_at: Set(now),
            published_at: if status == "published" {
                Set(Some(now))
            } else {
                Set(None)
            },
        }
        .insert(&txn)
        .await?;

        for translation in input.translations {
            let slug = resolve_slug(translation.slug, translation.title.as_ref())?;
            let translation_model = node_translation::ActiveModel {
                id: Set(rustok_core::generate_id()),
                node_id: Set(node_id),
                locale: Set(translation.locale.clone()),
                title: Set(translation.title),
                slug: Set(slug),
                excerpt: Set(translation.excerpt),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(&txn)
            .await?;

            if let Some(body_input) = translation
                .body
                .map(|body| NodeBodyInput {
                    locale: translation_model.locale.clone(),
                    body: Some(body),
                    format: translation.body_format.clone(),
                })
            {
                upsert_body(&txn, node_id, body_input, now).await?;
            }
        }

        for body_input in input.bodies {
            upsert_body(&txn, node_id, body_input, now).await?;
        }

        txn.commit().await?;

        self.event_bus.publish(
            tenant_id,
            actor_id,
            DomainEvent::NodeCreated {
                node_id,
                kind: input.kind,
                author_id: input.author_id,
            },
        )?;

        Ok(node_model)
    }

    pub async fn update_node(
        &self,
        node_id: Uuid,
        actor_id: Option<Uuid>,
        update: NodeUpdate,
    ) -> ContentResult<node::Model> {
        let node_model = self.find_node(node_id).await?;
        let mut active: node::ActiveModel = node_model.clone().into();
        let now = Utc::now().into();

        if let Some(parent_id) = update.parent_id {
            active.parent_id = Set(parent_id);
        }
        if let Some(author_id) = update.author_id {
            active.author_id = Set(author_id);
        }
        if let Some(category_id) = update.category_id {
            active.category_id = Set(category_id);
        }
        if let Some(status) = update.status.clone() {
            active.status = Set(status);
        }
        if let Some(position) = update.position {
            active.position = Set(position);
        }
        if let Some(depth) = update.depth {
            active.depth = Set(depth);
        }
        if let Some(reply_count) = update.reply_count {
            active.reply_count = Set(reply_count);
        }
        if let Some(metadata) = update.metadata {
            active.metadata = Set(metadata.into());
        }
        if let Some(published_at) = update.published_at {
            active.published_at = Set(published_at);
        }

        active.updated_at = Set(now);

        let updated = active.update(&self.db).await?;

        self.event_bus.publish(
            updated.tenant_id,
            actor_id,
            DomainEvent::NodeUpdated {
                node_id: updated.id,
                kind: updated.kind.clone(),
            },
        )?;

        Ok(updated)
    }

    pub async fn upsert_translation(
        &self,
        node_id: Uuid,
        actor_id: Option<Uuid>,
        input: NodeTranslationInput,
    ) -> ContentResult<node_translation::Model> {
        let node_model = self.find_node(node_id).await?;
        let now = Utc::now().into();

        let existing = node_translation::Entity::find()
            .filter(node_translation::Column::NodeId.eq(node_id))
            .filter(node_translation::Column::Locale.eq(input.locale.clone()))
            .one(&self.db)
            .await?;

        let slug = resolve_slug(input.slug, input.title.as_ref())?;

        let translation = if let Some(existing) = existing {
            let mut active: node_translation::ActiveModel = existing.into();
            if let Some(title) = input.title {
                active.title = Set(Some(title));
            }
            if slug.is_some() {
                active.slug = Set(slug);
            }
            if let Some(excerpt) = input.excerpt {
                active.excerpt = Set(Some(excerpt));
            }
            active.updated_at = Set(now);
            active.update(&self.db).await?
        } else {
            node_translation::ActiveModel {
                id: Set(rustok_core::generate_id()),
                node_id: Set(node_id),
                locale: Set(input.locale.clone()),
                title: Set(input.title),
                slug: Set(slug),
                excerpt: Set(input.excerpt),
                created_at: Set(now),
                updated_at: Set(now),
            }
            .insert(&self.db)
            .await?
        };

        if input.body.is_some() {
            let body_input = NodeBodyInput {
                locale: translation.locale.clone(),
                body: input.body,
                format: input.body_format,
            };
            upsert_body(&self.db, node_id, body_input, now).await?;

            self.event_bus.publish(
                node_model.tenant_id,
                actor_id,
                DomainEvent::BodyUpdated {
                    node_id: node_model.id,
                    locale: translation.locale.clone(),
                },
            )?;
        }

        self.event_bus.publish(
            node_model.tenant_id,
            actor_id,
            DomainEvent::NodeTranslationUpdated {
                node_id: node_model.id,
                locale: translation.locale.clone(),
            },
        )?;

        Ok(translation)
    }

    pub async fn publish_node(
        &self,
        node_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> ContentResult<node::Model> {
        let now = Utc::now().into();
        let update = NodeUpdate {
            status: Some("published".to_string()),
            published_at: Some(Some(now)),
            ..NodeUpdate::default()
        };
        let updated = self.update_node(node_id, actor_id, update).await?;

        self.event_bus.publish(
            updated.tenant_id,
            actor_id,
            DomainEvent::NodePublished {
                node_id: updated.id,
                kind: updated.kind.clone(),
            },
        )?;

        Ok(updated)
    }

    pub async fn unpublish_node(
        &self,
        node_id: Uuid,
        actor_id: Option<Uuid>,
    ) -> ContentResult<node::Model> {
        let update = NodeUpdate {
            status: Some("draft".to_string()),
            published_at: Some(None),
            ..NodeUpdate::default()
        };
        let updated = self.update_node(node_id, actor_id, update).await?;

        self.event_bus.publish(
            updated.tenant_id,
            actor_id,
            DomainEvent::NodeUnpublished {
                node_id: updated.id,
                kind: updated.kind.clone(),
            },
        )?;

        Ok(updated)
    }

    pub async fn delete_node(&self, node_id: Uuid, actor_id: Option<Uuid>) -> ContentResult<()> {
        let node_model = self.find_node(node_id).await?;
        node::Entity::delete_by_id(node_id).exec(&self.db).await?;

        self.event_bus.publish(
            node_model.tenant_id,
            actor_id,
            DomainEvent::NodeDeleted {
                node_id: node_model.id,
                kind: node_model.kind,
            },
        )?;

        Ok(())
    }

    pub async fn find_node(&self, node_id: Uuid) -> ContentResult<node::Model> {
        node::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?
            .ok_or(ContentError::NodeNotFound(node_id))
    }
}

fn resolve_slug(
    slug: Option<String>,
    title: Option<&String>,
) -> ContentResult<Option<String>> {
    if let Some(slug) = slug {
        return Ok(Some(slug));
    }

    if let Some(title) = title {
        return Ok(Some(slug::slugify(title)));
    }

    Ok(None)
}

async fn upsert_body<C>(
    db: &C,
    node_id: Uuid,
    input: NodeBodyInput,
    now: DateTimeWithTimeZone,
) -> ContentResult<body::Model>
where
    C: sea_orm::ConnectionTrait,
{
    let existing = body::Entity::find()
        .filter(body::Column::NodeId.eq(node_id))
        .filter(body::Column::Locale.eq(input.locale.clone()))
        .one(db)
        .await?;

    let format = input.format.unwrap_or_else(|| "markdown".to_string());

    let model = if let Some(existing) = existing {
        let mut active: body::ActiveModel = existing.into();
        if input.body.is_some() {
            active.body = Set(input.body);
        }
        active.format = Set(format);
        active.updated_at = Set(now);
        active.update(db).await?
    } else {
        body::ActiveModel {
            id: Set(rustok_core::generate_id()),
            node_id: Set(node_id),
            locale: Set(input.locale),
            body: Set(input.body),
            format: Set(format),
            updated_at: Set(now),
        }
        .insert(db)
        .await?
    };

    Ok(model)
}
