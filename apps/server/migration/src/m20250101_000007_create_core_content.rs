use sea_orm_migration::prelude::*;

use super::m20250101_000001_create_tenants::Tenants;
use super::m20250101_000002_create_users::Users;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TenantLocales::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TenantLocales::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TenantLocales::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(TenantLocales::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TenantLocales::IsDefault)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(TenantLocales::IsEnabled)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tenant_locales_tenant_id")
                            .from(TenantLocales::Table, TenantLocales::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_tenant_locales_unique")
                    .table(TenantLocales::Table)
                    .col(TenantLocales::TenantId)
                    .col(TenantLocales::Locale)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Categories::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Categories::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Categories::ParentId).uuid().null())
                    .col(
                        ColumnDef::new(Categories::Position)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Categories::Settings)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(Categories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_tenant_id")
                            .from(Categories::Table, Categories::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_categories_parent_id")
                            .from(Categories::Table, Categories::ParentId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CategoryTranslations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CategoryTranslations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CategoryTranslations::CategoryId).uuid().not_null())
                    .col(
                        ColumnDef::new(CategoryTranslations::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CategoryTranslations::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CategoryTranslations::Slug)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(CategoryTranslations::Description).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_category_translations_category_id")
                            .from(
                                CategoryTranslations::Table,
                                CategoryTranslations::CategoryId,
                            )
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_category_translations_unique")
                    .table(CategoryTranslations::Table)
                    .col(CategoryTranslations::CategoryId)
                    .col(CategoryTranslations::Locale)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_category_translations_slug")
                    .table(CategoryTranslations::Table)
                    .col(CategoryTranslations::Locale)
                    .col(CategoryTranslations::Slug)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Nodes::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Nodes::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Nodes::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Nodes::ParentId).uuid().null())
                    .col(ColumnDef::new(Nodes::AuthorId).uuid().null())
                    .col(ColumnDef::new(Nodes::Kind).string_len(64).not_null())
                    .col(ColumnDef::new(Nodes::CategoryId).uuid().null())
                    .col(
                        ColumnDef::new(Nodes::Status)
                            .string_len(32)
                            .not_null()
                            .default("draft"),
                    )
                    .col(
                        ColumnDef::new(Nodes::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(Nodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Nodes::PublishedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodes_tenant_id")
                            .from(Nodes::Table, Nodes::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodes_parent_id")
                            .from(Nodes::Table, Nodes::ParentId)
                            .to(Nodes::Table, Nodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodes_author_id")
                            .from(Nodes::Table, Nodes::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_nodes_category_id")
                            .from(Nodes::Table, Nodes::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(NodeTranslations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NodeTranslations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NodeTranslations::NodeId).uuid().not_null())
                    .col(
                        ColumnDef::new(NodeTranslations::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(ColumnDef::new(NodeTranslations::Title).string_len(255))
                    .col(ColumnDef::new(NodeTranslations::Slug).string_len(255))
                    .col(ColumnDef::new(NodeTranslations::Excerpt).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_node_translations_node_id")
                            .from(NodeTranslations::Table, NodeTranslations::NodeId)
                            .to(Nodes::Table, Nodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_node_translations_unique")
                    .table(NodeTranslations::Table)
                    .col(NodeTranslations::NodeId)
                    .col(NodeTranslations::Locale)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Bodies::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Bodies::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Bodies::NodeId).uuid().not_null())
                    .col(
                        ColumnDef::new(Bodies::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Bodies::Body).text())
                    .col(
                        ColumnDef::new(Bodies::Format)
                            .string_len(32)
                            .not_null()
                            .default("markdown"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_bodies_node_id")
                            .from(Bodies::Table, Bodies::NodeId)
                            .to(Nodes::Table, Nodes::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_bodies_unique")
                    .table(Bodies::Table)
                    .col(Bodies::NodeId)
                    .col(Bodies::Locale)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Tags::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tags::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tags::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(Tags::UseCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tags_tenant_id")
                            .from(Tags::Table, Tags::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TagTranslations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TagTranslations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TagTranslations::TagId).uuid().not_null())
                    .col(
                        ColumnDef::new(TagTranslations::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TagTranslations::Name)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TagTranslations::Slug)
                            .string_len(255)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tag_translations_tag_id")
                            .from(TagTranslations::Table, TagTranslations::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_tag_translations_unique")
                    .table(TagTranslations::Table)
                    .col(TagTranslations::TagId)
                    .col(TagTranslations::Locale)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Taggables::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Taggables::TagId).uuid().not_null())
                    .col(
                        ColumnDef::new(Taggables::TargetType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Taggables::TargetId).uuid().not_null())
                    .col(
                        ColumnDef::new(Taggables::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .primary_key(
                        Index::create()
                            .col(Taggables::TagId)
                            .col(Taggables::TargetType)
                            .col(Taggables::TargetId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_taggables_tag_id")
                            .from(Taggables::Table, Taggables::TagId)
                            .to(Tags::Table, Tags::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Meta::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Meta::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Meta::TenantId).uuid().not_null())
                    .col(
                        ColumnDef::new(Meta::TargetType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Meta::TargetId).uuid().not_null())
                    .col(
                        ColumnDef::new(Meta::NoIndex)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Meta::StructuredData).json_binary())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_meta_tenant_id")
                            .from(Meta::Table, Meta::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_meta_target")
                    .table(Meta::Table)
                    .col(Meta::TargetType)
                    .col(Meta::TargetId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MetaTranslations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MetaTranslations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MetaTranslations::MetaId).uuid().not_null())
                    .col(
                        ColumnDef::new(MetaTranslations::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(ColumnDef::new(MetaTranslations::Title).string_len(255))
                    .col(
                        ColumnDef::new(MetaTranslations::Description)
                            .string_len(500),
                    )
                    .col(ColumnDef::new(MetaTranslations::Keywords).string_len(255))
                    .col(ColumnDef::new(MetaTranslations::OgImage).string_len(500))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_meta_translations_meta_id")
                            .from(MetaTranslations::Table, MetaTranslations::MetaId)
                            .to(Meta::Table, Meta::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_meta_translations_unique")
                    .table(MetaTranslations::Table)
                    .col(MetaTranslations::MetaId)
                    .col(MetaTranslations::Locale)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Media::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Media::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Media::TenantId).uuid().not_null())
                    .col(ColumnDef::new(Media::UploadedBy).uuid().null())
                    .col(ColumnDef::new(Media::Filename).string_len(255).not_null())
                    .col(ColumnDef::new(Media::OriginalName).string_len(255).not_null())
                    .col(ColumnDef::new(Media::MimeType).string_len(100).not_null())
                    .col(ColumnDef::new(Media::Size).big_integer().not_null())
                    .col(ColumnDef::new(Media::StoragePath).string_len(500).not_null())
                    .col(
                        ColumnDef::new(Media::Metadata)
                            .json_binary()
                            .not_null()
                            .default("{}"),
                    )
                    .col(
                        ColumnDef::new(Media::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_tenant_id")
                            .from(Media::Table, Media::TenantId)
                            .to(Tenants::Table, Tenants::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_uploaded_by")
                            .from(Media::Table, Media::UploadedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MediaTranslations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MediaTranslations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MediaTranslations::MediaId).uuid().not_null())
                    .col(
                        ColumnDef::new(MediaTranslations::Locale)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(ColumnDef::new(MediaTranslations::Title).string_len(255))
                    .col(ColumnDef::new(MediaTranslations::AltText).string_len(255))
                    .col(ColumnDef::new(MediaTranslations::Caption).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_translations_media_id")
                            .from(MediaTranslations::Table, MediaTranslations::MediaId)
                            .to(Media::Table, Media::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("idx_media_translations_unique")
                    .table(MediaTranslations::Table)
                    .col(MediaTranslations::MediaId)
                    .col(MediaTranslations::Locale)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MediaTranslations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Media::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MetaTranslations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Meta::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Taggables::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TagTranslations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Bodies::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(NodeTranslations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Nodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(CategoryTranslations::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Categories::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TenantLocales::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TenantLocales {
    Table,
    Id,
    TenantId,
    Locale,
    IsDefault,
    IsEnabled,
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
    TenantId,
    ParentId,
    Position,
    Settings,
    CreatedAt,
}

#[derive(Iden)]
enum CategoryTranslations {
    Table,
    Id,
    CategoryId,
    Locale,
    Name,
    Slug,
    Description,
}

#[derive(Iden)]
enum Nodes {
    Table,
    Id,
    TenantId,
    ParentId,
    AuthorId,
    Kind,
    CategoryId,
    Status,
    Metadata,
    CreatedAt,
    PublishedAt,
}

#[derive(Iden)]
enum NodeTranslations {
    Table,
    Id,
    NodeId,
    Locale,
    Title,
    Slug,
    Excerpt,
}

#[derive(Iden)]
enum Bodies {
    Table,
    Id,
    NodeId,
    Locale,
    Body,
    Format,
}

#[derive(Iden)]
enum Tags {
    Table,
    Id,
    TenantId,
    UseCount,
}

#[derive(Iden)]
enum TagTranslations {
    Table,
    Id,
    TagId,
    Locale,
    Name,
    Slug,
}

#[derive(Iden)]
enum Taggables {
    Table,
    TagId,
    TargetType,
    TargetId,
    CreatedAt,
}

#[derive(Iden)]
enum Meta {
    Table,
    Id,
    TenantId,
    TargetType,
    TargetId,
    NoIndex,
    StructuredData,
}

#[derive(Iden)]
enum MetaTranslations {
    Table,
    Id,
    MetaId,
    Locale,
    Title,
    Description,
    Keywords,
    OgImage,
}

#[derive(Iden)]
enum Media {
    Table,
    Id,
    TenantId,
    UploadedBy,
    Filename,
    OriginalName,
    MimeType,
    Size,
    StoragePath,
    Metadata,
    CreatedAt,
}

#[derive(Iden)]
enum MediaTranslations {
    Table,
    Id,
    MediaId,
    Locale,
    Title,
    AltText,
    Caption,
}
