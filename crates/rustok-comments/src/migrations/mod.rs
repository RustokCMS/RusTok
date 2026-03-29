mod m20260328_000001_create_comments_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(m20260328_000001_create_comments_tables::Migration)]
}
