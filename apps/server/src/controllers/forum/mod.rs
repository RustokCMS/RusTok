use loco_rs::prelude::*;

pub mod threads;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/forum")
        .add("/health", get(super::health::health))
        .add("/threads", get(threads::list_threads).post(threads::create_thread))
        .add(
            "/threads/:id",
            get(threads::get_thread)
                .put(threads::update_thread)
                .delete(threads::delete_thread),
        )
}
