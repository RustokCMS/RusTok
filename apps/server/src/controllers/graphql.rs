use axum::{extract::State, routing::get, Extension, Json};
use loco_rs::prelude::*;

use crate::context::{AuthContext, TenantContext};
use crate::extractors::auth::OptionalCurrentUser;
use crate::graphql::{build_schema, AppSchema};
use rustok_core::ModuleRegistry;

async fn graphql_handler(
    State(ctx): State<AppContext>,
    Extension(schema): Extension<AppSchema>,
    Extension(registry): Extension<ModuleRegistry>,
    tenant_ctx: TenantContext,
    OptionalCurrentUser(current_user): OptionalCurrentUser,
    Json(req): Json<async_graphql::Request>,
) -> Json<async_graphql::Response> {
    let mut request = req.data(ctx).data(tenant_ctx).data(registry);

    if let Some(current_user) = current_user {
        let auth_ctx = AuthContext {
            user_id: current_user.user.id,
            tenant_id: current_user.user.tenant_id,
            role: current_user.user.role.clone(),
            permissions: current_user.permissions,
        };
        request = request.data(auth_ctx);
    }

    Json(schema.execute(request).await)
}

async fn graphql_playground() -> impl axum::response::IntoResponse {
    axum::response::Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/api/graphql"),
    ))
}

pub fn routes() -> Routes {
    let schema = build_schema();

    Routes::new()
        .prefix("api/graphql")
        .add("/", get(graphql_playground).post(graphql_handler))
        .layer(Extension(schema))
}
