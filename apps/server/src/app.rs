use async_trait::async_trait;
use axum::middleware as axum_middleware;
use axum::Extension;
use axum::Router as AxumRouter;
use loco_rs::{
    app::{AppContext, Hooks, Initializer},
    boot::{create_app, BootResult, StartMode},
    controller::AppRoutes,
    environment::Environment,
    task::Tasks,
    Result,
};
use sea_orm::DatabaseConnection;
use std::path::Path;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::controllers;
use crate::middleware;
use crate::modules;
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

    async fn boot(mode: StartMode, environment: &Environment) -> Result<BootResult> {
        create_app::<Self, Migrator>(mode, environment).await
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
        let registry = modules::build_registry();
        let engine = Arc::new(alloy_scripting::create_default_engine());
        let storage = Arc::new(alloy_scripting::SeaOrmStorage::new(ctx.db.clone()));
        let orchestrator = Arc::new(alloy_scripting::ScriptOrchestrator::new(
            engine.clone(),
            storage.clone(),
        ));
        let alloy_state = crate::graphql::alloy::AlloyState::new(engine, storage, orchestrator);

        Ok(router
            .merge(SwaggerUi::new("/swagger").url(
                "/api-docs/openapi.json",
                controllers::swagger::ApiDoc::openapi(),
            ))
            .layer(Extension(registry))
            .layer(Extension(alloy_state))
            .layer(axum_middleware::from_fn_with_state(
                ctx.clone(),
                middleware::tenant::resolve,
            )))
    }

    async fn truncate(_db: &DatabaseConnection) -> Result<()> {
        Ok(())
    }

    fn register_tasks(_tasks: &mut Tasks) {}

    async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
        Ok(vec![])
    }

    async fn connect_workers(_ctx: &AppContext, _queue: &Queue) -> Result<()> {
        Ok(())
    }

    async fn seed(_db: &DatabaseConnection, _path: &Path) -> Result<()> {
        Ok(())
    }
}
