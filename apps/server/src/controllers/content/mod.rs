use axum::routing::get;
use loco_rs::controller::Routes;

pub mod nodes;

pub fn routes() -> Routes {
    rustok_content::controllers::routes().add("/health", get(super::health::health))
}
