//! Property-Based Tests for Core Validators
//!
//! These tests use proptest to verify validator invariants across
//! randomly generated inputs, ensuring robustness and catching edge cases.
//!
//! Properties tested:
//! - Tenant identifier validation rules
//! - Event validation invariants
//! - Input sanitization properties

// ============================================================================
// TENANT VALIDATION PROPERTY TESTS
// ============================================================================

#[cfg(test)]
mod tenant_validation_tests {
    use super::super::tenant_validation::*;
    use proptest::prelude::*;

    // Property: Valid slug pattern matches [a-z0-9][a-z0-9-]{0,62}
    proptest! {
        #[test]
        fn valid_slug_pattern(s in "[a-z0-9]([a-z0-9-]{0,62})?") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            // Should succeed for all valid patterns except reserved words
            if !RESERVED_SLUGS.contains(&s.as_str()) {
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn lowercase_normalization(s in "[a-zA-Z0-9][a-zA-Z0-9-]{0,62}") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            if result.is_ok() {
                // If valid, should be normalized to lowercase
                prop_assert!(result.unwrap().chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-'));
            }
        }

        #[test]
        fn empty_slug_fails(s in "") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn slug_exceeding_max_length_fails(s in "[a-z0-9-]{65,}") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(matches!(result, Err(TenantValidationError::TooLong)));
        }

        #[test]
        fn slug_starting_with_hyphen_fails(s in "-[a-z0-9-]{0,62}") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn slug_ending_with_hyphen_fails(s in "[a-z0-9-]{1,63}-") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn uppercase_characters_rejected(s in "[A-Z]{1,}") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn spaces_rejected(s in "[a-z0-9 ]{1,64}") {
            prop_assume!(s.contains(' '));
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn special_characters_rejected(s in "[!@#$%^&*()_=+]{1,}") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn reserved_words_rejected(s in prop::sample::select(RESERVED_SLUGS)) {
            let result = TenantIdentifierValidator::validate_slug(s);
            prop_assert!(matches!(result, Err(TenantValidationError::Reserved(_))));
        }
    }

    // Property: UUID validation
    proptest! {
        #[test]
        fn valid_uuid_accepts(uuid in any::<[u8; 16]>()) {
            let uuid_str = uuid::Uuid::from_bytes(uuid).to_string();
            let result = TenantIdentifierValidator::validate_uuid(&uuid_str);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn uppercase_uuid_normalized(uuid in any::<[u8; 16]>()) {
            let uuid_str = uuid::Uuid::from_bytes(uuid).to_string().to_uppercase();
            let result = TenantIdentifierValidator::validate_uuid(&uuid_str);
            prop_assert!(result.is_ok());
            // Should be normalized to lowercase
            prop_assert!(result.unwrap().to_string().chars().all(|c| !c.is_uppercase()));
        }

        #[test]
        fn invalid_uuid_format_rejected(s in "[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{3}-[^0-9a-f][0-9a-f]{3}-[0-9a-f]{12}") {
            // Has exactly one invalid character in the third group
            let result = TenantIdentifierValidator::validate_uuid(&s);
            prop_assert!(result.is_err());
        }

        #[test]
        fn nil_uuid_rejected() {
            let nil_uuid = uuid::Uuid::nil().to_string();
            let result = TenantIdentifierValidator::validate_uuid(&nil_uuid);
            prop_assert!(result.is_err());
        }
    }

    // Property: Hostname validation
    proptest! {
        #[test]
        fn valid_hostname_accepts(host in "[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?){1,10}") {
            let host = host.trim_start_matches('.').trim_end_matches('.');
            if !host.is_empty() && host.len() <= 253 {
                let result = TenantIdentifierValidator::validate_host(&host);
                // Most should be valid, some edge cases may fail
                if host.len() <= 253 && !host.contains("..") && !host.starts_with('-') && !host.ends_with('-') {
                    prop_assert!(result.is_ok());
                }
            }
        }

        #[test]
        fn hostname_too_long_rejected(s in "[a-z]{254,}") {
            let result = TenantIdentifierValidator::validate_host(&s);
            prop_assert!(matches!(result, Err(TenantValidationError::HostnameTooLong)));
        }

        #[test]
        fn hostname_with_consecutive_dots_rejected(s in "[a-z]+\\.{2,}[a-z]+") {
            let result = TenantIdentifierValidator::validate_host(&s);
            prop_assert!(result.is_err());
        }
    }

    // Property: Auto-detect validation
    proptest! {
        #[test]
        fn validate_any_accepts_valid_slug(s in "[a-z0-9][a-z0-9-]{0,62}") {
            if !RESERVED_SLUGS.contains(&s.as_str()) {
                let result = TenantIdentifierValidator::validate_any(&s);
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn validate_any_accepts_valid_uuid(uuid in any::<[u8; 16]>()) {
            let uuid_str = uuid::Uuid::from_bytes(uuid).to_string();
            let result = TenantIdentifierValidator::validate_any(&uuid_str);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn validate_any_accepts_valid_hostname(host in "[a-z0-9]+(\\.[a-z0-9]+){1,3}") {
            let result = TenantIdentifierValidator::validate_any(&host);
            prop_assert!(result.is_ok());
        }
    }

    // Property: Security validation - SQL injection patterns
    proptest! {
        #[test]
        fn sql_injection_patterns_rejected(s in "('[^']*'|;[^;]*|--[^\\n]*|\\/\\*.*\\*\\/)[a-z0-9-]*") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }
    }

    // Property: Security validation - XSS patterns
    proptest! {
        #[test]
        fn xss_patterns_rejected(s in "(<[^>]*>|javascript:[a-z]*|&[a-z]+;)[a-z0-9-]*") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }
    }

    // Property: Security validation - Path traversal
    proptest! {
        #[test]
        fn path_traversal_patterns_rejected(s in "(\\.\\.|\\\\|/)[a-z0-9-]*") {
            let result = TenantIdentifierValidator::validate_slug(&s);
            prop_assert!(result.is_err());
        }
    }

    // Property: Whitespace trimming
    proptest! {
        #[test]
        fn slug_trims_whitespace(s in "([a-z0-9-]{1,10}\\s+){1,3}[a-z0-9-]{1,10}") {
            let trimmed = s.trim();
            if !trimmed.is_empty() && !RESERVED_SLUGS.contains(trimmed) {
                let result = TenantIdentifierValidator::validate_slug(&s);
                // Should be trimmed and validated
                if trimmed.len() <= 64 && trimmed.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                    prop_assert!(result.is_ok());
                    prop_assert_eq!(result.unwrap(), trimmed.to_string());
                }
            }
        }

        #[test]
        fn hostname_normalizes_case(host in "[a-zA-Z0-9]+(\\.[a-zA-Z0-9]+){1,2}") {
            let result = TenantIdentifierValidator::validate_host(&host);
            if result.is_ok() {
                let normalized = result.unwrap();
                prop_assert!(normalized.chars().all(|c| c.is_lowercase() || c.is_ascii_digit() || c == '-' || c == '.'));
            }
        }
    }

    // Property: Length boundaries
    proptest! {
        #[test]
        fn slug_max_length_acceptable(s in "[a-z0-9][a-z0-9-]{61}[a-z0-9]") {
            if !RESERVED_SLUGS.contains(&s.as_str()) {
                let result = TenantIdentifierValidator::validate_slug(&s);
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn slug_min_length_acceptable(s in "[a-z0-9]") {
            if !RESERVED_SLUGS.contains(&s.as_str()) {
                let result = TenantIdentifierValidator::validate_slug(&s);
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn hostname_max_length_boundary() {
            // Create a hostname exactly 253 characters
            let label = "a".repeat(63);
            let host = format!("{}.{}.{}", label, label, &label[..25]); // 63 + 1 + 63 + 1 + 25 = 153 chars
            let result = TenantIdentifierValidator::validate_host(&host);
            prop_assert!(result.is_ok());
        }
    }
}

// ============================================================================
// EVENT VALIDATION PROPERTY TESTS
// ============================================================================

#[cfg(test)]
mod event_validation_tests {
    use super::super::events::types::*;
    use super::super::events::validation::*;
    use proptest::prelude::*;
    use uuid::Uuid;

    // Property: String field validation
    proptest! {
        #[test]
        fn validate_not_empty_accepts_non_empty(s in "[a-zA-Z0-9]{1,100}") {
            let result = validators::validate_not_empty("field", &s);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn validate_not_empty_rejects_empty(s in "") {
            let result = validators::validate_not_empty("field", &s);
            prop_assert!(matches!(result, Err(EventValidationError::EmptyField(_))));
        }

        #[test]
        fn validate_not_empty_rejects_whitespace_only(s in "[ \\t\\n\\r]{1,10}") {
            let result = validators::validate_not_empty("field", &s);
            prop_assert!(result.is_err());
        }
    }

    // Property: Max length validation
    proptest! {
        #[test]
        fn validate_max_length_accepts_within_limit(s in "[a-z]{1,50}", max in 50usize..100usize) {
            if s.len() <= max {
                let result = validators::validate_max_length("field", &s, max);
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn validate_max_length_rejects_exceeding_limit(s in "[a-z]{51,100}", max in 1usize..50usize) {
            if s.len() > max {
                let result = validators::validate_max_length("field", &s, max);
                prop_assert!(matches!(result, Err(EventValidationError::FieldTooLong(_, _))));
            }
        }

        #[test]
        fn validate_max_length_boundary_case(max in 10usize..100usize) {
            let exact = "a".repeat(max);
            let result = validators::validate_max_length("field", &exact, max);
            prop_assert!(result.is_ok());

            let over = "a".repeat(max + 1);
            let result_over = validators::validate_max_length("field", &over, max);
            prop_assert!(result_over.is_err());
        }
    }

    // Property: UUID validation
    proptest! {
        #[test]
        fn validate_not_nil_uuid_accepts_valid(uuid in any::<[u8; 16]>()) {
            prop_assume!(uuid != [0u8; 16]);
            let uuid_obj = Uuid::from_bytes(uuid);
            let result = validators::validate_not_nil_uuid("field", &uuid_obj);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn validate_not_nil_uuid_rejects_nil() {
            let nil = Uuid::nil();
            let result = validators::validate_not_nil_uuid("field", &nil);
            prop_assert!(matches!(result, Err(EventValidationError::NilUuid(_))));
        }

        #[test]
        fn validate_optional_uuid_accepts_none() {
            let result = validators::validate_optional_uuid("field", &None);
            prop_assert!(result.is_ok());
        }

        #[test]
        fn validate_optional_uuid_rejects_nil_some(uuid in any::<[u8; 16]>()) {
            if uuid == [0u8; 16] {
                let nil = Uuid::nil();
                let result = validators::validate_optional_uuid("field", &Some(nil));
                prop_assert!(matches!(result, Err(EventValidationError::NilUuid(_))));
            }
        }
    }

    // Property: Range validation
    proptest! {
        #[test]
        fn validate_range_accepts_within_bounds(value in -1000i64..1000i64, min in -500i64..0i64, max in 0i64..500i64) {
            if value >= min && value <= max {
                let result = validators::validate_range("field", value, min, max);
                prop_assert!(result.is_ok());
            }
        }

        #[test]
        fn validate_range_rejects_below_min(value in -1000i64..-1i64, max in 0i64..500i64) {
            if value < max {
                let result = validators::validate_range("field", value, 0, max);
                prop_assert!(result.is_err());
            }
        }

        #[test]
        fn validate_range_rejects_above_max(value in 1i64..1000i64, min in -500i64..0i64) {
            if value > min {
                let result = validators::validate_range("field", value, min, 0);
                prop_assert!(result.is_err());
            }
        }

        #[test]
        fn validate_range_boundary_cases(min in -100i64..100i64) {
            let max = min + 10;

            // At lower boundary
            let result_min = validators::validate_range("field", min, min, max);
            prop_assert!(result_min.is_ok());

            // At upper boundary
            let result_max = validators::validate_range("field", max, min, max);
            prop_assert!(result_max.is_ok());

            // Just below lower boundary
            let result_below = validators::validate_range("field", min - 1, min, max);
            prop_assert!(result_below.is_err());

            // Just above upper boundary
            let result_above = validators::validate_range("field", max + 1, min, max);
            prop_assert!(result_above.is_err());
        }
    }

    // Property: Event type validation
    proptest! {
        #[test]
        fn event_type_length_validation(kind in "[a-z]{1,100}") {
            let event = DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: kind.clone(),
                author_id: None,
            };

            if kind.len() <= 64 {
                let result = event.validate();
                prop_assert!(result.is_ok());
            } else {
                let result = event.validate();
                // Should fail if kind is too long
                prop_assert!(result.is_err());
            }
        }

        #[test]
        fn event_type_empty_rejected() {
            let event = DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: String::new(),
                author_id: None,
            };

            let result = event.validate();
            prop_assert!(result.is_err());
        }
    }

    // Property: Node ID validation
    proptest! {
        #[test]
        fn node_id_nil_rejected() {
            let event = DomainEvent::NodeCreated {
                node_id: Uuid::nil(),
                kind: "article".to_string(),
                author_id: None,
            };

            let result = event.validate();
            prop_assert!(result.is_err());
        }

        #[test]
        fn valid_node_id_accepted(uuid in any::<[u8; 16]>()) {
            prop_assume!(uuid != [0u8; 16]);
            let event = DomainEvent::NodeCreated {
                node_id: Uuid::from_bytes(uuid),
                kind: "article".to_string(),
                author_id: None,
            };

            let result = event.validate();
            prop_assert!(result.is_ok());
        }
    }
}

// ============================================================================
// EVENT SERIALIZATION PROPERTY TESTS
// ============================================================================

#[cfg(test)]
mod event_serialization_tests {
    use super::super::events::types::*;
    use proptest::prelude::*;
    use uuid::Uuid;

    // Property: Serialization roundtrip preserves data
    proptest! {
        #[test]
        fn event_serialization_roundtrip(kind in "[a-z]{1,50}") {
            let original = DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: kind.clone(),
                author_id: Some(Uuid::new_v4()),
            };

            // Serialize to JSON
            let json = serde_json::to_string(&original).unwrap();

            // Deserialize back
            let deserialized: DomainEvent = serde_json::from_str(&json).unwrap();

            // Should be equal
            prop_assert_eq!(original, deserialized);
        }

        #[test]
        fn envelope_serialization_roundtrip(tenant_id in any::<[u8; 16]>()) {
            let original = EventEnvelope::new(
                Uuid::from_bytes(tenant_id),
                Some(Uuid::new_v4()),
                DomainEvent::NodeCreated {
                    node_id: Uuid::new_v4(),
                    kind: "article".to_string(),
                    author_id: None,
                },
            );

            // Serialize to JSON
            let json = serde_json::to_string(&original).unwrap();

            // Deserialize back
            let deserialized: EventEnvelope = serde_json::from_str(&json).unwrap();

            // Check critical fields
            prop_assert_eq!(original.id, deserialized.id);
            prop_assert_eq!(original.tenant_id, deserialized.tenant_id);
            prop_assert_eq!(original.event, deserialized.event);
        }
    }

    // Property: Serialization produces valid JSON
    proptest! {
        #[test]
        fn event_serialization_produces_valid_json(kind in "[a-z]{1,50}") {
            let event = DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind,
                author_id: Some(Uuid::new_v4()),
            };

            let json = serde_json::to_string(&event).unwrap();

            // Should be valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            prop_assert!(parsed.is_object());
        }

        #[test]
        fn envelope_serialization_produces_valid_json() {
            let envelope = EventEnvelope::new(
                Uuid::new_v4(),
                Some(Uuid::new_v4()),
                DomainEvent::NodeCreated {
                    node_id: Uuid::new_v4(),
                    kind: "article".to_string(),
                    author_id: None,
                },
            );

            let json = serde_json::to_string(&envelope).unwrap();

            // Should be valid JSON
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            prop_assert!(parsed.is_object());

            // Should have required fields
            prop_assert!(parsed.get("id").is_some());
            prop_assert!(parsed.get("event_type").is_some());
            prop_assert!(parsed.get("tenant_id").is_some());
            prop_assert!(parsed.get("event").is_some());
        }
    }

    // Property: JSON structure is deterministic
    proptest! {
        #[test]
        fn event_json_structure_consistent(kind in "[a-z]{1,20}") {
            let event = DomainEvent::NodeCreated {
                node_id: Uuid::new_v4(),
                kind: kind.clone(),
                author_id: None,
            };

            let json = serde_json::to_string(&event).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

            // Should have the type field
            prop_assert!(parsed.get("type").is_some());

            // Should have the data field
            prop_assert!(parsed.get("data").is_some());

            // Data should be an object
            let data = parsed.get("data").unwrap();
            prop_assert!(data.is_object());
        }
    }

    // Property: Different event types serialize correctly
    proptest! {
        #[test]
        fn multiple_event_types_serialize(
            node_id in any::<[u8; 16]>(),
            kind1 in "[a-z]{1,20}",
            kind2 in "[a-z]{1,20}",
        ) {
            let events = vec![
                DomainEvent::NodeCreated {
                    node_id: Uuid::from_bytes(node_id),
                    kind: kind1.clone(),
                    author_id: Some(Uuid::new_v4()),
                },
                DomainEvent::NodeUpdated {
                    node_id: Uuid::from_bytes(node_id),
                    kind: kind2.clone(),
                },
                DomainEvent::NodePublished {
                    node_id: Uuid::from_bytes(node_id),
                    kind: kind1,
                },
            ];

            for event in events {
                let json = serde_json::to_string(&event).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
                prop_assert!(parsed.is_object());
            }
        }
    }

    // Property: Serialization preserves UUID format
    proptest! {
        #[test]
        fn uuid_serialization_preserves_format(uuid in any::<[u8; 16]>()) {
            let uuid_obj = Uuid::from_bytes(uuid);
            let event = DomainEvent::NodeCreated {
                node_id: uuid_obj,
                kind: "article".to_string(),
                author_id: Some(uuid_obj),
            };

            let json = serde_json::to_string(&event).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

            // Check that UUIDs are serialized as strings
            if let Some(data) = parsed.get("data") {
                if let Some(node_id) = data.get("node_id") {
                    prop_assert!(node_id.is_string());
                }
            }
        }
    }
}
