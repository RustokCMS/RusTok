mod m20260316_000004_create_topic_field_definitions;
mod m20260328_000001_create_forum_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260316_000004_create_topic_field_definitions::Migration),
        Box::new(m20260328_000001_create_forum_tables::Migration),
    ]
}
