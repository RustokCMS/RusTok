//! Property test strategies for RusToK types
//!
//! This module provides proptest strategies for generating test data.

use proptest::prelude::*;
use uuid::Uuid;

/// Strategy for generating valid UUIDs
pub fn uuid_strategy() -> impl Strategy<Value = Uuid> {
    any::<[u8; 16]>().prop_map(|bytes| Uuid::from_bytes(bytes))
}

/// Strategy for generating tenant IDs (valid UUIDs)
pub fn tenant_id_strategy() -> impl Strategy<Value = Uuid> {
    uuid_strategy()
}

/// Strategy for generating non-empty strings
pub fn non_empty_string() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9]{1,100}".prop_map(|s| s.to_string())
}

/// Strategy for generating email addresses
pub fn email_strategy() -> impl Strategy<Value = String> {
    ("[a-z]{3,20}", "[a-z]{3,20}", "[a-z]{2,5}")
        .prop_map(|(name, domain, tld)| format!("{}@{}.{}", name, domain, tld))
}

/// Strategy for generating positive integers
pub fn positive_i64() -> impl Strategy<Value = i64> {
    1i64..=i64::MAX
}

/// Strategy for generating positive prices (in cents)
pub fn price_strategy() -> impl Strategy<Value = i64> {
    1i64..=1_000_000i64 // $0.01 to $10,000.00
}

/// Strategy for generating SKUs
pub fn sku_strategy() -> impl Strategy<Value = String> {
    "[A-Z]{2,5}-[0-9]{3,6}".prop_map(|s| s.to_string())
}

/// Strategy for generating slugs
pub fn slug_strategy() -> impl Strategy<Value = String> {
    "[a-z0-9]{3,20}(-[a-z0-9]{3,20}){0,3}".prop_map(|s| s.to_string())
}

/// Strategy for generating content titles
pub fn title_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z0-9 ]{5,100}".prop_map(|s| s.to_string())
}

/// Strategy for generating quantities
pub fn quantity_strategy() -> impl Strategy<Value = i32> {
    1i32..=1000i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::proptest;

    proptest! {
        #[test]
        fn test_uuid_strategy_generates_valid_uuids(uuid in uuid_strategy()) {
            // UUIDs should have valid string representation
            let uuid_str = uuid.to_string();
            assert_eq!(uuid_str.len(), 36); // UUID string length
            assert!(uuid_str.contains('-'));
        }

        #[test]
        fn test_non_empty_string_generates_non_empty(s in non_empty_string()) {
            assert!(!s.is_empty());
            assert!(s.len() <= 100);
        }

        #[test]
        fn test_email_strategy_generates_valid_format(email in email_strategy()) {
            assert!(email.contains('@'));
            assert!(email.contains('.'));
            let parts: Vec<&str> = email.split('@').collect();
            assert_eq!(parts.len(), 2);
        }

        #[test]
        fn test_positive_i64_generates_positive(n in positive_i64()) {
            assert!(n > 0);
        }

        #[test]
        fn test_price_strategy_generates_valid_prices(price in price_strategy()) {
            assert!(price > 0);
            assert!(price <= 1_000_000);
        }

        #[test]
        fn test_sku_strategy_generates_valid_format(sku in sku_strategy()) {
            assert!(sku.contains('-'));
            assert!(sku.len() >= 6); // Min: AB-123
        }

        #[test]
        fn test_slug_strategy_generates_valid_slugs(slug in slug_strategy()) {
            assert!(!slug.is_empty());
            assert!(!slug.starts_with('-'));
            assert!(!slug.ends_with('-'));
        }

        #[test]
        fn test_title_strategy_generates_valid_titles(title in title_strategy()) {
            assert!(title.len() >= 5);
            assert!(title.len() <= 100);
        }

        #[test]
        fn test_quantity_strategy_generates_positive_quantities(qty in quantity_strategy()) {
            assert!(qty > 0);
            assert!(qty <= 1000);
        }
    }
}
