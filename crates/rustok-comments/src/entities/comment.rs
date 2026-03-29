use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::dto::CommentStatus;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub thread_id: Uuid,
    pub author_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub status: CommentStatus,
    pub position: i64,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::comment_thread::Entity",
        from = "Column::ThreadId",
        to = "super::comment_thread::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Thread,
    #[sea_orm(has_many = "super::comment_body::Entity")]
    Bodies,
}

impl Related<super::comment_thread::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Thread.def()
    }
}

impl Related<super::comment_body::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bodies.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
