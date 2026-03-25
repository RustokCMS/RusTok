mod m20260324_000001_create_search_settings;
mod m20260324_000002_create_search_documents;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260324_000001_create_search_settings::Migration),
        Box::new(m20260324_000002_create_search_documents::Migration),
    ]
}
