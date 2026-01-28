#![allow(elided_lifetimes_in_paths)]

pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tenants;
mod m20240101_000002_create_users;
mod m20240101_000003_create_tenant_modules;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tenants::Migration),
            Box::new(m20240101_000002_create_users::Migration),
            Box::new(m20240101_000003_create_tenant_modules::Migration),
        ]
    }
}
