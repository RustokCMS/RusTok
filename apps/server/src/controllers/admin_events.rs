use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use loco_rs::prelude::*;
use rustok_outbox::entity::{self, SysEventStatus};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::auth::CurrentUser;

#[derive(Debug, Deserialize)]
pub struct DlqQuery {
    pub tenant_id: Option<Uuid>,
    pub event_type: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    #[serde(default = "default_limit")]
    pub limit: u64,
}

#[derive(Debug, Serialize)]
pub struct DlqEventItem {
    pub id: Uuid,
    pub event_type: String,
    pub schema_version: i16,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct DlqListResponse {
    pub total: usize,
    pub items: Vec<DlqEventItem>,
}

#[derive(Debug, Serialize)]
pub struct DlqReplayResponse {
    pub id: Uuid,
    pub status: &'static str,
}

pub async fn list_dlq(
    State(ctx): State<AppContext>,
    _user: CurrentUser,
    Query(query): Query<DlqQuery>,
) -> Result<Json<DlqListResponse>> {
    let limit = query.limit.clamp(1, 200);

    let mut db_query = entity::Entity::find()
        .filter(entity::Column::Status.eq(SysEventStatus::Failed))
        .order_by_desc(entity::Column::CreatedAt)
        .limit(limit);

    if let Some(event_type) = query.event_type.as_ref() {
        db_query = db_query.filter(entity::Column::EventType.eq(event_type.as_str()));
    }

    if let Some(created_after) = query.created_after {
        db_query = db_query.filter(entity::Column::CreatedAt.gte(created_after));
    }

    let models = db_query
        .all(&ctx.db)
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to load DLQ events: {e}")))?;

    let items = models
        .into_iter()
        .filter_map(|model| {
            if let Some(tenant_id) = query.tenant_id {
                let payload_tenant = model
                    .payload
                    .get("tenant_id")
                    .or_else(|| model.payload.get("event").and_then(|e| e.get("tenant_id")))
                    .and_then(|v| v.as_str())
                    .and_then(|raw| Uuid::parse_str(raw).ok());
                if payload_tenant != Some(tenant_id) {
                    return None;
                }
            }

            Some(DlqEventItem {
                id: model.id,
                event_type: model.event_type,
                schema_version: model.schema_version,
                retry_count: model.retry_count,
                last_error: model.last_error,
                created_at: model.created_at,
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(DlqListResponse {
        total: items.len(),
        items,
    }))
}

pub async fn replay_dlq_event(
    State(ctx): State<AppContext>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<DlqReplayResponse>> {
    let model = entity::Entity::find_by_id(id)
        .one(&ctx.db)
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to fetch sys_event: {e}")))?
        .ok_or_else(|| Error::NotFound)?;

    if model.status != SysEventStatus::Failed {
        return Err(Error::BadRequest(
            "Only failed (DLQ) events can be replayed".to_string(),
        ));
    }

    let mut active: entity::ActiveModel = model.into();
    active.status = Set(SysEventStatus::Pending);
    active.retry_count = Set(0);
    active.next_attempt_at = Set(None);
    active.last_error = Set(None);
    active.claimed_by = Set(None);
    active.claimed_at = Set(None);
    active.dispatched_at = Set(None);

    active
        .update(&ctx.db)
        .await
        .map_err(|e| Error::BadRequest(format!("Failed to replay sys_event: {e}")))?;

    Ok(Json(DlqReplayResponse {
        id,
        status: "requeued",
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/admin/events")
        .add("/dlq", axum::routing::get(list_dlq))
        .add("/dlq/{id}/replay", axum::routing::post(replay_dlq_event))
}

fn default_limit() -> u64 {
    100
}
