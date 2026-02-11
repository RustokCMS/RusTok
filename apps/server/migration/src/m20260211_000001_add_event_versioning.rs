use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add event_type and schema_version columns to sys_events table
        manager
            .alter_table(
                Table::alter()
                    .table(SysEvents::Table)
                    .add_column(
                        ColumnDef::new(SysEvents::EventType)
                            .string_len(128)
                            .not_null()
                            .default("unknown"),
                    )
                    .add_column(
                        ColumnDef::new(SysEvents::SchemaVersion)
                            .small_integer()
                            .not_null()
                            .default(1),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for filtering events by type and version
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_sys_events_type_version")
                    .table(SysEvents::Table)
                    .col(SysEvents::EventType)
                    .col(SysEvents::SchemaVersion)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop index first
        manager
            .drop_index(
                Index::drop()
                    .name("idx_sys_events_type_version")
                    .table(SysEvents::Table)
                    .to_owned(),
            )
            .await?;

        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(SysEvents::Table)
                    .drop_column(SysEvents::SchemaVersion)
                    .drop_column(SysEvents::EventType)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SysEvents {
    Table,
    EventType,
    SchemaVersion,
}
