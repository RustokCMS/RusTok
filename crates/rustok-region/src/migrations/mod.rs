mod m20250130_000014_create_regions;
mod m20260411_000001_add_region_translations;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20250130_000014_create_regions::Migration),
        Box::new(m20260411_000001_add_region_translations::Migration),
    ]
}
