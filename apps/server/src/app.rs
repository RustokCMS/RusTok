use async_trait::async_trait;
use axum::middleware as axum_middleware;
use axum::Extension;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    config::Config,
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use std::path::Path;

use crate::controllers;
use crate::middleware;
use crate::modules;
use crate::services::event_transport_factory::{build_event_runtime, spawn_outbox_relay_worker};
use loco_rs::prelude::Queue;
use migration::Migrator;
use std::sync::Arc;
use tokio::task::JoinHandle;

struct OutboxRelayWorkerHandle {
    _handle: JoinHandle<()>,
}

pub struct App;

#[async_trait]
impl Hooks for App {
    fn app_name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn app_version() -> String {
        format!(
            "{} ({})",
            env!("CARGO_PKG_VERSION"),
            option_env!("BUILD_SHA")
                .or(option_env!("GITHUB_SHA"))
                .unwrap_or("dev")
        )
    }

    async fn boot(
        mode: StartMode,
        environment: &Environment,
        config: Config,
    ) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment, config).await
    }

    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(controllers::health::routes())
            .add_route(controllers::metrics::routes())
            .add_route(controllers::auth::routes())
            .add_route(controllers::graphql::routes())
            .add_route(controllers::commerce::routes())
            .add_route(controllers::content::routes())
            .add_route(controllers::blog::routes())
            .add_route(controllers::forum::routes())
            .add_route(controllers::pages::routes())
    }

    async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
        let event_runtime = build_event_runtime(ctx).await?;
        ctx.shared_store.insert(event_runtime.transport.clone());
        let registry = modules::build_registry();
        middleware::tenant::init_tenant_cache_infrastructure(ctx).await;
        let engine = Arc::new(alloy_scripting::create_default_engine());
        let storage = Arc::new(alloy_scripting::SeaOrmStorage::new(ctx.db.clone()));
        let orchestrator = Arc::new(alloy_scripting::ScriptOrchestrator::new(
            engine.clone(),
            storage.clone(),
        ));
        let alloy_state = crate::graphql::alloy::AlloyState::new(engine, storage, orchestrator);

        Ok(router
            .layer(Extension(registry))
            .layer(Extension(alloy_state))
            .layer(axum_middleware::from_fn_with_state(
                ctx.clone(),
                middleware::tenant::resolve,
            )))
    }

    async fn truncate(_ctx: &AppContext) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![])
    }

    async fn connect_workers(ctx: &AppContext, _queue: &Queue) -> Result<()> {
        if ctx.shared_store.contains::<OutboxRelayWorkerHandle>() {
            return Ok(());
        }

        let event_runtime = build_event_runtime(ctx).await?;
        if let Some(relay_config) = event_runtime.relay_config {
            let handle = spawn_outbox_relay_worker(relay_config);
            ctx.shared_store
                .insert(OutboxRelayWorkerHandle { _handle: handle });
        }

        Ok(())
    }

    async fn seed(_ctx: &AppContext, _path: &Path) -> Result<()> {
        Ok(())
    }
}
