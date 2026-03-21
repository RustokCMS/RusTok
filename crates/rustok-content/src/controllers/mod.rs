use axum::routing::get;
use loco_rs::controller::Routes;

pub mod nodes;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/content")
        .add("/nodes", get(nodes::list_nodes).post(nodes::create_node))
        .add(
            "/nodes/{id}",
            get(nodes::get_node)
                .put(nodes::update_node)
                .delete(nodes::delete_node),
        )
}
