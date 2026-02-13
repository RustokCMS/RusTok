/// Distributed Tracing Utilities
///
/// Helper functions and macros for creating standardized spans across RusToK.
///
/// # Features
/// - Standardized span creation with common attributes
/// - Tenant and user correlation
/// - Error span recording
/// - Performance tracking

use tracing::{Span, Level};
use uuid::Uuid;

/// Common span attributes for RusToK operations
pub struct SpanAttributes {
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub operation: String,
    pub module: String,
}

impl SpanAttributes {
    pub fn new(operation: impl Into<String>, module: impl Into<String>) -> Self {
        Self {
            tenant_id: None,
            user_id: None,
            operation: operation.into(),
            module: module.into(),
        }
    }

    pub fn with_tenant(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }
}

/// Create a standardized info span
///
/// # Example
/// ```no_run
/// use rustok_core::tracing::{create_span, SpanAttributes};
/// use uuid::Uuid;
///
/// let tenant_id = Uuid::new_v4();
/// let attrs = SpanAttributes::new("fetch_user", "users")
///     .with_tenant(tenant_id);
///
/// let span = create_span("fetch_user", attrs);
/// let _guard = span.enter();
/// // Your code here
/// ```
pub fn create_span(name: &str, attrs: SpanAttributes) -> Span {
    let span = tracing::info_span!(
        name,
        module = %attrs.module,
        operation = %attrs.operation,
    );

    if let Some(tenant_id) = attrs.tenant_id {
        span.record("tenant_id", &tracing::field::display(tenant_id));
    }

    if let Some(user_id) = attrs.user_id {
        span.record("user_id", &tracing::field::display(user_id));
    }

    span
}

/// Record an error in the current span
///
/// # Example
/// ```no_run
/// use rustok_core::tracing::record_error;
///
/// let result: Result<(), &str> = Err("something failed");
/// if let Err(e) = result {
///     record_error(e, "operation_failed");
/// }
/// ```
pub fn record_error<E: std::fmt::Display>(error: E, error_type: &str) {
    let span = Span::current();
    span.record("error", &tracing::field::display(&error));
    span.record("error.type", error_type);
    span.record("error.occurred", true);
}

/// Record operation success with optional result data
pub fn record_success(message: &str) {
    let span = Span::current();
    span.record("success", true);
    span.record("result", message);
}

/// Create a database query span
///
/// # Example
/// ```no_run
/// use rustok_core::tracing::db_span;
///
/// let span = db_span("SELECT * FROM users WHERE id = $1", "postgres");
/// let _guard = span.enter();
/// // Execute query
/// ```
pub fn db_span(query: &str, db_type: &str) -> Span {
    tracing::debug_span!(
        "db.query",
        db.type = db_type,
        db.operation = query,
        otel.kind = "client",
    )
}

/// Create an HTTP client span
pub fn http_client_span(method: &str, url: &str) -> Span {
    tracing::info_span!(
        "http.client",
        http.method = method,
        http.url = url,
        otel.kind = "client",
    )
}

/// Create an event processing span
pub fn event_span(event_type: &str, event_id: &str) -> Span {
    tracing::info_span!(
        "event.process",
        event.type = event_type,
        event.id = event_id,
        otel.kind = "consumer",
    )
}

/// Macro for creating a traced async function
///
/// # Example
/// ```ignore
/// use rustok_core::traced;
///
/// #[traced(name = "fetch_user", tenant_id, user_id)]
/// async fn fetch_user(tenant_id: Uuid, user_id: Uuid) -> Result<User> {
///     // Function automatically gets a span with tenant_id and user_id
///     Ok(User::default())
/// }
/// ```
#[macro_export]
macro_rules! traced {
    (
        name = $name:expr,
        $($field:ident),* $(,)?
    ) => {
        tracing::instrument(
            name = $name,
            skip_all,
            fields(
                $($field = tracing::field::Empty,)*
            )
        )
    };
}

/// Measure operation duration and record as span attribute
///
/// # Example
/// ```no_run
/// use rustok_core::tracing::measure;
///
/// # async fn example() {
/// let result = measure("slow_operation", || async {
///     // Your slow operation
///     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
///     42
/// }).await;
/// # }
/// ```
pub async fn measure<F, Fut, T>(operation: &str, f: F) -> T
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f().await;
    let duration = start.elapsed();

    let span = Span::current();
    span.record("duration_ms", duration.as_millis() as u64);
    span.record("operation", operation);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_attributes() {
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let attrs = SpanAttributes::new("test_op", "test_module")
            .with_tenant(tenant_id)
            .with_user(user_id);

        assert_eq!(attrs.operation, "test_op");
        assert_eq!(attrs.module, "test_module");
        assert_eq!(attrs.tenant_id, Some(tenant_id));
        assert_eq!(attrs.user_id, Some(user_id));
    }

    #[test]
    fn test_create_span() {
        let attrs = SpanAttributes::new("test", "module");
        let span = create_span("test_span", attrs);
        assert_eq!(span.metadata().unwrap().name(), "test_span");
    }

    #[tokio::test]
    async fn test_measure() {
        let result = measure("test", || async {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            42
        }).await;

        assert_eq!(result, 42);
    }
}
