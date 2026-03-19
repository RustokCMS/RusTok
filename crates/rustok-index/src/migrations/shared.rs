use sea_orm_migration::prelude::*;

/// Platform-core table references used in index migration foreign keys.
#[allow(dead_code)]
#[derive(Iden)]
pub enum Tenants {
    Table,
    Id,
}
