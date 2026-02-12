//! Property-based tests for Order State Machine
//!
//! These tests verify that state machine invariants hold for all inputs.

use proptest::prelude::*;
use rust_decimal::Decimal;
use rustok_commerce::state_machine::{Order, Pending};
use rustok_test_utils::{non_empty_string, uuid_strategy};
use std::str::FromStr;
use uuid::Uuid;

// ============================================================================
// Helper Strategies
// ============================================================================

fn valid_order_amount() -> impl Strategy<Value = Decimal> {
    (1u64..=1_000_000u64).prop_map(|cents| Decimal::new(cents as i64, 2))
}

fn currency_strategy() -> impl Strategy<Value = String> {
    prop::sample::select(vec!["USD", "EUR", "GBP", "JPY"]).prop_map(|s| s.to_string())
}

fn create_test_order(
    id: Uuid,
    tenant_id: Uuid,
    customer_id: Uuid,
    amount: Decimal,
    currency: String,
) -> Order<Pending> {
    Order::new_pending(id, tenant_id, customer_id, amount, currency)
}

// ============================================================================
// PROPERTY 1: Valid State Transitions Only
// ============================================================================

proptest! {
    /// Property: Pending orders can only transition to Confirmed or Cancelled
    #[test]
    fn pending_transitions_only_to_confirmed_or_cancelled(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);

        // Can confirm
        let confirmed = order.clone().confirm();
        prop_assert!(confirmed.is_ok(), "Pending → Confirmed should be valid");

        // Can cancel
        let cancelled = order.cancel("Test cancellation".to_string());
        prop_assert_eq!(cancelled.id, id, "Cancelled order should preserve ID");
    }

    /// Property: Confirmed orders can only transition to Paid or Cancelled
    #[test]
    fn confirmed_transitions_only_to_paid_or_cancelled(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy(),
        payment_id in non_empty_string()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let confirmed = order.confirm().unwrap();

        // Can pay
        let paid = confirmed.clone().pay(payment_id.clone(), "credit_card".to_string());
        prop_assert!(paid.is_ok(), "Confirmed → Paid should be valid");

        // Can cancel
        let cancelled = confirmed.cancel("Test cancellation".to_string());
        prop_assert_eq!(cancelled.id, id, "Cancelled order should preserve ID");
    }

    /// Property: Paid orders can only transition to Shipped or Cancelled
    #[test]
    fn paid_transitions_only_to_shipped_or_cancelled(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy(),
        tracking in non_empty_string()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let paid = order.confirm().unwrap()
            .pay("pay_123".to_string(), "credit_card".to_string())
            .unwrap();

        // Can ship
        let shipped = paid.clone().ship(tracking.clone(), "UPS".to_string());
        prop_assert!(shipped.is_ok(), "Paid → Shipped should be valid");

        // Can cancel with refund
        let cancelled = paid.cancel("Test cancellation".to_string(), true);
        prop_assert_eq!(cancelled.id, id, "Cancelled order should preserve ID");
        prop_assert!(cancelled.state.refunded, "Cancelled paid order should be refunded");
    }

    /// Property: Shipped orders can only transition to Delivered
    #[test]
    fn shipped_transitions_only_to_delivered(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let shipped = order.confirm().unwrap()
            .pay("pay_123".to_string(), "credit_card".to_string()).unwrap()
            .ship("TRACK123".to_string(), "UPS".to_string()).unwrap();

        // Can deliver
        let delivered = shipped.deliver(None);
        prop_assert!(delivered.is_ok(), "Shipped → Delivered should be valid");
        prop_assert_eq!(delivered.unwrap().id, id, "Delivered order should preserve ID");
    }
}

// ============================================================================
// PROPERTY 2: State Transition Idempotency
// ============================================================================

proptest! {
    /// Property: Creating the same order multiple times with same parameters
    /// produces equivalent states
    #[test]
    fn order_creation_is_deterministic_for_same_inputs(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order1 = create_test_order(
            id, tenant_id, customer_id, 
            amount, currency.clone()
        );
        let order2 = create_test_order(
            id, tenant_id, customer_id,
            amount, currency
        );

        // Orders should have same core properties
        prop_assert_eq!(order1.id, order2.id);
        prop_assert_eq!(order1.tenant_id, order2.tenant_id);
        prop_assert_eq!(order1.customer_id, order2.customer_id);
        prop_assert_eq!(order1.total_amount, order2.total_amount);
        prop_assert_eq!(order1.currency, order2.currency);
    }

    /// Property: Applying the same transition twice should not be possible
    /// (or should produce the same end state)
    #[test]
    fn transition_to_same_state_is_consistent(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);

        // Confirm once
        let confirmed1 = order.confirm().unwrap();

        // Try to confirm from different pending order with same ID
        let order2 = create_test_order(
            id, tenant_id, customer_id,
            confirmed1.total_amount, confirmed1.currency.clone()
        );
        let confirmed2 = order2.confirm().unwrap();

        // Both confirmations should preserve the order ID and core properties
        prop_assert_eq!(confirmed1.id, confirmed2.id);
        prop_assert_eq!(confirmed1.tenant_id, confirmed2.tenant_id);
        prop_assert_eq!(confirmed1.total_amount, confirmed2.total_amount);
    }
}

// ============================================================================
// PROPERTY 3: State Data Integrity
// ============================================================================

proptest! {
    /// Property: Order ID is immutable across all state transitions
    #[test]
    fn order_id_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let original_id = order.id;

        // Confirm
        let confirmed = order.confirm().unwrap();
        prop_assert_eq!(confirmed.id, original_id, "ID should be immutable after confirm");

        // Pay
        let paid = confirmed.pay("pay_123".to_string(), "card".to_string()).unwrap();
        prop_assert_eq!(paid.id, original_id, "ID should be immutable after pay");

        // Ship
        let shipped = paid.ship("TRACK123".to_string(), "UPS".to_string()).unwrap();
        prop_assert_eq!(shipped.id, original_id, "ID should be immutable after ship");

        // Deliver
        let delivered = shipped.deliver(None).unwrap();
        prop_assert_eq!(delivered.id, original_id, "ID should be immutable after deliver");
    }

    /// Property: Tenant ID is immutable across all state transitions
    #[test]
    fn tenant_id_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let original_tenant = order.tenant_id;

        let confirmed = order.confirm().unwrap();
        prop_assert_eq!(confirmed.tenant_id, original_tenant);

        let paid = confirmed.pay("pay_123".to_string(), "card".to_string()).unwrap();
        prop_assert_eq!(paid.tenant_id, original_tenant);

        let shipped = paid.ship("TRACK123".to_string(), "UPS".to_string()).unwrap();
        prop_assert_eq!(shipped.tenant_id, original_tenant);
    }

    /// Property: Total amount is immutable across state transitions
    #[test]
    fn total_amount_immutable_across_transitions(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy()
    ) {
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let original_amount = order.total_amount;

        let confirmed = order.confirm().unwrap();
        prop_assert_eq!(confirmed.total_amount, original_amount);

        let paid = confirmed.pay("pay_123".to_string(), "card".to_string()).unwrap();
        prop_assert_eq!(paid.total_amount, original_amount);

        let cancelled = paid.cancel("Test".to_string(), true);
        prop_assert_eq!(cancelled.total_amount, original_amount);
    }
}

// ============================================================================
// PROPERTY 4: Cancellation Properties
// ============================================================================

proptest! {
    /// Property: Orders can be cancelled from any non-terminal state
    #[test]
    fn orders_can_be_cancelled_from_any_non_terminal_state(
        id in uuid_strategy(),
        tenant_id in uuid_strategy(),
        customer_id in uuid_strategy(),
        amount in valid_order_amount(),
        currency in currency_strategy(),
        reason in non_empty_string()
    ) {
        // From Pending
        let order = create_test_order(id, tenant_id, customer_id, amount, currency.clone());
        let cancelled = order.cancel(reason.clone());
        prop_assert_eq!(cancelled.state.reason, reason);
        prop_assert!(!cancelled.state.refunded, "Pending cancellation should not be refunded");

        // From Confirmed
        let order = create_test_order(id, tenant_id, customer_id, amount, currency.clone());
        let confirmed = order.confirm().unwrap();
        let cancelled = confirmed.cancel(reason.clone());
        prop_assert_eq!(cancelled.state.reason, reason);

        // From Paid (with refund)
        let order = create_test_order(id, tenant_id, customer_id, amount, currency);
        let paid = order.confirm().unwrap()
            .pay("pay_123".to_string(), "card".to_string()).unwrap();
        let cancelled = paid.cancel(reason.clone(), true);
        prop_assert_eq!(cancelled.state.reason, reason);
        prop_assert!(cancelled.state.refunded, "Paid cancellation should be refunded");
    }
}
