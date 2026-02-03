use async_graphql::SimpleObject;
use uuid::Uuid;

#[derive(Clone, Debug, SimpleObject)]
pub struct ForumThread {
    pub id: Uuid,
    pub locale: String,
    pub title: String,
}
