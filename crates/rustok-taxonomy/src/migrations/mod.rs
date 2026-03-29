mod m20260329_000001_create_taxonomy_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(m20260329_000001_create_taxonomy_tables::Migration)]
}
