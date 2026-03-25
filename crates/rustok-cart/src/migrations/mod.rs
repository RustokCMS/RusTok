mod m20260325_000102_create_cart_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(m20260325_000102_create_cart_tables::Migration)]
}
