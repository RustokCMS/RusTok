use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexFlexEntryModel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub schema_id: Uuid,
    pub schema_slug: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub status: String,
    pub data_preview: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
