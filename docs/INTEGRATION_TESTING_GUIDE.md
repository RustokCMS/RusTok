# Integration Testing Guide

This guide covers how to write and run integration tests for the RusToK platform.

---

## Overview

Integration tests in RusToK verify end-to-end functionality by:
- Testing complete request/response cycles
- Verifying database state changes
- Confirming event propagation
- Validating state machine transitions

---

## Quick Start

### Running Integration Tests

```bash
# Start PostgreSQL (required for integration tests)
docker-compose up -d postgres

# Run all integration tests
cargo test --package rustok-server --test integration -- --ignored

# Run a specific test
cargo test --package rustok-server --test integration test_complete_order_flow -- --ignored

# Run with output visible
cargo test --package rustok-server --test integration -- --ignored --nocapture
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TEST_DATABASE_URL` | `postgres://postgres:password@localhost:5432/rustok_test` | PostgreSQL connection URL |
| `TEST_SERVER_URL` | `http://localhost:3000` | Test server base URL |
| `TEST_AUTH_TOKEN` | `test_token` | Authentication token for tests |
| `TEST_TENANT_ID` | `test-tenant` | Default tenant identifier |
| `TEST_USER_ID` | Auto-generated | Default user UUID |

---

## Test Structure

### Test Organization

```
apps/server/tests/
├── integration/
│   ├── order_flow_test.rs    # Order lifecycle tests
│   ├── content_flow_test.rs  # Content/Node tests
│   └── event_flow_test.rs    # Event propagation tests
├── module_lifecycle.rs       # Module system tests
├── multi_tenant_isolation_test.rs
└── tenant_cache_*.rs         # Tenant cache tests
```

### Writing Integration Tests

```rust
use rustok_test_utils::*;
use rustok_commerce::dto::{CreateProductInput, CreateOrderInput, OrderItemInput};

#[tokio::test]
#[ignore] // Requires test server
async fn test_complete_order_flow() {
    // 1. Spawn test application
    let app = spawn_test_app().await;
    
    // 2. Create test data
    let product = app
        .create_product(CreateProductInput {
            sku: "TEST-001".to_string(),
            title: "Test Product".to_string(),
            price: 1000,
            currency: "USD".to_string(),
            inventory: 100,
            ..Default::default()
        })
        .await
        .expect("Failed to create product");
    
    // 3. Verify response
    assert_eq!(product.sku, "TEST-001");
    assert_eq!(product.price, 1000);
    
    // 4. Create order
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: 2,
                price: Some(1000),
            }],
        })
        .await
        .expect("Failed to create order");
    
    // 5. Verify order state
    assert_eq!(order.total, 2000); // 2 * $10.00
    
    // 6. Verify events
    let events = app.get_events_for_order(order.id).await;
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderCreated { .. })));
}
```

---

## Test Utilities

### TestApp

The `TestApp` struct provides a wrapper for making API calls:

```rust
pub struct TestApp {
    pub db: Arc<DatabaseConnection>,     // Database access
    pub client: reqwest::Client,          // HTTP client
    pub base_url: String,                 // Server URL
    pub auth_token: String,               // Auth token
    pub tenant_id: String,                // Tenant ID
    pub events: Arc<Mutex<Vec<DomainEvent>>>, // Captured events
    pub user_id: Uuid,                    // User ID
}
```

### Available Operations

#### Content Operations
- `create_node(input)` - Create a content node
- `get_node(id)` - Retrieve a node by ID
- `publish_node(id)` - Publish a draft node
- `add_translation(id, locale, input)` - Add a translation
- `search_nodes(query)` - Search for nodes

#### Commerce Operations
- `create_product(input)` - Create a product
- `get_product(id)` - Retrieve a product
- `create_order(input)` - Create an order
- `get_order(id)` - Retrieve an order
- `submit_order(id)` - Submit an order
- `process_payment(id, input)` - Process payment
- `search_orders(query)` - Search for orders

#### Event Operations
- `get_events_for_node(id)` - Get events for a node
- `get_events_for_order(id)` - Get events for an order
- `get_outbox_events()` - Get all outbox events
- `get_relayed_events()` - Get relayed event count

### Fixtures

Pre-defined test data generators:

```rust
use rustok_test_utils::fixtures::*;

// IDs
test_uuid();                          // Random UUID
test_deterministic_uuid(42);          // Deterministic UUID
test_user_id();                       // Standard test user
test_customer_id();                   // Standard test customer
test_tenant_id();                     // Standard test tenant

// Content
test_node_input();                    // Default node input
test_node_input_with_title("Title");  // Node with custom title
test_body_input();                    // Default body input

// Commerce
test_product_input();                 // Default product
test_product_input_with_sku("SKU");   // Product with SKU
test_order_input();                   // Default order
test_payment_input();                 // Default payment
```

---

## Test Server

### Spawning a Test Server

```rust
use rustok_test_utils::test_server::*;

#[tokio::test]
async fn test_with_server() {
    // Default configuration
    let server = spawn_test_server_default().await;
    
    // Custom configuration
    let config = TestServerConfig {
        database_url: Some("postgres://...".to_string()),
        port: Some(8080),
        run_migrations: true,
        tenant_id: "my-tenant".to_string(),
        auth_token: "my-token".to_string(),
    };
    let server = spawn_test_server(config).await;
    
    // Make requests
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health", server.base_url()))
        .send()
        .await;
    
    // Cleanup
    server.shutdown().await;
}
```

### Test Server with PostgreSQL (testcontainers)

```rust
#[tokio::test]
async fn test_with_postgres() {
    use rustok_test_utils::test_server::postgres::*;
    
    let (server, _docker) = spawn_test_server_with_postgres().await;
    
    // Run tests...
    
    server.shutdown().await;
}
```

---

## Best Practices

### 1. Test Isolation

Each test should be independent:

```rust
#[tokio::test]
#[ignore]
async fn test_isolated() {
    let app = spawn_test_app().await;
    
    // Create unique test data
    let unique_sku = format!("TEST-{}", test_uuid());
    
    let product = app
        .create_product(test_product_input_with_sku(&unique_sku))
        .await
        .expect("Failed to create product");
    
    // Test only uses data created in this test
    assert_eq!(product.sku, unique_sku);
}
```

### 2. Cleanup

Tests should clean up after themselves:

```rust
#[tokio::test]
#[ignore]
async fn test_with_cleanup() {
    let app = spawn_test_app().await;
    
    // Test code...
    
    // Optional: explicit cleanup
    app.shutdown().await;
}
```

### 3. Assertions

Make specific assertions:

```rust
// Good
assert_eq!(order.status, OrderStatus::Paid.to_string());
assert_eq!(product.inventory, 98); // 100 - 2 items

// Avoid vague assertions
assert!(order.status.contains("paid")); // Too vague
```

### 4. Error Handling

Always handle errors explicitly:

```rust
// Good
let result = app.create_order(invalid_input).await;
assert!(result.is_err(), "Should fail with invalid input");

// Better - check specific error
match result {
    Err(TestAppError::ApiError { status, .. }) => {
        assert_eq!(status, 400);
    }
    _ => panic!("Expected validation error"),
}
```

### 5. Event Verification

Verify events are emitted correctly:

```rust
let events = app.get_events_for_order(order.id).await;

// Check specific event types
assert!(
    events.iter().any(|e| matches!(e, DomainEvent::OrderCreated { .. })),
    "OrderCreated event should be emitted"
);

// Check event order
let created_idx = events.iter().position(|e| matches!(e, DomainEvent::OrderCreated { .. }));
let paid_idx = events.iter().position(|e| matches!(e, DomainEvent::OrderPaid { .. }));
assert!(created_idx < paid_idx, "OrderCreated should come before OrderPaid");
```

---

## CI/CD Integration

Integration tests run automatically in CI:

```yaml
# .github/workflows/ci.yml
integration-tests:
  runs-on: ubuntu-latest
  services:
    postgres:
      image: postgres:16
      env:
        POSTGRES_USER: postgres
        POSTGRES_PASSWORD: postgres
        POSTGRES_DB: rustok_test
      ports:
        - 5432:5432
  steps:
    - uses: actions/checkout@v4
    - name: Run migrations
      run: sea-orm-cli migrate up -d apps/server/migration
    - name: Run integration tests
      run: cargo test --package rustok-server --test integration -- --ignored
```

---

## Troubleshooting

### Tests Fail with Connection Error

```bash
# Ensure PostgreSQL is running
docker-compose up -d postgres

# Check connection
psql postgres://postgres:password@localhost:5432/rustok_test -c "SELECT 1"

# Run migrations
sea-orm-cli migrate up -d apps/server/migration -u postgres://postgres:password@localhost:5432/rustok_test
```

### Port Already in Use

```bash
# Find process using port 3000
lsof -i :3000

# Kill the process
kill -9 <PID>
```

### Database Locked

```bash
# Reset test database
dropdb rustok_test
createdb rustok_test
sea-orm-cli migrate up -d apps/server/migration -u postgres://postgres:password@localhost:5432/rustok_test
```

---

## Coverage

Current integration test coverage:

| Module | Scenarios | LOC |
|--------|-----------|-----|
| Order Flow | 6 | 380 |
| Content Flow | 9 | 440 |
| Event Flow | 13 | 380 |
| **Total** | **28** | **1200** |

---

## Next Steps

- [ ] Add performance benchmarks
- [ ] Add property-based tests
- [ ] Add security audit tests
- [ ] Add load testing

---

## References

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [SeaORM Migrations](https://www.sea-ql.org/SeaORM/docs/migration/setting-up-migration/)
- [Testcontainers Rust](https://docs.rs/testcontainers/latest/testcontainers/)
