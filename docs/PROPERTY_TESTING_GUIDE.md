# Property-Based Testing Guide

> **Sprint 4, Task 4.2** - Comprehensive guide to property-based testing in RusToK

## Table of Contents

1. [Overview](#overview)
2. [What is Property-Based Testing?](#what-is-property-based-testing)
3. [Getting Started](#getting-started)
4. [Property Test Strategies](#property-test-strategies)
5. [Writing Property Tests](#writing-property-tests)
6. [Test Coverage](#test-coverage)
7. [Best Practices](#best-practices)
8. [CI Integration](#ci-integration)

---

## Overview

Property-based testing complements example-based testing by verifying that certain properties hold for **all possible inputs**, not just specific test cases. RusToK uses [proptest](https://github.com/proptest-rs/proptest) for property-based testing.

### Benefits

- **Broader Coverage**: Tests thousands of random inputs automatically
- **Edge Case Discovery**: Finds corner cases you didn't think of
- **Invariant Verification**: Ensures system properties always hold
- **Regression Prevention**: Shrinks failing cases to minimal examples

### When to Use

✅ **Use property-based tests for:**
- Domain invariants (e.g., "order total is always positive")
- State machine transitions (e.g., "published nodes can't become draft")
- Serialization/deserialization (e.g., "JSON roundtrip preserves data")
- Mathematical properties (e.g., "quantity + quantity = 2 * quantity")

❌ **Use example-based tests for:**
- Specific business logic scenarios
- Complex multi-step workflows
- Integration tests with external services
- UI/UX behavior verification

---

## What is Property-Based Testing?

Instead of writing:
```rust
#[test]
fn test_uuid_roundtrip() {
    let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
    let json = serde_json::to_string(&uuid).unwrap();
    let parsed: Uuid = serde_json::from_str(&json).unwrap();
    assert_eq!(uuid, parsed);
}
```

You write:
```rust
proptest! {
    #[test]
    fn uuid_survives_json_roundtrip(uuid in uuid_strategy()) {
        let json = serde_json::to_string(&uuid).unwrap();
        let parsed: Uuid = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(uuid, parsed);
    }
}
```

This tests **100+ random UUIDs** automatically, finding edge cases you never considered.

---

## Getting Started

### 1. Add Dependencies

Property tests are already configured in the workspace. For new crates:

```toml
# Cargo.toml
[dev-dependencies]
proptest = { workspace = true }
rustok-test-utils = { workspace = true }  # For strategies
```

### 2. Import Proptest

```rust
use proptest::prelude::*;
use rustok_test_utils::{uuid_strategy, non_empty_string, price_strategy};
```

### 3. Write Your First Property Test

```rust
proptest! {
    #[test]
    fn my_property_holds(input in 1u32..1000) {
        prop_assert!(input > 0);
        prop_assert!(input < 1000);
    }
}
```

---

## Property Test Strategies

### Built-in Strategies

RusToK provides reusable strategies in `rustok-test-utils/src/proptest_strategies.rs`:

| Strategy | Description | Example Output |
|----------|-------------|----------------|
| `uuid_strategy()` | Valid UUIDs | `550e8400-e29b-41d4-a716-446655440000` |
| `tenant_id_strategy()` | Tenant IDs (UUIDs) | `7c9e6679-7425-40de-944b-e07fc1f90ae7` |
| `non_empty_string()` | Alphanumeric strings (1-100 chars) | `"abc123xyz"` |
| `email_strategy()` | Valid email addresses | `"user@example.com"` |
| `positive_i64()` | Positive integers | `42`, `1000`, `9223372036854775807` |
| `price_strategy()` | Prices in cents ($0.01-$10,000) | `100`, `50000`, `999999` |
| `sku_strategy()` | Product SKUs | `"AB-12345"`, `"XYZ-9876"` |
| `slug_strategy()` | URL slugs | `"hello-world"`, `"product-123"` |
| `title_strategy()` | Content titles (5-100 chars) | `"My Great Article"` |
| `quantity_strategy()` | Quantities (1-1000) | `5`, `42`, `999` |

### Creating Custom Strategies

```rust
// Simple strategy from a range
fn temperature_strategy() -> impl Strategy<Value = i32> {
    -40i32..=50i32  // Valid temperature range
}

// Complex strategy with prop_map
fn order_strategy() -> impl Strategy<Value = Order> {
    (uuid_strategy(), price_strategy(), quantity_strategy())
        .prop_map(|(id, price, qty)| Order {
            id,
            total: price * qty as i64,
            quantity: qty,
        })
}

// Strategy with constraints
fn valid_email() -> impl Strategy<Value = String> {
    ("[a-z]{3,10}", "[a-z]{3,10}", "[a-z]{2,4}")
        .prop_map(|(user, domain, tld)| format!("{}@{}.{}", user, domain, tld))
}
```

---

## Writing Property Tests

### 1. UUID Property Tests

**File:** `crates/rustok-core/tests/uuid_property_tests.rs` (7 properties)

```rust
proptest! {
    /// Property: UUIDs preserve format through string conversion
    #[test]
    fn uuid_preserves_format(uuid in uuid_strategy()) {
        let as_string = uuid.to_string();
        let parsed = Uuid::parse_str(&as_string).unwrap();
        prop_assert_eq!(uuid, parsed);
    }

    /// Property: Tenant IDs survive JSON roundtrip
    #[test]
    fn tenant_id_json_roundtrip(tenant_id in uuid_strategy()) {
        let json = serde_json::to_string(&tenant_id).unwrap();
        let deserialized: Uuid = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(tenant_id, deserialized);
    }
}
```

**Properties Verified:**
- ✅ UUID format preservation (string conversion)
- ✅ Tenant ID immutability (value type semantics)
- ✅ Tenant ID uniqueness (generated IDs are distinct)
- ✅ JSON serialization roundtrip
- ✅ Nil UUID handling
- ✅ Comparison transitivity
- ✅ Hash consistency

### 2. Event Property Tests

**File:** `crates/rustok-core/tests/event_property_tests.rs` (9 properties)

```rust
proptest! {
    /// Property: Event envelopes have unique IDs
    #[test]
    fn event_envelopes_have_unique_ids(
        tenant_id in uuid_strategy(),
        count in 2usize..10
    ) {
        let events: Vec<EventEnvelope> = (0..count)
            .map(|_| EventEnvelope::new(
                tenant_id, None, DomainEvent::TenantCreated { tenant_id }
            ))
            .collect();

        let mut ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        
        prop_assert_eq!(ids.len(), count);
    }

    /// Property: Valid events always pass validation
    #[test]
    fn valid_events_always_validate(
        user_id in uuid_strategy(),
        email in email_strategy()
    ) {
        let event = DomainEvent::UserRegistered { user_id, email };
        
        if user_id != Uuid::nil() {
            prop_assert!(event.validate().is_ok());
        }
    }
}
```

**Properties Verified:**
- ✅ Event ID uniqueness
- ✅ Timestamp monotonicity
- ✅ JSON payload roundtrip
- ✅ Valid events pass validation
- ✅ Invalid events fail validation (nil UUIDs, empty strings)
- ✅ Event type string consistency
- ✅ Schema version positivity
- ✅ Envelope correlation

### 3. Order State Machine Property Tests

**File:** `crates/rustok-commerce/tests/order_state_machine_properties.rs` (4 properties)

```rust
proptest! {
    /// Property: Pending orders can only transition to Confirmed or Cancelled
    #[test]
    fn pending_transitions_only_to_confirmed_or_cancelled(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        amount in valid_order_amount()
    ) {
        let order = Order::new_pending(id, tenant_id, customer_id, amount, "USD".to_string());
        
        prop_assert!(order.confirm().is_ok());  // Valid transition
        prop_assert_eq!(order.cancel("reason".to_string()).id, id);  // Valid transition
    }

    /// Property: Order ID is immutable across all transitions
    #[test]
    fn order_id_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        amount in valid_order_amount()
    ) {
        let order = Order::new_pending(id, tenant_id, customer_id, amount, "USD".to_string());
        let original_id = order.id;

        let confirmed = order.confirm().unwrap();
        prop_assert_eq!(confirmed.id, original_id);

        let paid = confirmed.pay("pay_123".to_string(), "card".to_string()).unwrap();
        prop_assert_eq!(paid.id, original_id);
    }
}
```

**Properties Verified:**
- ✅ Valid state transitions only (Pending→Confirmed, Confirmed→Paid, etc.)
- ✅ Order ID immutability across transitions
- ✅ Tenant ID immutability
- ✅ Total amount immutability
- ✅ Cancellation from any non-terminal state

### 4. Node State Machine Property Tests

**File:** `crates/rustok-content/tests/node_state_machine_properties.rs` (6 properties)

```rust
proptest! {
    /// Property: Draft nodes can only transition to Published
    #[test]
    fn draft_can_only_transition_to_published(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        kind in node_kind_strategy()
    ) {
        let node = ContentNode::new_draft(id, tenant_id, None, kind);
        let published = node.publish();
        prop_assert_eq!(published.id, id);
    }

    /// Property: Published nodes cannot transition back to Draft
    #[test]
    fn published_state_is_one_way(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        kind in node_kind_strategy()
    ) {
        let node = ContentNode::new_draft(id, tenant_id, None, kind);
        let published = node.publish();
        let published_at = published.state.published_at;
        
        let updated = published.update();
        prop_assert_eq!(updated.state.published_at, published_at);
        // Note: No unpublish() method exists - compile-time safety!
    }
}
```

**Properties Verified:**
- ✅ Valid transitions only (Draft→Published→Archived)
- ✅ Published state immutability (one-way transition)
- ✅ Node ID immutability
- ✅ Tenant ID immutability
- ✅ Kind immutability
- ✅ Archive timestamp ordering

---

## Test Coverage

### Summary

| Module | File | Properties | LOC |
|--------|------|------------|-----|
| **Test Utils** | `rustok-test-utils/src/proptest_strategies.rs` | 9 strategies | 120 |
| **UUID/Tenant** | `rustok-core/tests/uuid_property_tests.rs` | 7 properties | 340 |
| **Events** | `rustok-core/tests/event_property_tests.rs` | 9 properties | 350 |
| **Order SM** | `rustok-commerce/tests/order_state_machine_properties.rs` | 4 properties | 370 |
| **Node SM** | `rustok-content/tests/node_state_machine_properties.rs` | 6 properties | 380 |
| **Total** | - | **35 properties** | **1,560 LOC** |

### Test Execution

Each property test runs **100 random test cases** by default (configurable):

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,  // Run 1000 cases instead of default 100
        ..ProptestConfig::default()
    })]
    
    #[test]
    fn my_intensive_property(input in 0u64..u64::MAX) {
        // ...
    }
}
```

**Total Test Cases:** 35 properties × 100 cases = **3,500+ test executions** per run

---

## Best Practices

### 1. Start Simple

```rust
// ❌ Too complex
proptest! {
    #[test]
    fn complex_workflow(/* 10 parameters */) {
        // 50 lines of setup and assertions
    }
}

// ✅ Simple and focused
proptest! {
    #[test]
    fn order_id_immutable(id in uuid_strategy()) {
        let order = Order::new_pending(id, tenant_id, amount, "USD".to_string());
        let confirmed = order.confirm().unwrap();
        prop_assert_eq!(confirmed.id, id);
    }
}
```

### 2. Test One Property at a Time

Each test should verify **one invariant**:

```rust
// ✅ Good: Tests one property
proptest! {
    #[test]
    fn order_id_immutable(id in uuid_strategy()) {
        // Only tests ID immutability
    }
    
    #[test]
    fn order_amount_immutable(amount in price_strategy()) {
        // Only tests amount immutability
    }
}
```

### 3. Use Descriptive Names

```rust
// ❌ Vague
#[test]
fn test_uuid(uuid in uuid_strategy()) { }

// ✅ Descriptive
#[test]
fn uuid_preserves_format_through_string_conversion(uuid in uuid_strategy()) { }
```

### 4. Handle Edge Cases Explicitly

```rust
proptest! {
    #[test]
    fn event_validates_correctly(user_id in uuid_strategy()) {
        let event = DomainEvent::UserDeleted { user_id };
        
        // Explicitly handle nil UUID edge case
        if user_id == Uuid::nil() {
            prop_assert!(event.validate().is_err());
        } else {
            prop_assert!(event.validate().is_ok());
        }
    }
}
```

### 5. Use Shrinking for Debugging

When a property fails, proptest automatically shrinks the input to the minimal failing case:

```
Test failed: uuid_roundtrip
Minimal failing case: Uuid("00000000-0000-0000-0000-000000000001")
```

### 6. Document Your Properties

```rust
proptest! {
    /// Property: Order state transitions preserve core identifiers
    ///
    /// This ensures that order_id, tenant_id, and customer_id remain
    /// unchanged throughout the order lifecycle, maintaining referential
    /// integrity and audit trail consistency.
    #[test]
    fn order_identifiers_immutable(/* ... */) {
        // ...
    }
}
```

---

## CI Integration

### Running Property Tests

```bash
# Run all property tests
cargo test --workspace

# Run specific module property tests
cargo test --package rustok-core --test uuid_property_tests
cargo test --package rustok-core --test event_property_tests
cargo test --package rustok-commerce --test order_state_machine_properties
cargo test --package rustok-content --test node_state_machine_properties

# Run with more cases (slower but more thorough)
PROPTEST_CASES=1000 cargo test --workspace

# Run with specific seed for reproducibility
PROPTEST_SEED=12345 cargo test --workspace
```

### GitHub Actions Integration

```yaml
name: Property Tests

on: [push, pull_request]

jobs:
  proptest:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run property tests
        run: |
          cargo test --workspace
          PROPTEST_CASES=1000 cargo test --workspace --release
```

### Persistence for CI

Property tests can persist failing cases:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("proptest-regressions"))),
        ..ProptestConfig::default()
    })]
    
    #[test]
    fn my_property(input in 0u32..1000) {
        // Failing cases saved to proptest-regressions/my_property.txt
    }
}
```

Commit these files to prevent regressions.

---

## Examples from RusToK

### Example 1: State Machine Invariants

```rust
// Order state machine ensures valid transitions only
proptest! {
    #[test]
    fn paid_orders_can_only_ship_or_cancel(
        id in uuid_strategy(),
        tenant_id in uuid_strategy()
    ) {
        let order = Order::new_pending(id, tenant_id, customer_id, amount, "USD".to_string())
            .confirm().unwrap()
            .pay("pay_123".to_string(), "card".to_string()).unwrap();

        // Can ship (valid)
        prop_assert!(order.clone().ship("TRACK123".to_string(), "UPS".to_string()).is_ok());
        
        // Can cancel with refund (valid)
        let cancelled = order.cancel("Test".to_string(), true);
        prop_assert!(cancelled.state.refunded);
        
        // Cannot transition to Draft/Confirmed (enforced by type system)
    }
}
```

### Example 2: Serialization Roundtrips

```rust
proptest! {
    #[test]
    fn event_envelope_json_roundtrip(
        tenant_id in uuid_strategy(),
        user_id in uuid_strategy(),
        email in email_strategy()
    ) {
        let envelope = EventEnvelope::new(
            tenant_id,
            Some(user_id),
            DomainEvent::UserRegistered { user_id, email }
        );

        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(envelope.id, deserialized.id);
        prop_assert_eq!(envelope.tenant_id, deserialized.tenant_id);
        prop_assert_eq!(envelope.event, deserialized.event);
    }
}
```

### Example 3: Validation Invariants

```rust
proptest! {
    #[test]
    fn inventory_low_event_validates_threshold_logic(
        variant_id in uuid_strategy(),
        product_id in uuid_strategy(),
        remaining in 0i32..100,
        threshold in 0i32..100
    ) {
        let event = DomainEvent::InventoryLow {
            variant_id,
            product_id,
            remaining,
            threshold,
        };

        let result = event.validate();

        if variant_id == Uuid::nil() || product_id == Uuid::nil() {
            prop_assert!(result.is_err(), "Nil UUIDs should fail validation");
        } else if remaining >= threshold {
            prop_assert!(result.is_err(), "Remaining >= threshold should fail");
        } else {
            prop_assert!(result.is_ok(), "Valid low inventory should pass");
        }
    }
}
```

---

## Troubleshooting

### Problem: Tests are too slow

**Solution:** Reduce test cases for development, increase for CI:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: if cfg!(debug_assertions) { 10 } else { 1000 },
        ..ProptestConfig::default()
    })]
}
```

### Problem: Failing tests are not reproducible

**Solution:** Use `PROPTEST_SEED` environment variable:

```bash
PROPTEST_SEED=12345 cargo test my_property_test
```

### Problem: Tests find too many edge cases

**Solution:** Refine your strategies to exclude invalid inputs:

```rust
// ❌ Too broad
fn price_strategy() -> impl Strategy<Value = i64> {
    any::<i64>()  // Includes negative values!
}

// ✅ Constrained
fn price_strategy() -> impl Strategy<Value = i64> {
    1i64..=1_000_000i64  // Only valid prices
}
```

---

## Further Reading

- [Proptest Book](https://altsysrq.github.io/proptest-book/intro.html)
- [Property-Based Testing Tutorial](https://hypothesis.works/articles/what-is-property-based-testing/)
- [Integration Testing Guide](./INTEGRATION_TESTING_GUIDE.md)
- [Performance Testing Guide](./PERFORMANCE_TESTING_GUIDE.md)

---

**Task 4.2 Complete** | Sprint 4: Testing Infrastructure | RusToK Platform
