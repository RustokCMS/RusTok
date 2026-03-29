//! SeaORM entities for forum-owned persistence.

pub mod forum_category;
pub mod forum_category_translation;
pub mod forum_reply;
pub mod forum_reply_body;
pub mod forum_topic;
pub mod forum_topic_channel_access;
pub mod forum_topic_translation;

pub use forum_category::Entity as ForumCategory;
pub use forum_reply::Entity as ForumReply;
pub use forum_topic::Entity as ForumTopic;
