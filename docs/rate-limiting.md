# Rate Limiting in RusToK

## Overview

RusToK implements a sliding window rate limiter to protect API endpoints from abuse and DoS attacks. The rate limiter tracks requests per client (by IP or user ID) and enforces configurable limits.

## Features

- ✅ Sliding window algorithm for accurate rate limiting
- ✅ Per-user and per-IP tracking
- ✅ Configurable limits and time windows
- ✅ Standard HTTP headers (`X-RateLimit-*`)
- ✅ Automatic cleanup of expired entries
- ✅ Can be disabled for development/testing
- ✅ Zero-allocation for non-limited requests

## Configuration

### Default Configuration

```rust
use rustok_server::middleware::rate_limit::RateLimitConfig;

// Default: 100 requests per 60 seconds
let config = RateLimitConfig::default();
```

### Custom Configuration

```rust
// 1000 requests per 5 minutes
let config = RateLimitConfig::new(1000, 300);

// Disable rate limiting (e.g., for development)
let config = RateLimitConfig::disabled();
```

## Integration

### Axum Middleware

Add the rate limiter to your Axum application:

```rust
use axum::Router;
use rustok_server::middleware::rate_limit::{RateLimiter, RateLimitConfig, rate_limit_middleware};
use std::sync::Arc;

async fn setup_app() -> Router {
    // Create rate limiter
    let config = RateLimitConfig::new(100, 60); // 100 req/min
    let rate_limiter = Arc::new(RateLimiter::new(config));

    // Start cleanup task
    let limiter_clone = rate_limiter.clone();
    tokio::spawn(async move {
        rustok_server::middleware::rate_limit::cleanup_task(limiter_clone).await;
    });

    // Add to router
    Router::new()
        .route("/api/nodes", post(create_node))
        .route("/api/nodes/:id", get(get_node))
        .layer(axum::middleware::from_fn_with_state(
            rate_limiter.clone(),
            rate_limit_middleware,
        ))
}
```

## Client Identification

The rate limiter identifies clients in the following priority order:

1. **User ID** from `X-User-ID` header (set by auth middleware)
2. **X-Forwarded-For** header (behind proxy)
3. **X-Real-IP** header (behind nginx)
4. **Fallback** to `"ip:unknown"`

### Example Headers

```http
# Authenticated user
X-User-ID: 550e8400-e29b-41d4-a716-446655440000

# Behind proxy
X-Forwarded-For: 203.0.113.42, 198.51.100.17

# Behind nginx
X-Real-IP: 203.0.113.42
```

## Response Headers

The middleware adds standard rate limit headers to all responses:

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 73
X-RateLimit-Reset: 1704063600

...
```

### When Rate Limited

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 42
Content-Type: text/plain

Rate limit exceeded
```

## Per-Endpoint Configuration

You can apply different rate limits to different route groups:

```rust
use tower::ServiceBuilder;

// Strict limits for expensive operations
let strict_limiter = Arc::new(RateLimiter::new(
    RateLimitConfig::new(10, 60) // 10 req/min
));

// Relaxed limits for read operations
let relaxed_limiter = Arc::new(RateLimiter::new(
    RateLimitConfig::new(1000, 60) // 1000 req/min
));

Router::new()
    // Expensive mutations
    .route("/api/nodes", post(create_node))
    .route("/api/nodes/:id", put(update_node))
    .layer(axum::middleware::from_fn_with_state(
        strict_limiter,
        rate_limit_middleware,
    ))
    
    // Fast reads
    .route("/api/nodes", get(list_nodes))
    .route("/api/nodes/:id", get(get_node))
    .layer(axum::middleware::from_fn_with_state(
        relaxed_limiter,
        rate_limit_middleware,
    ))
```

## Monitoring

### Metrics

Add Prometheus metrics to track rate limiting:

```rust
use prometheus::{register_counter_vec, CounterVec};
use once_cell::sync::Lazy;

static RATE_LIMIT_HITS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "rustok_rate_limit_hits_total",
        "Total rate limit rejections",
        &["client_type"]
    ).unwrap()
});

// In middleware, when rate limited:
RATE_LIMIT_HITS
    .with_label_values(&[if key.starts_with("user:") { "user" } else { "ip" }])
    .inc();
```

### Logs

The middleware logs rate limit events:

```rust
tracing::warn!(
    key = %key,
    count = counter.count,
    limit = self.config.max_requests,
    retry_after = retry_after,
    "Rate limit exceeded"
);
```

## Testing

### Unit Tests

```rust
use rustok_server::middleware::rate_limit::{RateLimiter, RateLimitConfig};

#[tokio::test]
async fn test_rate_limiting() {
    let config = RateLimitConfig::new(3, 60);
    let limiter = RateLimiter::new(config);

    // First 3 requests succeed
    assert!(limiter.check_rate_limit("test-client").await.is_ok());
    assert!(limiter.check_rate_limit("test-client").await.is_ok());
    assert!(limiter.check_rate_limit("test-client").await.is_ok());

    // 4th request is rate limited
    assert!(limiter.check_rate_limit("test-client").await.is_err());
}
```

### Integration Tests

```rust
use axum::http::StatusCode;
use axum_test::TestServer;

#[tokio::test]
async fn test_api_rate_limiting() {
    let app = setup_app_with_rate_limit().await;
    let server = TestServer::new(app).unwrap();

    // Make requests until rate limited
    for i in 1..=100 {
        let resp = server.get("/api/nodes").await;
        assert_eq!(resp.status_code(), StatusCode::OK);
    }

    // 101st request should be rate limited
    let resp = server.get("/api/nodes").await;
    assert_eq!(resp.status_code(), StatusCode::TOO_MANY_REQUESTS);
}
```

## Best Practices

### 1. Set Appropriate Limits

Consider your API usage patterns:

- **Read operations**: High limits (1000+ req/min)
- **Write operations**: Moderate limits (100 req/min)
- **Expensive operations**: Low limits (10 req/min)

### 2. Use Different Limits for Authenticated vs Anonymous

```rust
// Higher limits for authenticated users
let user_config = RateLimitConfig::new(1000, 60);

// Lower limits for anonymous/IP-based
let anon_config = RateLimitConfig::new(100, 60);
```

### 3. Monitor Rate Limit Hits

Set up alerts when rate limiting is frequently triggered:

```yaml
# Prometheus alert
- alert: HighRateLimitRejections
  expr: rate(rustok_rate_limit_hits_total[5m]) > 10
  annotations:
    summary: "High rate of rate limit rejections"
```

### 4. Document Limits in API Docs

Include rate limit information in your OpenAPI spec:

```yaml
paths:
  /api/nodes:
    post:
      summary: Create a node
      x-rate-limit:
        limit: 100
        window: 60
```

### 5. Cleanup Periodically

The built-in cleanup task runs every 5 minutes. Adjust if needed:

```rust
// Custom cleanup interval (every 10 minutes)
let mut interval = tokio::time::interval(Duration::from_secs(600));
loop {
    interval.tick().await;
    limiter.cleanup_expired().await;
}
```

## Performance

### Memory Usage

- Each client entry: ~100 bytes
- 10,000 active clients: ~1 MB
- Old entries cleaned up every 5 minutes

### CPU Usage

- Per request: 1 hash map lookup + 1 timestamp comparison
- Negligible overhead (<1μs per request)

### Concurrency

- Uses `RwLock` for thread-safe access
- Read operations don't block each other
- Write operations (incrementing counter) are fast

## Troubleshooting

### "Rate limit exceeded" but I'm making few requests

**Cause**: Multiple clients sharing the same IP (NAT, proxy)

**Solution**: Ensure `X-User-ID` header is set for authenticated users

### Rate limiting not working

**Check**:
1. Is rate limiting enabled? (`config.enabled == true`)
2. Is middleware added to the correct routes?
3. Are client IDs being extracted correctly?

### Memory growing over time

**Cause**: Cleanup task not running

**Solution**: Ensure cleanup task is spawned:

```rust
tokio::spawn(async move {
    cleanup_task(limiter).await;
});
```

## Migration from Other Systems

### From nginx `limit_req`

```nginx
# nginx config
limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
limit_req zone=api burst=20 nodelay;
```

Equivalent RusToK config:

```rust
RateLimitConfig::new(10 * 60, 60) // 10 req/s = 600 req/min
```

### From API Gateway

Most API gateways support standard `X-RateLimit-*` headers, so clients can adapt seamlessly.

## Future Enhancements

- [ ] Distributed rate limiting (Redis-based)
- [ ] Token bucket algorithm option
- [ ] Per-endpoint metrics
- [ ] Dynamic limit adjustment based on load
- [ ] Rate limit quotas (daily/monthly limits)

---

**See Also:**
- [Security Best Practices](./security.md)
- [API Documentation](./api.md)
- [Monitoring Guide](./monitoring.md)
