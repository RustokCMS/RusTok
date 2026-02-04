use serde::Serialize;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct FieldSchema {
    pub name: &'static str,
    pub data_type: &'static str,
    pub optional: bool,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct EventSchema {
    pub event_type: &'static str,
    pub version: u16,
    pub description: &'static str,
    pub fields: &'static [FieldSchema],
}

impl EventSchema {
    pub fn to_json_schema(&self) -> serde_json::Value {
        let properties: serde_json::Map<String, serde_json::Value> = self
            .fields
            .iter()
            .map(|field| {
                let mut schema = serde_json::json!({ "type": field.data_type });
                if field.optional {
                    schema["nullable"] = serde_json::Value::Bool(true);
                }
                (field.name.to_string(), schema)
            })
            .collect();

        let required: Vec<&str> = self
            .fields
            .iter()
            .filter(|field| !field.optional)
            .map(|field| field.name)
            .collect();

        serde_json::json!({
            "title": self.event_type,
            "type": "object",
            "description": self.description,
            "properties": properties,
            "required": required,
            "version": self.version,
        })
    }
}

macro_rules! field {
    ($name:literal, $data_type:literal) => {
        FieldSchema {
            name: $name,
            data_type: $data_type,
            optional: false,
        }
    };
    ($name:literal, $data_type:literal, optional) => {
        FieldSchema {
            name: $name,
            data_type: $data_type,
            optional: true,
        }
    };
}

const NODE_CREATED_FIELDS: &[FieldSchema] = &[
    field!("node_id", "uuid"),
    field!("kind", "string"),
    field!("author_id", "uuid", optional),
];
const NODE_UPDATED_FIELDS: &[FieldSchema] = &[field!("node_id", "uuid"), field!("kind", "string")];
const NODE_TRANSLATION_UPDATED_FIELDS: &[FieldSchema] =
    &[field!("node_id", "uuid"), field!("locale", "string")];
const NODE_PUBLISHED_FIELDS: &[FieldSchema] =
    &[field!("node_id", "uuid"), field!("kind", "string")];
const NODE_UNPUBLISHED_FIELDS: &[FieldSchema] =
    &[field!("node_id", "uuid"), field!("kind", "string")];
const NODE_DELETED_FIELDS: &[FieldSchema] = &[field!("node_id", "uuid"), field!("kind", "string")];
const BODY_UPDATED_FIELDS: &[FieldSchema] =
    &[field!("node_id", "uuid"), field!("locale", "string")];

const CATEGORY_ID_FIELDS: &[FieldSchema] = &[field!("category_id", "uuid")];
const TAG_ID_FIELDS: &[FieldSchema] = &[field!("tag_id", "uuid")];
const TAG_RELATION_FIELDS: &[FieldSchema] = &[
    field!("tag_id", "uuid"),
    field!("target_type", "string"),
    field!("target_id", "uuid"),
];

const MEDIA_UPLOADED_FIELDS: &[FieldSchema] = &[
    field!("media_id", "uuid"),
    field!("mime_type", "string"),
    field!("size", "int64"),
];
const MEDIA_DELETED_FIELDS: &[FieldSchema] = &[field!("media_id", "uuid")];

const USER_REGISTERED_FIELDS: &[FieldSchema] =
    &[field!("user_id", "uuid"), field!("email", "string")];
const USER_LOGGED_IN_FIELDS: &[FieldSchema] = &[field!("user_id", "uuid")];
const USER_UPDATED_FIELDS: &[FieldSchema] = &[field!("user_id", "uuid")];
const USER_DELETED_FIELDS: &[FieldSchema] = &[field!("user_id", "uuid")];

const PRODUCT_ID_FIELDS: &[FieldSchema] = &[field!("product_id", "uuid")];
const VARIANT_FIELDS: &[FieldSchema] =
    &[field!("variant_id", "uuid"), field!("product_id", "uuid")];
const INVENTORY_UPDATED_FIELDS: &[FieldSchema] = &[
    field!("variant_id", "uuid"),
    field!("product_id", "uuid"),
    field!("location_id", "uuid"),
    field!("old_quantity", "int32"),
    field!("new_quantity", "int32"),
];
const INVENTORY_LOW_FIELDS: &[FieldSchema] = &[
    field!("variant_id", "uuid"),
    field!("product_id", "uuid"),
    field!("remaining", "int32"),
    field!("threshold", "int32"),
];
const PRICE_UPDATED_FIELDS: &[FieldSchema] = &[
    field!("variant_id", "uuid"),
    field!("product_id", "uuid"),
    field!("currency", "string"),
    field!("old_amount", "int64", optional),
    field!("new_amount", "int64"),
];
const ORDER_PLACED_FIELDS: &[FieldSchema] = &[
    field!("order_id", "uuid"),
    field!("customer_id", "uuid", optional),
    field!("total", "int64"),
    field!("currency", "string"),
];
const ORDER_STATUS_CHANGED_FIELDS: &[FieldSchema] = &[
    field!("order_id", "uuid"),
    field!("old_status", "string"),
    field!("new_status", "string"),
];
const ORDER_COMPLETED_FIELDS: &[FieldSchema] = &[field!("order_id", "uuid")];
const ORDER_CANCELLED_FIELDS: &[FieldSchema] = &[
    field!("order_id", "uuid"),
    field!("reason", "string", optional),
];

const REINDEX_REQUESTED_FIELDS: &[FieldSchema] = &[
    field!("target_type", "string"),
    field!("target_id", "uuid", optional),
];
const INDEX_UPDATED_FIELDS: &[FieldSchema] =
    &[field!("index_name", "string"), field!("target_id", "uuid")];

const TENANT_ID_FIELDS: &[FieldSchema] = &[field!("tenant_id", "uuid")];
const LOCALE_FIELDS: &[FieldSchema] = &[field!("tenant_id", "uuid"), field!("locale", "string")];

pub const EVENT_SCHEMAS: &[EventSchema] = &[
    EventSchema {
        event_type: "node.created",
        version: 1,
        description: "A content node was created.",
        fields: NODE_CREATED_FIELDS,
    },
    EventSchema {
        event_type: "node.updated",
        version: 1,
        description: "A content node was updated.",
        fields: NODE_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "node.translation.updated",
        version: 1,
        description: "A node translation was updated.",
        fields: NODE_TRANSLATION_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "node.published",
        version: 1,
        description: "A content node was published.",
        fields: NODE_PUBLISHED_FIELDS,
    },
    EventSchema {
        event_type: "node.unpublished",
        version: 1,
        description: "A content node was unpublished.",
        fields: NODE_UNPUBLISHED_FIELDS,
    },
    EventSchema {
        event_type: "node.deleted",
        version: 1,
        description: "A content node was deleted.",
        fields: NODE_DELETED_FIELDS,
    },
    EventSchema {
        event_type: "body.updated",
        version: 1,
        description: "A node body was updated.",
        fields: BODY_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "category.created",
        version: 1,
        description: "A category was created.",
        fields: CATEGORY_ID_FIELDS,
    },
    EventSchema {
        event_type: "category.updated",
        version: 1,
        description: "A category was updated.",
        fields: CATEGORY_ID_FIELDS,
    },
    EventSchema {
        event_type: "category.deleted",
        version: 1,
        description: "A category was deleted.",
        fields: CATEGORY_ID_FIELDS,
    },
    EventSchema {
        event_type: "tag.created",
        version: 1,
        description: "A tag was created.",
        fields: TAG_ID_FIELDS,
    },
    EventSchema {
        event_type: "tag.attached",
        version: 1,
        description: "A tag was attached to a target.",
        fields: TAG_RELATION_FIELDS,
    },
    EventSchema {
        event_type: "tag.detached",
        version: 1,
        description: "A tag was detached from a target.",
        fields: TAG_RELATION_FIELDS,
    },
    EventSchema {
        event_type: "media.uploaded",
        version: 1,
        description: "Media asset uploaded.",
        fields: MEDIA_UPLOADED_FIELDS,
    },
    EventSchema {
        event_type: "media.deleted",
        version: 1,
        description: "Media asset deleted.",
        fields: MEDIA_DELETED_FIELDS,
    },
    EventSchema {
        event_type: "user.registered",
        version: 1,
        description: "A user registered.",
        fields: USER_REGISTERED_FIELDS,
    },
    EventSchema {
        event_type: "user.logged_in",
        version: 1,
        description: "A user logged in.",
        fields: USER_LOGGED_IN_FIELDS,
    },
    EventSchema {
        event_type: "user.updated",
        version: 1,
        description: "A user profile was updated.",
        fields: USER_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "user.deleted",
        version: 1,
        description: "A user was deleted.",
        fields: USER_DELETED_FIELDS,
    },
    EventSchema {
        event_type: "product.created",
        version: 1,
        description: "A product was created.",
        fields: PRODUCT_ID_FIELDS,
    },
    EventSchema {
        event_type: "product.updated",
        version: 1,
        description: "A product was updated.",
        fields: PRODUCT_ID_FIELDS,
    },
    EventSchema {
        event_type: "product.published",
        version: 1,
        description: "A product was published.",
        fields: PRODUCT_ID_FIELDS,
    },
    EventSchema {
        event_type: "product.deleted",
        version: 1,
        description: "A product was deleted.",
        fields: PRODUCT_ID_FIELDS,
    },
    EventSchema {
        event_type: "variant.created",
        version: 1,
        description: "A product variant was created.",
        fields: VARIANT_FIELDS,
    },
    EventSchema {
        event_type: "variant.updated",
        version: 1,
        description: "A product variant was updated.",
        fields: VARIANT_FIELDS,
    },
    EventSchema {
        event_type: "variant.deleted",
        version: 1,
        description: "A product variant was deleted.",
        fields: VARIANT_FIELDS,
    },
    EventSchema {
        event_type: "inventory.updated",
        version: 1,
        description: "Inventory was updated.",
        fields: INVENTORY_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "inventory.low",
        version: 1,
        description: "Inventory low threshold reached.",
        fields: INVENTORY_LOW_FIELDS,
    },
    EventSchema {
        event_type: "price.updated",
        version: 1,
        description: "Price was updated.",
        fields: PRICE_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "order.placed",
        version: 1,
        description: "Order was placed.",
        fields: ORDER_PLACED_FIELDS,
    },
    EventSchema {
        event_type: "order.status_changed",
        version: 1,
        description: "Order status changed.",
        fields: ORDER_STATUS_CHANGED_FIELDS,
    },
    EventSchema {
        event_type: "order.completed",
        version: 1,
        description: "Order completed.",
        fields: ORDER_COMPLETED_FIELDS,
    },
    EventSchema {
        event_type: "order.cancelled",
        version: 1,
        description: "Order cancelled.",
        fields: ORDER_CANCELLED_FIELDS,
    },
    EventSchema {
        event_type: "index.reindex_requested",
        version: 1,
        description: "Index rebuild requested.",
        fields: REINDEX_REQUESTED_FIELDS,
    },
    EventSchema {
        event_type: "index.updated",
        version: 1,
        description: "Index entry updated.",
        fields: INDEX_UPDATED_FIELDS,
    },
    EventSchema {
        event_type: "tenant.created",
        version: 1,
        description: "Tenant created.",
        fields: TENANT_ID_FIELDS,
    },
    EventSchema {
        event_type: "tenant.updated",
        version: 1,
        description: "Tenant updated.",
        fields: TENANT_ID_FIELDS,
    },
    EventSchema {
        event_type: "locale.enabled",
        version: 1,
        description: "Locale enabled for tenant.",
        fields: LOCALE_FIELDS,
    },
    EventSchema {
        event_type: "locale.disabled",
        version: 1,
        description: "Locale disabled for tenant.",
        fields: LOCALE_FIELDS,
    },
];

pub fn event_schema(event_type: &str) -> Option<&'static EventSchema> {
    EVENT_SCHEMAS
        .iter()
        .find(|schema| schema.event_type == event_type)
}
