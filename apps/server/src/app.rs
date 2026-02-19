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

use sea_orm::EntityTrait;

use crate::controllers;
use crate::initializers;
use crate::middleware;
use crate::modules;
use crate::seeds;
use crate::services::event_transport_factory::{build_event_runtime, spawn_outbox_relay_worker};
use crate::tasks;
use loco_rs::prelude::Queue;
use migration::Migrator;
use std::sync::Arc;

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

    async fn truncate(ctx: &AppContext) -> Result<()> {
        tracing::info!("Truncating database...");

        // Delete in dependency order to satisfy foreign key constraints.
        let releases = crate::models::release::Entity::delete_many()
            .exec(&ctx.db)
            .await?;
        let builds = crate::models::build::Entity::delete_many()
            .exec(&ctx.db)
            .await?;
        let tenant_modules = crate::models::_entities::tenant_modules::Entity::delete_many()
            .exec(&ctx.db)
            .await?;
        let sessions = crate::models::sessions::Entity::delete_many()
            .exec(&ctx.db)
            .await?;
        let users = crate::models::users::Entity::delete_many()
            .exec(&ctx.db)
            .await?;
        let tenants = crate::models::tenants::Entity::delete_many()
            .exec(&ctx.db)
            .await?;

        tracing::info!(
            releases = releases.rows_affected,
            builds = builds.rows_affected,
            tenant_modules = tenant_modules.rows_affected,
            sessions = sessions.rows_affected,
            users = users.rows_affected,
            tenants = tenants.rows_affected,
            "Database truncation complete"
        );
        Ok(())
    }

    fn register_tasks(tasks: &mut Tasks) {
        tasks::register(tasks);
    }

    async fn initializers(ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        initializers::create(ctx).await
    }

    async fn connect_workers(ctx: &AppContext, _queue: &Queue) -> Result<()> {
        let event_runtime = build_event_runtime(ctx).await?;
        if let Some(relay_config) = event_runtime.relay_config {
            let _handle = spawn_outbox_relay_worker(relay_config);
        }

        Ok(())
    }

    async fn seed(ctx: &AppContext, path: &Path) -> Result<()> {
        seeds::seed(ctx, path).await
    }
}
