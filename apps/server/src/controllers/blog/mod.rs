pub mod posts;

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/blog")
        .add("/health", super::health::routes())
        .add(
            "/posts",
            get(posts::list_posts).post(posts::create_post),
        )
        .add(
            "/posts/:id",
            get(posts::get_post)
                .put(posts::update_post)
                .delete(posts::delete_post),
        )
}
