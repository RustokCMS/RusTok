//! # Mock Services
//!
//! Provides mock implementations for external services used in tests.

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// Payment Gateway Mock
// ============================================================================

/// Mock payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPaymentRequest {
    pub card_token: String,
    pub amount: i64,
    pub currency: String,
    pub customer_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

/// Mock payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPaymentResponse {
    pub success: bool,
    pub payment_id: String,
    pub transaction_id: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub amount: i64,
    pub currency: String,
}

/// Mock payment gateway
#[derive(Debug, Clone)]
pub struct MockPaymentGateway {
    /// Tokens that will succeed
    success_tokens: Vec<String>,
    /// Tokens that will fail with specific error codes
    failure_tokens: HashMap<String, (String, String)>, // token -> (error_code, error_message)
    /// Payment ID prefix
    id_prefix: String,
}

impl Default for MockPaymentGateway {
    fn default() -> Self {
        Self::new()
    }
}

impl MockPaymentGateway {
    /// Create a new mock payment gateway
    pub fn new() -> Self {
        Self {
            success_tokens: vec![
                "tok_test_visa".to_string(),
                "tok_test_mastercard".to_string(),
                "tok_test_amex".to_string(),
            ],
            failure_tokens: HashMap::from([
                ("tok_fail".to_string(), ("card_declined".to_string(), "Card declined".to_string())),
                ("tok_expired".to_string(), ("expired_card".to_string(), "Card has expired".to_string())),
                ("tok_insufficient".to_string(), ("insufficient_funds".to_string(), "Insufficient funds".to_string())),
            ]),
            id_prefix: "pay_mock_".to_string(),
        }
    }

    /// Add a token that will succeed
    pub fn with_success_token(mut self, token: &str) -> Self {
        self.success_tokens.push(token.to_string());
        self
    }

    /// Add a token that will fail with specific error
    pub fn with_failure_token(mut self, token: &str, error_code: &str, error_message: &str) -> Self {
        self.failure_tokens.insert(token.to_string(), (error_code.to_string(), error_message.to_string()));
        self
    }

    /// Set payment ID prefix
    pub fn with_id_prefix(mut self, prefix: &str) -> Self {
        self.id_prefix = prefix.to_string();
        self
    }

    /// Process a payment
    pub fn process_payment(&self, request: MockPaymentRequest) -> MockPaymentResponse {
        // Check for specific failure tokens
        if let Some((error_code, error_message)) = self.failure_tokens.get(&request.card_token) {
            return MockPaymentResponse {
                success: false,
                payment_id: format!("{}failed", self.id_prefix),
                transaction_id: None,
                error_code: Some(error_code.clone()),
                error_message: Some(error_message.clone()),
                amount: request.amount,
                currency: request.currency,
            };
        }

        // Check for success tokens
        if self.success_tokens.contains(&request.card_token) {
            return MockPaymentResponse {
                success: true,
                payment_id: format!("{}{}", self.id_prefix, Uuid::new_v4()),
                transaction_id: Some(format!("txn_{}", Uuid::new_v4())),
                error_code: None,
                error_message: None,
                amount: request.amount,
                currency: request.currency,
            };
        }

        // Unknown token - treat as failure
        MockPaymentResponse {
            success: false,
            payment_id: format!("{}unknown", self.id_prefix),
            transaction_id: None,
            error_code: Some("invalid_token".to_string()),
            error_message: Some("Invalid card token".to_string()),
            amount: request.amount,
            currency: request.currency,
        }
    }

    /// Check if a token will succeed
    pub fn will_succeed(&self, token: &str) -> bool {
        self.success_tokens.contains(token)
    }
}

// ============================================================================
// Email Service Mock
// ============================================================================

/// Mock email request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockEmailRequest {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub html: Option<bool>,
}

/// Mock email response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockEmailResponse {
    pub success: bool,
    pub message_id: String,
    pub error: Option<String>,
}

/// Mock email service
#[derive(Debug, Clone, Default)]
pub struct MockEmailService {
    sent_emails: Vec<MockEmailRequest>,
    id_prefix: String,
}

impl MockEmailService {
    /// Create a new mock email service
    pub fn new() -> Self {
        Self {
            sent_emails: Vec::new(),
            id_prefix: "email_mock_".to_string(),
        }
    }

    /// Set message ID prefix
    pub fn with_id_prefix(mut self, prefix: &str) -> Self {
        self.id_prefix = prefix.to_string();
        self
    }

    /// Send an email
    pub fn send_email(&mut self, request: MockEmailRequest) -> MockEmailResponse {
        self.sent_emails.push(request.clone());

        MockEmailResponse {
            success: true,
            message_id: format!("{}{}", self.id_prefix, Uuid::new_v4()),
            error: None,
        }
    }

    /// Get all sent emails
    pub fn get_sent_emails(&self) -> &[MockEmailRequest] {
        &self.sent_emails
    }

    /// Find email by recipient
    pub fn find_email_to(&self, to: &str) -> Option<&MockEmailRequest> {
        self.sent_emails.iter().find(|e| e.to == to)
    }

    /// Find emails by subject
    pub fn find_emails_by_subject(&self, subject: &str) -> Vec<&MockEmailRequest> {
        self.sent_emails.iter().filter(|e| e.subject == subject).collect()
    }

    /// Clear all sent emails
    pub fn clear(&mut self) {
        self.sent_emails.clear();
    }
}

// ============================================================================
// SMS Service Mock
// ============================================================================

/// Mock SMS request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSmsRequest {
    pub to: String,
    pub message: String,
}

/// Mock SMS response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSmsResponse {
    pub success: bool,
    pub message_id: String,
    pub error: Option<String>,
}

/// Mock SMS service
#[derive(Debug, Clone, Default)]
pub struct MockSmsService {
    sent_messages: Vec<MockSmsRequest>,
    id_prefix: String,
}

impl MockSmsService {
    /// Create a new mock SMS service
    pub fn new() -> Self {
        Self {
            sent_messages: Vec::new(),
            id_prefix: "sms_mock_".to_string(),
        }
    }

    /// Set message ID prefix
    pub fn with_id_prefix(mut self, prefix: &str) -> Self {
        self.id_prefix = prefix.to_string();
        self
    }

    /// Send an SMS
    pub fn send_sms(&mut self, request: MockSmsRequest) -> MockSmsResponse {
        self.sent_messages.push(request.clone());

        MockSmsResponse {
            success: true,
            message_id: format!("{}{}", self.id_prefix, Uuid::new_v4()),
            error: None,
        }
    }

    /// Get all sent messages
    pub fn get_sent_messages(&self) -> &[MockSmsRequest] {
        &self.sent_messages
    }

    /// Find message by recipient
    pub fn find_message_to(&self, to: &str) -> Option<&MockSmsRequest> {
        self.sent_messages.iter().find(|m| m.to == to)
    }

    /// Clear all sent messages
    pub fn clear(&mut self) {
        self.sent_messages.clear();
    }
}

// ============================================================================
// External API Mock
// ============================================================================

/// Mock API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// Mock external API client
#[derive(Debug, Clone, Default)]
pub struct MockApiClient {
    responses: HashMap<String, serde_json::Value>,
    requests: Vec<(String, serde_json::Value)>,
}

impl MockApiClient {
    /// Create a new mock API client
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            requests: Vec::new(),
        }
    }

    /// Set a mock response for a specific endpoint
    pub fn set_response(&mut self, endpoint: &str, response: serde_json::Value) {
        self.responses.insert(endpoint.to_string(), response);
    }

    /// Make a mock API call
    pub fn call(&mut self, endpoint: &str, request: serde_json::Value) -> MockApiResponse<serde_json::Value> {
        // Record the request
        self.requests.push((endpoint.to_string(), request));

        // Return the mock response
        if let Some(response) = self.responses.get(endpoint) {
            MockApiResponse {
                success: true,
                data: Some(response.clone()),
                error: None,
            }
        } else {
            MockApiResponse {
                success: false,
                data: None,
                error: Some(format!("No mock response set for endpoint: {}", endpoint)),
            }
        }
    }

    /// Get all requests made
    pub fn get_requests(&self) -> &[(String, serde_json::Value)] {
        &self.requests
    }

    /// Find requests by endpoint
    pub fn find_requests_by_endpoint(&self, endpoint: &str) -> Vec<&(String, serde_json::Value)> {
        self.requests.iter().filter(|(e, _)| e == endpoint).collect()
    }

    /// Clear all requests
    pub fn clear_requests(&mut self) {
        self.requests.clear();
    }

    /// Clear all responses
    pub fn clear_responses(&mut self) {
        self.responses.clear();
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a successful payment response
pub fn mock_successful_payment_response(amount: i64, currency: &str) -> MockPaymentResponse {
    MockPaymentResponse {
        success: true,
        payment_id: format!("pay_mock_{}", Uuid::new_v4()),
        transaction_id: Some(format!("txn_{}", Uuid::new_v4())),
        error_code: None,
        error_message: None,
        amount,
        currency: currency.to_string(),
    }
}

/// Create a failed payment response
pub fn mock_failed_payment_response(
    amount: i64,
    currency: &str,
    error_code: &str,
    error_message: &str,
) -> MockPaymentResponse {
    MockPaymentResponse {
        success: false,
        payment_id: format!("pay_mock_failed_{}", Uuid::new_v4()),
        transaction_id: None,
        error_code: Some(error_code.to_string()),
        error_message: Some(error_message.to_string()),
        amount,
        currency: currency.to_string(),
    }
}

/// Create a test payment gateway with default settings
pub fn test_payment_gateway() -> MockPaymentGateway {
    MockPaymentGateway::new()
}

/// Create a test email service
pub fn test_email_service() -> MockEmailService {
    MockEmailService::new()
}

/// Create a test SMS service
pub fn test_sms_service() -> MockSmsService {
    MockSmsService::new()
}

/// Create a test API client
pub fn test_api_client() -> MockApiClient {
    MockApiClient::new()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_gateway_success() {
        let gateway = MockPaymentGateway::new();

        let request = MockPaymentRequest {
            card_token: "tok_test_visa".to_string(),
            amount: 1000,
            currency: "USD".to_string(),
            customer_id: None,
            metadata: None,
        };

        let response = gateway.process_payment(request);

        assert!(response.success);
        assert!(response.payment_id.starts_with("pay_mock_"));
        assert!(response.transaction_id.is_some());
        assert!(response.error_code.is_none());
    }

    #[test]
    fn test_payment_gateway_failure() {
        let gateway = MockPaymentGateway::new();

        let request = MockPaymentRequest {
            card_token: "tok_fail".to_string(),
            amount: 1000,
            currency: "USD".to_string(),
            customer_id: None,
            metadata: None,
        };

        let response = gateway.process_payment(request);

        assert!(!response.success);
        assert_eq!(response.error_code, Some("card_declined".to_string()));
        assert_eq!(response.error_message, Some("Card declined".to_string()));
    }

    #[test]
    fn test_email_service() {
        let mut service = MockEmailService::new();

        let request = MockEmailRequest {
            to: "test@example.com".to_string(),
            subject: "Test Subject".to_string(),
            body: "Test Body".to_string(),
            html: None,
        };

        let response = service.send_email(request);

        assert!(response.success);
        assert_eq!(service.get_sent_emails().len(), 1);
        assert_eq!(service.find_email_to("test@example.com").unwrap().to, "test@example.com");
    }

    #[test]
    fn test_sms_service() {
        let mut service = MockSmsService::new();

        let request = MockSmsRequest {
            to: "+1234567890".to_string(),
            message: "Test SMS".to_string(),
        };

        let response = service.send_sms(request);

        assert!(response.success);
        assert_eq!(service.get_sent_messages().len(), 1);
        assert_eq!(service.find_message_to("+1234567890").unwrap().to, "+1234567890");
    }

    #[test]
    fn test_api_client() {
        let mut client = MockApiClient::new();

        // Set mock response
        client.set_response(
            "/api/data",
            serde_json::json!({"status": "success", "value": 42}),
        );

        // Make API call
        let response = client.call(
            "/api/data",
            serde_json::json!({"query": "test"}),
        );

        assert!(response.success);
        assert_eq!(client.get_requests().len(), 1);
    }
}
