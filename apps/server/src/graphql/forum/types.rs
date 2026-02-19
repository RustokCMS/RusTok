use async_graphql::SimpleObject;
use uuid::Uuid;

#[derive(Clone, Debug, SimpleObject)]
pub struct ForumCategory {
    pub id: Uuid,
    pub locale: String,
    pub name: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ForumTopic {
    pub id: Uuid,
    pub locale: String,
    pub title: String,
}

#[derive(Clone, Debug, SimpleObject)]
pub struct ForumReply {
    pub id: Uuid,
    pub locale: String,
    pub content: String,
}
