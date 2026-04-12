use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(IndexFlexEntries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(IndexFlexEntries::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(IndexFlexEntries::TenantId).uuid().not_null())
                    .col(ColumnDef::new(IndexFlexEntries::SchemaId).uuid().not_null())
                    .col(
                        ColumnDef::new(IndexFlexEntries::SchemaSlug)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(IndexFlexEntries::EntityType).string_len(64))
                    .col(ColumnDef::new(IndexFlexEntries::EntityId).uuid())
                    .col(
                        ColumnDef::new(IndexFlexEntries::Status)
                            .string_len(32)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IndexFlexEntries::DataPreview)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(IndexFlexEntries::SearchVector)
                            .custom(Alias::new("tsvector")),
                    )
                    .col(
                        ColumnDef::new(IndexFlexEntries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IndexFlexEntries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(IndexFlexEntries::IndexedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_index_flex_entries_schema")
                    .table(IndexFlexEntries::Table)
                    .col(IndexFlexEntries::TenantId)
                    .col(IndexFlexEntries::SchemaId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_index_flex_entries_binding")
                    .table(IndexFlexEntries::Table)
                    .col(IndexFlexEntries::TenantId)
                    .col(IndexFlexEntries::EntityType)
                    .col(IndexFlexEntries::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_index_flex_entries_slug_status")
                    .table(IndexFlexEntries::Table)
                    .col(IndexFlexEntries::TenantId)
                    .col(IndexFlexEntries::SchemaSlug)
                    .col(IndexFlexEntries::Status)
                    .to_owned(),
            )
            .await?;

        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared(
                    "CREATE INDEX idx_index_flex_entries_search ON index_flex_entries USING GIN (search_vector)",
                )
                .await?;

            manager
                .get_connection()
                .execute_unprepared(
                    "CREATE INDEX idx_index_flex_entries_preview ON index_flex_entries USING GIN (data_preview jsonb_path_ops)",
                )
                .await?;

            manager
                .get_connection()
                .execute_unprepared(
                    r#"
                CREATE OR REPLACE FUNCTION index_flex_entries_search_trigger() RETURNS trigger AS $$
                BEGIN
                    NEW.search_vector :=
                        setweight(to_tsvector('simple', COALESCE(NEW.schema_slug, '')), 'A') ||
                        setweight(to_tsvector('simple', COALESCE(NEW.status, '')), 'C') ||
                        setweight(to_tsvector('simple', COALESCE(NEW.entity_type, '')), 'C') ||
                        setweight(to_tsvector('simple', COALESCE(NEW.data_preview::text, '')), 'B');
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;

                CREATE TRIGGER index_flex_entries_search_update
                    BEFORE INSERT OR UPDATE ON index_flex_entries
                    FOR EACH ROW
                    EXECUTE FUNCTION index_flex_entries_search_trigger();
            "#,
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        if manager.get_database_backend() == DatabaseBackend::Postgres {
            manager
                .get_connection()
                .execute_unprepared(
                    "DROP TRIGGER IF EXISTS index_flex_entries_search_update ON index_flex_entries",
                )
                .await?;
            manager
                .get_connection()
                .execute_unprepared("DROP FUNCTION IF EXISTS index_flex_entries_search_trigger()")
                .await?;
        }

        manager
            .drop_table(
                Table::drop()
                    .table(IndexFlexEntries::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum IndexFlexEntries {
    Table,
    Id,
    TenantId,
    SchemaId,
    SchemaSlug,
    EntityType,
    EntityId,
    Status,
    DataPreview,
    SearchVector,
    CreatedAt,
    UpdatedAt,
    IndexedAt,
}
