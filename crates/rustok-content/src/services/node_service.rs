use chrono::Utc;
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Set, TransactionTrait,
};
use uuid::Uuid;

use rustok_core::{
    Action, DomainEvent, PermissionScope, Resource, SecurityContext, TransactionalEventBus,
};

use crate::dto::{
    BodyInput, BodyResponse, CreateNodeInput, ListNodesFilter, NodeListItem, NodeResponse,
    NodeTranslationResponse, UpdateNodeInput,
};
use crate::entities::{body, node, node_translation};
use crate::error::{ContentError, ContentResult};

pub struct NodeService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
}

impl NodeService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self { db, event_bus }
    }

    fn kind_to_resource(kind: &str) -> Resource {
        match kind {
            "post" => Resource::Posts,
            "page" => Resource::Pages,
            "comment" => Resource::Comments,
            _ => Resource::Posts,
        }
    }

    pub async fn create_node(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        mut input: CreateNodeInput,
    ) -> ContentResult<NodeResponse> {
        // Scope Enforcement
        let resource = Self::kind_to_resource(&input.kind);
        let scope = security.get_scope(resource, Action::Create);

        match scope {
            PermissionScope::All => {}
            PermissionScope::Own => {
                // Force user_id as author_id
                input.author_id = security.user_id;
            }
            PermissionScope::None => {
                return Err(ContentError::Forbidden("Permission denied".into()));
            }
        }

        let now = Utc::now().into();
        let node_id = rustok_core::generate_id();
        let status = input
            .status
            .unwrap_or(crate::entities::node::ContentStatus::Draft);
        let metadata = input.metadata;
        if input.translations.is_empty() {
            return Err(ContentError::Validation(
                "At least one translation is required".to_string(),
            ));
        }

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
            metadata: Set(metadata),
            created_at: Set(now),
            updated_at: Set(now),
            published_at: if status == crate::entities::node::ContentStatus::Published {
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

            let _ = translation_model;
        }

        for body_input in input.bodies {
            upsert_body(&txn, node_id, body_input, now).await?;
        }

        self.event_bus
            .publish_in_tx(
                &txn,
                tenant_id,
                security.user_id,
                DomainEvent::NodeCreated {
                    node_id,
                    kind: input.kind.clone(),
                    author_id: input.author_id,
                },
            )
            .await?;

        txn.commit().await?;

        let response = self.get_node(node_model.id).await?;
        Ok(response)
    }

    pub async fn update_node(
        &self,
        node_id: Uuid,
        security: SecurityContext,
        update: UpdateNodeInput,
    ) -> ContentResult<NodeResponse> {
        let node_model = self.find_node(node_id).await?;

        // Scope Enforcement
        let resource = Self::kind_to_resource(&node_model.kind);
        let scope = security.get_scope(resource, Action::Update);

        match scope {
            PermissionScope::All => {}
            PermissionScope::Own => {
                if node_model.author_id != security.user_id {
                    return Err(ContentError::Forbidden(
                        "Permission denied: Not the author".into(),
                    ));
                }
                if update.author_id.is_some() {
                    return Err(ContentError::Forbidden(
                        "Permission denied: cannot change author".into(),
                    ));
                }
            }
            PermissionScope::None => {
                return Err(ContentError::Forbidden("Permission denied".into()));
            }
        }

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
            active.metadata = Set(metadata);
        }
        if let Some(published_at) = update.published_at {
            active.published_at = Set(published_at);
        }

        active.updated_at = Set(now);

        let txn = self.db.begin().await?;

        let updated = active.update(&txn).await?;

        if let Some(translations) = update.translations {
            node_translation::Entity::delete_many()
                .filter(node_translation::Column::NodeId.eq(node_id))
                .exec(&txn)
                .await?;

            for translation in translations {
                let slug = resolve_slug(translation.slug, translation.title.as_ref())?;
                node_translation::ActiveModel {
                    id: Set(rustok_core::generate_id()),
                    node_id: Set(node_id),
                    locale: Set(translation.locale),
                    title: Set(translation.title),
                    slug: Set(slug),
                    excerpt: Set(translation.excerpt),
                    created_at: Set(now),
                    updated_at: Set(now),
                }
                .insert(&txn)
                .await?;
            }
        }

        if let Some(bodies) = update.bodies {
            body::Entity::delete_many()
                .filter(body::Column::NodeId.eq(node_id))
                .exec(&txn)
                .await?;

            for body_input in bodies {
                upsert_body(&txn, node_id, body_input, now).await?;
            }
        }

        self.event_bus
            .publish_in_tx(
                &txn,
                updated.tenant_id,
                security.user_id,
                DomainEvent::NodeUpdated {
                    node_id: updated.id,
                    kind: updated.kind.clone(),
                },
            )
            .await?;

        txn.commit().await?;

        let translations = node_translation::Entity::find()
            .filter(node_translation::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await?;
        let bodies = body::Entity::find()
            .filter(body::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await?;

        Ok(Self::to_response(updated, translations, bodies))
    }

    pub async fn publish_node(
        &self,
        node_id: Uuid,
        security: SecurityContext,
    ) -> ContentResult<NodeResponse> {
        let now = Utc::now().into();
        let update = UpdateNodeInput {
            status: Some(crate::entities::node::ContentStatus::Published),
            published_at: Some(Some(now)),
            ..UpdateNodeInput::default()
        };
        let updated = self.update_node(node_id, security.clone(), update).await?;

        self.event_bus
            .publish(
                updated.tenant_id,
                security.user_id,
                DomainEvent::NodePublished {
                    node_id: updated.id,
                    kind: updated.kind.clone(),
                },
            )
            .await?;

        Ok(updated)
    }

    pub async fn unpublish_node(
        &self,
        node_id: Uuid,
        security: SecurityContext,
    ) -> ContentResult<NodeResponse> {
        let update = UpdateNodeInput {
            status: Some(crate::entities::node::ContentStatus::Draft),
            published_at: Some(None),
            ..UpdateNodeInput::default()
        };
        let updated = self.update_node(node_id, security.clone(), update).await?;

        self.event_bus
            .publish(
                updated.tenant_id,
                security.user_id,
                DomainEvent::NodeUnpublished {
                    node_id: updated.id,
                    kind: updated.kind.clone(),
                },
            )
            .await?;

        Ok(updated)
    }

    pub async fn delete_node(&self, node_id: Uuid, security: SecurityContext) -> ContentResult<()> {
        let node_model = self.find_node(node_id).await?;

        // Scope Enforcement
        let resource = Self::kind_to_resource(&node_model.kind);
        let scope = security.get_scope(resource, Action::Delete);

        match scope {
            PermissionScope::All => {}
            PermissionScope::Own => {
                if node_model.author_id != security.user_id {
                    return Err(ContentError::Forbidden(
                        "Permission denied: Not the author".into(),
                    ));
                }
            }
            PermissionScope::None => {
                return Err(ContentError::Forbidden("Permission denied".into()));
            }
        }

        let txn = self.db.begin().await?;

        node::Entity::delete_by_id(node_id).exec(&txn).await?;

        self.event_bus
            .publish_in_tx(
                &txn,
                node_model.tenant_id,
                security.user_id,
                DomainEvent::NodeDeleted {
                    node_id: node_model.id,
                    kind: node_model.kind,
                },
            )
            .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn find_node(&self, node_id: Uuid) -> ContentResult<node::Model> {
        node::Entity::find_by_id(node_id)
            .one(&self.db)
            .await?
            .ok_or(ContentError::NodeNotFound(node_id))
    }

    pub async fn get_node(&self, node_id: Uuid) -> ContentResult<NodeResponse> {
        let node_model = self.find_node(node_id).await?;
        let translations = node_translation::Entity::find()
            .filter(node_translation::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await?;
        let bodies = body::Entity::find()
            .filter(body::Column::NodeId.eq(node_id))
            .all(&self.db)
            .await?;

        Ok(Self::to_response(node_model, translations, bodies))
    }

    pub async fn get_by_slug(
        &self,
        tenant_id: Uuid,
        kind: &str,
        locale: &str,
        slug: &str,
    ) -> ContentResult<Option<NodeResponse>> {
        let result = node::Entity::find()
            .inner_join(node_translation::Entity)
            .filter(node::Column::TenantId.eq(tenant_id))
            .filter(node::Column::Kind.eq(kind))
            .filter(node_translation::Column::Locale.eq(locale))
            .filter(node_translation::Column::Slug.eq(slug))
            .one(&self.db)
            .await?;

        match result {
            Some(node_model) => Ok(Some(self.get_node(node_model.id).await?)),
            None => Ok(None),
        }
    }

    pub async fn list_nodes(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        mut filter: ListNodesFilter,
    ) -> ContentResult<(Vec<NodeListItem>, u64)> {
        // Scope Enforcement for List
        // We assume 'kind' is provided or we use a generic Resource::Posts if not.
        let resource = filter
            .kind
            .as_deref()
            .map(Self::kind_to_resource)
            .unwrap_or(Resource::Posts);
        let scope = security.get_scope(resource, Action::List);

        match scope {
            PermissionScope::All => {}
            PermissionScope::Own => {
                // Force user_id filter
                filter.author_id = security.user_id;
            }
            PermissionScope::None => {
                return Err(ContentError::Forbidden("Permission denied".into()));
            }
        }

        let locale = filter.locale.clone().unwrap_or_else(|| "en".to_string());
        let mut query = node::Entity::find().filter(node::Column::TenantId.eq(tenant_id));

        if let Some(kind) = filter.kind {
            query = query.filter(node::Column::Kind.eq(kind));
        }
        if let Some(status) = filter.status {
            query = query.filter(node::Column::Status.eq(status));
        }
        if let Some(parent_id) = filter.parent_id {
            query = query.filter(node::Column::ParentId.eq(parent_id));
        }
        if let Some(author_id) = filter.author_id {
            query = query.filter(node::Column::AuthorId.eq(author_id));
        }

        let paginator = query.clone().paginate(&self.db, filter.per_page);
        let total = paginator.num_items().await?;
        let nodes = paginator.fetch_page(filter.page.saturating_sub(1)).await?;

        let node_ids: Vec<Uuid> = nodes.iter().map(|node| node.id).collect();
        let translations = node_translation::Entity::find()
            .filter(node_translation::Column::NodeId.is_in(node_ids))
            .filter(node_translation::Column::Locale.eq(locale))
            .all(&self.db)
            .await?;

        let mut translations_map = std::collections::HashMap::new();
        for translation in translations {
            translations_map.insert(translation.node_id, translation);
        }

        let items = nodes
            .into_iter()
            .map(|node| {
                let translation = translations_map.get(&node.id);
                NodeListItem {
                    id: node.id,
                    kind: node.kind,
                    status: node.status,
                    title: translation.and_then(|t| t.title.clone()),
                    slug: translation.and_then(|t| t.slug.clone()),
                    excerpt: translation.and_then(|t| t.excerpt.clone()),
                    author_id: node.author_id,
                    created_at: node.created_at.to_rfc3339(),
                    published_at: node.published_at.map(|date| date.to_rfc3339()),
                }
            })
            .collect();

        Ok((items, total))
    }
}

fn resolve_slug(slug: Option<String>, title: Option<&String>) -> ContentResult<Option<String>> {
    if let Some(slug) = slug {
        return Ok(Some(slug));
    }

    if let Some(title) = title {
        return Ok(Some(slug::slugify(title)));
    }

    Err(ContentError::Validation(
        "Slug or title must be provided".to_string(),
    ))
}

async fn upsert_body<C>(
    db: &C,
    node_id: Uuid,
    input: BodyInput,
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

impl NodeService {
    fn to_response(
        node: node::Model,
        translations: Vec<node_translation::Model>,
        bodies: Vec<body::Model>,
    ) -> NodeResponse {
        NodeResponse {
            id: node.id,
            tenant_id: node.tenant_id,
            kind: node.kind,
            status: node.status,
            parent_id: node.parent_id,
            author_id: node.author_id,
            category_id: node.category_id,
            position: node.position,
            depth: node.depth,
            reply_count: node.reply_count,
            metadata: node.metadata,
            created_at: node.created_at.to_rfc3339(),
            updated_at: node.updated_at.to_rfc3339(),
            published_at: node.published_at.map(|date| date.to_rfc3339()),
            translations: translations
                .into_iter()
                .map(|translation| NodeTranslationResponse {
                    locale: translation.locale,
                    title: translation.title,
                    slug: translation.slug,
                    excerpt: translation.excerpt,
                })
                .collect(),
            bodies: bodies
                .into_iter()
                .map(|body| BodyResponse {
                    locale: body.locale,
                    body: body.body,
                    format: body.format,
                    updated_at: body.updated_at.to_rfc3339(),
                })
                .collect(),
        }
    }
}
