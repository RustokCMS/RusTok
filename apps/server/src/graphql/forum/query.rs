use async_graphql::Object;

use super::{ForumCategory, ForumReply, ForumTopic};

#[derive(Default)]
pub struct ForumQuery;

#[Object]
impl ForumQuery {
    async fn forum_categories(&self) -> Vec<ForumCategory> {
        Vec::new()
    }

    async fn forum_topics(&self) -> Vec<ForumTopic> {
        Vec::new()
    }

    async fn forum_replies(&self) -> Vec<ForumReply> {
        Vec::new()
    }
}
