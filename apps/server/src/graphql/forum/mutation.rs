use async_graphql::{InputObject, Object};
use uuid::Uuid;

#[derive(Default)]
pub struct ForumMutation;

#[derive(InputObject)]
pub struct CreateForumTopicInput {
    pub locale: String,
    pub title: String,
    pub body: String,
}

#[Object]
impl ForumMutation {
    async fn create_forum_topic(&self, _input: CreateForumTopicInput) -> Uuid {
        rustok_core::generate_id()
    }
}
