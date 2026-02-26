use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, StatusCode},
    response::{IntoResponse, Response},
};
use loco_rs::{app::AppContext, controller::Routes, prelude::*, Result};
use rustok_outbox::entity::{Column as SysEventsColumn, Entity as SysEventsEntity, SysEventStatus};
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbBackend, EntityTrait, PaginatorTrait, QueryFilter, Statement,
};

use crate::middleware::tenant::tenant_cache_stats;
use crate::services::auth::AuthService;

pub async fn metrics(State(ctx): State<AppContext>) -> Result<Response> {
    match rustok_telemetry::metrics_handle() {
        Some(handle) => {
            let mut payload = handle.render();
            payload.push('\n');
            payload.push_str(&render_tenant_cache_metrics(&ctx).await);
            payload.push_str(&render_outbox_metrics(&ctx).await);
            payload.push_str(&render_rbac_metrics());

            Ok((
                StatusCode::OK,
                [(CONTENT_TYPE, "text/plain; version=0.0.4; charset=utf-8")],
                payload,
            )
                .into_response())
        }
        None => Ok((StatusCode::SERVICE_UNAVAILABLE, "metrics disabled").into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new().prefix("metrics").add("/", get(metrics))
}

async fn render_tenant_cache_metrics(ctx: &AppContext) -> String {
    let stats = tenant_cache_stats(ctx).await;
    format!(
        "rustok_tenant_cache_hits {hits}\n\
rustok_tenant_cache_misses {misses}\n\
rustok_tenant_cache_evictions {evictions}\n\
rustok_tenant_cache_entries {entries}\n\
rustok_tenant_cache_negative_hits {negative_hits}\n\
rustok_tenant_cache_negative_misses {negative_misses}\n\
rustok_tenant_cache_negative_evictions {negative_evictions}\n\
rustok_tenant_cache_negative_entries {negative_entries}\n\
rustok_tenant_cache_negative_inserts {negative_inserts}\n",
        hits = stats.hits,
        misses = stats.misses,
        evictions = stats.evictions,
        entries = stats.entries,
        negative_hits = stats.negative_hits,
        negative_misses = stats.negative_misses,
        negative_evictions = stats.negative_evictions,
        negative_entries = stats.negative_entries,
        negative_inserts = stats.negative_inserts,
    )
}

async fn render_outbox_metrics(ctx: &AppContext) -> String {
    let backlog_size = SysEventsEntity::find()
        .filter(SysEventsColumn::Status.eq(SysEventStatus::Pending))
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let dlq_total = SysEventsEntity::find()
        .filter(SysEventsColumn::Status.eq(SysEventStatus::Failed))
        .count(&ctx.db)
        .await
        .unwrap_or(0);

    let retries_total = ctx
        .db
        .query_one(Statement::from_string(
            DbBackend::Postgres,
            "SELECT COALESCE(SUM(retry_count), 0) AS total FROM sys_events".to_string(),
        ))
        .await
        .ok()
        .flatten()
        .and_then(|row| row.try_get::<i64>("", "total").ok())
        .unwrap_or(0);

    format!(
        "outbox_backlog_size {backlog_size}\n\
outbox_dlq_total {dlq_total}\n\
outbox_retries_total {retries_total}\n",
    )
}

fn render_rbac_metrics() -> String {
    let stats = AuthService::metrics_snapshot();
    format!(
        "rustok_rbac_permission_cache_hits {cache_hits}\n\
rustok_rbac_permission_cache_misses {cache_misses}\n\
rustok_rbac_permission_checks_allowed {checks_allowed}\n\
rustok_rbac_permission_checks_denied {checks_denied}\n\
rustok_rbac_permission_check_latency_ms_total {check_latency_ms_total}\n\
rustok_rbac_permission_check_latency_samples {check_latency_samples}\n\
rustok_rbac_permission_lookup_latency_ms_total {lookup_latency_ms_total}\n\
rustok_rbac_permission_lookup_latency_samples {lookup_latency_samples}\n\
rustok_rbac_permission_denied_reason_no_permissions_resolved {denied_no_permissions_resolved}\n\
rustok_rbac_permission_denied_reason_missing_permissions {denied_missing_permissions}\n\
rustok_rbac_permission_denied_reason_unknown {denied_unknown}\n",
        cache_hits = stats.permission_cache_hits,
        cache_misses = stats.permission_cache_misses,
        checks_allowed = stats.permission_checks_allowed,
        checks_denied = stats.permission_checks_denied,
        check_latency_ms_total = stats.permission_check_latency_ms_total,
        check_latency_samples = stats.permission_check_latency_samples,
        lookup_latency_ms_total = stats.permission_lookup_latency_ms_total,
        lookup_latency_samples = stats.permission_lookup_latency_samples,
        denied_no_permissions_resolved = stats.denied_no_permissions_resolved,
        denied_missing_permissions = stats.denied_missing_permissions,
        denied_unknown = stats.denied_unknown,
    )
}
