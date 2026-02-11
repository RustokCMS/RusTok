use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    /// Event type string for fast filtering and routing
    pub event_type: String,
    /// Schema version for this event type (for evolution tracking)
    pub schema_version: u16,
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
        let event_type = event.event_type().to_string();
        let schema_version = event.schema_version();
        Self {
            id,
            event_type,
            schema_version,
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

    /// Returns the schema version for this event type.
    /// Increment this version when making breaking changes to the event structure.
    ///
    /// Version History:
    /// - v1: Initial schema for all events
    pub fn schema_version(&self) -> u16 {
        match self {
            // Content events (v1)
            Self::NodeCreated { .. } => 1,
            Self::NodeUpdated { .. } => 1,
            Self::NodeTranslationUpdated { .. } => 1,
            Self::NodePublished { .. } => 1,
            Self::NodeUnpublished { .. } => 1,
            Self::NodeDeleted { .. } => 1,
            Self::BodyUpdated { .. } => 1,

            // Category events (v1)
            Self::CategoryCreated { .. } => 1,
            Self::CategoryUpdated { .. } => 1,
            Self::CategoryDeleted { .. } => 1,

            // Tag events (v1)
            Self::TagCreated { .. } => 1,
            Self::TagAttached { .. } => 1,
            Self::TagDetached { .. } => 1,

            // Media events (v1)
            Self::MediaUploaded { .. } => 1,
            Self::MediaDeleted { .. } => 1,

            // User events (v1)
            Self::UserRegistered { .. } => 1,
            Self::UserLoggedIn { .. } => 1,
            Self::UserUpdated { .. } => 1,
            Self::UserDeleted { .. } => 1,

            // Commerce events (v1)
            Self::ProductCreated { .. } => 1,
            Self::ProductUpdated { .. } => 1,
            Self::ProductPublished { .. } => 1,
            Self::ProductDeleted { .. } => 1,
            Self::VariantCreated { .. } => 1,
            Self::VariantUpdated { .. } => 1,
            Self::VariantDeleted { .. } => 1,
            Self::InventoryUpdated { .. } => 1,
            Self::InventoryLow { .. } => 1,
            Self::PriceUpdated { .. } => 1,
            Self::OrderPlaced { .. } => 1,
            Self::OrderStatusChanged { .. } => 1,
            Self::OrderCompleted { .. } => 1,
            Self::OrderCancelled { .. } => 1,

            // Index events (v1)
            Self::ReindexRequested { .. } => 1,
            Self::IndexUpdated { .. } => 1,

            // Tenant events (v1)
            Self::TenantCreated { .. } => 1,
            Self::TenantUpdated { .. } => 1,
            Self::LocaleEnabled { .. } => 1,
            Self::LocaleDisabled { .. } => 1,
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
