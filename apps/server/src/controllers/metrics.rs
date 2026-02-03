use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use loco_rs::prelude::*;

pub async fn metrics() -> Result<Response> {
    match rustok_telemetry::metrics_handle() {
        Some(handle) => Ok((
            [(header::CONTENT_TYPE, "text/plain; version=0.0.4")],
            handle.render(),
        )
            .into_response()),
        None => Ok((StatusCode::SERVICE_UNAVAILABLE, "metrics disabled").into_response()),
    }
}

pub fn routes() -> Routes {
    Routes::new().prefix("metrics").add("/", get(metrics))
}
