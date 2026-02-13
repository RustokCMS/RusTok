# Error Handling Policy

> **Date:** 2026-02-12  
> **Sprint:** Sprint 2 - Simplification  
> **Status:** Draft → Review → **Active**

---

## Overview

This document defines the error handling standards and patterns for the RusToK platform. Consistent error handling improves reliability, debuggability, and user experience.

### Goals

✅ **Consistency** - Predictable error types across the codebase  
✅ **Debuggability** - Rich context for troubleshooting  
✅ **User Experience** - Clear, actionable error messages  
✅ **Observability** - Proper logging and metrics  
✅ **Security** - No information leakage in errors  

---

## Error Architecture

### Core Error Type

**Location:** `crates/rustok-core/src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(String),
    
    #[error("cache error: {0}")]
    Cache(String),
    
    #[error("validation error: {0}")]
    Validation(String),
    
    #[error("authentication error: {0}")]
    Authentication(String),
    
    #[error("authorization error: {0}")]
    Authorization(String),
    
    #[error("not found: {0}")]
    NotFound(String),
    
    #[error("conflict: {0}")]
    Conflict(String),
    
    #[error("external service error: {0}")]
    ExternalService(String),
    
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### Error Categories

| Category | HTTP Status | When to Use | Log Level |
|----------|-------------|-------------|-----------|
| `Validation` | 400 | Invalid input data | `INFO` |
| `Authentication` | 401 | Auth required or failed | `WARN` |
| `Authorization` | 403 | User lacks permission | `WARN` |
| `NotFound` | 404 | Resource doesn't exist | `INFO` |
| `Conflict` | 409 | Resource already exists | `INFO` |
| `Database` | 500 | Database operation failed | `ERROR` |
| `Cache` | 500 | Cache operation failed | `WARN`* |
| `ExternalService` | 502 | External API failed | `ERROR` |
| `Internal` | 500 | Unexpected internal error | `ERROR` |

*Cache errors are `WARN` because they're often non-critical (fallback to DB)

---

## Error Handling Patterns

### 1. Input Validation

**Pattern:** Validate early, fail fast

```rust
pub async fn create_product(
    ctx: &AppContext,
    tenant_id: Uuid,
    dto: CreateProductDto,
) -> Result<Product> {
    // ❌ BAD - validate inside transaction
    // let product = Product::new(dto);
    // db.insert(product).await?;
    
    // ✅ GOOD - validate before any side effects
    dto.validate()
        .map_err(|e| Error::Validation(format!("Invalid product data: {}", e)))?;
    
    let product = Product::new(dto);
    db.insert(product).await?;
    
    Ok(product)
}
```

**Rules:**
- Validate input **before** any side effects
- Use specific validation error messages
- Return `Error::Validation` for bad input
- Don't expose internal validation details to users

### 2. Database Errors

**Pattern:** Catch and categorize

```rust
pub async fn get_product(db: &DatabaseConnection, id: Uuid) -> Result<Product> {
    let product = products::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|e| {
            tracing::error!("Database error fetching product {}: {}", id, e);
            Error::Database("Failed to fetch product".to_string())
        })?
        .ok_or_else(|| Error::NotFound(format!("Product {} not found", id)))?;
    
    Ok(product)
}
```

**Rules:**
- Log database errors with context (`tracing::error!`)
- Map `DbErr` to appropriate `Error` type
- Use `NotFound` for missing resources
- Use `Conflict` for uniqueness violations
- Use `Database` for other DB errors
- **Never** expose SQL errors to users

### 3. External Service Errors

**Pattern:** Wrap and add context

```rust
pub async fn fetch_exchange_rate(currency: &str) -> Result<f64> {
    let response = reqwest::get(format!("https://api.example.com/rates/{}", currency))
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch exchange rate for {}: {}", currency, e);
            Error::ExternalService(format!("Currency service unavailable"))
        })?;
    
    if !response.status().is_success() {
        tracing::warn!("Exchange rate API returned {}: {}", response.status(), currency);
        return Err(Error::ExternalService("Currency service error".to_string()));
    }
    
    let rate = response.json::<ExchangeRateResponse>()
        .await
        .map_err(|e| {
            tracing::error!("Failed to parse exchange rate response: {}", e);
            Error::ExternalService("Invalid response from currency service".to_string())
        })?;
    
    Ok(rate.value)
}
```

**Rules:**
- Use `Error::ExternalService` for external API failures
- Log the detailed error internally
- Return generic error message to user
- Include service name in error message
- Consider circuit breaker for reliability

### 4. Cache Errors (Non-Critical)

**Pattern:** Fallback and warn

```rust
pub async fn get_user_cached(
    cache: &impl CacheBackend,
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<User> {
    let cache_key = format!("user:{}", id);
    
    // Try cache first
    match cache.get(&cache_key).await {
        Ok(Some(bytes)) => {
            // Cache hit
            return serde_json::from_slice(&bytes)
                .map_err(|e| Error::Internal(format!("Corrupt cache data: {}", e)));
        }
        Ok(None) => {
            // Cache miss - proceed to database
        }
        Err(e) => {
            // Cache error - log and fallback to database
            tracing::warn!("Cache error for {}: {}", cache_key, e);
            // Don't fail the request, just skip cache
        }
    }
    
    // Fetch from database
    let user = get_user_from_db(db, id).await?;
    
    // Try to cache (best effort)
    if let Err(e) = cache.set(cache_key, serde_json::to_vec(&user)?).await {
        tracing::warn!("Failed to cache user {}: {}", id, e);
        // Don't fail the request
    }
    
    Ok(user)
}
```

**Rules:**
- Cache errors should **not** fail requests
- Log cache errors at `WARN` level
- Always have a fallback (usually database)
- Consider circuit breaker to avoid repeated failures

### 5. Authorization Errors

**Pattern:** Check permissions explicitly

```rust
pub async fn delete_product(
    ctx: &AppContext,
    tenant_id: Uuid,
    product_id: Uuid,
    user: &User,
) -> Result<()> {
    // Check permission
    if !user.has_permission("products:delete") {
        tracing::warn!(
            "User {} attempted to delete product {} without permission",
            user.id,
            product_id
        );
        return Err(Error::Authorization(
            "You don't have permission to delete products".to_string()
        ));
    }
    
    // Verify tenant ownership
    let product = get_product(&ctx.db, product_id).await?;
    if product.tenant_id != tenant_id {
        tracing::warn!(
            "User {} attempted to delete product {} from different tenant",
            user.id,
            product_id
        );
        return Err(Error::NotFound(format!("Product {} not found", product_id)));
    }
    
    // Perform deletion
    delete_product_from_db(&ctx.db, product_id).await?;
    
    Ok(())
}
```

**Rules:**
- Use `Error::Authorization` for permission failures
- Log authorization failures at `WARN` level
- Include user ID and attempted action in logs
- **Return `NotFound` instead of `Authorization`** for cross-tenant access (security)
- Don't expose tenant isolation in error messages

### 6. Internal Errors (Unexpected)

**Pattern:** Log and return generic error

```rust
pub async fn complex_operation(ctx: &AppContext) -> Result<()> {
    let data = prepare_data().await?;
    
    // Unexpected error that "should never happen"
    if data.is_empty() {
        tracing::error!("UNEXPECTED: prepare_data() returned empty, this should be impossible");
        return Err(Error::Internal(
            "An unexpected error occurred. Please try again or contact support.".to_string()
        ));
    }
    
    // ... rest of logic
    Ok(())
}
```

**Rules:**
- Use `Error::Internal` for unexpected errors
- Log with `tracing::error!` and mark as "UNEXPECTED"
- Return **generic** error message to users
- Include request ID in logs for debugging
- Consider alerting on internal errors

---

## Conversion Guidelines

### From thiserror

```rust
// Define custom error types with thiserror
#[derive(Debug, thiserror::Error)]
pub enum ProductError {
    #[error("invalid price: {0}")]
    InvalidPrice(String),
    
    #[error("out of stock")]
    OutOfStock,
}

// Convert to core Error
impl From<ProductError> for Error {
    fn from(err: ProductError) -> Self {
        match err {
            ProductError::InvalidPrice(msg) => Error::Validation(format!("Product: {}", msg)),
            ProductError::OutOfStock => Error::Conflict("Product is out of stock".to_string()),
        }
    }
}
```

### From anyhow

```rust
// ❌ AVOID - anyhow loses error type information
pub async fn do_something() -> anyhow::Result<()> { ... }

// ✅ PREFER - typed errors
pub async fn do_something() -> Result<()> { ... }

// If you must convert from anyhow:
anyhow_result
    .map_err(|e| {
        tracing::error!("Operation failed: {:#}", e);
        Error::Internal("Operation failed".to_string())
    })?
```

### From SeaORM

```rust
use sea_orm::DbErr;

// Convert database errors
impl From<DbErr> for Error {
    fn from(err: DbErr) -> Self {
        match err {
            DbErr::RecordNotFound(_) => Error::NotFound("Resource not found".to_string()),
            DbErr::Exec(msg) if msg.contains("unique constraint") => {
                Error::Conflict("Resource already exists".to_string())
            }
            _ => {
                tracing::error!("Database error: {}", err);
                Error::Database("Database operation failed".to_string())
            }
        }
    }
}
```

---

## API Error Responses

### REST API

**Format:**

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid product data: price must be positive",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

**Error Codes:**

| HTTP Status | Error Code | Error Type |
|-------------|------------|------------|
| 400 | `VALIDATION_ERROR` | `Error::Validation` |
| 401 | `AUTHENTICATION_REQUIRED` | `Error::Authentication` |
| 403 | `FORBIDDEN` | `Error::Authorization` |
| 404 | `NOT_FOUND` | `Error::NotFound` |
| 409 | `CONFLICT` | `Error::Conflict` |
| 500 | `INTERNAL_ERROR` | `Error::Internal` |
| 502 | `SERVICE_UNAVAILABLE` | `Error::ExternalService` |
| 500 | `DATABASE_ERROR` | `Error::Database` |

**Implementation:**

```rust
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Error::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg),
            Error::Authentication(msg) => (StatusCode::UNAUTHORIZED, "AUTHENTICATION_REQUIRED", msg),
            Error::Authorization(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg),
            Error::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg),
            Error::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg),
            Error::ExternalService(msg) => (StatusCode::BAD_GATEWAY, "SERVICE_UNAVAILABLE", msg),
            Error::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "A database error occurred".to_string(),
            ),
            Error::Cache(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
            Error::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
            ),
        };
        
        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });
        
        (status, Json(body)).into_response()
    }
}
```

### GraphQL API

**Format:**

```json
{
  "errors": [
    {
      "message": "Invalid product data: price must be positive",
      "extensions": {
        "code": "VALIDATION_ERROR"
      }
    }
  ]
}
```

**Implementation:**

```rust
use async_graphql::{Error as GraphQLError, ErrorExtensions};

impl From<Error> for GraphQLError {
    fn from(err: Error) -> Self {
        let code = match &err {
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Authentication(_) => "AUTHENTICATION_REQUIRED",
            Error::Authorization(_) => "FORBIDDEN",
            Error::NotFound(_) => "NOT_FOUND",
            Error::Conflict(_) => "CONFLICT",
            _ => "INTERNAL_ERROR",
        };
        
        GraphQLError::new(err.to_string())
            .extend_with(|_, e| e.set("code", code))
    }
}
```

---

## Logging Best Practices

### Log Levels

```rust
// TRACE - Very detailed, high volume
tracing::trace!("Cache lookup for key: {}", key);

// DEBUG - Diagnostic information
tracing::debug!("Processing {} items", items.len());

// INFO - Normal operations
tracing::info!("User {} logged in", user_id);

// WARN - Recoverable errors, degraded state
tracing::warn!("Cache unavailable, using database fallback");

// ERROR - Errors requiring attention
tracing::error!("Database query failed: {}", error);
```

### Structured Logging

```rust
use tracing::{info, error, instrument};

#[instrument(skip(ctx, dto))]
pub async fn create_product(
    ctx: &AppContext,
    tenant_id: Uuid,
    dto: CreateProductDto,
) -> Result<Product> {
    info!(
        tenant_id = %tenant_id,
        sku = %dto.sku,
        "Creating product"
    );
    
    // ... operation
    
    if let Err(e) = result {
        error!(
            tenant_id = %tenant_id,
            sku = %dto.sku,
            error = %e,
            "Failed to create product"
        );
        return Err(e);
    }
    
    Ok(product)
}
```

### Don't Log Sensitive Data

```rust
// ❌ BAD - logs password
tracing::info!("User login: {}, password: {}", email, password);

// ✅ GOOD - no sensitive data
tracing::info!("User login attempt: {}", email);

// ❌ BAD - logs credit card
tracing::debug!("Payment: card={}", card_number);

// ✅ GOOD - logs last 4 digits only
tracing::debug!("Payment: card=****{}", &card_number[card_number.len()-4..]);
```

---

## Testing Error Handling

### Unit Tests

```rust
#[tokio::test]
async fn test_create_product_validation_error() {
    let dto = CreateProductDto {
        name: "".to_string(), // Invalid: empty name
        price: -10.0,          // Invalid: negative price
    };
    
    let result = create_product(&ctx, tenant_id, dto).await;
    
    assert!(matches!(result, Err(Error::Validation(_))));
}

#[tokio::test]
async fn test_get_product_not_found() {
    let result = get_product(&db, Uuid::new_v4()).await;
    
    assert!(matches!(result, Err(Error::NotFound(_))));
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_api_returns_correct_error_status() {
    let app = setup_test_app().await;
    
    // Test 404
    let response = app.get("/api/v1/products/non-existent").await;
    assert_eq!(response.status(), 404);
    
    let body: ErrorResponse = response.json().await;
    assert_eq!(body.error.code, "NOT_FOUND");
}
```

---

## Metrics

Track error rates for monitoring:

```rust
use metrics::counter;

// Increment error counters
match result {
    Err(Error::Database(_)) => counter!("errors.database", 1),
    Err(Error::ExternalService(_)) => counter!("errors.external_service", 1),
    Err(Error::Validation(_)) => counter!("errors.validation", 1),
    _ => {}
}
```

---

## Migration Checklist

When updating existing code to follow this policy:

- [ ] Replace `anyhow::Result` with `rustok_core::Result`
- [ ] Add proper error categorization
- [ ] Add structured logging with context
- [ ] Remove sensitive data from logs
- [ ] Add error tests
- [ ] Update API documentation
- [ ] Add metrics counters
- [ ] Verify error messages are user-friendly

---

## Examples

### Complete Example

```rust
use rustok_core::{Error, Result};
use tracing::{info, warn, error, instrument};
use uuid::Uuid;

#[instrument(skip(ctx, dto))]
pub async fn create_order(
    ctx: &AppContext,
    tenant_id: Uuid,
    user_id: Uuid,
    dto: CreateOrderDto,
) -> Result<Order> {
    // 1. Validate input
    dto.validate()
        .map_err(|e| Error::Validation(format!("Invalid order: {}", e)))?;
    
    info!(
        tenant_id = %tenant_id,
        user_id = %user_id,
        items = dto.items.len(),
        "Creating order"
    );
    
    // 2. Check authorization
    let user = get_user(&ctx.db, user_id).await?;
    if !user.has_permission("orders:create") {
        warn!("User {} lacks permission to create orders", user_id);
        return Err(Error::Authorization(
            "You don't have permission to create orders".to_string()
        ));
    }
    
    // 3. Validate business rules
    let total = dto.calculate_total();
    if total <= 0.0 {
        return Err(Error::Validation("Order total must be positive".to_string()));
    }
    
    // 4. External service call with error handling
    let shipping_quote = match get_shipping_quote(&dto.address).await {
        Ok(quote) => quote,
        Err(e) => {
            error!("Failed to get shipping quote: {}", e);
            return Err(Error::ExternalService(
                "Shipping service unavailable. Please try again.".to_string()
            ));
        }
    };
    
    // 5. Database transaction
    let order = Order {
        id: Uuid::new_v4(),
        tenant_id,
        user_id,
        total,
        shipping_cost: shipping_quote.cost,
        status: OrderStatus::Pending,
    };
    
    let result = orders::Entity::insert(order.clone().into())
        .exec(&ctx.db)
        .await
        .map_err(|e| {
            error!("Failed to insert order: {}", e);
            Error::Database("Failed to create order".to_string())
        })?;
    
    // 6. Best-effort cache
    if let Err(e) = ctx.cache.set(
        format!("order:{}", order.id),
        serde_json::to_vec(&order)?
    ).await {
        warn!("Failed to cache order {}: {}", order.id, e);
        // Don't fail the request
    }
    
    info!(
        tenant_id = %tenant_id,
        user_id = %user_id,
        order_id = %order.id,
        "Order created successfully"
    );
    
    Ok(order)
}
```

---

## Related Documentation

- [REFACTORING_ROADMAP.md](./REFACTORING_ROADMAP.md) - Sprint 2, Task 2.4
- [CIRCUIT_BREAKER_GUIDE.md](./CIRCUIT_BREAKER_GUIDE.md) - External service error handling
- [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) - Progress tracking

---

**Version:** 1.0  
**Last Updated:** 2026-02-12  
**Status:** Active  
**Sprint:** Sprint 2 - Simplification
