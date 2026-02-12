# Integration Testing Guide

This guide explains how to run and write integration tests for the RusToK platform.

## Overview

Integration tests verify that different components of the system work together correctly. They test complete workflows including:

- Order lifecycle (create → submit → pay → ship)
- Content lifecycle (create → translate → publish → search)
- Event propagation (publish → persist → relay → consume)

## Test Structure

Integration tests are located in:
```
apps/server/tests/integration/
```

Key files:
- `order_flow_test.rs` - Order-related workflows
- `content_flow_test.rs` - Content/node workflows  
- `event_flow_test.rs` - Event system workflows

## Test Utilities

The `rustok-test-utils` crate provides helper functions and fixtures:

- `TestApp` - Main test application wrapper
- `spawn_test_app()` - Create a test application instance
- Fixtures for test data (products, nodes, users, etc.)
- Event capture and verification helpers

## Running Tests

### Locally

1. **Start dependencies**: Ensure PostgreSQL is running
2. **Set environment variables**:
   ```bash
   export DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"
   export TEST_DATABASE_URL="postgres://postgres:password@localhost:5432/rustok_test"
   export TEST_SERVER_URL="http://localhost:3000"
   export TEST_AUTH_TOKEN="test_token"
   export TEST_TENANT_ID="test-tenant"
   ```

3. **Start the server**:
   ```bash
   cd apps/server
   cargo run --release
   ```

4. **Run integration tests** (in another terminal):
   ```bash
   cargo test --package rustok-server --test integration
   ```

### In CI/CD

The CI/CD pipeline automatically runs integration tests in the `integration-tests` job:

1. Starts PostgreSQL service
2. Builds and starts the test server
3. Runs all integration tests
4. Cleans up resources

## Writing Tests

### Test Structure Example

```rust
use rustok_test_utils::*;

#[tokio::test]
async fn test_order_flow() {
    // Setup
    let app = spawn_test_app().await;
    
    // Create product
    let product = app.create_product(test_product_input()).await.unwrap();
    
    // Create order
    let order = app.create_order(test_order_input(product.id)).await.unwrap();
    
    // Submit order
    let submitted = app.submit_order(order.id).await.unwrap();
    
    // Verify state
    assert_eq!(submitted.status, OrderStatus::PendingPayment);
    
    // Process payment
    let payment = app.process_payment(order.id, test_payment_input()).await.unwrap();
    assert!(payment.success);
    
    // Verify final state
    let final_order = app.get_order(order.id).await.unwrap();
    assert_eq!(final_order.status, OrderStatus::Paid);
}
```

### Best Practices

1. **Test complete workflows**: Test from creation to final state
2. **Verify events**: Check that domain events are emitted correctly
3. **Test edge cases**: Include validation errors and failure scenarios
4. **Use fixtures**: Leverage test utilities for common test data
5. **Clean up**: Tests should clean up after themselves

## Test Coverage

Current integration test coverage includes:

### Order Flow (6 tests)
- Complete order lifecycle
- Multiple items
- Validation scenarios
- Payment failures
- Order retrieval and search
- State transitions

### Content Flow (9 tests)
- Complete node lifecycle
- Different content types
- Translations
- Search functionality
- Validation
- State transitions
- Retrieval
- Slug uniqueness
- Body formats

### Event Flow (13 tests)
- Event propagation
- Outbox persistence
- Event relay
- Event ordering
- Correlation IDs
- Error handling
- Cross-module events
- Tenant isolation
- Event validation
- Payload size limits
- Event replay
- Deduplication
- Batching

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `TEST_DATABASE_URL` | Test database connection | `postgres://postgres:password@localhost:5432/rustok_test` |
| `TEST_SERVER_URL` | Test server base URL | `http://localhost:3000` |
| `TEST_AUTH_TOKEN` | Authentication token | `test_token` |
| `TEST_TENANT_ID` | Tenant identifier | `test-tenant` |
| `TEST_USER_ID` | User ID for test operations | Random UUID |

## Troubleshooting

### Server not starting
- Check PostgreSQL is running
- Verify database credentials
- Check for port conflicts

### Tests timing out
- Increase timeout in `TestApp::new()`
- Check server logs for errors
- Ensure all dependencies are available

### Event not captured
- Verify event subscription is set up
- Check event filtering logic
- Ensure events are being published correctly

## CI/CD Integration

The integration tests are automatically run in GitHub Actions:

```yaml
integration-tests:
  name: Integration Tests
  runs-on: ubuntu-latest
  needs: build-server
  services:
    postgres:
      image: postgres:16
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - name: Start test server
      run: |
        cd apps/server
        cargo build --release
        cargo run --release &
        # Wait for server readiness
    - name: Run integration tests
      run: cargo test --package rustok-server --test integration
```

## Future Improvements

- Add test database migrations
- Mock external services (payment gateway, email, etc.)
- Performance regression testing
- Parallel test execution
- Test coverage reporting
- Automated test data seeding
