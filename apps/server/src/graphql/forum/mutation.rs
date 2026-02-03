use async_graphql::{InputObject, Object};
use uuid::Uuid;

#[derive(Default)]
pub struct ForumMutation;

#[derive(InputObject)]
pub struct CreateForumThreadInput {
    pub locale: String,
    pub title: String,
    pub body: String,
}

#[Object]
impl ForumMutation {
    async fn create_forum_thread(&self, _input: CreateForumThreadInput) -> Uuid {
        rustok_core::generate_id()
    }
}
