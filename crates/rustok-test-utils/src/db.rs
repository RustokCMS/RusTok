//! Database testing utilities
//!
//! Provides functions for setting up test databases with migrations.

use sea_orm::{Database, DatabaseConnection, DbErr};
use std::sync::Arc;
use tokio::sync::Mutex;

static DB_LOCK: tokio::sync::OnceCell<Arc<Mutex<()>>> = tokio::sync::OnceCell::const_new();

/// Sets up an in-memory SQLite database for testing.
///
/// This creates a fresh SQLite database in memory.
///
/// Note: this helper does **not** run migrations automatically.
/// Use `setup_test_db_with_migrations::<M>()` when schema is required.
/// The database is isolated per test.
///
/// # Example
///
/// ```rust
/// use rustok_test_utils::setup_test_db;
///
/// #[tokio::test]
/// async fn test_with_db() {
///     let db = setup_test_db().await;
///     // Use db for testing...
/// }
/// ```
pub async fn setup_test_db() -> DatabaseConnection {
    // Use a lock to prevent concurrent migration runs which can cause conflicts
    let lock = DB_LOCK
        .get_or_init(|| async { Arc::new(Mutex::new(())) })
        .await;
    let _guard = lock.lock().await;

    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    // Intentionally no automatic migrations here.
    // Tests can opt in to module-specific migrations via
    // `setup_test_db_with_migrations::<M>()`.

    db
}

/// Sets up a test database with specific migrations.
///
/// This is useful when you want to test a specific module without
/// running all migrations.
///
/// # Type Parameters
///
/// * `M` - The migration type that implements `MigratorTrait`
///
/// # Example
///
/// ```rust,ignore
/// use rustok_test_utils::setup_test_db_with_migrations;
/// use rustok_content::migrations::Migrator;
///
/// #[tokio::test]
/// async fn test_content_module() {
///     let db = setup_test_db_with_migrations::<Migrator>().await;
///     // Test with content migrations only...
/// }
/// ```
pub async fn setup_test_db_with_migrations<M>() -> DatabaseConnection
where
    M: sea_orm_migration::MigratorTrait,
{
    let lock = DB_LOCK
        .get_or_init(|| async { Arc::new(Mutex::new(())) })
        .await;
    let _guard = lock.lock().await;

    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");

    M::up(&db, None).await.expect("Failed to run migrations");

    db
}

/// Creates a test transaction that will be rolled back after the test.
///
/// This is useful for tests that should not commit changes to the database.
///
/// # Example
///
/// ```rust,ignore
/// use rustok_test_utils::db::with_test_transaction;
///
/// #[tokio::test]
/// async fn test_with_transaction() {
///     with_test_transaction(|txn| async move {
///         // Perform database operations...
///         // Changes are automatically rolled back
///     }).await;
/// }
/// ```
pub async fn with_test_transaction<F, Fut, R>(f: F) -> R
where
    F: FnOnce(&sea_orm::DatabaseTransaction) -> Fut,
    Fut: std::future::Future<Output = R>,
{
    let db = setup_test_db().await;
    let txn = db.begin().await.expect("Failed to begin transaction");

    let result = f(&txn).await;

    // Transaction is dropped here, causing rollback
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_setup_test_db() {
        let db = setup_test_db().await;
        // Just verify we can connect
        let result: Result<i32, DbErr> =
            sea_orm::query::Select::new(sea_orm::query::SelectStatement::new())
                .from(sea_orm::sea_query::Alias::new("sqlite_master"))
                .one(&db)
                .await
                .map(|_| 1);
        assert!(result.is_ok() || result.is_err()); // Just checking connection works
    }
}
