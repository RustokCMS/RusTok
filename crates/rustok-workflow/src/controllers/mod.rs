use axum::routing::{get, post, put};
use loco_rs::controller::Routes;

pub mod executions;
pub mod steps;
pub mod webhook;
pub mod workflows;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/workflows")
        .add("/", get(workflows::list).post(workflows::create))
        .add(
            "/{id}",
            get(workflows::get)
                .put(workflows::update)
                .delete(workflows::delete_workflow),
        )
        .add("/{id}/activate", post(workflows::activate))
        .add("/{id}/pause", post(workflows::pause))
        .add("/{id}/trigger", post(workflows::trigger_manual))
        .add("/{id}/steps", post(steps::add_step))
        .add(
            "/{id}/steps/{step_id}",
            put(steps::update_step).delete(steps::delete_step),
        )
        .add("/{id}/executions", get(executions::list_executions))
        .add("/executions/{execution_id}", get(executions::get_execution))
}

pub fn webhook_routes() -> Routes {
    Routes::new()
        .prefix("webhooks")
        .add("/{tenant_slug}/{webhook_slug}", post(webhook::receive))
}
