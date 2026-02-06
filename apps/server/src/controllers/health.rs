//! Health check endpoints for K8s probes and module health aggregation

use axum::routing::get;
use axum::Extension;
use loco_rs::prelude::*;
use rustok_core::{HealthStatus, ModuleRegistry};
use sea_orm::DatabaseConnection;
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    app: &'static str,
    version: &'static str,
}

#[derive(Serialize)]
struct ReadinessResponse {
    status: &'static str,
    database: &'static str,
}

#[derive(Serialize)]
struct ModuleHealth {
    slug: String,
    name: String,
    status: String,
}

#[derive(Serialize)]
struct ModulesHealthResponse {
    status: &'static str,
    modules: Vec<ModuleHealth>,
}

/// GET /health - Basic health check
pub async fn health() -> Result<Response> {
    format::json(HealthResponse {
        status: "ok",
        app: "rustok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// GET /health/live - K8s liveness probe
/// Always returns 200 if the process is running
pub async fn live() -> Result<Response> {
    format::json(serde_json::json!({ "status": "ok" }))
}

/// GET /health/ready - K8s readiness probe
/// Checks database connectivity
pub async fn ready(State(ctx): State<AppContext>) -> Result<Response> {
    let db_status = check_database(&ctx.db).await;
    
    let status = if db_status == "connected" { "ok" } else { "degraded" };
    
    format::json(ReadinessResponse {
        status,
        database: db_status,
    })
}

/// GET /health/modules - Module health aggregation
/// Reports health status of all registered modules
pub async fn modules(Extension(registry): Extension<ModuleRegistry>) -> Result<Response> {
    let mut modules_health = Vec::new();
    let mut overall_healthy = true;

    for module in registry.modules() {
        let health = module.health().await;
        let status_str = match health {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => {
                overall_healthy = false;
                "degraded"
            }
            HealthStatus::Unhealthy => {
                overall_healthy = false;
                "unhealthy"
            }
        };

        modules_health.push(ModuleHealth {
            slug: module.slug().to_string(),
            name: module.name().to_string(),
            status: status_str.to_string(),
        });
    }

    format::json(ModulesHealthResponse {
        status: if overall_healthy { "ok" } else { "degraded" },
        modules: modules_health,
    })
}

async fn check_database(db: &DatabaseConnection) -> &'static str {
    use sea_orm::ConnectionTrait;
    
    match db.execute_unprepared("SELECT 1").await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("health")
        .add("/", get(health))
        .add("/live", get(live))
        .add("/ready", get(ready))
        .add("/modules", get(modules))
}
