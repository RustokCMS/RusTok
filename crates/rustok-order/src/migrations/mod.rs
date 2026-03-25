mod m20260325_000101_create_order_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(m20260325_000101_create_order_tables::Migration)]
}
