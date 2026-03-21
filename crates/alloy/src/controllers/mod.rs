use std::sync::Arc;

use alloy_scripting::{create_router, AppState, SeaOrmStorage};
use axum::Router;

pub type AlloyAppState = AppState<SeaOrmStorage>;

pub fn router(state: Arc<AlloyAppState>) -> Router {
    create_router(state)
}
