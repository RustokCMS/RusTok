use std::sync::Arc;

use rustok_core::{MemoryTransport, MigrationSource, SecurityContext, UserRole};
use rustok_forum::{
    CategoryService, CreateCategoryInput, CreateTopicInput, ForumModule, ListTopicsFilter,
    TopicService, UpdateCategoryInput, UpdateTopicInput,
};
use rustok_outbox::TransactionalEventBus;
use rustok_taxonomy::TaxonomyModule;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::SchemaManager;
use tokio::sync::broadcast;
use uuid::Uuid;

async fn setup_forum_test_db() -> DatabaseConnection {
    let db_url = format!(
        "sqlite:file:forum_locale_contract_{}?mode=memory&cache=shared",
        Uuid::new_v4()
    );
    let mut opts = ConnectOptions::new(db_url);
    opts.max_connections(5)
        .min_connections(1)
        .sqlx_logging(false);

    Database::connect(opts)
        .await
        .expect("failed to connect forum sqlite database")
}

async fn setup() -> (
    DatabaseConnection,
    TransactionalEventBus,
    broadcast::Receiver<rustok_events::EventEnvelope>,
    Uuid,
) {
    let db = setup_forum_test_db().await;
    let schema = SchemaManager::new(&db);
    for migration in TaxonomyModule.migrations() {
        migration
            .up(&schema)
            .await
            .expect("taxonomy migration should apply");
    }
    let module = ForumModule;
    for migration in module.migrations() {
        migration
            .up(&schema)
            .await
            .expect("forum migration should apply");
    }

    let transport = MemoryTransport::new();
    let receiver = transport.subscribe();
    let event_bus = TransactionalEventBus::new(Arc::new(transport));
    (db, event_bus, receiver, Uuid::new_v4())
}

#[tokio::test]
async fn category_list_exposes_requested_effective_and_available_locales() {
    let (db, _event_bus, _events, tenant_id) = setup().await;
    let service = CategoryService::new(db);
    let admin = SecurityContext::new(UserRole::Admin, Some(Uuid::new_v4()));

    let category = service
        .create(
            tenant_id,
            admin.clone(),
            CreateCategoryInput {
                locale: "en".to_string(),
                name: "General".to_string(),
                slug: "general".to_string(),
                description: Some("General discussion".to_string()),
                icon: None,
                color: None,
                parent_id: None,
                position: Some(0),
                moderated: false,
            },
        )
        .await
        .expect("category should be created");

    service
        .update(
            tenant_id,
            category.id,
            admin.clone(),
            UpdateCategoryInput {
                locale: "ru".to_string(),
                name: Some("Общее".to_string()),
                slug: Some("obschee".to_string()),
                description: Some("Общие обсуждения".to_string()),
                icon: None,
                color: None,
                position: None,
                moderated: None,
            },
        )
        .await
        .expect("ru category translation should be saved");

    let (items, total) = service
        .list_paginated_with_locale_fallback(tenant_id, admin, "fr-FR", 1, 20, Some("ru"))
        .await
        .expect("category list should load");

    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].requested_locale, "fr-fr");
    assert_eq!(items[0].locale, "fr-fr");
    assert_eq!(items[0].effective_locale, "ru");
    assert_eq!(
        items[0].available_locales,
        vec!["en".to_string(), "ru".to_string()]
    );
    assert_eq!(items[0].name, "Общее");
    assert_eq!(items[0].slug, "obschee");
}

#[tokio::test]
async fn topic_list_exposes_requested_effective_and_available_locales() {
    let (db, event_bus, _events, tenant_id) = setup().await;
    let category_service = CategoryService::new(db.clone());
    let topic_service = TopicService::new(db, event_bus);
    let admin = SecurityContext::new(UserRole::Admin, Some(Uuid::new_v4()));

    let category = category_service
        .create(
            tenant_id,
            admin.clone(),
            CreateCategoryInput {
                locale: "en".to_string(),
                name: "General".to_string(),
                slug: "general".to_string(),
                description: None,
                icon: None,
                color: None,
                parent_id: None,
                position: Some(0),
                moderated: false,
            },
        )
        .await
        .expect("category should be created");

    let topic = topic_service
        .create(
            tenant_id,
            admin.clone(),
            CreateTopicInput {
                locale: "en".to_string(),
                category_id: category.id,
                title: "General thread".to_string(),
                slug: Some("general-thread".to_string()),
                body: "English body".to_string(),
                body_format: "markdown".to_string(),
                content_json: None,
                tags: vec![],
                channel_slugs: None,
            },
        )
        .await
        .expect("topic should be created");

    topic_service
        .update(
            tenant_id,
            topic.id,
            admin.clone(),
            UpdateTopicInput {
                locale: "ru".to_string(),
                title: Some("Общая тема".to_string()),
                body: Some("Русское тело".to_string()),
                body_format: Some("markdown".to_string()),
                content_json: None,
                tags: None,
                channel_slugs: None,
            },
        )
        .await
        .expect("ru topic translation should be saved");

    let (items, total) = topic_service
        .list_with_locale_fallback(
            tenant_id,
            admin,
            ListTopicsFilter {
                category_id: Some(category.id),
                status: None,
                locale: Some("fr-FR".to_string()),
                page: 1,
                per_page: 20,
            },
            Some("ru"),
        )
        .await
        .expect("topic list should load");

    assert_eq!(total, 1);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].requested_locale, "fr-fr");
    assert_eq!(items[0].locale, "fr-fr");
    assert_eq!(items[0].effective_locale, "ru");
    assert_eq!(
        items[0].available_locales,
        vec!["en".to_string(), "ru".to_string()]
    );
    assert_eq!(items[0].title, "Общая тема");
    assert_eq!(items[0].slug, "general-thread");
}
