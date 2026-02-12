//! # Test Application
//!
//! Provides a test application wrapper for integration testing.

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::Mutex;
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use rustok_content::dto::{CreateNodeInput, NodeResponse, TranslationInput};
use rustok_commerce::dto::{
    CreateProductInput, ProductResponse,
    CreateOrderInput, OrderResponse, OrderItemInput, ProcessPaymentInput, PaymentResponse
};
use rustok_core::events::types::DomainEvent;

use crate::test_server::{TestServer, TestServerConfig, spawn_test_server};

/// Test application for integration testing
pub struct TestApp {
    /// Database connection
    pub db: Arc<sea_orm::DatabaseConnection>,
    /// HTTP client for API calls
    pub client: reqwest::Client,
    /// Base URL for the test server
    pub base_url: String,
    /// Authentication token
    pub auth_token: String,
    /// Tenant identifier
    pub tenant_id: String,
    /// Captured events
    pub events: Arc<Mutex<Vec<DomainEvent>>>,
    /// User ID
    pub user_id: Uuid,
    /// Optional test server handle (if spawned)
    pub server: Option<TestServer>,
}

impl TestApp {
    /// Create a new test application
    pub async fn new() -> Result<Self, TestAppError> {
        let db = Self::create_test_db().await?;
        let base_url = std::env::var("TEST_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:3000".to_string());

        let auth_token = std::env::var("TEST_AUTH_TOKEN")
            .unwrap_or_else(|_| "test_token".to_string());

        let tenant_id = std::env::var("TEST_TENANT_ID")
            .unwrap_or_else(|_| "test-tenant".to_string());

        let user_id = std::env::var("TEST_USER_ID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::new_v4);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| TestAppError::ClientError(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
            client,
            base_url,
            auth_token,
            tenant_id,
            events: Arc::new(Mutex::new(Vec::new())),
            user_id,
        })
    }

    /// Create a new test application with a custom server URL
    pub async fn with_server_url(base_url: String) -> Result<Self, TestAppError> {
        let db = Self::create_test_db().await?;

        let auth_token = std::env::var("TEST_AUTH_TOKEN")
            .unwrap_or_else(|_| "test_token".to_string());

        let tenant_id = std::env::var("TEST_TENANT_ID")
            .unwrap_or_else(|_| "test-tenant".to_string());

        let user_id = std::env::var("TEST_USER_ID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::new_v4);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| TestAppError::ClientError(e.to_string()))?;

        Ok(Self {
            db: Arc::new(db),
            client,
            base_url,
            auth_token,
            tenant_id,
            events: Arc::new(Mutex::new(Vec::new())),
            user_id,
            server: None,
        })
    }

    /// Create a new test application with a spawned server
    pub async fn new_with_server() -> Result<Self, TestAppError> {
        let config = TestServerConfig::default();
        Self::new_with_config(config).await
    }

    /// Create a new test application with custom server configuration
    pub async fn new_with_config(config: TestServerConfig) -> Result<Self, TestAppError> {
        let server = spawn_test_server(config.clone()).await;
        let db = Self::create_test_db().await?;
        
        let auth_token = config.auth_token;
        let tenant_id = config.tenant_id;
        
        let user_id = std::env::var("TEST_USER_ID")
            .ok()
            .and_then(|s| Uuid::parse_str(&s).ok())
            .unwrap_or_else(Uuid::new_v4);
        
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| TestAppError::ClientError(e.to_string()))?;
        
        Ok(Self {
            db: Arc::new(db),
            client,
            base_url: server.base_url().to_string(),
            auth_token,
            tenant_id,
            events: Arc::new(Mutex::new(Vec::new())),
            user_id,
            server: Some(server),
        })
    }

    /// Shutdown the test application and server
    pub async fn shutdown(self) {
        if let Some(server) = self.server {
            server.shutdown().await;
        }
    }
    
    /// Create a test database connection
    async fn create_test_db() -> Result<sea_orm::DatabaseConnection, TestAppError> {
        use sea_orm::{Database, ConnectOptions};
        use std::time::Duration;
        
        let database_url = std::env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/rustok_test".to_string());
        
        let mut opt = ConnectOptions::new(database_url);
        opt.max_connections(5)
            .connect_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(300))
            .sqlx_logging(false);
        
        Database::connect(opt).await
            .map_err(|e| TestAppError::DatabaseError(e.to_string()))
    }
    
    /// Get the authorization header value
    pub fn auth_header(&self) -> String {
        format!("Bearer {}", self.auth_token)
    }
    
    /// Subscribe to events
    pub async fn subscribe_to_events(&self) {
        // In a real implementation, this would set up event subscriptions
        // For now, we'll just capture events in the in-memory buffer
    }
    
    // ========================================================================
    // Content/Node Operations
    // ========================================================================
    
    /// Create a node
    pub async fn create_node(&self, input: CreateNodeInput) -> Result<NodeResponse, TestAppError> {
        let response = self
            .client
            .post(format!("{}/api/v1/nodes", self.base_url))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&input)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let node: NodeResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(node)
    }
    
    /// Get a node by ID
    pub async fn get_node(&self, node_id: Uuid) -> Result<NodeResponse, TestAppError> {
        let response = self
            .client
            .get(format!("{}/api/v1/nodes/{}", self.base_url, node_id))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let node: NodeResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(node)
    }
    
    /// Publish a node
    pub async fn publish_node(&self, node_id: Uuid) -> Result<NodeResponse, TestAppError> {
        #[derive(Serialize)]
        struct PublishInput {
            status: String,
        }
        
        let response = self
            .client
            .patch(format!("{}/api/v1/nodes/{}", self.base_url, node_id))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&PublishInput {
                status: "published".to_string(),
            })
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let node: NodeResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(node)
    }
    
    /// Add a translation to a node
    pub async fn add_translation(
        &self,
        node_id: Uuid,
        locale: &str,
        input: TranslationInput,
    ) -> Result<NodeResponse, TestAppError> {
        let response = self
            .client
            .post(format!(
                "{}/api/v1/nodes/{}/translations/{}",
                self.base_url, node_id, locale
            ))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&input)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let node: NodeResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(node)
    }
    
    /// Search nodes
    pub async fn search_nodes(&self, query: &str) -> Result<Vec<NodeResponse>, TestAppError> {
        let response = self
            .client
            .get(format!("{}/api/v1/nodes/search", self.base_url))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let nodes: Vec<NodeResponse> = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(nodes)
    }
    
    // ========================================================================
    // Commerce/Product Operations
    // ========================================================================
    
    /// Create a product
    pub async fn create_product(&self, input: CreateProductInput) -> Result<ProductResponse, TestAppError> {
        let response = self
            .client
            .post(format!("{}/api/v1/products", self.base_url))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&input)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let product: ProductResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(product)
    }
    
    /// Get a product by ID
    pub async fn get_product(&self, product_id: Uuid) -> Result<ProductResponse, TestAppError> {
        let response = self
            .client
            .get(format!("{}/api/v1/products/{}", self.base_url, product_id))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let product: ProductResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(product)
    }
    
    // ========================================================================
    // Commerce/Order Operations
    // ========================================================================
    
    /// Create an order
    pub async fn create_order(&self, input: CreateOrderInput) -> Result<OrderResponse, TestAppError> {
        let response = self
            .client
            .post(format!("{}/api/v1/orders", self.base_url))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&input)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let order: OrderResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(order)
    }
    
    /// Get an order by ID
    pub async fn get_order(&self, order_id: Uuid) -> Result<OrderResponse, TestAppError> {
        let response = self
            .client
            .get(format!("{}/api/v1/orders/{}", self.base_url, order_id))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let order: OrderResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(order)
    }
    
    /// Submit an order
    pub async fn submit_order(&self, order_id: Uuid) -> Result<OrderResponse, TestAppError> {
        #[derive(Serialize)]
        struct SubmitInput {
            action: String,
        }
        
        let response = self
            .client
            .post(format!("{}/api/v1/orders/{}/submit", self.base_url, order_id))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&SubmitInput {
                action: "submit".to_string(),
            })
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let order: OrderResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(order)
    }
    
    /// Process payment for an order
    pub async fn process_payment(&self, order_id: Uuid, input: ProcessPaymentInput) -> Result<PaymentResponse, TestAppError> {
        let response = self
            .client
            .post(format!("{}/api/v1/orders/{}/payment", self.base_url, order_id))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .json(&input)
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let payment: PaymentResponse = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(payment)
    }
    
    /// Search orders
    pub async fn search_orders(&self, query: &str) -> Result<Vec<OrderResponse>, TestAppError> {
        let response = self
            .client
            .get(format!("{}/api/v1/orders/search", self.base_url))
            .header("Authorization", self.auth_header())
            .header("X-Tenant-Id", &self.tenant_id)
            .query(&[("q", query)])
            .send()
            .await
            .map_err(|e| TestAppError::RequestError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(TestAppError::ApiError {
                status: status.as_u16(),
                message: text,
            });
        }
        
        let orders: Vec<OrderResponse> = response
            .json()
            .await
            .map_err(|e| TestAppError::ResponseError(e.to_string()))?;
        
        Ok(orders)
    }
    
    // ========================================================================
    // Event Operations
    // ========================================================================
    
    /// Get events for a specific node
    pub async fn get_events_for_node(&self, node_id: Uuid) -> Vec<DomainEvent> {
        // In a real implementation, this would query the outbox table
        // For now, return captured events from memory
        let events = self.events.lock().await;
        events
            .iter()
            .filter(|e| {
                if let DomainEvent::NodeCreated { node_id: id, .. } = e {
                    *id == node_id
                } else if let DomainEvent::NodePublished { node_id: id, .. } = e {
                    *id == node_id
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// Get events for a specific order
    pub async fn get_events_for_order(&self, order_id: Uuid) -> Vec<DomainEvent> {
        let events = self.events.lock().await;
        events
            .iter()
            .filter(|e| {
                if let DomainEvent::OrderCreated { order_id: id, .. } = e {
                    *id == order_id
                } else if let DomainEvent::OrderPaid { order_id: id, .. } = e {
                    *id == order_id
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }
    
    /// Get all outbox events
    pub async fn get_outbox_events(&self) -> Vec<serde_json::Value> {
        // In a real implementation, this would query the outbox table
        vec![]
    }
    
    /// Get relayed events count
    pub async fn get_relayed_events(&self) -> i64 {
        // In a real implementation, this would query metrics
        0
    }
}

/// Error type for test application
#[derive(Debug, thiserror::Error)]
pub enum TestAppError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Client error: {0}")]
    ClientError(String),
    
    #[error("Request error: {0}")]
    RequestError(String),
    
    #[error("Response error: {0}")]
    ResponseError(String),
    
    #[error("API error: status={status}, message={message}")]
    ApiError {
        status: u16,
        message: String,
    },
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
}

/// Helper function to spawn a test application
pub async fn spawn_test_app() -> TestApp {
    TestApp::new()
        .await
        .expect("Failed to create test application")
}

/// Helper function to spawn a test application with a running server
pub async fn spawn_test_app_with_server() -> TestApp {
    TestApp::new_with_server()
        .await
        .expect("Failed to create test application with server")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_creation() {
        let app = spawn_test_app().await;
        assert!(!app.auth_token.is_empty());
        assert!(!app.tenant_id.is_empty());
    }

    #[tokio::test]
    async fn test_auth_header_format() {
        let app = TestApp {
            db: Arc::new(
                sea_orm::Database::connect("sqlite::memory:")
                    .await
                    .unwrap()
            ),
            client: reqwest::Client::new(),
            base_url: "http://localhost".to_string(),
            auth_token: "test_token".to_string(),
            tenant_id: "test-tenant".to_string(),
            events: Arc::new(Mutex::new(vec![])),
            user_id: Uuid::new_v4(),
            server: None,
        };
        
        assert_eq!(app.auth_header(), "Bearer test_token");
    }

    #[tokio::test]
    #[ignore] // Requires server implementation
    async fn test_app_with_server() {
        let app = spawn_test_app_with_server().await;
        assert!(!app.base_url.is_empty());
        assert!(app.server.is_some());
        app.shutdown().await;
    }
}
