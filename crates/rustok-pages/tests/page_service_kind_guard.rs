use rustok_core::{MigrationSource, SecurityContext};
use rustok_pages::dto::{BlockType, CreateBlockInput, CreatePageInput, PageTranslationInput};
use rustok_pages::error::PagesError;
use rustok_pages::services::{BlockService, PageService};
use rustok_pages::PagesModule;
use rustok_test_utils::{db::setup_test_db, helpers::admin_context, mock_transactional_event_bus};
use sea_orm_migration::SchemaManager;
use uuid::Uuid;

async fn setup() -> (PageService, BlockService, Uuid, SecurityContext) {
    let db = setup_test_db().await;
    let module = PagesModule;
    let schema = SchemaManager::new(&db);
    for migration in module.migrations() {
        migration
            .up(&schema)
            .await
            .expect("failed to apply pages migrations");
    }

    let event_bus = mock_transactional_event_bus();
    let page_service = PageService::new(db.clone(), event_bus.clone());
    let block_service = BlockService::new(db, event_bus);

    (page_service, block_service, Uuid::new_v4(), admin_context())
}

async fn create_page(
    page_service: &PageService,
    tenant_id: Uuid,
    security: SecurityContext,
) -> rustok_pages::dto::PageResponse {
    page_service
        .create(
            tenant_id,
            security,
            CreatePageInput {
                translations: vec![PageTranslationInput {
                    locale: "en".to_string(),
                    title: "Page".to_string(),
                    slug: Some("page".to_string()),
                    meta_title: None,
                    meta_description: None,
                }],
                template: Some("default".to_string()),
                body: None,
                blocks: None,
                channel_slugs: None,
                publish: false,
            },
        )
        .await
        .expect("failed to create page")
}

async fn create_block(
    block_service: &BlockService,
    tenant_id: Uuid,
    security: SecurityContext,
    page_id: Uuid,
) -> rustok_pages::dto::BlockResponse {
    block_service
        .create(
            tenant_id,
            security,
            page_id,
            CreateBlockInput {
                block_type: BlockType::Text,
                position: 0,
                data: serde_json::json!({ "text": "hello" }),
                translations: None,
            },
        )
        .await
        .expect("failed to create block")
}

#[tokio::test]
async fn publish_returns_page_not_found_for_block_id_and_keeps_page_status() {
    let (page_service, block_service, tenant_id, security) = setup().await;
    let page = create_page(&page_service, tenant_id, security.clone()).await;
    let block = create_block(&block_service, tenant_id, security.clone(), page.id).await;

    let result = page_service
        .publish(tenant_id, security.clone(), block.id)
        .await;

    assert!(matches!(result, Err(PagesError::PageNotFound(id)) if id == block.id));

    let unchanged = page_service
        .get(tenant_id, security, page.id)
        .await
        .expect("page should remain accessible");
    assert_eq!(unchanged.status, page.status);
}

#[tokio::test]
async fn unpublish_returns_page_not_found_for_block_id_and_keeps_page_status() {
    let (page_service, block_service, tenant_id, security) = setup().await;
    let page = create_page(&page_service, tenant_id, security.clone()).await;
    let block = create_block(&block_service, tenant_id, security.clone(), page.id).await;

    let result = page_service
        .unpublish(tenant_id, security.clone(), block.id)
        .await;

    assert!(matches!(result, Err(PagesError::PageNotFound(id)) if id == block.id));

    let unchanged = page_service
        .get(tenant_id, security, page.id)
        .await
        .expect("page should remain accessible");
    assert_eq!(unchanged.status, page.status);
}

#[tokio::test]
async fn delete_returns_page_not_found_for_block_id_and_keeps_page_record() {
    let (page_service, block_service, tenant_id, security) = setup().await;
    let page = create_page(&page_service, tenant_id, security.clone()).await;
    let block = create_block(&block_service, tenant_id, security.clone(), page.id).await;

    let result = page_service
        .delete(tenant_id, security.clone(), block.id)
        .await;

    assert!(matches!(result, Err(PagesError::PageNotFound(id)) if id == block.id));

    let unchanged = page_service
        .get(tenant_id, security, page.id)
        .await
        .expect("page should remain accessible");
    assert_eq!(unchanged.status, page.status);
}
