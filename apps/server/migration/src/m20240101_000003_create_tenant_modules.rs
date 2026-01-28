use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TenantModules::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TenantModules::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TenantModules::TenantId).uuid().not_null())
                    .col(ColumnDef::new(TenantModules::ModuleSlug).string().not_null())
                    .col(
                        ColumnDef::new(TenantModules::Enabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(TenantModules::Config).json_binary().not_null())
                    .col(
                        ColumnDef::new(TenantModules::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(TenantModules::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-tenant-modules-tenant_id")
                            .from(TenantModules::Table, TenantModules::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx-tenant-module-unique")
                            .table(TenantModules::Table)
                            .col(TenantModules::TenantId)
                            .col(TenantModules::ModuleSlug),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TenantModules::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TenantModules {
    Table,
    Id,
    TenantId,
    ModuleSlug,
    Enabled,
    Config,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Tenants {
    Table,
    Id,
}
