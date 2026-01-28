use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use axum::{routing::get, Extension, Json};
use loco_rs::prelude::*;

#[derive(Default)]
pub struct Query;

#[Object]
impl Query {
    async fn health(&self) -> &str {
        "GraphQL is working!"
    }

    async fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
}

pub type AppSchema = Schema<Query, EmptyMutation, EmptySubscription>;

fn build_schema() -> AppSchema {
    Schema::build(Query, EmptyMutation, EmptySubscription).finish()
}

async fn graphql_handler(
    Extension(schema): Extension<AppSchema>,
    Json(req): Json<async_graphql::Request>,
) -> Json<async_graphql::Response> {
    Json(schema.execute(req).await)
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
