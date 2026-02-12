//! Property-based tests for event validation and invariants
//!
//! These tests verify that event properties hold for all possible inputs.

use proptest::prelude::*;
use rustok_core::events::{DomainEvent, EventEnvelope, ValidateEvent};
use rustok_test_utils::{email_strategy, non_empty_string, price_strategy, uuid_strategy};
use uuid::Uuid;

// ============================================================================
// PROPERTY 1: Event ID Uniqueness
// ============================================================================

proptest! {
    /// Property: Every event envelope has a unique ID
    #[test]
    fn event_envelopes_have_unique_ids(
        tenant_id in uuid_strategy(),
        count in 2usize..10
    ) {
        let events: Vec<EventEnvelope> = (0..count)
            .map(|_| {
                EventEnvelope::new(
                    tenant_id,
                    None,
                    DomainEvent::TenantCreated { tenant_id }
                )
            })
            .collect();

        // All event IDs should be unique
        let mut ids: Vec<Uuid> = events.iter().map(|e| e.id).collect();
        ids.sort();
        ids.dedup();
        
        prop_assert_eq!(ids.len(), count, "Event IDs should be unique");
    }
}

// ============================================================================
// PROPERTY 2: Event Timestamp Monotonicity
// ============================================================================

proptest! {
    /// Property: Event timestamps progress forward (or stay same) in sequence
    #[test]
    fn event_timestamps_are_monotonic(
        tenant_id in uuid_strategy(),
        count in 2usize..10
    ) {
        let events: Vec<EventEnvelope> = (0..count)
            .map(|_| {
                std::thread::sleep(std::time::Duration::from_micros(100));
                EventEnvelope::new(
                    tenant_id,
                    None,
                    DomainEvent::TenantCreated { tenant_id }
                )
            })
            .collect();

        // Check that timestamps are monotonically increasing (or equal)
        for i in 1..events.len() {
            prop_assert!(
                events[i].timestamp >= events[i-1].timestamp,
                "Event timestamps should be monotonically increasing"
            );
        }
    }
}

// ============================================================================
// PROPERTY 3: Event Payload JSON Roundtrip
// ============================================================================

proptest! {
    /// Property: Event payloads survive JSON serialization/deserialization
    #[test]
    fn event_payload_json_roundtrip(
        node_id in uuid_strategy(),
        kind in non_empty_string(),
        author_id in proptest::option::of(uuid_strategy())
    ) {
        let event = DomainEvent::NodeCreated {
            node_id,
            kind: kind.chars().take(64).collect(), // Limit to valid length
            author_id,
        };

        // Serialize and deserialize
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(event, deserialized, "Event should survive JSON roundtrip");
    }
}

// ============================================================================
// PROPERTY 4: Valid Events Always Pass Validation
// ============================================================================

proptest! {
    /// Property: Well-formed TenantCreated events always validate
    #[test]
    fn valid_tenant_created_always_validates(tenant_id in uuid_strategy()) {
        // UUID::new_v4() always generates non-nil UUIDs
        let event = DomainEvent::TenantCreated { 
            tenant_id 
        };

        // Should validate successfully if tenant_id is not nil
        let result = event.validate();
        if tenant_id != Uuid::nil() {
            prop_assert!(result.is_ok(), "Valid TenantCreated should pass validation");
        } else {
            prop_assert!(result.is_err(), "Nil UUID should fail validation");
        }
    }

    /// Property: Well-formed UserRegistered events always validate
    #[test]
    fn valid_user_registered_always_validates(
        user_id in uuid_strategy(),
        email in email_strategy()
    ) {
        let event = DomainEvent::UserRegistered {
            user_id,
            email,
        };

        // Should validate if user_id is not nil
        let result = event.validate();
        if user_id != Uuid::nil() {
            prop_assert!(result.is_ok(), "Valid UserRegistered should pass validation");
        } else {
            prop_assert!(result.is_err(), "Nil UUID should fail validation");
        }
    }

    /// Property: Well-formed OrderPlaced events always validate
    #[test]
    fn valid_order_placed_always_validates(
        order_id in uuid_strategy(),
        customer_id in proptest::option::of(uuid_strategy()),
        total in price_strategy()
    ) {
        let event = DomainEvent::OrderPlaced {
            order_id,
            customer_id,
            total,
            currency: "USD".to_string(),
        };

        // Should validate if order_id is not nil and total is positive
        let result = event.validate();
        if order_id != Uuid::nil() && total > 0 {
            prop_assert!(result.is_ok(), "Valid OrderPlaced should pass validation");
        }
    }
}

// ============================================================================
// PROPERTY 5: Invalid Events Always Fail Validation
// ============================================================================

proptest! {
    /// Property: Events with nil UUIDs always fail validation
    #[test]
    fn events_with_nil_uuids_fail_validation(event_type in 0u8..5) {
        let nil_uuid = Uuid::nil();
        
        let event = match event_type {
            0 => DomainEvent::TenantCreated { tenant_id: nil_uuid },
            1 => DomainEvent::NodeCreated { 
                node_id: nil_uuid, 
                kind: "post".to_string(),
                author_id: None 
            },
            2 => DomainEvent::ProductCreated { product_id: nil_uuid },
            3 => DomainEvent::OrderPlaced {
                order_id: nil_uuid,
                customer_id: None,
                total: 100,
                currency: "USD".to_string(),
            },
            _ => DomainEvent::UserDeleted { user_id: nil_uuid },
        };

        let result = event.validate();
        prop_assert!(result.is_err(), "Events with nil UUIDs should fail validation");
    }

    /// Property: Events with empty required strings always fail validation
    #[test]
    fn events_with_empty_strings_fail_validation() {
        let events = vec![
            DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: "".to_string(), // Empty kind
                author_id: None,
            },
            DomainEvent::UserRegistered {
                user_id: Uuid::new_v4(),
                email: "".to_string(), // Empty email
            },
            DomainEvent::MediaUploaded {
                media_id: Uuid::new_v4(),
                mime_type: "".to_string(), // Empty mime_type
                size: 1000,
            },
        ];

        for event in events {
            let result = event.validate();
            prop_assert!(result.is_err(), "Events with empty strings should fail validation");
        }
    }

    /// Property: OrderStatusChanged with same status always fails
    #[test]
    fn order_status_same_always_fails(
        order_id in uuid_strategy(),
        status in "[a-z]{5,20}"
    ) {
        let event = DomainEvent::OrderStatusChanged {
            order_id,
            old_status: status.clone(),
            new_status: status, // Same as old
        };

        let result = event.validate();
        if order_id != Uuid::nil() {
            prop_assert!(result.is_err(), "Same old/new status should fail validation");
        }
    }
}

// ============================================================================
// PROPERTY 6: Event Type String Consistency
// ============================================================================

proptest! {
    /// Property: Event type strings are consistent and non-empty
    #[test]
    fn event_type_strings_are_consistent(
        id in uuid_strategy()
    ) {
        let events = vec![
            DomainEvent::TenantCreated { tenant_id: id },
            DomainEvent::ProductCreated { product_id: id },
            DomainEvent::NodeCreated { node_id: id, kind: "post".to_string(), author_id: None },
            DomainEvent::OrderPlaced { order_id: id, customer_id: None, total: 100, currency: "USD".to_string() },
        ];

        for event in events {
            let event_type = event.event_type();
            prop_assert!(!event_type.is_empty(), "Event type should not be empty");
            prop_assert!(event_type.contains('.'), "Event type should contain namespace separator");
            
            // Event type should be consistent for same variant
            let event_type2 = event.event_type();
            prop_assert_eq!(event_type, event_type2, "Event type should be consistent");
        }
    }

    /// Property: Schema version is always positive
    #[test]
    fn schema_version_is_positive(
        id in uuid_strategy()
    ) {
        let events = vec![
            DomainEvent::TenantCreated { tenant_id: id },
            DomainEvent::ProductCreated { product_id: id },
            DomainEvent::NodePublished { node_id: id, kind: "post".to_string() },
        ];

        for event in events {
            let version = event.schema_version();
            prop_assert!(version > 0, "Schema version should be positive");
        }
    }
}

// ============================================================================
// PROPERTY 7: Event Envelope Correlation
// ============================================================================

proptest! {
    /// Property: New event envelopes have correlation_id == id initially
    #[test]
    fn new_envelope_correlation_equals_id(
        tenant_id in uuid_strategy()
    ) {
        let envelope = EventEnvelope::new(
            tenant_id,
            None,
            DomainEvent::TenantCreated { tenant_id }
        );

        prop_assert_eq!(
            envelope.id, 
            envelope.correlation_id,
            "New envelopes should have correlation_id == id"
        );
        prop_assert_eq!(
            envelope.causation_id,
            None,
            "New envelopes should have no causation_id initially"
        );
    }

    /// Property: Event envelope preserves tenant_id
    #[test]
    fn envelope_preserves_tenant_id(
        tenant_id in uuid_strategy()
    ) {
        let envelope = EventEnvelope::new(
            tenant_id,
            None,
            DomainEvent::TenantCreated { tenant_id }
        );

        prop_assert_eq!(
            envelope.tenant_id,
            tenant_id,
            "Envelope should preserve tenant_id"
        );
    }
}
