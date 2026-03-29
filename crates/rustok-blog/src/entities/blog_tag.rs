use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "blog_tags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub use_count: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::blog_tag_translation::Entity")]
    Translations,
    #[sea_orm(has_many = "super::blog_post_tag::Entity")]
    PostTags,
}

impl Related<super::blog_tag_translation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Translations.def()
    }
}

impl Related<super::blog_post_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::PostTags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
