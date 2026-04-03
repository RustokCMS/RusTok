use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AiChatSessions::Table)
                    .add_column(
                        ColumnDef::new(AiChatSessions::RequestedLocale)
                            .string_len(32)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(AiChatSessions::ResolvedLocale)
                            .string_len(32)
                            .not_null()
                            .default("en"),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AiChatRuns::Table)
                    .add_column(
                        ColumnDef::new(AiChatRuns::RequestedLocale)
                            .string_len(32)
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(AiChatRuns::ResolvedLocale)
                            .string_len(32)
                            .not_null()
                            .default("en"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(AiChatRuns::Table)
                    .drop_column(AiChatRuns::ResolvedLocale)
                    .drop_column(AiChatRuns::RequestedLocale)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(AiChatSessions::Table)
                    .drop_column(AiChatSessions::ResolvedLocale)
                    .drop_column(AiChatSessions::RequestedLocale)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AiChatSessions {
    Table,
    RequestedLocale,
    ResolvedLocale,
}

#[derive(DeriveIden)]
enum AiChatRuns {
    Table,
    RequestedLocale,
    ResolvedLocale,
}
