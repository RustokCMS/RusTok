mod m20260325_000101_create_order_tables;
mod m20260402_000102_add_order_channel_columns;
mod m20260405_000103_add_order_line_item_shipping_profiles;
mod m20260409_000104_add_order_line_item_seller_id;
mod m20260410_000105_create_order_adjustments;
mod m20260411_000106_add_order_tax_lines;
mod m20260411_000107_add_order_line_item_translations;
mod m20260412_000108_add_order_shipping_total;
mod m20260412_000109_add_order_tax_line_provider_id;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260325_000101_create_order_tables::Migration),
        Box::new(m20260402_000102_add_order_channel_columns::Migration),
        Box::new(m20260405_000103_add_order_line_item_shipping_profiles::Migration),
        Box::new(m20260409_000104_add_order_line_item_seller_id::Migration),
        Box::new(m20260410_000105_create_order_adjustments::Migration),
        Box::new(m20260411_000106_add_order_tax_lines::Migration),
        Box::new(m20260411_000107_add_order_line_item_translations::Migration),
        Box::new(m20260412_000108_add_order_shipping_total::Migration),
        Box::new(m20260412_000109_add_order_tax_line_provider_id::Migration),
    ]
}
