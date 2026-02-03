use async_graphql::Object;

use super::ForumThread;

#[derive(Default)]
pub struct ForumQuery;

#[Object]
impl ForumQuery {
    async fn forum_threads(&self) -> Vec<ForumThread> {
        Vec::new()
    }
}
