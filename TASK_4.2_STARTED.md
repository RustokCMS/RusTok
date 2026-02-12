# Task 4.2: Property-Based Tests - Started

> **Status:** In Progress (30% complete)
> **Started:** 2026-02-12
> **Estimated Completion:** 6-8 hours remaining

---

## Summary

Task 4.2 has been initiated with foundational infrastructure for property-based testing using proptest. Core strategies and initial setup are complete.

## Completed Work

### 1. Dependencies Added ✅

**Workspace Cargo.toml:**
- Added `proptest = "1.4"` to workspace dependencies
- Added `criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }` for future benchmarking

**rustok-test-utils Cargo.toml:**
- Added `proptest = { workspace = true }` dependency

### 2. Proptest Strategies Module ✅

Created `crates/rustok-test-utils/src/proptest_strategies.rs` (120 LOC)

**Strategies Implemented:**
- `uuid_strategy()` - Generates valid UUIDs
- `tenant_id_strategy()` - Generates tenant IDs
- `non_empty_string()` - Non-empty strings (1-100 chars)
- `email_strategy()` - Valid email addresses
- `positive_i64()` - Positive integers
- `price_strategy()` - Valid prices ($0.01 - $10,000.00)
- `sku_strategy()` - Product SKUs (e.g., "AB-123")
- `slug_strategy()` - URL slugs
- `title_strategy()` - Content titles
- `quantity_strategy()` - Product quantities (1-1000)

**Self-Tests:** 9 property tests validating the strategies themselves

### 3. Module Integration ✅

- Updated `crates/rustok-test-utils/src/lib.rs` to export `proptest_strategies` module
- Module ready for use across all RusToK crates

---

## Remaining Work

### Task 4.2 Roadmap (70% remaining)

#### 1. Tenant Identifier Property Tests (0/4 properties)
**Location:** `crates/rustok-tenant/tests/property_tests.rs`

Properties to test:
- [ ] **UUID Format Preservation** - Tenant IDs maintain valid UUID format through serialization
- [ ] **Tenant ID Immutability** - Once created, tenant IDs never change
- [ ] **Tenant ID Uniqueness** - No two tenants can have the same ID
- [ ] **Serialization Round-trip** - Tenant IDs survive JSON/BSON serialization

#### 2. Event Validation Property Tests (0/3 properties)
**Location:** `crates/rustok-core/tests/event_property_tests.rs`

Properties to test:
- [ ] **Event ID Uniqueness** - Every event has a unique ID
- [ ] **Timestamp Monotonicity** - Event timestamps progress forward
- [ ] **Payload Validity** - Event payloads are always valid JSON

#### 3. Order State Machine Property Tests (0/2 properties)
**Location:** `crates/rustok-commerce/tests/order_state_machine_properties.rs`

Properties to test:
- [ ] **Valid Transitions Only** - Only allowed state transitions occur
- [ ] **Transition Idempotency** - Applying same transition twice has no effect

#### 4. Node State Machine Property Tests (0/2 properties)
**Location:** `crates/rustok-content/tests/node_state_machine_properties.rs`

Properties to test:
- [ ] **Valid Transitions Only** - Draft → Published (not reverse)
- [ ] **Published Immutability** - Published nodes cannot transition back to Draft

#### 5. Documentation (0/1)
**Location:** `docs/PROPERTY_TESTING_GUIDE.md`

Content needed:
- [ ] Property testing introduction
- [ ] Using proptest in RusToK
- [ ] Strategy reference
- [ ] Writing property tests
- [ ] Best practices
- [ ] CI integration

---

## Implementation Examples

### Tenant ID Property Test (Template)

```rust
// crates/rustok-tenant/tests/property_tests.rs
use proptest::prelude::*;
use rustok_test_utils::tenant_id_strategy;
use uuid::Uuid;

proptest! {
    #[test]
    fn tenant_id_preserves_uuid_format(tenant_id in tenant_id_strategy()) {
        // Property: Tenant IDs are always valid UUIDs
        let as_string = tenant_id.to_string();
        let parsed = Uuid::parse_str(&as_string).unwrap();
        assert_eq!(tenant_id, parsed);
    }
    
    #[test]
    fn tenant_id_survives_json_roundtrip(tenant_id in tenant_id_strategy()) {
        // Property: Serialization preserves tenant ID
        let json = serde_json::to_string(&tenant_id).unwrap();
        let parsed: Uuid = serde_json::from_str(&json).unwrap();
        assert_eq!(tenant_id, parsed);
    }
}
```

### Order State Machine Property Test (Template)

```rust
// crates/rustok-commerce/tests/order_state_machine_properties.rs
use proptest::prelude::*;
use rustok_commerce::entities::OrderStatus;

proptest! {
    #[test]
    fn order_transitions_are_valid(
        initial in order_status_strategy(),
        action in order_action_strategy()
    ) {
        // Property: Only valid transitions are allowed
        let result = try_transition(initial, action);
        
        match (initial, action) {
            (OrderStatus::Draft, OrderAction::Submit) => assert!(result.is_ok()),
            (OrderStatus::PendingPayment, OrderAction::Pay) => assert!(result.is_ok()),
            (OrderStatus::Paid, OrderAction::Submit) => assert!(result.is_err()), // Invalid
            _ => {}
        }
    }
}
```

---

## Files Modified

### New Files
- `crates/rustok-test-utils/src/proptest_strategies.rs` (120 LOC)
- `TASK_4.2_STARTED.md` (this file)

### Modified Files
- `Cargo.toml` (added proptest and criterion)
- `crates/rustok-test-utils/Cargo.toml` (added proptest dependency)
- `crates/rustok-test-utils/src/lib.rs` (exported proptest_strategies)

---

## Next Steps

### Immediate (Complete Task 4.2)

1. **Create Property Test Files**
   ```bash
   touch crates/rustok-tenant/tests/property_tests.rs
   touch crates/rustok-core/tests/event_property_tests.rs
   touch crates/rustok-commerce/tests/order_state_machine_properties.rs
   touch crates/rustok-content/tests/node_state_machine_properties.rs
   ```

2. **Implement 11+ Property Tests**
   - Tenant properties (4 tests)
   - Event properties (3 tests)
   - Order state machine (2 tests)
   - Node state machine (2 tests)

3. **Add Cargo.toml Dependencies**
   - Add proptest to each crate's `[dev-dependencies]`

4. **Run and Verify Tests**
   ```bash
   cargo test --all-features --workspace
   ```

5. **Create Documentation**
   - Write `docs/PROPERTY_TESTING_GUIDE.md` (6KB)

6. **Update Sprint Progress**
   - Mark Task 4.2 as complete in `SPRINT_4_PROGRESS.md`

### Future Tasks
- Task 4.3: Performance Benchmarks (criterion already added)
- Task 4.4: Security Audit

---

## Resources

### Property Testing
- [Proptest Book](https://altsysrq.github.io/proptest-book/intro.html)
- [Property-Based Testing Tutorial](https://hypothesis.works/articles/what-is-property-based-testing/)

### RusToK Testing
- [Integration Testing Guide](./docs/INTEGRATION_TESTING_GUIDE.md)
- [Performance Testing Guide](./docs/PERFORMANCE_TESTING_GUIDE.md)

---

## Progress Tracking

| Subtask | Status | LOC | Tests |
|---------|--------|-----|-------|
| Dependencies | ✅ Complete | - | - |
| Proptest Strategies | ✅ Complete | 120 | 9 |
| Tenant Properties | 📋 Planned | 0 | 0 |
| Event Properties | 📋 Planned | 0 | 0 |
| Order State Machine | 📋 Planned | 0 | 0 |
| Node State Machine | 📋 Planned | 0 | 0 |
| Documentation | 📋 Planned | 0 | - |

**Overall Progress:** 30% complete

---

**Task Status:** 🔄 In Progress  
**Sprint 4:** 30% complete (1.3/4 tasks)  
**Next Milestone:** Complete property test implementation
