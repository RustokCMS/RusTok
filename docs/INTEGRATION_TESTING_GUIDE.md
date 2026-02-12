# Integration Testing Guide

> Comprehensive guide to writing and running integration tests for the RusToK platform.

## Table of Contents

- [Overview](#overview)
- [Test Architecture](#test-architecture)
- [Getting Started](#getting-started)
- [Writing Integration Tests](#writing-integration-tests)
- [Test Utilities](#test-utilities)
- [Running Tests](#running-tests)
- [CI/CD Integration](#cicd-integration)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## Overview

Integration tests verify that multiple components of the system work together correctly. Unlike unit tests that test individual functions in isolation, integration tests:

- Test complete user flows (e.g., order creation → payment → fulfillment)
- Verify database interactions and data persistence
- Test HTTP API endpoints end-to-end
- Verify event propagation across modules
- Test multi-tenant isolation
- Validate state machine transitions

### Test Coverage

Current integration test suites:

| Test Suite | Scenarios | Coverage | Location |
|------------|-----------|----------|----------|
| Order Flow | 6 | Complete order lifecycle | `apps/server/tests/integration/order_flow_test.rs` |
| Content Flow | 9 | Node CRUD, translations, search | `apps/server/tests/integration/content_flow_test.rs` |
| Event Flow | 13 | Event propagation, outbox, relay | `apps/server/tests/integration/event_flow_test.rs` |

**Total:** 28 integration test scenarios covering core platform functionality.

---

## Test Architecture

### Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Integration Tests                         │
│  (apps/server/tests/integration/*.rs)                       │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ uses
               ▼
┌─────────────────────────────────────────────────────────────┐
│              rustok-test-utils                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Fixtures   │  │   TestApp    │  │  Assertions  │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└──────────────┬──────────────────────────────────────────────┘
               │
               │ interacts with
               ▼
┌─────────────────────────────────────────────────────────────┐
│                Test Server Instance                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   HTTP API   │  │   Database   │  │  Event Bus   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

### Test Layers

1. **Test Case** - Individual test function (`#[tokio::test]`)
2. **TestApp** - HTTP client wrapper with helper methods
3. **Fixtures** - Reusable test data generators
4. **Test Server** - In-memory server instance with test database
5. **Real Components** - Actual application code (controllers, services, database)

---

## Getting Started

### Prerequisites

1. **PostgreSQL** - For integration tests
   ```bash
   # Start via Docker Compose
   docker-compose up -d postgres
   ```

2. **Environment Variables**
   ```bash
   export DATABASE_URL="postgres://postgres:postgres@localhost:5432/rustok_test"
   ```

3. **Dependencies** - Already configured in workspace

### Running Your First Integration Test

```bash
# Run all integration tests
cargo test --test '*' --package rustok-server

# Run specific test suite
cargo test --test order_flow_test --package rustok-server

# Run with output
cargo test --test order_flow_test --package rustok-server -- --nocapture

# Run single test
cargo test test_complete_order_flow --package rustok-server -- --nocapture
```

---

## Writing Integration Tests

### Basic Structure

```rust
//! # My Integration Test Suite
//!
//! Description of what this test suite covers.

use rustok_test_utils::*;
use rustok_commerce::dto::*;

/// Test a specific scenario
#[tokio::test]
async fn test_my_scenario() {
    // 1. Setup - Create test app
    let app = spawn_test_app().await;
    
    // 2. Execute - Perform actions
    let result = app.create_order(/* ... */).await;
    
    // 3. Assert - Verify outcomes
    assert!(result.is_ok());
    
    // 4. Verify side effects (events, database state, etc.)
    let events = app.get_events_for_order(result.unwrap().id).await;
    assert_eq!(events.len(), 1);
}
```

### Example: Complete Order Flow Test

```rust
use rustok_test_utils::*;
use rustok_commerce::dto::{CreateProductInput, CreateOrderInput, OrderItemInput};
use rustok_commerce::entities::OrderStatus;

#[tokio::test]
async fn test_complete_order_flow() {
    let app = spawn_test_app().await;
    
    // Step 1: Create product
    let product = app
        .create_product(CreateProductInput {
            sku: "TEST-001".to_string(),
            title: "Test Product".to_string(),
            description: Some("Test description".to_string()),
            price: 1999, // $19.99
            currency: "USD".to_string(),
            inventory: 100,
            status: None,
        })
        .await
        .expect("Failed to create product");
    
    assert_eq!(product.sku, "TEST-001");
    assert_eq!(product.price, 1999);
    
    // Step 2: Create order
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: 2,
                price: product.price,
            }],
            shipping_address: test_shipping_address(),
            billing_address: None,
        })
        .await
        .expect("Failed to create order");
    
    assert_eq!(order.status, OrderStatus::Draft);
    assert_eq!(order.items.len(), 1);
    assert_eq!(order.total, 3998); // 2 * $19.99
    
    // Step 3: Submit order
    let submitted = app
        .submit_order(order.id)
        .await
        .expect("Failed to submit order");
    
    assert_eq!(submitted.status, OrderStatus::PendingPayment);
    
    // Step 4: Process payment
    let paid = app
        .process_payment(ProcessPaymentInput {
            order_id: order.id,
            payment_method: PaymentMethod::CreditCard {
                token: "tok_visa".to_string(),
            },
            amount: order.total,
        })
        .await
        .expect("Failed to process payment");
    
    assert_eq!(paid.status, OrderStatus::Paid);
    
    // Step 5: Verify events
    let events = app.get_events_for_order(order.id).await;
    assert!(events.iter().any(|e| e.event_type == "OrderCreated"));
    assert!(events.iter().any(|e| e.event_type == "OrderSubmitted"));
    assert!(events.iter().any(|e| e.event_type == "OrderPaid"));
}
```

### Testing Error Cases

```rust
#[tokio::test]
async fn test_order_validation_errors() {
    let app = spawn_test_app().await;
    
    // Test 1: Invalid product ID
    let result = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: Uuid::new_v4(), // Non-existent product
                quantity: 1,
                price: 1000,
            }],
            shipping_address: test_shipping_address(),
            billing_address: None,
        })
        .await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Product not found"));
    
    // Test 2: Negative quantity
    let product = app.create_product(test_product_input()).await.unwrap();
    
    let result = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![OrderItemInput {
                product_id: product.id,
                quantity: -1, // Invalid
                price: 1000,
            }],
            shipping_address: test_shipping_address(),
            billing_address: None,
        })
        .await;
    
    assert!(result.is_err());
}
```

---

## Test Utilities

### TestApp Wrapper

The `TestApp` struct provides high-level methods for interacting with the test server:

```rust
pub struct TestApp {
    pub client: reqwest::Client,
    pub base_url: String,
    pub auth_token: Option<String>,
}

impl TestApp {
    // Content operations
    pub async fn create_node(&self, input: CreateNodeInput) -> Result<Node>;
    pub async fn get_node(&self, id: Uuid) -> Result<Node>;
    pub async fn publish_node(&self, id: Uuid) -> Result<Node>;
    pub async fn add_translation(&self, id: Uuid, input: TranslationInput) -> Result<Node>;
    pub async fn search_nodes(&self, query: &str) -> Result<Vec<Node>>;
    
    // Commerce operations
    pub async fn create_product(&self, input: CreateProductInput) -> Result<Product>;
    pub async fn get_product(&self, id: Uuid) -> Result<Product>;
    pub async fn create_order(&self, input: CreateOrderInput) -> Result<Order>;
    pub async fn submit_order(&self, order_id: Uuid) -> Result<Order>;
    pub async fn process_payment(&self, input: ProcessPaymentInput) -> Result<Order>;
    
    // Event operations
    pub async fn get_events_for_node(&self, node_id: Uuid) -> Vec<DomainEvent>;
    pub async fn get_events_for_order(&self, order_id: Uuid) -> Vec<DomainEvent>;
    pub async fn get_outbox_events(&self) -> Vec<OutboxEvent>;
}
```

### Fixtures

Reusable test data generators in `rustok-test-utils/src/fixtures.rs`:

```rust
// ID generators
pub fn test_uuid() -> Uuid;
pub fn test_node_id() -> Uuid;
pub fn test_tenant_id() -> Uuid;
pub fn test_customer_id() -> Uuid;

// Entity fixtures
pub fn test_create_node_input() -> CreateNodeInput;
pub fn test_create_product_input() -> CreateProductInput;
pub fn test_create_order_input() -> CreateOrderInput;
pub fn test_shipping_address() -> Address;
pub fn test_payment_input() -> ProcessPaymentInput;

// Customizable builders
pub fn node_input_with_title(title: &str) -> CreateNodeInput;
pub fn product_input_with_sku(sku: &str) -> CreateProductInput;
```

### Assertions

Common assertion helpers:

```rust
// Event assertions
pub fn assert_event_exists(events: &[DomainEvent], event_type: &str);
pub fn assert_event_count(events: &[DomainEvent], event_type: &str, count: usize);

// ID assertions
pub fn assert_valid_uuid(id: &Uuid);
pub fn assert_matching_tenant_id(entity: &impl HasTenantId, expected: Uuid);
```

---

## Running Tests

### Local Development

```bash
# Run all integration tests
cargo test --package rustok-server --test '*'

# Run specific test suite
cargo test --test order_flow_test

# Run with detailed output
cargo test --test order_flow_test -- --nocapture --test-threads=1

# Run single test
cargo test test_complete_order_flow -- --nocapture
```

### Test Database Setup

Integration tests automatically set up a test database. You can also manually prepare:

```bash
# Create test database
createdb rustok_test

# Run migrations
cargo run --bin migration -- up

# Or use make targets (if available)
make test-db-setup
```

### Parallel vs Sequential Execution

By default, Cargo runs tests in parallel. For integration tests that share database state:

```bash
# Run sequentially
cargo test --test order_flow_test -- --test-threads=1

# Or use #[serial] attribute from serial_test crate
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_with_shared_state() {
    // ...
}
```

---

## CI/CD Integration

### GitHub Actions

Integration tests run automatically in CI via `.github/workflows/ci.yml`:

```yaml
test:
  name: Tests
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
      options: >-
        --health-cmd pg_isready
        --health-interval 10s
        --health-timeout 5s
        --health-retries 5
  env:
    DATABASE_URL: postgres://postgres:postgres@localhost:5432/rustok_test
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    - run: cargo test --workspace --all-targets --all-features
```

### Coverage Reports

Code coverage is tracked via `cargo-llvm-cov`:

```yaml
coverage:
  name: Code Coverage
  runs-on: ubuntu-latest
  services:
    postgres: # ... same as test job
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: llvm-tools-preview
    - name: Install cargo-llvm-cov
      run: cargo install cargo-llvm-cov --locked
    - run: cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

---

## Best Practices

### 1. Test Isolation

Each test should be independent and not rely on state from other tests:

```rust
// ✅ Good - Each test creates its own data
#[tokio::test]
async fn test_order_creation() {
    let app = spawn_test_app().await;
    let product = app.create_product(test_product_input()).await.unwrap();
    // ...
}

// ❌ Bad - Relies on global state
static SHARED_PRODUCT: OnceCell<Product> = OnceCell::new();

#[tokio::test]
async fn test_order_creation() {
    let product = SHARED_PRODUCT.get().unwrap();
    // ...
}
```

### 2. Use Fixtures

DRY (Don't Repeat Yourself) - use fixtures for common test data:

```rust
// ✅ Good - Use fixtures
#[tokio::test]
async fn test_order() {
    let app = spawn_test_app().await;
    let input = test_create_order_input(); // Reusable fixture
    let order = app.create_order(input).await.unwrap();
}

// ❌ Bad - Duplicate setup code
#[tokio::test]
async fn test_order() {
    let app = spawn_test_app().await;
    let input = CreateOrderInput {
        customer_id: Uuid::new_v4(),
        items: vec![/* ... */],
        // ... lots of fields
    };
    let order = app.create_order(input).await.unwrap();
}
```

### 3. Test Both Happy Path and Error Cases

```rust
#[tokio::test]
async fn test_order_happy_path() {
    // Test successful order creation
}

#[tokio::test]
async fn test_order_invalid_product() {
    // Test error handling for invalid product
}

#[tokio::test]
async fn test_order_insufficient_inventory() {
    // Test error handling for insufficient inventory
}
```

### 4. Verify Side Effects

Don't just test the main result - verify events, database state, etc.:

```rust
#[tokio::test]
async fn test_order_lifecycle() {
    let app = spawn_test_app().await;
    
    // Main action
    let order = app.create_order(test_order_input()).await.unwrap();
    
    // Verify side effects
    let events = app.get_events_for_order(order.id).await;
    assert!(events.iter().any(|e| e.event_type == "OrderCreated"));
    
    let from_db = app.get_order(order.id).await.unwrap();
    assert_eq!(from_db.status, OrderStatus::Draft);
}
```

### 5. Use Descriptive Test Names

```rust
// ✅ Good - Clear what is being tested
#[tokio::test]
async fn test_order_submission_transitions_from_draft_to_pending_payment() {
    // ...
}

// ❌ Bad - Unclear
#[tokio::test]
async fn test_order() {
    // ...
}
```

### 6. Keep Tests Fast

- Use in-memory databases when possible
- Minimize external service calls
- Use mocks for slow dependencies
- Run tests in parallel when safe

### 7. Document Complex Tests

```rust
/// Test the complete order lifecycle including:
/// 1. Product creation
/// 2. Order creation with multiple items
/// 3. Order submission (Draft → PendingPayment)
/// 4. Payment processing (PendingPayment → Paid)
/// 5. Event verification (OrderCreated, OrderSubmitted, OrderPaid)
/// 6. Inventory updates
#[tokio::test]
async fn test_complete_order_flow_with_inventory_updates() {
    // ...
}
```

---

## Troubleshooting

### Test Database Connection Issues

**Problem:** Tests fail with "connection refused" or "database does not exist"

**Solution:**
```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Verify DATABASE_URL
echo $DATABASE_URL

# Create test database
createdb rustok_test

# Run migrations
cargo run --bin migration -- up
```

### Tests Fail in CI but Pass Locally

**Problem:** Integration tests pass locally but fail in GitHub Actions

**Solutions:**
1. **Timing issues** - Add retries or longer timeouts for CI
2. **Database differences** - Ensure same PostgreSQL version (16)
3. **Parallel execution** - Use `--test-threads=1` for flaky tests
4. **Environment variables** - Check CI env vars match local

### Flaky Tests

**Problem:** Tests pass sometimes and fail other times

**Solutions:**
1. **Race conditions** - Add explicit waits or use `#[serial]`
2. **Shared state** - Ensure tests are isolated
3. **Timeouts** - Increase timeouts for slow operations
4. **Random data** - Use deterministic IDs/data in tests

### Out of Memory Errors

**Problem:** Tests fail with OOM when running full suite

**Solutions:**
```bash
# Run tests in smaller batches
cargo test --test order_flow_test
cargo test --test content_flow_test
cargo test --test event_flow_test

# Reduce parallel threads
cargo test -- --test-threads=2

# Increase Docker memory limit
docker-compose up -d postgres --memory=4g
```

### Slow Test Execution

**Problem:** Integration tests take too long to run

**Solutions:**
1. **Optimize database setup** - Use fixtures instead of real DB when possible
2. **Parallel execution** - Enable parallel tests (default)
3. **Selective running** - Run only affected tests during development
4. **Profile tests** - Use `cargo test -- --nocapture --test-threads=1 --show-output`

---

## Advanced Topics

### Custom Test Server Configuration

```rust
use rustok_test_utils::TestAppBuilder;

#[tokio::test]
async fn test_with_custom_config() {
    let app = TestAppBuilder::new()
        .with_port(8081)
        .with_auth_enabled(false)
        .with_event_bus(MockEventBus::new())
        .build()
        .await;
    
    // Test with custom configuration
}
```

### Testing Multi-Tenant Scenarios

```rust
#[tokio::test]
async fn test_tenant_isolation() {
    let app = spawn_test_app().await;
    
    let tenant1 = Uuid::new_v4();
    let tenant2 = Uuid::new_v4();
    
    // Create data for tenant1
    app.set_tenant(tenant1);
    let node1 = app.create_node(test_node_input()).await.unwrap();
    
    // Create data for tenant2
    app.set_tenant(tenant2);
    let node2 = app.create_node(test_node_input()).await.unwrap();
    
    // Verify isolation
    app.set_tenant(tenant1);
    let nodes = app.search_nodes("").await.unwrap();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].id, node1.id);
}
```

### Testing Event Propagation

```rust
#[tokio::test]
async fn test_event_propagation_to_read_model() {
    let app = spawn_test_app().await;
    
    // Create entity (triggers event)
    let node = app.create_node(test_node_input()).await.unwrap();
    
    // Wait for event processing
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Verify read model updated
    let from_index = app.search_nodes(&node.title).await.unwrap();
    assert_eq!(from_index.len(), 1);
    assert_eq!(from_index[0].id, node.id);
}
```

---

## Resources

### Internal Documentation
- [rustok-test-utils README](../crates/rustok-test-utils/README.md) - Test utilities documentation
- [SPRINT_4_PROGRESS.md](../SPRINT_4_PROGRESS.md) - Current sprint progress
- [ARCHITECTURE_IMPROVEMENT_PLAN.md](../ARCHITECTURE_IMPROVEMENT_PLAN.md) - Overall architecture plan

### External Resources
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html) - Official Rust testing guide
- [Tokio Testing](https://tokio.rs/tokio/topics/testing) - Async testing with Tokio
- [reqwest Documentation](https://docs.rs/reqwest/) - HTTP client for tests
- [cargo-nextest](https://nexte.st/) - Next-generation test runner

---

## Contributing

When adding new integration tests:

1. **Choose the right test suite** - Add to existing suite or create new one
2. **Follow naming conventions** - `test_<scenario>_<expected_outcome>`
3. **Document complex scenarios** - Add doc comments explaining what's tested
4. **Add to progress tracking** - Update `SPRINT_4_PROGRESS.md` with new tests
5. **Verify CI passes** - Ensure tests pass in GitHub Actions

### Test Suite Template

```rust
//! # <Module> Integration Tests
//!
//! Tests the complete <module> lifecycle:
//! 1. <Step 1>
//! 2. <Step 2>
//! 3. <Step 3>

use rustok_test_utils::*;
use rustok_<module>::dto::*;

/// Test <scenario> - <expected outcome>
#[tokio::test]
async fn test_<scenario>() {
    // Setup
    let app = spawn_test_app().await;
    
    // Execute
    // ...
    
    // Assert
    // ...
    
    // Verify side effects
    // ...
}
```

---

**Last Updated:** 2026-02-12  
**Version:** Sprint 4 - Task 4.1  
**Status:** ✅ Complete
