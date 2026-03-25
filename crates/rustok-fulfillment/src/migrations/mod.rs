mod m20260325_000105_create_fulfillment_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(
        m20260325_000105_create_fulfillment_tables::Migration,
    )]
}
