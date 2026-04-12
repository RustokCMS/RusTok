mod shared;

mod m20250130_000015_create_commerce_prices;
mod m20260325_000002_add_decimal_runtime_columns;
mod m20260410_000003_add_price_list_rules;
mod m20260410_000004_add_pricing_channel_scope;
mod m20260411_000005_add_price_list_translations;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20250130_000015_create_commerce_prices::Migration),
        Box::new(m20260325_000002_add_decimal_runtime_columns::Migration),
        Box::new(m20260410_000003_add_price_list_rules::Migration),
        Box::new(m20260410_000004_add_pricing_channel_scope::Migration),
        Box::new(m20260411_000005_add_price_list_translations::Migration),
    ]
}
