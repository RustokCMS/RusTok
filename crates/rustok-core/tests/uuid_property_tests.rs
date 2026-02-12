//! Property-based tests for UUID and Tenant ID handling
//!
//! These tests verify that UUID properties and tenant ID invariants hold.

use proptest::prelude::*;
use rustok_test_utils::uuid_strategy;
use serde_json;
use uuid::Uuid;

// ============================================================================
// PROPERTY 1: UUID Format Preservation
// ============================================================================

proptest! {
    /// Property: UUIDs preserve their format through string conversion
    #[test]
    fn uuid_preserves_format_through_string_conversion(uuid in uuid_strategy()) {
        let as_string = uuid.to_string();
        let parsed = Uuid::parse_str(&as_string).unwrap();
        
        prop_assert_eq!(uuid, parsed, "UUID should be identical after string roundtrip");
        prop_assert_eq!(as_string.len(), 36, "UUID string should be 36 characters");
        prop_assert_eq!(as_string.matches('-').count(), 4, "UUID should have 4 hyphens");
    }

    /// Property: UUIDs have consistent hyphenated format
    #[test]
    fn uuid_format_is_consistent(uuid in uuid_strategy()) {
        let s = uuid.to_string();
        
        // Format: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
        prop_assert_eq!(s.chars().nth(8), Some('-'), "Hyphen at position 8");
        prop_assert_eq!(s.chars().nth(13), Some('-'), "Hyphen at position 13");
        prop_assert_eq!(s.chars().nth(18), Some('-'), "Hyphen at position 18");
        prop_assert_eq!(s.chars().nth(23), Some('-'), "Hyphen at position 23");
        
        // All other characters should be hex digits
        for (i, ch) in s.chars().enumerate() {
            if ![8, 13, 18, 23].contains(&i) {
                prop_assert!(ch.is_ascii_hexdigit(), "Non-hyphen chars should be hex digits");
            }
        }
    }
}

// ============================================================================
// PROPERTY 2: Tenant ID Immutability
// ============================================================================

proptest! {
    /// Property: Tenant IDs maintain their value when cloned
    #[test]
    fn tenant_id_immutable_on_clone(tenant_id in uuid_strategy()) {
        let cloned = tenant_id;
        
        prop_assert_eq!(
            tenant_id, 
            cloned,
            "Cloned tenant ID should be identical"
        );
        
        // Verify both have same string representation
        prop_assert_eq!(
            tenant_id.to_string(),
            cloned.to_string(),
            "String representations should match"
        );
    }

    /// Property: Tenant IDs are immutable (value type semantics)
    #[test]
    fn tenant_id_is_immutable_value_type(tenant_id in uuid_strategy()) {
        let original_string = tenant_id.to_string();
        
        // Create a "modified" reference (which can't actually modify due to Copy trait)
        let reference = tenant_id;
        let _ = reference.to_string(); // Use the reference
        
        // Original should be unchanged
        prop_assert_eq!(
            tenant_id.to_string(),
            original_string,
            "Tenant ID should be immutable"
        );
    }
}

// ============================================================================
// PROPERTY 3: Tenant ID Uniqueness
// ============================================================================

proptest! {
    /// Property: Generated UUIDs are unique
    #[test]
    fn generated_uuids_are_unique(count in 2usize..20) {
        let mut uuids = Vec::new();
        
        for _ in 0..count {
            uuids.push(Uuid::new_v4());
        }
        
        // Check for uniqueness
        let mut sorted = uuids.clone();
        sorted.sort();
        sorted.dedup();
        
        prop_assert_eq!(
            sorted.len(),
            uuids.len(),
            "All generated UUIDs should be unique"
        );
    }

    /// Property: Different tenant IDs are not equal
    #[test]
    fn different_tenant_ids_not_equal(
        tenant_id1 in uuid_strategy(),
        tenant_id2 in uuid_strategy()
    ) {
        if tenant_id1 != tenant_id2 {
            prop_assert_ne!(
                tenant_id1,
                tenant_id2,
                "Different UUIDs should not be equal"
            );
            prop_assert_ne!(
                tenant_id1.to_string(),
                tenant_id2.to_string(),
                "String representations should differ"
            );
        }
    }
}

// ============================================================================
// PROPERTY 4: Tenant ID Serialization/Deserialization
// ============================================================================

proptest! {
    /// Property: Tenant IDs survive JSON serialization roundtrip
    #[test]
    fn tenant_id_json_roundtrip(tenant_id in uuid_strategy()) {
        let json = serde_json::to_string(&tenant_id).unwrap();
        let deserialized: Uuid = serde_json::from_str(&json).unwrap();
        
        prop_assert_eq!(
            tenant_id,
            deserialized,
            "Tenant ID should survive JSON roundtrip"
        );
    }

    /// Property: Tenant IDs serialize to quoted UUID string in JSON
    #[test]
    fn tenant_id_json_format(tenant_id in uuid_strategy()) {
        let json = serde_json::to_string(&tenant_id).unwrap();
        
        // Should be quoted string
        prop_assert!(json.starts_with('"'), "JSON should start with quote");
        prop_assert!(json.ends_with('"'), "JSON should end with quote");
        
        // Length should be 38 (36 chars + 2 quotes)
        prop_assert_eq!(json.len(), 38, "JSON length should be 38");
        
        // Should contain the UUID string
        let uuid_str = tenant_id.to_string();
        prop_assert!(json.contains(&uuid_str), "JSON should contain UUID string");
    }

    /// Property: Tenant IDs in structs survive serialization
    #[test]
    fn tenant_id_in_struct_roundtrip(
        tenant_id in uuid_strategy(),
        name in "[a-z]{5,20}"
    ) {
        use serde::{Serialize, Deserialize};
        
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestStruct {
            tenant_id: Uuid,
            name: String,
        }
        
        let original = TestStruct {
            tenant_id,
            name,
        };
        
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TestStruct = serde_json::from_str(&json).unwrap();
        
        prop_assert_eq!(
            original,
            deserialized,
            "Struct with tenant_id should survive JSON roundtrip"
        );
    }
}

// ============================================================================
// PROPERTY 5: UUID Nil Handling
// ============================================================================

proptest! {
    /// Property: Nil UUID is distinguishable from random UUIDs
    #[test]
    fn nil_uuid_is_distinguishable(uuid in uuid_strategy()) {
        let nil = Uuid::nil();
        
        // Nil should be all zeros
        prop_assert_eq!(
            nil.to_string(),
            "00000000-0000-0000-0000-000000000000",
            "Nil UUID should be all zeros"
        );
        
        // Random UUIDs should almost never be nil (probability is infinitesimal)
        // We can't assert != because theoretically possible, but very unlikely
        if uuid == nil {
            // If it happens (astronomically unlikely), document it
            prop_assert!(false, "Generated UUID should not be nil (extremely rare)");
        }
    }

    /// Property: Nil UUID comparison is consistent
    #[test]
    fn nil_uuid_comparison_consistent() {
        let nil1 = Uuid::nil();
        let nil2 = Uuid::nil();
        
        prop_assert_eq!(nil1, nil2, "Nil UUIDs should be equal");
        prop_assert_eq!(
            nil1.to_string(),
            nil2.to_string(),
            "Nil UUID strings should match"
        );
    }
}

// ============================================================================
// PROPERTY 6: UUID Comparison and Ordering
// ============================================================================

proptest! {
    /// Property: UUID comparison is transitive
    #[test]
    fn uuid_comparison_is_transitive(
        a in uuid_strategy(),
        b in uuid_strategy(),
        c in uuid_strategy()
    ) {
        // If a <= b and b <= c, then a <= c
        if a <= b && b <= c {
            prop_assert!(a <= c, "UUID comparison should be transitive");
        }
    }

    /// Property: UUID equality is reflexive
    #[test]
    fn uuid_equality_is_reflexive(uuid in uuid_strategy()) {
        prop_assert_eq!(uuid, uuid, "UUID should equal itself");
        
        let cloned = uuid;
        prop_assert_eq!(uuid, cloned, "UUID should equal its clone");
    }

    /// Property: UUID equality is symmetric
    #[test]
    fn uuid_equality_is_symmetric(
        uuid1 in uuid_strategy(),
        uuid2 in uuid_strategy()
    ) {
        if uuid1 == uuid2 {
            prop_assert_eq!(uuid2, uuid1, "UUID equality should be symmetric");
        }
    }
}

// ============================================================================
// PROPERTY 7: UUID Hash Consistency
// ============================================================================

proptest! {
    /// Property: Equal UUIDs produce the same hash
    #[test]
    fn equal_uuids_same_hash(uuid in uuid_strategy()) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher1 = DefaultHasher::new();
        uuid.hash(&mut hasher1);
        let hash1 = hasher1.finish();
        
        let mut hasher2 = DefaultHasher::new();
        uuid.hash(&mut hasher2);
        let hash2 = hasher2.finish();
        
        prop_assert_eq!(hash1, hash2, "Equal UUIDs should produce same hash");
    }

    /// Property: UUIDs can be used as HashMap keys
    #[test]
    fn uuids_work_as_hashmap_keys(
        uuids in prop::collection::vec(uuid_strategy(), 1..20)
    ) {
        use std::collections::HashMap;
        
        let mut map = HashMap::new();
        
        for (i, uuid) in uuids.iter().enumerate() {
            map.insert(*uuid, i);
        }
        
        // All UUIDs should be retrievable
        for (i, uuid) in uuids.iter().enumerate() {
            if let Some(&value) = map.get(uuid) {
                // If UUID is unique in the vec, value should match
                if uuids.iter().filter(|&u| u == uuid).count() == 1 {
                    prop_assert_eq!(value, i, "HashMap should retrieve correct value");
                }
            }
        }
        
        prop_assert!(map.len() <= uuids.len(), "HashMap size should not exceed input size");
    }
}
