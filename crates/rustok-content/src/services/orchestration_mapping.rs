use chrono::Utc;
use serde_json::{json, Map, Value};

use crate::dto::{BodyInput, CreateNodeInput, NodeTranslationInput};
use crate::entities::node::{self, ContentStatus};

pub const KIND_TOPIC: &str = "forum_topic";
pub const KIND_POST: &str = "blog_post";

pub struct AuditStamp<'a> {
    pub operation: &'a str,
    pub actor_id: Option<uuid::Uuid>,
    pub reason: Option<&'a str>,
}

pub fn map_topic_to_post_input(
    topic: &node::Model,
    translations: Vec<NodeTranslationInput>,
    bodies: Vec<BodyInput>,
) -> CreateNodeInput {
    let translations = retarget_translations(translations, topic.id);
    CreateNodeInput {
        kind: KIND_POST.to_string(),
        status: map_topic_status_to_post_status(&topic.status, &topic.metadata),
        parent_id: None,
        author_id: topic.author_id,
        category_id: topic.category_id,
        position: Some(topic.position),
        depth: Some(0),
        reply_count: Some(topic.reply_count),
        metadata: map_topic_metadata_to_post_metadata(topic.metadata.clone()),
        translations,
        bodies,
    }
}

pub fn map_post_to_topic_input(
    post: &node::Model,
    translations: Vec<NodeTranslationInput>,
    bodies: Vec<BodyInput>,
) -> CreateNodeInput {
    let translations = retarget_translations(translations, post.id);
    CreateNodeInput {
        kind: KIND_TOPIC.to_string(),
        status: map_post_status_to_topic_status(&post.status, &post.metadata),
        parent_id: None,
        author_id: post.author_id,
        category_id: post.category_id,
        position: Some(post.position),
        depth: Some(0),
        reply_count: Some(post.reply_count),
        metadata: map_post_metadata_to_topic_metadata(post.metadata.clone()),
        translations,
        bodies,
    }
}

pub fn map_topic_status_to_post_status(
    status: &ContentStatus,
    metadata: &Value,
) -> Option<ContentStatus> {
    if metadata
        .get("forum_status")
        .and_then(Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case("archived"))
    {
        return Some(ContentStatus::Archived);
    }

    Some(match status {
        ContentStatus::Draft => ContentStatus::Draft,
        ContentStatus::Published => ContentStatus::Published,
        ContentStatus::Archived => ContentStatus::Archived,
    })
}

pub fn map_post_status_to_topic_status(
    status: &ContentStatus,
    metadata: &Value,
) -> Option<ContentStatus> {
    if metadata
        .get("commenting_enabled")
        .and_then(Value::as_bool)
        .is_some_and(|enabled| !enabled)
    {
        return Some(ContentStatus::Archived);
    }

    Some(match status {
        ContentStatus::Draft => ContentStatus::Draft,
        ContentStatus::Published => ContentStatus::Published,
        ContentStatus::Archived => ContentStatus::Archived,
    })
}

pub fn map_topic_metadata_to_post_metadata(metadata: Value) -> Value {
    let mut mapped = as_object(metadata);
    if let Some(status) = mapped.remove("forum_status") {
        mapped.insert("discussion_status".to_string(), status);
    }
    mapped.insert(
        "origin_kind".to_string(),
        Value::String(KIND_TOPIC.to_string()),
    );
    Value::Object(mapped)
}

pub fn map_post_metadata_to_topic_metadata(metadata: Value) -> Value {
    let mut mapped = as_object(metadata);
    if let Some(status) = mapped.remove("discussion_status") {
        mapped.insert("forum_status".to_string(), status);
    }
    mapped.insert(
        "origin_kind".to_string(),
        Value::String(KIND_POST.to_string()),
    );
    Value::Object(mapped)
}

pub fn stamp_audit_metadata(metadata: &mut Value, stamp: AuditStamp<'_>) {
    let root = as_object(metadata.take());
    let mut root = root;

    let mut audit = root
        .remove("audit")
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();

    audit.push(json!({
        "operation": stamp.operation,
        "actor_id": stamp.actor_id,
        "reason": stamp.reason,
        "at": Utc::now().to_rfc3339(),
    }));

    root.insert("audit".to_string(), Value::Array(audit));
    *metadata = Value::Object(root);
}

fn as_object(value: Value) -> Map<String, Value> {
    value.as_object().cloned().unwrap_or_default()
}

fn retarget_translations(
    translations: Vec<NodeTranslationInput>,
    source_id: uuid::Uuid,
) -> Vec<NodeTranslationInput> {
    let suffix = source_id.simple().to_string();
    let suffix = &suffix[..8];

    translations
        .into_iter()
        .map(|mut tr| {
            if let Some(slug) = tr.slug.as_ref() {
                tr.slug = Some(format!("{slug}-{suffix}"));
                return tr;
            }

            if let Some(title) = tr.title.as_ref() {
                tr.slug = Some(format!("{}-{suffix}", slug::slugify(title)));
            }

            tr
        })
        .collect()
}
