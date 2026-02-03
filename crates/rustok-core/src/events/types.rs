use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub trace_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub event: DomainEvent,
    pub retry_count: u32,
}

impl EventEnvelope {
    pub fn new(tenant_id: Uuid, actor_id: Option<Uuid>, event: DomainEvent) -> Self {
        let id = crate::id::generate_id();
        Self {
            id,
            correlation_id: id,
            causation_id: None,
            tenant_id,
            trace_id: rustok_telemetry::current_trace_id(),
            timestamp: Utc::now(),
            actor_id,
            event,
            retry_count: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum DomainEvent {
    // ════════════════════════════════════════════════════════════════
    // CONTENT EVENTS (nodes, bodies)
    // ════════════════════════════════════════════════════════════════
    NodeCreated {
        node_id: Uuid,
        kind: String,
        author_id: Option<Uuid>,
    },
    NodeUpdated {
        node_id: Uuid,
        kind: String,
    },
    NodeTranslationUpdated {
        node_id: Uuid,
        locale: String,
    },
    NodePublished {
        node_id: Uuid,
        kind: String,
    },
    NodeUnpublished {
        node_id: Uuid,
        kind: String,
    },
    NodeDeleted {
        node_id: Uuid,
        kind: String,
    },
    BodyUpdated {
        node_id: Uuid,
        locale: String,
    },

    // ════════════════════════════════════════════════════════════════
    // CATEGORY EVENTS
    // ════════════════════════════════════════════════════════════════
    CategoryCreated {
        category_id: Uuid,
    },
    CategoryUpdated {
        category_id: Uuid,
    },
    CategoryDeleted {
        category_id: Uuid,
    },

    // ════════════════════════════════════════════════════════════════
    // TAG EVENTS
    // ════════════════════════════════════════════════════════════════
    TagCreated {
        tag_id: Uuid,
    },
    TagAttached {
        tag_id: Uuid,
        target_type: String,
        target_id: Uuid,
    },
    TagDetached {
        tag_id: Uuid,
        target_type: String,
        target_id: Uuid,
    },

    // ════════════════════════════════════════════════════════════════
    // MEDIA EVENTS
    // ════════════════════════════════════════════════════════════════
    MediaUploaded {
        media_id: Uuid,
        mime_type: String,
        size: i64,
    },
    MediaDeleted {
        media_id: Uuid,
    },

    // ════════════════════════════════════════════════════════════════
    // USER EVENTS
    // ════════════════════════════════════════════════════════════════
    UserRegistered {
        user_id: Uuid,
        email: String,
    },
    UserLoggedIn {
        user_id: Uuid,
    },
    UserUpdated {
        user_id: Uuid,
    },
    UserDeleted {
        user_id: Uuid,
    },

    // ════════════════════════════════════════════════════════════════
    // COMMERCE EVENTS (для будущего модуля)
    // ════════════════════════════════════════════════════════════════
    ProductCreated {
        product_id: Uuid,
    },
    ProductUpdated {
        product_id: Uuid,
    },
    ProductPublished {
        product_id: Uuid,
    },
    ProductDeleted {
        product_id: Uuid,
    },
    VariantCreated {
        variant_id: Uuid,
        product_id: Uuid,
    },
    VariantUpdated {
        variant_id: Uuid,
        product_id: Uuid,
    },
    VariantDeleted {
        variant_id: Uuid,
        product_id: Uuid,
    },
    InventoryUpdated {
        variant_id: Uuid,
        product_id: Uuid,
        location_id: Uuid,
        old_quantity: i32,
        new_quantity: i32,
    },
    InventoryLow {
        variant_id: Uuid,
        product_id: Uuid,
        remaining: i32,
        threshold: i32,
    },
    PriceUpdated {
        variant_id: Uuid,
        product_id: Uuid,
        currency: String,
        old_amount: Option<i64>,
        new_amount: i64,
    },
    OrderPlaced {
        order_id: Uuid,
        customer_id: Option<Uuid>,
        total: i64,
        currency: String,
    },
    OrderStatusChanged {
        order_id: Uuid,
        old_status: String,
        new_status: String,
    },
    OrderCompleted {
        order_id: Uuid,
    },
    OrderCancelled {
        order_id: Uuid,
        reason: Option<String>,
    },

    // ════════════════════════════════════════════════════════════════
    // INDEX EVENTS (CQRS)
    // ════════════════════════════════════════════════════════════════
    ReindexRequested {
        target_type: String,
        target_id: Option<Uuid>,
    },
    IndexUpdated {
        index_name: String,
        target_id: Uuid,
    },

    // ════════════════════════════════════════════════════════════════
    // TENANT EVENTS
    // ════════════════════════════════════════════════════════════════
    TenantCreated {
        tenant_id: Uuid,
    },
    TenantUpdated {
        tenant_id: Uuid,
    },
    LocaleEnabled {
        tenant_id: Uuid,
        locale: String,
    },
    LocaleDisabled {
        tenant_id: Uuid,
        locale: String,
    },
}

impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::NodeCreated { .. } => "node.created",
            Self::NodeUpdated { .. } => "node.updated",
            Self::NodeTranslationUpdated { .. } => "node.translation.updated",
            Self::NodePublished { .. } => "node.published",
            Self::NodeUnpublished { .. } => "node.unpublished",
            Self::NodeDeleted { .. } => "node.deleted",
            Self::BodyUpdated { .. } => "body.updated",

            Self::CategoryCreated { .. } => "category.created",
            Self::CategoryUpdated { .. } => "category.updated",
            Self::CategoryDeleted { .. } => "category.deleted",

            Self::TagCreated { .. } => "tag.created",
            Self::TagAttached { .. } => "tag.attached",
            Self::TagDetached { .. } => "tag.detached",

            Self::MediaUploaded { .. } => "media.uploaded",
            Self::MediaDeleted { .. } => "media.deleted",

            Self::UserRegistered { .. } => "user.registered",
            Self::UserLoggedIn { .. } => "user.logged_in",
            Self::UserUpdated { .. } => "user.updated",
            Self::UserDeleted { .. } => "user.deleted",

            Self::ProductCreated { .. } => "product.created",
            Self::ProductUpdated { .. } => "product.updated",
            Self::ProductPublished { .. } => "product.published",
            Self::ProductDeleted { .. } => "product.deleted",
            Self::VariantCreated { .. } => "variant.created",
            Self::VariantUpdated { .. } => "variant.updated",
            Self::VariantDeleted { .. } => "variant.deleted",
            Self::InventoryUpdated { .. } => "inventory.updated",
            Self::InventoryLow { .. } => "inventory.low",
            Self::PriceUpdated { .. } => "price.updated",
            Self::OrderPlaced { .. } => "order.placed",
            Self::OrderStatusChanged { .. } => "order.status_changed",
            Self::OrderCompleted { .. } => "order.completed",
            Self::OrderCancelled { .. } => "order.cancelled",

            Self::ReindexRequested { .. } => "index.reindex_requested",
            Self::IndexUpdated { .. } => "index.updated",

            Self::TenantCreated { .. } => "tenant.created",
            Self::TenantUpdated { .. } => "tenant.updated",
            Self::LocaleEnabled { .. } => "locale.enabled",
            Self::LocaleDisabled { .. } => "locale.disabled",
        }
    }

    pub fn affects_index(&self) -> bool {
        matches!(
            self,
            Self::NodeCreated { .. }
                | Self::NodeUpdated { .. }
                | Self::NodeTranslationUpdated { .. }
                | Self::NodePublished { .. }
                | Self::NodeUnpublished { .. }
                | Self::NodeDeleted { .. }
                | Self::BodyUpdated { .. }
                | Self::ProductCreated { .. }
                | Self::ProductUpdated { .. }
                | Self::ProductPublished { .. }
                | Self::ProductDeleted { .. }
                | Self::VariantUpdated { .. }
                | Self::InventoryUpdated { .. }
                | Self::PriceUpdated { .. }
                | Self::TagAttached { .. }
                | Self::TagDetached { .. }
        )
    }
}
