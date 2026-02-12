# Circuit Breaker Pattern - Implementation Guide

> **Date:** 2026-02-12  
> **Sprint:** Sprint 2 - Simplification  
> **Component:** `rustok-core::circuit_breaker`

---

## Overview

The Circuit Breaker pattern protects services from cascading failures by temporarily blocking requests when a failure threshold is reached. This implementation is generic and can be used to protect any async operation.

### Use Cases

1. **Redis connections** - Prevent connection storms when Redis is down
2. **External API calls** - Protect against slow/failing third-party services
3. **Database queries** - Prevent overwhelming a struggling database
4. **Microservice calls** - Stop calling a failing downstream service

---

## Architecture

### States

The circuit breaker has three states:

```
┌─────────┐
│ CLOSED  │ ◄──────────────────────┐
└────┬────┘                        │
     │                             │
     │ Failures ≥ threshold        │ Successes ≥ threshold
     │                             │
     ▼                             │
┌─────────┐                   ┌────┴─────┐
│  OPEN   │ ───────────────► │ HALF_OPEN│
└─────────┘  After timeout    └──────────┘
     │                             │
     │                             │
     └──── Reject requests         └──── Limited requests
```

**CLOSED (Normal operation)**
- Requests flow through normally
- Failures are counted
- If failures ≥ threshold → OPEN

**OPEN (Circuit tripped)**
- Requests are immediately rejected
- No calls to upstream service
- After timeout → HALF_OPEN

**HALF_OPEN (Testing recovery)**
- Limited requests allowed (configurable)
- Testing if service recovered
- Success → increment counter
  - If successes ≥ threshold → CLOSED
- Failure → OPEN (back to square one)

---

## Configuration

```rust
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening (default: 5)
    pub failure_threshold: u32,
    
    /// Number of consecutive successes in half-open to close (default: 2)
    pub success_threshold: u32,
    
    /// Time to wait before attempting to close (default: 60s)
    pub timeout: Duration,
    
    /// Max concurrent requests in half-open state (default: 3)
    pub half_open_max_requests: u32,
}
```

### Tuning Guidelines

**Conservative (production default):**
```rust
CircuitBreakerConfig {
    failure_threshold: 5,      // Allow some transient errors
    success_threshold: 2,      // Need 2 successes to close
    timeout: Duration::from_secs(60),
    half_open_max_requests: 3,
}
```

**Aggressive (fail-fast):**
```rust
CircuitBreakerConfig {
    failure_threshold: 3,      // Trip quickly
    success_threshold: 3,      // Need more proof to close
    timeout: Duration::from_secs(30),
    half_open_max_requests: 1, // Minimal testing
}
```

**Tolerant (for flaky services):**
```rust
CircuitBreakerConfig {
    failure_threshold: 10,     // Allow more errors
    success_threshold: 1,      // Quick recovery
    timeout: Duration::from_secs(120),
    half_open_max_requests: 5,
}
```

---

## Usage Examples

### Basic Example

```rust
use rustok_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let config = CircuitBreakerConfig::default();
    let breaker = CircuitBreaker::new(config);
    
    // Wrap any async operation
    let result = breaker.call(async {
        // Your potentially failing operation
        make_api_call().await
    }).await;
    
    match result {
        Ok(value) => println!("Success: {:?}", value),
        Err(CircuitBreakerError::Open) => {
            println!("Circuit breaker is open, service unavailable");
        }
        Err(CircuitBreakerError::Upstream(e)) => {
            println!("Upstream error: {}", e);
        }
    }
}
```

### Redis with Circuit Breaker

```rust
use rustok_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;

pub struct ProtectedRedisCache {
    client: redis::Client,
    breaker: Arc<CircuitBreaker>,
}

impl ProtectedRedisCache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            success_threshold: 2,
            timeout: Duration::from_secs(30),
            half_open_max_requests: 2,
        };
        
        Ok(Self {
            client,
            breaker: Arc::new(CircuitBreaker::new(config)),
        })
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let client = self.client.clone();
        let key = key.to_string();
        
        self.breaker.call(async move {
            let mut conn = client.get_multiplexed_async_connection().await?;
            let value: Option<String> = conn.get(&key).await?;
            Ok(value)
        }).await
        .map_err(|e| match e {
            CircuitBreakerError::Open => CacheError::CircuitOpen,
            CircuitBreakerError::Upstream(e) => CacheError::Redis(e),
        })
    }
    
    pub async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let client = self.client.clone();
        let key = key.to_string();
        let value = value.to_string();
        let ttl_secs = ttl.as_secs();
        
        self.breaker.call(async move {
            let mut conn = client.get_multiplexed_async_connection().await?;
            conn.set_ex(&key, &value, ttl_secs).await?;
            Ok(())
        }).await
        .map_err(|e| match e {
            CircuitBreakerError::Open => CacheError::CircuitOpen,
            CircuitBreakerError::Upstream(e) => CacheError::Redis(e),
        })
    }
}

#[derive(Debug)]
pub enum CacheError {
    Redis(redis::RedisError),
    CircuitOpen,
}
```

### HTTP Client with Circuit Breaker

```rust
use rustok_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use std::sync::Arc;

pub struct ProtectedHttpClient {
    client: reqwest::Client,
    breaker: Arc<CircuitBreaker>,
}

impl ProtectedHttpClient {
    pub fn new() -> Self {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_requests: 3,
        };
        
        Self {
            client: reqwest::Client::new(),
            breaker: Arc::new(CircuitBreaker::new(config)),
        }
    }
    
    pub async fn get(&self, url: &str) -> Result<String, HttpError> {
        let client = self.client.clone();
        let url = url.to_string();
        
        self.breaker.call(async move {
            let response = client.get(&url)
                .timeout(Duration::from_secs(5))
                .send()
                .await?;
            
            let text = response.text().await?;
            Ok(text)
        }).await
        .map_err(|e| match e {
            CircuitBreakerError::Open => HttpError::ServiceUnavailable,
            CircuitBreakerError::Upstream(e) => HttpError::Request(e),
        })
    }
}
```

### Database with Circuit Breaker

```rust
use rustok_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::sync::Arc;

pub struct ProtectedRepository<E: EntityTrait> {
    db: DatabaseConnection,
    breaker: Arc<CircuitBreaker>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: EntityTrait> ProtectedRepository<E> {
    pub fn new(db: DatabaseConnection) -> Self {
        let config = CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(30),
            half_open_max_requests: 2,
        };
        
        Self {
            db,
            breaker: Arc::new(CircuitBreaker::new(config)),
            _phantom: std::marker::PhantomData,
        }
    }
    
    pub async fn find_all(&self) -> Result<Vec<E::Model>, DbError> {
        let db = self.db.clone();
        
        self.breaker.call(async move {
            E::find().all(&db).await
        }).await
        .map_err(|e| match e {
            CircuitBreakerError::Open => DbError::CircuitOpen,
            CircuitBreakerError::Upstream(e) => DbError::Database(e),
        })
    }
}
```

---

## Monitoring

### Checking State

```rust
let state = breaker.get_state();
match state {
    State::Closed => println!("✅ Circuit healthy"),
    State::Open => println!("❌ Circuit open - service down"),
    State::HalfOpen => println!("⚠️ Circuit testing recovery"),
}
```

### Metrics

```rust
let failures = breaker.failure_count();
let successes = breaker.success_count();

// Export to Prometheus
gauge!("circuit_breaker.failure_count", failures as f64);
gauge!("circuit_breaker.success_count", successes as f64);
gauge!("circuit_breaker.state", state as u32 as f64);
```

### Manual Reset

```rust
// Force reset (use carefully)
breaker.reset();
tracing::info!("Circuit breaker manually reset");
```

---

## Testing

### Unit Test Example

```rust
#[tokio::test]
async fn test_circuit_opens_after_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        ..Default::default()
    };
    let breaker = CircuitBreaker::new(config);
    
    // Fail 3 times
    for _ in 0..3 {
        let _ = breaker.call(async { 
            Err::<(), _>("error") 
        }).await;
    }
    
    // Circuit should be open
    assert_eq!(breaker.get_state(), State::Open);
    
    // Next call should be rejected
    let result = breaker.call(async { 
        Ok::<(), String>(()) 
    }).await;
    
    assert!(matches!(result, Err(CircuitBreakerError::Open)));
}
```

### Integration Test Example

```rust
#[tokio::test]
async fn test_redis_circuit_breaker() {
    // Start with invalid Redis URL (will fail)
    let cache = ProtectedRedisCache::new("redis://invalid:6379").unwrap();
    
    // Try to get - should fail and count towards threshold
    for _ in 0..5 {
        let _ = cache.get("test").await;
    }
    
    // Circuit should be open now
    let result = cache.get("test").await;
    assert!(matches!(result, Err(CacheError::CircuitOpen)));
}
```

---

## Best Practices

### 1. Use Shared Circuit Breakers

```rust
// ❌ Bad - each instance has its own circuit
fn bad_example() {
    let breaker1 = CircuitBreaker::new(config);
    let breaker2 = CircuitBreaker::new(config); // Independent!
}

// ✅ Good - shared circuit breaker
fn good_example() {
    let breaker = Arc::new(CircuitBreaker::new(config));
    let breaker_clone = breaker.clone(); // Shares state
}
```

### 2. Tune for Your Service

```rust
// Fast-changing data (can tolerate brief unavailability)
let aggressive = CircuitBreakerConfig {
    failure_threshold: 3,
    timeout: Duration::from_secs(15),
    ..Default::default()
};

// Critical data (be more patient)
let conservative = CircuitBreakerConfig {
    failure_threshold: 10,
    timeout: Duration::from_secs(120),
    ..Default::default()
};
```

### 3. Log State Changes

Circuit breaker automatically logs state transitions:
- `WARN` when circuit opens
- `INFO` when transitioning to half-open
- `INFO` when closing after recovery

Add custom logging:
```rust
match breaker.call(operation).await {
    Err(CircuitBreakerError::Open) => {
        tracing::error!(
            service = "redis",
            "Circuit breaker open, using fallback"
        );
        // Return cached value or default
    }
    result => result,
}
```

### 4. Implement Fallbacks

```rust
async fn get_user_with_fallback(
    id: Uuid,
    cache: &ProtectedRedisCache,
    db: &DatabaseConnection,
) -> Result<User, Error> {
    // Try cache first
    match cache.get(&id.to_string()).await {
        Ok(Some(json)) => {
            // Cache hit
            Ok(serde_json::from_str(&json)?)
        }
        Err(CacheError::CircuitOpen) => {
            // Circuit open, go directly to database
            tracing::warn!("Cache circuit open, querying database");
            get_user_from_db(id, db).await
        }
        _ => {
            // Cache miss, query database
            get_user_from_db(id, db).await
        }
    }
}
```

### 5. Monitor and Alert

```rust
// Check circuit state periodically
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        
        let state = breaker.get_state();
        if state == State::Open {
            // Send alert
            alert::send(Alert {
                severity: Severity::High,
                message: "Redis circuit breaker is OPEN",
                service: "redis-cache",
            }).await;
        }
    }
});
```

---

## Performance Impact

Circuit breaker adds minimal overhead:

| Operation | Overhead | Notes |
|-----------|----------|-------|
| Successful call (closed) | ~1-2μs | Atomic operations only |
| Rejected call (open) | ~0.5μs | Immediate rejection |
| Half-open state check | ~1-3μs | Includes mutex lock |

**Memory:** ~200 bytes per circuit breaker instance.

---

## Comparison with Alternatives

### vs. Retry Logic

| Feature | Circuit Breaker | Retry Logic |
|---------|----------------|-------------|
| Prevents cascading failures | ✅ Yes | ❌ No |
| Reduces load on failing service | ✅ Yes | ❌ Makes it worse |
| Fast-fail | ✅ Yes | ❌ Slow (retries) |
| Automatic recovery detection | ✅ Yes | ⚠️ Manual |

**Use both:** Circuit breaker outside, limited retries inside.

### vs. Timeout

| Feature | Circuit Breaker | Timeout |
|---------|----------------|---------|
| Protects from complete failures | ✅ Yes | ❌ No |
| Protects from slow responses | ⚠️ Indirect | ✅ Yes |
| Memory of past failures | ✅ Yes | ❌ No |

**Use both:** Timeouts for individual calls, circuit breaker for service health.

---

## Troubleshooting

### Circuit Opens Too Quickly

**Problem:** Circuit opens with only a few failures.

**Solutions:**
1. Increase `failure_threshold`
2. Check if errors are transient vs. permanent
3. Add retry logic before circuit breaker

### Circuit Stays Open Too Long

**Problem:** Service recovered but circuit still open.

**Solutions:**
1. Decrease `timeout`
2. Increase `half_open_max_requests` (more testing capacity)
3. Check logs for half-open failures

### Circuit Never Opens

**Problem:** Service is clearly down but circuit stays closed.

**Solutions:**
1. Verify errors are being propagated correctly
2. Check if `failure_threshold` is too high
3. Ensure circuit breaker is actually wrapping the calls

### Flapping (Open → Closed → Open)

**Problem:** Circuit rapidly changes states.

**Solutions:**
1. Increase `success_threshold` (need more proof of recovery)
2. Increase `timeout` (give service more time to stabilize)
3. Check for intermittent issues (network, load balancer)

---

## Migration from Existing Code

### Before (No Circuit Breaker)

```rust
async fn get_from_cache(key: &str) -> Result<String, Error> {
    let mut conn = redis_client.get_async_connection().await?;
    let value: String = conn.get(key).await?;
    Ok(value)
}
```

### After (With Circuit Breaker)

```rust
async fn get_from_cache(
    key: &str, 
    breaker: &CircuitBreaker
) -> Result<String, Error> {
    breaker.call(async {
        let mut conn = redis_client.get_async_connection().await?;
        let value: String = conn.get(key).await?;
        Ok(value)
    }).await
    .map_err(|e| match e {
        CircuitBreakerError::Open => Error::ServiceUnavailable,
        CircuitBreakerError::Upstream(e) => Error::Cache(e),
    })
}
```

---

## References

- [Martin Fowler - Circuit Breaker](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Release It! by Michael Nygard](https://pragprog.com/titles/mnee2/release-it-second-edition/)
- [Hystrix Design Principles](https://github.com/Netflix/Hystrix/wiki)
- [REFACTORING_ROADMAP.md](./REFACTORING_ROADMAP.md) - Sprint 2, Task 2.2

---

**Last Updated:** 2026-02-12  
**Component:** `rustok-core::circuit_breaker`  
**Sprint 2 Task:** 2.2 - Circuit Breaker Implementation
