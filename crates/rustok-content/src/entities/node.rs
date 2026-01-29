use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "nodes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub kind: String,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub position: i32,
    pub depth: i32,
    pub reply_count: i32,
    pub metadata: Json,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub published_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::node_translation::Entity")]
    Translations,
    #[sea_orm(has_many = "super::body::Entity")]
    Bodies,
    #[sea_orm(belongs_to = "Entity", from = "Column::ParentId", to = "Column::Id")]
    Parent,
    #[sea_orm(has_many = "Entity")]
    Children,
}

impl Related<super::node_translation::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Translations.def()
    }
}

impl Related<super::body::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bodies.def()
    }
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Children.def()
    }
}

pub struct SelfReferencing;

impl Linked for SelfReferencing {
    type FromEntity = Entity;
    type ToEntity = Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![Relation::Children.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
