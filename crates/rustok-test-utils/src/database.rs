//! Database testing utilities
//!
//! Provides helpers for setting up test databases, running migrations, and managing test data.

use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use std::time::Duration;
use tracing::info;
use uuid::Uuid;

/// Test database configuration
#[derive(Debug, Clone)]
pub struct TestDbConfig {
    /// Database host
    pub host: String,
    /// Database port
    pub port: u16,
    /// Database user
    pub user: String,
    /// Database password
    pub password: String,
    /// Base database name (will be appended with test UUID)
    pub base_name: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Whether to drop database on cleanup
    pub auto_cleanup: bool,
}

impl Default for TestDbConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("TEST_DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("TEST_DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            user: std::env::var("TEST_DB_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("TEST_DB_PASSWORD")
                .unwrap_or_else(|_| "postgres".to_string()),
            base_name: std::env::var("TEST_DB_NAME")
                .unwrap_or_else(|_| "rustok_test".to_string()),
            max_connections: 5,
            connect_timeout: Duration::from_secs(10),
            auto_cleanup: true,
        }
    }
}

impl TestDbConfig {
    /// Create a new test database configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the database host
    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    /// Set the database port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the database user
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = user.into();
        self
    }

    /// Set the database password
    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = password.into();
        self
    }

    /// Set the base database name
    pub fn with_base_name(mut self, base_name: impl Into<String>) -> Self {
        self.base_name = base_name.into();
        self
    }

    /// Set maximum number of connections
    pub fn with_max_connections(mut self, max_connections: u32) -> Self {
        self.max_connections = max_connections;
        self
    }

    /// Set connection timeout
    pub fn with_connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Enable/disable automatic cleanup
    pub fn with_auto_cleanup(mut self, auto_cleanup: bool) -> Self {
        self.auto_cleanup = auto_cleanup;
        self
    }

    /// Generate a unique database name for this test run
    pub fn unique_db_name(&self) -> String {
        format!("{}_{}", self.base_name, Uuid::new_v4().to_string().replace('-', ""))
    }

    /// Build the database URL
    pub fn database_url(&self, db_name: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.user, self.password, self.host, self.port, db_name
        )
    }

    /// Build the connection URL (to postgres database for admin operations)
    pub fn admin_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/postgres",
            self.user, self.password, self.host, self.port
        )
    }
}

/// Test database instance
///
/// Automatically creates a unique test database and cleans it up on drop.
pub struct TestDatabase {
    config: TestDbConfig,
    db_name: String,
    connection: DatabaseConnection,
}

impl TestDatabase {
    /// Create a new test database with default configuration
    pub async fn new() -> Result<Self, DbErr> {
        Self::with_config(TestDbConfig::default()).await
    }

    /// Create a new test database with custom configuration
    pub async fn with_config(config: TestDbConfig) -> Result<Self, DbErr> {
        let db_name = config.unique_db_name();

        info!("Creating test database: {}", db_name);

        // Connect to postgres database to create test database
        let admin_conn = Database::connect(&config.admin_url()).await?;

        // Create test database
        let create_db_sql = format!("CREATE DATABASE \"{}\"", db_name);
        admin_conn
            .execute(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                create_db_sql,
            ))
            .await?;

        drop(admin_conn);

        // Connect to the new test database
        let mut opt = ConnectOptions::new(config.database_url(&db_name));
        opt.max_connections(config.max_connections)
            .connect_timeout(config.connect_timeout)
            .sqlx_logging(false); // Disable query logging in tests

        let connection = Database::connect(opt).await?;

        info!("Test database ready: {}", db_name);

        Ok(Self {
            config,
            db_name,
            connection,
        })
    }

    /// Get the database connection
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Get the database name
    pub fn db_name(&self) -> &str {
        &self.db_name
    }

    /// Get the database URL
    pub fn database_url(&self) -> String {
        self.config.database_url(&self.db_name)
    }

    /// Run migrations (requires migration runner to be implemented)
    ///
    /// Example usage:
    /// ```ignore
    /// use migration::Migrator;
    /// use sea_orm_migration::MigratorTrait;
    ///
    /// let db = TestDatabase::new().await?;
    /// db.run_migrations::<Migrator>().await?;
    /// ```
    pub async fn run_migrations<M>(&self) -> Result<(), DbErr>
    where
        M: sea_orm_migration::MigratorTrait,
    {
        info!("Running migrations on test database: {}", self.db_name);
        M::up(&self.connection, None).await?;
        info!("Migrations completed");
        Ok(())
    }

    /// Truncate all tables (useful for test cleanup without dropping database)
    pub async fn truncate_all_tables(&self) -> Result<(), DbErr> {
        info!("Truncating all tables in test database: {}", self.db_name);

        // Get all table names
        let tables_query = r#"
            SELECT tablename 
            FROM pg_tables 
            WHERE schemaname = 'public'
        "#;

        let tables: Vec<(String,)> = sqlx::query_as(tables_query)
            .fetch_all(self.connection.get_postgres_connection_pool())
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get table names: {}", e)))?;

        // Truncate each table
        for (table_name,) in tables {
            let truncate_sql = format!("TRUNCATE TABLE \"{}\" CASCADE", table_name);
            self.connection
                .execute(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    truncate_sql,
                ))
                .await?;
        }

        info!("All tables truncated");
        Ok(())
    }

    /// Reset sequences to start from 1
    pub async fn reset_sequences(&self) -> Result<(), DbErr> {
        info!("Resetting sequences in test database: {}", self.db_name);

        let sequences_query = r#"
            SELECT sequence_name
            FROM information_schema.sequences
            WHERE sequence_schema = 'public'
        "#;

        let sequences: Vec<(String,)> = sqlx::query_as(sequences_query)
            .fetch_all(self.connection.get_postgres_connection_pool())
            .await
            .map_err(|e| DbErr::Custom(format!("Failed to get sequence names: {}", e)))?;

        for (sequence_name,) in sequences {
            let reset_sql = format!("ALTER SEQUENCE \"{}\" RESTART WITH 1", sequence_name);
            self.connection
                .execute(sea_orm::Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    reset_sql,
                ))
                .await?;
        }

        info!("Sequences reset");
        Ok(())
    }

    /// Clean up test data (truncate tables and reset sequences)
    pub async fn cleanup(&self) -> Result<(), DbErr> {
        self.truncate_all_tables().await?;
        self.reset_sequences().await?;
        Ok(())
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if !self.config.auto_cleanup {
            info!("Skipping cleanup for test database: {}", self.db_name);
            return;
        }

        info!("Cleaning up test database: {}", self.db_name);

        // Drop the database (requires async runtime)
        let db_name = self.db_name.clone();
        let admin_url = self.config.admin_url();

        tokio::task::spawn(async move {
            match Database::connect(&admin_url).await {
                Ok(admin_conn) => {
                    // Terminate existing connections
                    let terminate_sql = format!(
                        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
                        db_name
                    );

                    let _ = admin_conn
                        .execute(sea_orm::Statement::from_string(
                            sea_orm::DatabaseBackend::Postgres,
                            terminate_sql,
                        ))
                        .await;

                    // Drop database
                    let drop_db_sql = format!("DROP DATABASE IF EXISTS \"{}\"", db_name);
                    match admin_conn
                        .execute(sea_orm::Statement::from_string(
                            sea_orm::DatabaseBackend::Postgres,
                            drop_db_sql,
                        ))
                        .await
                    {
                        Ok(_) => info!("Test database dropped: {}", db_name),
                        Err(e) => tracing::warn!("Failed to drop test database {}: {}", db_name, e),
                    }
                }
                Err(e) => tracing::warn!("Failed to connect to admin database: {}", e),
            }
        });
    }
}

/// Create a test database with default configuration
///
/// # Example
///
/// ```ignore
/// use rustok_test_utils::database::setup_test_db;
///
/// #[tokio::test]
/// async fn test_with_database() {
///     let db = setup_test_db().await;
///     // Use db.connection() for queries
/// }
/// ```
pub async fn setup_test_db() -> TestDatabase {
    TestDatabase::new()
        .await
        .expect("Failed to create test database")
}

/// Create a test database with migrations
///
/// # Example
///
/// ```ignore
/// use rustok_test_utils::database::setup_test_db_with_migrations;
/// use migration::Migrator;
///
/// #[tokio::test]
/// async fn test_with_database() {
///     let db = setup_test_db_with_migrations::<Migrator>().await;
///     // Use db.connection() for queries
/// }
/// ```
pub async fn setup_test_db_with_migrations<M>() -> TestDatabase
where
    M: sea_orm_migration::MigratorTrait,
{
    let db = setup_test_db().await;
    db.run_migrations::<M>()
        .await
        .expect("Failed to run migrations");
    db
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires PostgreSQL running
    async fn test_create_and_drop_database() {
        let db = TestDatabase::new().await.expect("Failed to create test database");
        assert!(!db.db_name().is_empty());
        assert!(db.database_url().contains(&db.db_name));
        // Database will be automatically dropped when db goes out of scope
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL running
    async fn test_truncate_tables() {
        let db = TestDatabase::new().await.expect("Failed to create test database");
        db.truncate_all_tables()
            .await
            .expect("Failed to truncate tables");
    }

    #[tokio::test]
    #[ignore] // Requires PostgreSQL running
    async fn test_custom_config() {
        let config = TestDbConfig::new()
            .with_base_name("custom_test")
            .with_max_connections(3)
            .with_auto_cleanup(false);

        let db = TestDatabase::with_config(config)
            .await
            .expect("Failed to create test database");

        assert!(db.db_name().starts_with("custom_test_"));
    }
}
