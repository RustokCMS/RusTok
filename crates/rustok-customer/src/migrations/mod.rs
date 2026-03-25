mod m20260325_000103_create_customers_table;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![Box::new(m20260325_000103_create_customers_table::Migration)]
}
