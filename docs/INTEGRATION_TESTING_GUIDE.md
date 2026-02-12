# Integration Testing Guide

> **Version:** 1.0  
> **Last Updated:** 2026-02-12  
> **Applies to:** RusToK Server Integration Tests

---

## Overview

This guide covers the integration testing framework for the RusToK project, focusing on end-to-end testing of critical business flows.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Running Tests](#running-tests)
- [Writing Tests](#writing-tests)
- [Test Utilities](#test-utilities)
- [Mock Services](#mock-services)
- [CI/CD Integration](#cicd-integration)
- [Best Practices](#best-practices)
- [Troubleshooting](#troubleshooting)

---

## Architecture

### Test Structure

```
apps/server/tests/
├── integration/
│   ├── order_flow_test.rs      # Order lifecycle tests
│   ├── content_flow_test.rs    # Content management tests
│   └── event_flow_test.rs      # Event propagation tests
├── tenant_cache_v2_test.rs     # Tenant cache tests
├── module_lifecycle.rs         # Module lifecycle tests
└── multi_tenant_isolation_test.rs
```

### Test Categories

| Category | Purpose | Example |
|----------|---------|---------|
| **Unit Tests** | Test individual functions/modules | `cargo test --lib` |
| **Integration Tests** | Test complete flows with real services | `cargo test --test integration` |
| **E2E Tests** | Test from user perspective | Requires running server |

---

## Running Tests

### Prerequisites

```bash
# PostgreSQL running locally
docker-compose up -d postgres

# Or use your local PostgreSQL
export DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"
```

### Run All Tests

```bash
# Run all tests (unit + integration)
cargo test --workspace --all-features

# Run only integration tests
cargo test --test integration

# Run specific test file
cargo test --test order_flow_test

# Run with output
cargo test --test order_flow_test -- --nocapture
```

### Run Tests Requiring Server

```bash
# Start the test server first
cargo run -p rustok-server

# In another terminal, run integration tests
cargo test --test '*' -- --ignored
```

### Run Tests with Mock Services

```bash
# Set environment variables
export USE_MOCK_PAYMENT=true
export MOCK_PAYMENT_AUTO_APPROVE=true

# Run tests
cargo test --test integration
```

---

## Writing Tests

### Test Structure

```rust
use rustok_test_utils::*;
use rustok_commerce::dto::{CreateProductInput, CreateOrderInput};

#[tokio::test]
async fn test_complete_order_flow() {
    // 1. Setup
    let app = spawn_test_app().await;
    
    // 2. Execute
    let product = app
        .create_product(test_product_input())
        .await
        .expect("Failed to create product");
    
    let order = app
        .create_order(CreateOrderInput {
            customer_id: test_customer_id(),
            items: vec![/* ... */],
        })
        .await
        .expect("Failed to create order");
    
    // 3. Verify
    assert_eq!(order.status, OrderStatus::Draft.to_string());
    assert_eq!(order.total, expected_total);
}
```

### Test Categories

#### Happy Path Tests

```rust
#[tokio::test]
async fn test_order_creation_success() {
    let app = spawn_test_app().await;
    
    let order = app
        .create_order(valid_order_input())
        .await
        .expect("Should create order");
    
    assert_eq!(order.status, "Draft");
}
```

#### Error Handling Tests

```rust
#[tokio::test]
async fn test_order_creation_invalid_product() {
    let app = spawn_test_app().await;
    
    let result = app
        .create_order(invalid_order_input())
        .await;
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), TestAppError::ApiError { .. }));
}
```

#### State Transition Tests

```rust
#[tokio::test]
async fn test_order_state_transitions() {
    let app = spawn_test_app().await;
    let product = app.create_product(test_product_input()).await.unwrap();
    
    // Draft
    let order = app.create_order(/* ... */).await.unwrap();
    assert_eq!(order.status, "Draft");
    
    // Draft → PendingPayment
    let order = app.submit_order(order.id).await.unwrap();
    assert_eq!(order.status, "PendingPayment");
    
    // PendingPayment → Paid
    app.process_payment(order.id, test_payment_input()).await.unwrap();
    let order = app.get_order(order.id).await.unwrap();
    assert_eq!(order.status, "Paid");
}
```

---

## Test Utilities

### Fixtures

```rust
use rustok_test_utils::fixtures::*;

// ID generators
let id = test_uuid();
let customer_id = test_customer_id();

// Domain fixtures
let product_input = test_product_input();
let order_input = test_order_input();
let node_input = test_node_input();

// HTTP fixtures
let client = test_http_client();
let auth_header = test_auth_header();
```

### TestApp API

```rust
let app = spawn_test_app().await;

// Content operations
let node = app.create_node(input).await?;
let node = app.get_node(id).await?;
let node = app.publish_node(id).await?;
let nodes = app.search_nodes(query).await?;

// Commerce operations
let product = app.create_product(input).await?;
let order = app.create_order(input).await?;
let order = app.submit_order(id).await?;
let payment = app.process_payment(order_id, input).await?;

// Event operations
let events = app.get_events_for_order(order_id).await;
let events = app.get_outbox_events().await;
```

---

## Mock Services

### Mock Payment Service

```rust
use rustok_test_utils::mock_payment::MockPaymentService;

let payment_service = MockPaymentService::new();

// Process payment
let result = payment_service
    .process_payment(order_id, 1000, "USD", "tok_test_visa")
    .await;

assert!(result.is_ok());

// Test failure scenario
let result = payment_service
    .process_payment(order_id, 1000, "USD", "tok_fail")
    .await;

assert!(result.is_err());
```

### Custom Mock Configuration

```rust
let payment_service = MockPaymentService::with_tokens(
    vec!["tok_custom_ok".to_string()],
    vec!["tok_custom_fail".to_string()],
);

// Disable service to test error handling
payment_service.disable().await;
```

---

## CI/CD Integration

### GitHub Actions

The integration tests run automatically in CI:

```yaml
- name: Run integration tests
  run: cargo test --package rustok-server --test '*' -- --ignored
```

### Local CI Simulation

```bash
# Run the same checks as CI
make ci-check

# Or manually:
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace --all-features
cargo test --test '*' -- --ignored
```

---

## Best Practices

### 1. Test Independence

```rust
// ❌ Bad: Tests depend on each other
#[tokio::test]
async fn test_create() { /* creates resource */ }

#[tokio::test]
async fn test_update() { /* assumes resource exists */ }

// ✅ Good: Each test is independent
#[tokio::test]
async fn test_create_and_update() {
    let app = spawn_test_app().await;
    let resource = app.create_resource().await?;
    let updated = app.update_resource(resource.id).await?;
}
```

### 2. Descriptive Assertions

```rust
// ❌ Bad
assert_eq!(order.total, 1000);

// ✅ Good
assert_eq!(
    order.total, 1000,
    "Order total should be $10.00 for 1 item at $10.00 each"
);
```

### 3. Use Test Fixtures

```rust
// ❌ Bad: Hardcoded test data
let input = CreateProductInput {
    sku: "TEST-123".to_string(),
    title: "Test Product".to_string(),
    // ...
};

// ✅ Good: Use fixtures
let input = test_product_input();
```

### 4. Handle Async Properly

```rust
// ❌ Bad: Not waiting for async operations
let result = app.create_order(input);
assert!(result.is_ok());

// ✅ Good: Properly await async operations
let result = app.create_order(input).await;
assert!(result.is_ok());
```

### 5. Clean Up Resources

```rust
#[tokio::test]
async fn test_with_cleanup() {
    let app = spawn_test_app().await;
    
    // Create test resource
    let resource = app.create_resource().await?;
    
    // Test operations
    // ...
    
    // Cleanup (if needed)
    app.delete_resource(resource.id).await.ok();
}
```

---

## Troubleshooting

### Common Issues

#### Database Connection Failed

```
Error: DatabaseError("connection refused")
```

**Solution:**
```bash
# Check PostgreSQL is running
docker-compose up -d postgres

# Verify connection string
export DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"
```

#### Server Not Available

```
Error: RequestError("connection refused")
```

**Solution:**
```bash
# Start the server
cargo run -p rustok-server

# Or skip tests requiring server
cargo test --lib
```

#### Test Timeouts

```
Test timed out after 60s
```

**Solution:**
```rust
// Increase timeout in test
let app = TestApp::builder()
    .timeout(Duration::from_secs(120))
    .build()
    .await?;
```

#### Port Already in Use

```
Error: Address already in use (os error 48)
```

**Solution:**
```bash
# Kill existing server
pkill -f rustok-server

# Or use different port
export TEST_SERVER_URL="http://localhost:3001"
```

---

## Coverage Reporting

### Generate Coverage Report

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate LCOV report
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

# Generate HTML report
cargo llvm-cov --workspace --all-features --html
```

### View Coverage

Open `target/llvm-cov/html/index.html` in your browser.

---

## Related Documentation

- [Architecture Improvement Plan](../ARCHITECTURE_IMPROVEMENT_PLAN.md)
- [Sprint 4 Progress](../SPRINT_4_PROGRESS.md)
- [Test Utilities](../crates/rustok-test-utils/)
- [State Machine Guide](STATE_MACHINE_GUIDE.md)
- [Error Handling Guide](ERROR_HANDLING_GUIDE.md)

---

## Changelog

| Date | Version | Changes |
|------|---------|---------|
| 2026-02-12 | 1.0 | Initial documentation |

---

**Questions?** Contact the development team or open an issue.
