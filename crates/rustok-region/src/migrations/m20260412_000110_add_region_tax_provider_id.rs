use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Regions::Table)
                    .add_column(ColumnDef::new(Regions::TaxProviderId).string_len(64).null())
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_connection().get_database_backend();
        manager
            .get_connection()
            .execute(Statement::from_string(
                backend,
                "UPDATE regions
                 SET tax_provider_id = NULLIF(LOWER(TRIM(COALESCE(metadata->>'tax_provider_id', ''))), '')"
                    .to_string(),
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Regions::Table)
                    .drop_column(Regions::TaxProviderId)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Regions {
    Table,
    TaxProviderId,
}
