use rustok_core::events::{DomainEvent, EventEnvelope, TransactionalEventBus};
use rustok_outbox::OutboxTransport;
use sea_orm::{Database, DatabaseConnection, EntityTrait, ModelTrait};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use uuid::Uuid;

async fn setup_test_db() -> (DatabaseConnection, Pool<Sqlite>) {
    let database_url = "sqlite::memory:";

    // Setup SeaORM connection
    let sea_orm_db = Database::connect(database_url)
        .await
        .expect("Failed to connect SeaORM to test database");

    // Setup sqlx connection for manual queries
    let sqlx_pool = sqlx::Sqlite::create(&database_url).await.unwrap();

    (sea_orm_db, sqlx_pool)
}

#[tokio::test]
async fn test_transactional_event_publishing_rollback() {
    let (sea_orm_db, sqlx_pool) = setup_test_db().await;
    let transport = Arc::new(OutboxTransport::new(sea_orm_db.clone()));
    let event_bus = TransactionalEventBus::new(transport.clone());

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Start a transaction and publish event
    let mut txn = sea_orm_db.begin().await.unwrap();

    let event = DomainEvent::NodeCreated {
        node_id: Uuid::new_v4(),
        kind: "post".to_string(),
        author_id: Some(user_id),
    };

    // Publish event in transaction
    event_bus
        .publish_in_tx(&txn, tenant_id, Some(user_id), event.clone())
        .await
        .expect("Failed to publish event in transaction");

    // Rollback transaction - event should not be persisted
    txn.rollback().await.unwrap();

    // Check that event was not persisted (outbox should be empty)
    let count: i64 = sqlx::query("SELECT COUNT(*) FROM sys_events")
        .fetch_one(&sqlx_pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(
        count, 0,
        "Events should not be persisted after transaction rollback"
    );
}

#[tokio::test]
async fn test_transactional_event_publishing_commit() {
    let (sea_orm_db, sqlx_pool) = setup_test_db().await;
    let transport = Arc::new(OutboxTransport::new(sea_orm_db.clone()));
    let event_bus = TransactionalEventBus::new(transport.clone());

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();

    // Start a transaction and publish event
    let mut txn = sea_orm_db.begin().await.unwrap();

    let event = DomainEvent::NodeCreated {
        node_id,
        kind: "post".to_string(),
        author_id: Some(user_id),
    };

    // Publish event in transaction
    event_bus
        .publish_in_tx(&txn, tenant_id, Some(user_id), event.clone())
        .await
        .expect("Failed to publish event in transaction");

    // Commit transaction - event should be persisted
    txn.commit().await.unwrap();

    // Check that event was persisted
    let count: i64 = sqlx::query("SELECT COUNT(*) FROM sys_events")
        .fetch_one(&sqlx_pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(
        count, 1,
        "Event should be persisted after transaction commit"
    );

    // Verify event content
    let persisted_event: (String, i32, String, Option<String>, Uuid, Option<Uuid>, String, serde_json::Value) = sqlx::query(
        r#"SELECT event_type, schema_version, aggregate_id, aggregate_type, tenant_id, actor_id, event_data, metadata 
           FROM sys_events 
           LIMIT 1"#
    )
    .fetch_one(&sqlx_pool)
    .await
    .unwrap();

    assert_eq!(persisted_event.0, "node.created");
    assert_eq!(persisted_event.1, 1); // schema_version
    assert_eq!(persisted_event.4, tenant_id);
    assert_eq!(persisted_event.5, Some(user_id));
    assert_eq!(persisted_event.2, node_id.to_string());
}

#[tokio::test]
async fn test_mixed_transactional_and_non_transactional_events() {
    let (sea_orm_db, sqlx_pool) = setup_test_db().await;
    let transport = Arc::new(OutboxTransport::new(sea_orm_db.clone()));
    let event_bus = TransactionalEventBus::new(transport.clone());

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();

    // Publish non-transactional event first
    let non_tx_event = DomainEvent::NodeCreated {
        node_id: Uuid::new_v4(),
        kind: "page".to_string(),
        author_id: Some(user_id),
    };

    event_bus
        .publish(tenant_id, Some(user_id), non_tx_event.clone())
        .await
        .expect("Failed to publish non-transactional event");

    // Publish transactional event in transaction
    let mut txn = sea_orm_db.begin().await.unwrap();

    let tx_event = DomainEvent::NodeCreated {
        node_id,
        kind: "post".to_string(),
        author_id: Some(user_id),
    };

    event_bus
        .publish_in_tx(&txn, tenant_id, Some(user_id), tx_event.clone())
        .await
        .expect("Failed to publish transactional event");

    txn.commit().await.unwrap();

    // Check that both events were persisted
    let count: i64 = sqlx::query("SELECT COUNT(*) FROM sys_events")
        .fetch_one(&sqlx_pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(
        count, 2,
        "Both transactional and non-transactional events should be persisted"
    );
}

#[tokio::test]
async fn test_multiple_events_in_single_transaction() {
    let (sea_orm_db, sqlx_pool) = setup_test_db().await;
    let transport = Arc::new(OutboxTransport::new(sea_orm_db.clone()));
    let event_bus = TransactionalEventBus::new(transport.clone());

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();

    // Start transaction and publish multiple events
    let mut txn = sea_orm_db.begin().await.unwrap();

    let events = vec![
        DomainEvent::NodeCreated {
            node_id,
            kind: "post".to_string(),
            author_id: Some(user_id),
        },
        DomainEvent::NodeUpdated {
            node_id,
            kind: "post".to_string(),
        },
        DomainEvent::NodePublished {
            node_id,
            kind: "post".to_string(),
        },
    ];

    for event in &events {
        event_bus
            .publish_in_tx(&txn, tenant_id, Some(user_id), event.clone())
            .await
            .expect("Failed to publish event in transaction");
    }

    txn.commit().await.unwrap();

    // Check that all events were persisted
    let count: i64 = sqlx::query("SELECT COUNT(*) FROM sys_events")
        .fetch_one(&sqlx_pool)
        .await
        .unwrap()
        .get(0);

    assert_eq!(count, 3, "All events in transaction should be persisted");
}

#[tokio::test]
async fn test_event_persistence_on_db_failure() {
    let (sea_orm_db, sqlx_pool) = setup_test_db().await;
    let transport = Arc::new(OutboxTransport::new(sea_orm_db.clone()));
    let event_bus = TransactionalEventBus::new(transport.clone());

    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Start transaction
    let mut txn = sea_orm_db.begin().await.unwrap();

    let event = DomainEvent::NodeCreated {
        node_id: Uuid::new_v4(),
        kind: "post".to_string(),
        author_id: Some(user_id),
    };

    // Publish event in transaction
    event_bus
        .publish_in_tx(&txn, tenant_id, Some(user_id), event.clone())
        .await
        .expect("Failed to publish event in transaction");

    // Manually close database connection to simulate failure
    drop(sea_orm_db);
    drop(sqlx_pool);

    // Transaction should fail to commit due to connection loss
    let result = txn.commit().await;
    assert!(
        result.is_err(),
        "Transaction commit should fail on connection loss"
    );
}
