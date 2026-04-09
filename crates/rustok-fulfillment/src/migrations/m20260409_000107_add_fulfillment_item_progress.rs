use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(FulfillmentItems::Table)
                    .add_column(
                        ColumnDef::new(FulfillmentItems::ShippedQuantity)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(
                        ColumnDef::new(FulfillmentItems::DeliveredQuantity)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(FulfillmentItems::Table)
                    .drop_column(FulfillmentItems::ShippedQuantity)
                    .drop_column(FulfillmentItems::DeliveredQuantity)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum FulfillmentItems {
    Table,
    ShippedQuantity,
    DeliveredQuantity,
}
