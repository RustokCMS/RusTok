# rustok-test-utils

Test utilities for the RusToK platform. This crate provides fixtures, mocks, and helpers for writing tests across all RusToK modules.

## Features

- **Database utilities** - Setup test databases with migrations, cleanup, and reset
- **Mock services** - Payment gateway, email service, storage service
- **Test fixtures** - Builder patterns for users, tenants, nodes, products, orders
- **Test application wrapper** - High-level API for integration testing
- **Helper functions** - Common testing utilities and assertions

## Usage

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
rustok-test-utils = { path = "../rustok-test-utils" }
```

## Examples

### Database Setup

```rust
use rustok_test_utils::setup_test_db;

#[tokio::test]
async fn test_with_database() {
    let db = setup_test_db().await;
    // In-memory DB only (without automatic migrations).
}
```


For schema-aware tests use explicit migrations:

```rust,ignore
use rustok_test_utils::db::setup_test_db_with_migrations;

// let db = setup_test_db_with_migrations::<YourMigrator>().await;
```

### Mock Event Bus

```rust
use rustok_test_utils::MockEventBus;
use rustok_core::{DomainEvent, EventBus};
use uuid::Uuid;

#[tokio::test]
async fn test_event_publishing() {
    let mock_bus = MockEventBus::new();
    let tenant_id = Uuid::new_v4();

    // Publish an event
    mock_bus.publish(tenant_id, None, DomainEvent::NodeCreated {
        id: Uuid::new_v4(),
        kind: "post".to_string(),
        tenant_id,
    }).unwrap();

    // Verify event was recorded
    assert_eq!(mock_bus.event_count(), 1);
    assert!(mock_bus.has_event_of_type("NodeCreated"));
}
```

### Fixtures

```rust
use rustok_test_utils::fixtures::{UserFixture, NodeFixture, ProductFixture};

#[test]
fn test_fixtures() {
    // Create a test user
    let admin = UserFixture::admin()
        .with_email("admin@example.com")
        .build();

    // Create a test node
    let post = NodeFixture::post()
        .with_title("My Post")
        .build();

    // Create a test product
    let product = ProductFixture::new()
        .with_name("Test Product")
        .with_price(99.99)
        .build();
}
```

### Security Context Helpers

```rust
use rustok_test_utils::helpers::{admin_context, customer_context, super_admin_context};

#[test]
fn test_with_security_context() {
    let admin = admin_context();
    let customer = customer_context();
    let super_admin = super_admin_context();
}
```

### Mock External Services

```rust
use rustok_test_utils::mocks::{MockPaymentGateway, MockEmailService};

#[tokio::test]
async fn test_payment_processing() {
    let gateway = MockPaymentGateway::new().await;
    gateway.configure_successful_payment("tok_visa", "txn_123").await;
    
    // Make payment request to gateway.url()
    // ...
    
    // Verify transaction
    let txn = gateway.get_transaction("tok_visa").unwrap();
    assert_eq!(txn.transaction_id, "txn_123");
}

#[tokio::test]
async fn test_email_sending() {
    let email_service = MockEmailService::new().await;
    email_service.mount().await;
    
    // Send email to email_service.url()
    // ...
    
    // Verify email was sent
    assert!(email_service.was_sent_to("user@example.com"));
    assert_eq!(email_service.sent_count(), 1);
}
```

## Modules

- `database` - Database testing utilities with migrations and cleanup
- `fixtures` - Test data builders for entities
- `mocks` - Mock implementations of external services
- `test_app` - Test application wrapper for integration tests

## License

MIT
