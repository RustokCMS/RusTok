//! # Mock Payment Service
//!
//! Provides a mock payment gateway for integration testing.
//! Simulates payment processing without real external calls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Mock payment service for testing
#[derive(Debug, Clone)]
pub struct MockPaymentService {
    /// Successful card tokens
    valid_tokens: Vec<String>,
    /// Failed card tokens
    failed_tokens: Vec<String>,
    /// Payment history
    payments: Arc<Mutex<HashMap<Uuid, MockPaymentRecord>>>,
    /// Service enabled flag
    enabled: Arc<Mutex<bool>>,
}

/// Mock payment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPaymentRecord {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount: i64,
    pub currency: String,
    pub status: MockPaymentStatus,
    pub card_token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Mock payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MockPaymentStatus {
    Pending,
    Approved,
    Declined,
    Error,
}

impl MockPaymentService {
    /// Create a new mock payment service with default test tokens
    pub fn new() -> Self {
        Self {
            valid_tokens: vec![
                "tok_test_visa".to_string(),
                "tok_test_mastercard".to_string(),
                "tok_test_amex".to_string(),
                "tok_test".to_string(),
            ],
            failed_tokens: vec![
                "tok_fail".to_string(),
                "tok_declined".to_string(),
                "tok_error".to_string(),
            ],
            payments: Arc::new(Mutex::new(HashMap::new())),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Create a new mock payment service with custom tokens
    pub fn with_tokens(valid: Vec<String>, failed: Vec<String>) -> Self {
        Self {
            valid_tokens: valid,
            failed_tokens: failed,
            payments: Arc::new(Mutex::new(HashMap::new())),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// Enable the service
    pub async fn enable(&self) {
        let mut enabled = self.enabled.lock().await;
        *enabled = true;
    }

    /// Disable the service (simulates gateway downtime)
    pub async fn disable(&self) {
        let mut enabled = self.enabled.lock().await;
        *enabled = false;
    }

    /// Check if service is enabled
    pub async fn is_enabled(&self) -> bool {
        *self.enabled.lock().await
    }

    /// Process a payment
    pub async fn process_payment(
        &self,
        order_id: Uuid,
        amount: i64,
        currency: &str,
        card_token: &str,
    ) -> Result<MockPaymentRecord, MockPaymentError> {
        // Check if service is enabled
        if !self.is_enabled().await {
            return Err(MockPaymentError::ServiceUnavailable);
        }

        // Validate amount
        if amount <= 0 {
            return Err(MockPaymentError::InvalidAmount);
        }

        // Check if token is valid
        let status = if self.valid_tokens.contains(&card_token.to_string()) {
            MockPaymentStatus::Approved
        } else if self.failed_tokens.contains(&card_token.to_string()) {
            MockPaymentStatus::Declined
        } else {
            // Unknown token - random behavior for testing
            MockPaymentStatus::Error
        };

        let payment = MockPaymentRecord {
            id: Uuid::new_v4(),
            order_id,
            amount,
            currency: currency.to_string(),
            status,
            card_token: card_token.to_string(),
            created_at: chrono::Utc::now(),
        };

        // Store payment record
        let mut payments = self.payments.lock().await;
        payments.insert(payment.id, payment.clone());

        // Return based on status
        match status {
            MockPaymentStatus::Approved => Ok(payment),
            MockPaymentStatus::Declined => Err(MockPaymentError::CardDeclined),
            MockPaymentStatus::Error => Err(MockPaymentError::ProcessingError),
            _ => Err(MockPaymentError::Unknown),
        }
    }

    /// Get a payment by ID
    pub async fn get_payment(&self, payment_id: Uuid) -> Option<MockPaymentRecord> {
        let payments = self.payments.lock().await;
        payments.get(&payment_id).cloned()
    }

    /// Get all payments for an order
    pub async fn get_payments_for_order(&self, order_id: Uuid) -> Vec<MockPaymentRecord> {
        let payments = self.payments.lock().await;
        payments
            .values()
            .filter(|p| p.order_id == order_id)
            .cloned()
            .collect()
    }

    /// Get total successful payments for an order
    pub async fn get_total_paid(&self, order_id: Uuid) -> i64 {
        let payments = self.payments.lock().await;
        payments
            .values()
            .filter(|p| p.order_id == order_id && p.status == MockPaymentStatus::Approved)
            .map(|p| p.amount)
            .sum()
    }

    /// Clear all payment records
    pub async fn clear(&self) {
        let mut payments = self.payments.lock().await;
        payments.clear();
    }

    /// Add a valid token
    pub fn add_valid_token(&mut self, token: &str) {
        self.valid_tokens.push(token.to_string());
    }

    /// Add a failed token
    pub fn add_failed_token(&mut self, token: &str) {
        self.failed_tokens.push(token.to_string());
    }
}

impl Default for MockPaymentService {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock payment errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MockPaymentError {
    ServiceUnavailable,
    InvalidAmount,
    CardDeclined,
    ProcessingError,
    Unknown,
}

impl std::fmt::Display for MockPaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServiceUnavailable => write!(f, "Payment service is unavailable"),
            Self::InvalidAmount => write!(f, "Invalid payment amount"),
            Self::CardDeclined => write!(f, "Card was declined"),
            Self::ProcessingError => write!(f, "Payment processing error"),
            Self::Unknown => write!(f, "Unknown payment error"),
        }
    }
}

impl std::error::Error for MockPaymentError {}

/// Test payment configuration
#[derive(Debug, Clone)]
pub struct TestPaymentConfig {
    pub use_mock: bool,
    pub auto_approve: bool,
    pub delay_ms: u64,
}

impl Default for TestPaymentConfig {
    fn default() -> Self {
        Self {
            use_mock: true,
            auto_approve: true,
            delay_ms: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_payment_success() {
        let service = MockPaymentService::new();
        
        let result = service
            .process_payment(Uuid::new_v4(), 1000, "USD", "tok_test_visa")
            .await;
        
        assert!(result.is_ok());
        let payment = result.unwrap();
        assert_eq!(payment.status, MockPaymentStatus::Approved);
        assert_eq!(payment.amount, 1000);
    }

    #[tokio::test]
    async fn test_mock_payment_failure() {
        let service = MockPaymentService::new();
        
        let result = service
            .process_payment(Uuid::new_v4(), 1000, "USD", "tok_fail")
            .await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MockPaymentError::CardDeclined);
    }

    #[tokio::test]
    async fn test_mock_payment_service_disabled() {
        let service = MockPaymentService::new();
        service.disable().await;
        
        let result = service
            .process_payment(Uuid::new_v4(), 1000, "USD", "tok_test_visa")
            .await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MockPaymentError::ServiceUnavailable);
    }

    #[tokio::test]
    async fn test_mock_payment_invalid_amount() {
        let service = MockPaymentService::new();
        
        let result = service
            .process_payment(Uuid::new_v4(), -100, "USD", "tok_test_visa")
            .await;
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), MockPaymentError::InvalidAmount);
    }

    #[tokio::test]
    async fn test_get_payments_for_order() {
        let service = MockPaymentService::new();
        let order_id = Uuid::new_v4();
        
        // Process multiple payments for the same order
        let _ = service
            .process_payment(order_id, 1000, "USD", "tok_test_visa")
            .await;
        let _ = service
            .process_payment(order_id, 500, "USD", "tok_test_visa")
            .await;
        
        let payments = service.get_payments_for_order(order_id).await;
        assert_eq!(payments.len(), 2);
        
        let total = service.get_total_paid(order_id).await;
        assert_eq!(total, 1000); // Only successful payments count
    }
}
