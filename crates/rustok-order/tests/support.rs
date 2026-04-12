use rustok_order::entities::{
    order, order_adjustment, order_line_item, order_line_item_translation,
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Schema};

mod order_field_definitions {
    rustok_core::define_field_definitions_entity!("order_field_definitions");
}

pub async fn ensure_order_schema(db: &DatabaseConnection) {
    if db.get_database_backend() != DbBackend::Sqlite {
        return;
    }

    let builder = db.get_database_backend();
    let schema = Schema::new(builder);

    create_entity_table(db, &builder, schema.create_table_from_entity(order::Entity)).await;
    create_entity_table(
        db,
        &builder,
        schema.create_table_from_entity(order_line_item::Entity),
    )
    .await;
    create_entity_table(
        db,
        &builder,
        schema.create_table_from_entity(order_line_item_translation::Entity),
    )
    .await;
    create_entity_table(
        db,
        &builder,
        schema.create_table_from_entity(order_adjustment::Entity),
    )
    .await;
    create_entity_table(
        db,
        &builder,
        schema.create_table_from_entity(order_field_definitions::Entity),
    )
    .await;
    let attached_table = sea_orm::sea_query::Table::create()
        .table(sea_orm::sea_query::Alias::new(
            "flex_attached_localized_values",
        ))
        .if_not_exists()
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("id"))
                .uuid()
                .not_null()
                .primary_key(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("tenant_id"))
                .uuid()
                .not_null(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("entity_type"))
                .string_len(64)
                .not_null(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("entity_id"))
                .uuid()
                .not_null(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("field_key"))
                .string_len(128)
                .not_null(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("locale"))
                .string_len(32)
                .not_null(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("value"))
                .json_binary()
                .not_null(),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("created_at"))
                .timestamp_with_time_zone()
                .not_null()
                .default(sea_orm::sea_query::Expr::current_timestamp()),
        )
        .col(
            sea_orm::sea_query::ColumnDef::new(sea_orm::sea_query::Alias::new("updated_at"))
                .timestamp_with_time_zone()
                .not_null()
                .default(sea_orm::sea_query::Expr::current_timestamp()),
        )
        .to_owned();
    db.execute(builder.build(&attached_table))
        .await
        .expect("flex attached localized values table should be created");
}

async fn create_entity_table(
    db: &DatabaseConnection,
    builder: &DbBackend,
    mut statement: sea_orm::sea_query::TableCreateStatement,
) {
    statement.if_not_exists();
    db.execute(builder.build(&statement))
        .await
        .expect("failed to create order test table");
}
