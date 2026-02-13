# rustok-test-utils

Test utilities for the RusToK platform. This crate provides fixtures, mocks, and helpers for writing tests across all RusToK modules.

## Features

- **Database utilities** - Setup test databases (with optional explicit migrations)
- **Mock event bus** - Record and verify event publishing
- **Test fixtures** - Builder patterns for users, tenants, nodes, products
- **Helper functions** - Common testing utilities and macros

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

## Modules

- `db` - Database testing utilities
- `events` - Mock event bus for testing
- `fixtures` - Test data builders
- `helpers` - Common testing utilities and macros

## License

MIT
