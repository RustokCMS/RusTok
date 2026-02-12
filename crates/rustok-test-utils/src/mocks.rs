//! Mock implementations for external services
//!
//! This module provides mock implementations of external services for testing purposes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, Request, ResponseTemplate,
};

/// Mock payment gateway for testing payment processing
///
/// # Example
///
/// ```
/// use rustok_test_utils::mocks::MockPaymentGateway;
///
/// #[tokio::test]
/// async fn test_payment_processing() {
///     let gateway = MockPaymentGateway::new().await;
///     
///     // Configure successful payment
///     gateway.configure_successful_payment("tok_visa", "txn_123").await;
///     
///     // Make payment request to gateway.url()
///     // ...
/// }
/// ```
pub struct MockPaymentGateway {
    server: MockServer,
    transactions: Arc<Mutex<HashMap<String, PaymentTransaction>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTransaction {
    pub token: String,
    pub amount: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub transaction_id: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Success,
    Failed,
    Pending,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChargeRequest {
    pub token: String,
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChargeResponse {
    pub transaction_id: String,
    pub status: String,
    pub amount: i64,
    pub currency: String,
    pub error: Option<String>,
}

impl MockPaymentGateway {
    /// Create a new mock payment gateway
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let transactions = Arc::new(Mutex::new(HashMap::new()));

        Self {
            server,
            transactions,
        }
    }

    /// Get the base URL of the mock server
    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Configure a successful payment for a given token
    pub async fn configure_successful_payment(&self, token: &str, transaction_id: &str) {
        let transactions = self.transactions.clone();
        let token_owned = token.to_string();
        let txn_id_owned = transaction_id.to_string();

        Mock::given(method("POST"))
            .and(path("/charge"))
            .respond_with(move |req: &Request| {
                let body: ChargeRequest = serde_json::from_slice(&req.body).unwrap();

                let mut txns = transactions.lock().unwrap();
                txns.insert(
                    body.token.clone(),
                    PaymentTransaction {
                        token: body.token.clone(),
                        amount: body.amount,
                        currency: body.currency.clone(),
                        status: PaymentStatus::Success,
                        transaction_id: txn_id_owned.clone(),
                        error_message: None,
                    },
                );

                ResponseTemplate::new(200).set_body_json(ChargeResponse {
                    transaction_id: txn_id_owned.clone(),
                    status: "success".to_string(),
                    amount: body.amount,
                    currency: body.currency,
                    error: None,
                })
            })
            .mount(&self.server)
            .await;
    }

    /// Configure a failed payment for a given token
    pub async fn configure_failed_payment(&self, token: &str, error_message: &str) {
        let transactions = self.transactions.clone();
        let token_owned = token.to_string();
        let error_owned = error_message.to_string();

        Mock::given(method("POST"))
            .and(path("/charge"))
            .respond_with(move |req: &Request| {
                let body: ChargeRequest = serde_json::from_slice(&req.body).unwrap();

                let mut txns = transactions.lock().unwrap();
                txns.insert(
                    body.token.clone(),
                    PaymentTransaction {
                        token: body.token.clone(),
                        amount: body.amount,
                        currency: body.currency.clone(),
                        status: PaymentStatus::Failed,
                        transaction_id: "".to_string(),
                        error_message: Some(error_owned.clone()),
                    },
                );

                ResponseTemplate::new(402).set_body_json(ChargeResponse {
                    transaction_id: "".to_string(),
                    status: "failed".to_string(),
                    amount: body.amount,
                    currency: body.currency,
                    error: Some(error_owned.clone()),
                })
            })
            .mount(&self.server)
            .await;
    }

    /// Get all recorded transactions
    pub fn transactions(&self) -> Vec<PaymentTransaction> {
        self.transactions
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Get a specific transaction by token
    pub fn get_transaction(&self, token: &str) -> Option<PaymentTransaction> {
        self.transactions.lock().unwrap().get(token).cloned()
    }

    /// Clear all recorded transactions
    pub fn clear_transactions(&self) {
        self.transactions.lock().unwrap().clear();
    }
}

/// Mock email service for testing email sending
pub struct MockEmailService {
    server: MockServer,
    sent_emails: Arc<Mutex<Vec<EmailMessage>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub is_html: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailRequest {
    pub to: String,
    pub from: String,
    pub subject: String,
    pub body: String,
    pub html: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendEmailResponse {
    pub message_id: String,
    pub status: String,
}

impl MockEmailService {
    /// Create a new mock email service
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let sent_emails = Arc::new(Mutex::new(Vec::new()));

        Self {
            server,
            sent_emails,
        }
    }

    /// Get the base URL of the mock server
    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Mount the mock email endpoint
    pub async fn mount(&self) {
        let emails = self.sent_emails.clone();

        Mock::given(method("POST"))
            .and(path("/send"))
            .respond_with(move |req: &Request| {
                let body: SendEmailRequest = serde_json::from_slice(&req.body).unwrap();

                let mut email_list = emails.lock().unwrap();
                email_list.push(EmailMessage {
                    to: body.to,
                    from: body.from,
                    subject: body.subject,
                    body: body.body,
                    is_html: body.html.unwrap_or(false),
                });

                ResponseTemplate::new(200).set_body_json(SendEmailResponse {
                    message_id: uuid::Uuid::new_v4().to_string(),
                    status: "sent".to_string(),
                })
            })
            .mount(&self.server)
            .await;
    }

    /// Get all sent emails
    pub fn sent_emails(&self) -> Vec<EmailMessage> {
        self.sent_emails.lock().unwrap().clone()
    }

    /// Get the count of sent emails
    pub fn sent_count(&self) -> usize {
        self.sent_emails.lock().unwrap().len()
    }

    /// Check if an email was sent to a specific address
    pub fn was_sent_to(&self, email: &str) -> bool {
        self.sent_emails
            .lock()
            .unwrap()
            .iter()
            .any(|e| e.to == email)
    }

    /// Clear all sent emails
    pub fn clear_sent_emails(&self) {
        self.sent_emails.lock().unwrap().clear();
    }
}

/// Mock storage service (e.g., S3) for testing file uploads
pub struct MockStorageService {
    server: MockServer,
    stored_files: Arc<Mutex<HashMap<String, StoredFile>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredFile {
    pub key: String,
    pub content: Vec<u8>,
    pub content_type: String,
    pub size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadResponse {
    pub key: String,
    pub url: String,
    pub size: usize,
}

impl MockStorageService {
    /// Create a new mock storage service
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        let stored_files = Arc::new(Mutex::new(HashMap::new()));

        Self {
            server,
            stored_files,
        }
    }

    /// Get the base URL of the mock server
    pub fn url(&self) -> String {
        self.server.uri()
    }

    /// Mount the mock upload endpoint
    pub async fn mount(&self) {
        let files = self.stored_files.clone();
        let base_url = self.url();

        Mock::given(method("POST"))
            .and(path("/upload"))
            .respond_with(move |req: &Request| {
                let content = req.body.to_vec();
                let key = format!("file_{}", uuid::Uuid::new_v4());
                let content_type = req
                    .headers
                    .get("content-type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_string();

                let size = content.len();

                let mut file_map = files.lock().unwrap();
                file_map.insert(
                    key.clone(),
                    StoredFile {
                        key: key.clone(),
                        content: content.clone(),
                        content_type: content_type.clone(),
                        size,
                    },
                );

                ResponseTemplate::new(200).set_body_json(UploadResponse {
                    key: key.clone(),
                    url: format!("{}/files/{}", base_url, key),
                    size,
                })
            })
            .mount(&self.server)
            .await;
    }

    /// Get all stored files
    pub fn stored_files(&self) -> Vec<StoredFile> {
        self.stored_files.lock().unwrap().values().cloned().collect()
    }

    /// Get a specific file by key
    pub fn get_file(&self, key: &str) -> Option<StoredFile> {
        self.stored_files.lock().unwrap().get(key).cloned()
    }

    /// Clear all stored files
    pub fn clear_files(&self) {
        self.stored_files.lock().unwrap().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_payment_gateway_success() {
        let gateway = MockPaymentGateway::new().await;
        gateway
            .configure_successful_payment("tok_visa", "txn_123")
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/charge", gateway.url()))
            .json(&ChargeRequest {
                token: "tok_visa".to_string(),
                amount: 1000,
                currency: "USD".to_string(),
            })
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: ChargeResponse = response.json().await.unwrap();
        assert_eq!(body.transaction_id, "txn_123");
        assert_eq!(body.status, "success");

        let txn = gateway.get_transaction("tok_visa").unwrap();
        assert_eq!(txn.status, PaymentStatus::Success);
    }

    #[tokio::test]
    async fn test_mock_payment_gateway_failure() {
        let gateway = MockPaymentGateway::new().await;
        gateway
            .configure_failed_payment("tok_invalid", "Card declined")
            .await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/charge", gateway.url()))
            .json(&ChargeRequest {
                token: "tok_invalid".to_string(),
                amount: 1000,
                currency: "USD".to_string(),
            })
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 402);

        let body: ChargeResponse = response.json().await.unwrap();
        assert_eq!(body.status, "failed");
        assert_eq!(body.error, Some("Card declined".to_string()));
    }

    #[tokio::test]
    async fn test_mock_email_service() {
        let email_service = MockEmailService::new().await;
        email_service.mount().await;

        let client = reqwest::Client::new();
        client
            .post(format!("{}/send", email_service.url()))
            .json(&SendEmailRequest {
                to: "test@example.com".to_string(),
                from: "noreply@example.com".to_string(),
                subject: "Test".to_string(),
                body: "Hello".to_string(),
                html: Some(false),
            })
            .send()
            .await
            .unwrap();

        assert_eq!(email_service.sent_count(), 1);
        assert!(email_service.was_sent_to("test@example.com"));
    }

    #[tokio::test]
    async fn test_mock_storage_service() {
        let storage = MockStorageService::new().await;
        storage.mount().await;

        let client = reqwest::Client::new();
        let response = client
            .post(format!("{}/upload", storage.url()))
            .body(b"test file content".to_vec())
            .header("content-type", "text/plain")
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body: UploadResponse = response.json().await.unwrap();
        assert!(!body.key.is_empty());
        assert_eq!(body.size, 17);

        let file = storage.get_file(&body.key).unwrap();
        assert_eq!(file.content, b"test file content");
        assert_eq!(file.content_type, "text/plain");
    }
}
