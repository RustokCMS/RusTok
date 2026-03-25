use rustok_customer::dto::{CreateCustomerInput, UpdateCustomerInput};
use rustok_customer::error::CustomerError;
use rustok_customer::services::CustomerService;
use rustok_test_utils::db::setup_test_db;
use uuid::Uuid;

mod support;

async fn setup() -> CustomerService {
    let db = setup_test_db().await;
    support::ensure_customer_schema(&db).await;
    CustomerService::new(db)
}

fn create_input() -> CreateCustomerInput {
    CreateCustomerInput {
        user_id: Some(Uuid::new_v4()),
        email: "customer@example.com".to_string(),
        first_name: Some("Jane".to_string()),
        last_name: Some("Doe".to_string()),
        phone: Some("+123456789".to_string()),
        locale: Some("en".to_string()),
        metadata: serde_json::json!({ "source": "customer-test" }),
    }
}

#[tokio::test]
async fn create_and_get_customer() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let created = service
        .create_customer(tenant_id, create_input())
        .await
        .unwrap();
    assert_eq!(created.email, "customer@example.com");

    let fetched = service.get_customer(tenant_id, created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.first_name.as_deref(), Some("Jane"));
}

#[tokio::test]
async fn update_customer_profile() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    let created = service
        .create_customer(tenant_id, create_input())
        .await
        .unwrap();
    let updated = service
        .update_customer(
            tenant_id,
            created.id,
            UpdateCustomerInput {
                email: Some("updated@example.com".to_string()),
                first_name: Some("Janet".to_string()),
                last_name: None,
                phone: Some("+987654321".to_string()),
                locale: Some("ru".to_string()),
                metadata: Some(serde_json::json!({ "source": "updated" })),
            },
        )
        .await
        .unwrap();

    assert_eq!(updated.email, "updated@example.com");
    assert_eq!(updated.first_name.as_deref(), Some("Janet"));
    assert_eq!(updated.phone.as_deref(), Some("+987654321"));
    assert_eq!(updated.locale.as_deref(), Some("ru"));
}

#[tokio::test]
async fn duplicate_email_is_rejected() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();

    service
        .create_customer(tenant_id, create_input())
        .await
        .unwrap();
    let error = service
        .create_customer(
            tenant_id,
            CreateCustomerInput {
                user_id: Some(Uuid::new_v4()),
                ..create_input()
            },
        )
        .await
        .unwrap_err();

    match error {
        CustomerError::DuplicateEmail(email) => assert_eq!(email, "customer@example.com"),
        other => panic!("expected duplicate email error, got {other:?}"),
    }
}

#[tokio::test]
async fn upsert_customer_for_user_updates_existing_profile() {
    let service = setup().await;
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let created = service
        .upsert_customer_for_user(
            tenant_id,
            user_id,
            CreateCustomerInput {
                user_id: Some(user_id),
                ..create_input()
            },
        )
        .await
        .unwrap();
    let updated = service
        .upsert_customer_for_user(
            tenant_id,
            user_id,
            CreateCustomerInput {
                user_id: Some(user_id),
                email: "customer-updated@example.com".to_string(),
                first_name: Some("Updated".to_string()),
                last_name: Some("User".to_string()),
                phone: None,
                locale: Some("de".to_string()),
                metadata: serde_json::json!({ "step": 2 }),
            },
        )
        .await
        .unwrap();

    assert_eq!(created.id, updated.id);
    assert_eq!(updated.email, "customer-updated@example.com");
    assert_eq!(updated.locale.as_deref(), Some("de"));
}
