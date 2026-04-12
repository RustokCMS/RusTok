mod shared;

mod m20250130_000016_create_commerce_inventory;
mod m20260411_000001_add_stock_location_translations;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(
        m20250130_000016_create_commerce_inventory::Migration,
    ), Box::new(m20260411_000001_add_stock_location_translations::Migration)]
}
