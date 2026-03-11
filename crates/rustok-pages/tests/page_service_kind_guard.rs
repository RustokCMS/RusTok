use rustok_content::{CreateNodeInput, NodeService, NodeTranslationInput};
use rustok_core::SecurityContext;
use rustok_pages::error::PagesError;
use rustok_pages::services::PageService;
use rustok_test_utils::{db::setup_test_db, helpers::admin_context, mock_transactional_event_bus};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};
use uuid::Uuid;

async fn ensure_content_schema(db: &DatabaseConnection) {
    if db.get_database_backend() != DbBackend::Sqlite {
        return;
    }

    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS nodes (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            parent_id TEXT NULL,
            author_id TEXT NULL,
            kind TEXT NOT NULL,
            category_id TEXT NULL,
            status TEXT NOT NULL,
            position INTEGER NOT NULL,
            depth INTEGER NOT NULL,
            reply_count INTEGER NOT NULL,
            metadata TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            published_at TEXT NULL,
            deleted_at TEXT NULL,
            version INTEGER NOT NULL DEFAULT 1
        )"
        .to_string(),
    ))
    .await
    .expect("failed to create nodes test table");

    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS node_translations (
            id TEXT PRIMARY KEY,
            node_id TEXT NOT NULL,
            locale TEXT NOT NULL,
            title TEXT NULL,
            slug TEXT NULL,
            excerpt TEXT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(node_id) REFERENCES nodes(id)
        )"
        .to_string(),
    ))
    .await
    .expect("failed to create node_translations test table");

    db.execute(Statement::from_string(
        DbBackend::Sqlite,
        "CREATE TABLE IF NOT EXISTS bodies (
            id TEXT PRIMARY KEY,
            node_id TEXT NOT NULL,
            locale TEXT NOT NULL,
            body TEXT NULL,
            format TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY(node_id) REFERENCES nodes(id)
        )"
        .to_string(),
    ))
    .await
    .expect("failed to create bodies test table");
}

async fn setup() -> (PageService, NodeService, Uuid, SecurityContext) {
    let db = setup_test_db().await;
    ensure_content_schema(&db).await;
    let event_bus = mock_transactional_event_bus();
    let page_service = PageService::new(db.clone(), event_bus.clone());
    let node_service = NodeService::new(db, event_bus);

    (page_service, node_service, Uuid::new_v4(), admin_context())
}

async fn create_post_node(
    node_service: &NodeService,
    tenant_id: Uuid,
    security: SecurityContext,
) -> rustok_content::dto::NodeResponse {
    node_service
        .create_node(
            tenant_id,
            security,
            CreateNodeInput {
                kind: "post".to_string(),
                status: Some(rustok_content::entities::node::ContentStatus::Draft),
                parent_id: None,
                author_id: None,
                category_id: None,
                position: Some(0),
                depth: Some(0),
                reply_count: Some(0),
                metadata: serde_json::json!({}),
                translations: vec![NodeTranslationInput {
                    locale: "en".to_string(),
                    title: Some("Post".to_string()),
                    slug: None,
                    excerpt: None,
                }],
                bodies: vec![],
            },
        )
        .await
        .expect("failed to create post node")
}

#[tokio::test]
async fn publish_returns_page_not_found_for_post_id_and_keeps_status() {
    let (page_service, node_service, tenant_id, security) = setup().await;
    let post = create_post_node(&node_service, tenant_id, security.clone()).await;

    let result = page_service
        .publish(tenant_id, security.clone(), post.id)
        .await;

    assert!(matches!(result, Err(PagesError::PageNotFound(id)) if id == post.id));

    let unchanged = node_service
        .get_node(tenant_id, post.id)
        .await
        .expect("post node should remain accessible");
    assert_eq!(unchanged.status, post.status);
    assert!(unchanged.deleted_at.is_none());
}

#[tokio::test]
async fn unpublish_returns_page_not_found_for_post_id_and_keeps_status() {
    let (page_service, node_service, tenant_id, security) = setup().await;
    let post = create_post_node(&node_service, tenant_id, security.clone()).await;

    let result = page_service
        .unpublish(tenant_id, security.clone(), post.id)
        .await;

    assert!(matches!(result, Err(PagesError::PageNotFound(id)) if id == post.id));

    let unchanged = node_service
        .get_node(tenant_id, post.id)
        .await
        .expect("post node should remain accessible");
    assert_eq!(unchanged.status, post.status);
    assert!(unchanged.deleted_at.is_none());
}

#[tokio::test]
async fn delete_returns_page_not_found_for_post_id_and_keeps_record() {
    let (page_service, node_service, tenant_id, security) = setup().await;
    let post = create_post_node(&node_service, tenant_id, security.clone()).await;

    let result = page_service
        .delete(tenant_id, security.clone(), post.id)
        .await;

    assert!(matches!(result, Err(PagesError::PageNotFound(id)) if id == post.id));

    let unchanged = node_service
        .get_node(tenant_id, post.id)
        .await
        .expect("post node should remain accessible");
    assert_eq!(unchanged.status, post.status);
    assert!(unchanged.deleted_at.is_none());
}
