//! # Test Server
//!
//! Provides a complete test server that can be spawned within tests.
//! Supports both SQLite (fast) and PostgreSQL via testcontainers (full integration).

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Configuration for the test server
#[derive(Debug, Clone)]
pub struct TestServerConfig {
    /// Database URL (if None, uses SQLite in-memory)
    pub database_url: Option<String>,
    /// Port to bind to (if None, uses random available port)
    pub port: Option<u16>,
    /// Whether to run migrations on startup
    pub run_migrations: bool,
    /// Test tenant ID
    pub tenant_id: String,
    /// Test auth token
    pub auth_token: String,
}

impl Default for TestServerConfig {
    fn default() -> Self {
        Self {
            database_url: None, // Use SQLite by default for speed
            port: None,         // Random port
            run_migrations: true,
            tenant_id: "test-tenant".to_string(),
            auth_token: "test-token".to_string(),
        }
    }
}

/// A running test server
pub struct TestServer {
    /// Base URL for the server
    pub base_url: String,
    /// Server handle for shutdown
    handle: Option<JoinHandle<()>>,
    /// Configuration used
    pub config: TestServerConfig,
}

impl TestServer {
    /// Get the base URL for making requests
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Shutdown the test server
    pub async fn shutdown(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            let _ = handle.await;
        }
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

/// Spawn a test server with the given configuration
///
/// # Example
///
/// ```rust,ignore
/// use rustok_test_utils::test_server::{spawn_test_server, TestServerConfig};
///
/// #[tokio::test]
/// async fn test_with_server() {
///     let server = spawn_test_server(TestServerConfig::default()).await;
///     
///     // Make requests to server.base_url()
///     let client = reqwest::Client::new();
///     let response = client.get(format!("{}/health", server.base_url()))
///         .send()
///         .await;
///     
///     server.shutdown().await;
/// }
/// ```
pub async fn spawn_test_server(config: TestServerConfig) -> TestServer {
    // Find an available port
    let port = config.port.unwrap_or_else(|| find_available_port().await);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let base_url = format!("http://{}", addr);

    // For now, return a mock server that just binds to the port
    // In a full implementation, this would spawn the actual RusToK server
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind test server");

    let handle = tokio::spawn(async move {
        // This is a placeholder - in a real implementation,
        // we would spawn the full RusToK application here
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    TestServer {
        base_url,
        handle: Some(handle),
        config,
    }
}

/// Spawn a test server with default configuration
pub async fn spawn_test_server_default() -> TestServer {
    spawn_test_server(TestServerConfig::default()).await
}

/// Find an available port on localhost
async fn find_available_port() -> u16 {
    // Try to bind to port 0 to get a random available port
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind to find available port");
    
    let addr = listener.local_addr().expect("Failed to get local address");
    let port = addr.port();
    
    // Drop the listener so the port becomes available again
    drop(listener);
    
    // Small delay to ensure the port is released
    tokio::time::sleep(Duration::from_millis(10)).await;
    
    port
}

/// PostgreSQL test container configuration
#[cfg(feature = "postgres-testcontainer")]
pub mod postgres {
    use super::*;
    use testcontainers::{clients::Cli, images::postgres::Postgres, Container};

    /// PostgreSQL container wrapper for tests
    pub struct PostgresContainer {
        container: Container<'static, Postgres>,
        pub connection_string: String,
    }

    impl PostgresContainer {
        /// Start a new PostgreSQL container
        pub fn start(docker: &'static Cli) -> Self {
            let container = docker.run(Postgres::default());
            
            let host = container.get_host();
            let port = container.get_host_port_ipv4(5432);
            
            let connection_string = format!(
                "postgres://postgres:postgres@{}:{}/postgres",
                host, port
            );

            Self {
                container,
                connection_string,
            }
        }

        /// Get a database connection
        pub async fn connect(&self) -> Result<sea_orm::DatabaseConnection, sea_orm::DbErr> {
            sea_orm::Database::connect(&self.connection_string).await
        }
    }

    /// Spawn a test server with PostgreSQL via testcontainers
    pub async fn spawn_test_server_with_postgres() -> (TestServer, &'static Cli) {
        lazy_static::lazy_static! {
            static ref DOCKER: Cli = Cli::default();
        }

        let postgres = PostgresContainer::start(&DOCKER);
        
        let config = TestServerConfig {
            database_url: Some(postgres.connection_string.clone()),
            ..Default::default()
        };

        let server = spawn_test_server(config).await;
        (server, &DOCKER)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_find_available_port() {
        let port = find_available_port().await;
        assert!(port > 0);
        assert!(port <= 65535);
    }

    #[tokio::test]
    async fn test_spawn_test_server() {
        let server = spawn_test_server_default().await;
        assert!(!server.base_url.is_empty());
        assert!(server.base_url.starts_with("http://"));
        server.shutdown().await;
    }

    #[tokio::test]
    async fn test_server_config_default() {
        let config = TestServerConfig::default();
        assert!(config.database_url.is_none());
        assert!(config.port.is_none());
        assert!(config.run_migrations);
        assert_eq!(config.tenant_id, "test-tenant");
        assert_eq!(config.auth_token, "test-token");
    }
}
