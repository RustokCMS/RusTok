# Redis Circuit Breaker Integration

> **Date:** 2026-02-12  
> **Sprint:** Sprint 2 - Simplification  
> **Component:** `rustok-core::cache::RedisCacheBackend`

---

## Overview

The `RedisCacheBackend` now includes built-in circuit breaker protection to prevent cascading failures when Redis is unavailable or experiencing issues.

### Benefits

✅ **Automatic failure detection** - Opens circuit after threshold failures  
✅ **Fast-fail when Redis is down** - Returns error immediately (~0.5μs)  
✅ **Automatic recovery detection** - Tests Redis health periodically  
✅ **Configurable thresholds** - Tune for your environment  
✅ **Zero code changes** - Works with existing `CacheBackend` trait  

---

## Usage

### Basic Usage (Default Configuration)

```rust
use rustok_core::RedisCacheBackend;
use std::time::Duration;

// Uses default circuit breaker config
let cache = RedisCacheBackend::new(
    "redis://localhost:6379",
    "myapp",
    Duration::from_secs(300),
)?;

// All operations are now protected
cache.set("key".to_string(), b"value".to_vec()).await?;
let value = cache.get("key").await?;
```

**Default Configuration:**
- `failure_threshold: 5` - Open after 5 consecutive failures
- `success_threshold: 2` - Close after 2 consecutive successes
- `timeout: 60s` - Wait 60s before testing recovery
- `half_open_max_requests: 3` - Allow 3 concurrent tests

### Custom Circuit Breaker Configuration

```rust
use rustok_core::{RedisCacheBackend, CircuitBreakerConfig};
use std::time::Duration;

let breaker_config = CircuitBreakerConfig {
    failure_threshold: 3,      // Trip faster
    success_threshold: 2,
    timeout: Duration::from_secs(30),
    half_open_max_requests: 2,
};

let cache = RedisCacheBackend::with_circuit_breaker(
    "redis://localhost:6379",
    "myapp",
    Duration::from_secs(300),
    breaker_config,
)?;
```

### Monitoring Circuit Breaker State

```rust
use rustok_core::RedisCacheBackend;

let cache = RedisCacheBackend::new(/* ... */)?;

// Access the circuit breaker
let breaker = cache.circuit_breaker();

// Check failure count
let failures = breaker.failure_count();
println!("Current failures: {}", failures);

// Check state
match breaker.get_state() {
    State::Closed => println!("✅ Redis healthy"),
    State::Open => println!("❌ Redis circuit open"),
    State::HalfOpen => println!("⚠️ Testing recovery"),
}
```

---

## Behavior

### Normal Operation (Circuit Closed)

```rust
// Redis is healthy
let result = cache.get("key").await;
// Result: Ok(Some(value)) or Ok(None)
```

### Redis Failure (Circuit Opening)

```rust
// Redis connection fails 5 times (default threshold)
for _ in 0..5 {
    let _ = cache.get("key").await; // Each fails
}

// Circuit is now OPEN
let result = cache.get("key").await;
// Result: Err(Error::Cache("Redis unavailable (circuit breaker open)"))
// Returns immediately, no attempt to connect to Redis
```

### Recovery Testing (Half-Open State)

```rust
// After timeout (60s default), circuit enters HALF_OPEN
// Limited requests are allowed to test recovery

let result1 = cache.get("key1").await; // Test request
let result2 = cache.get("key2").await; // Test request

// If 2 consecutive successes (default threshold)
// Circuit closes and normal operation resumes
```

---

## Error Handling

### Handling Circuit Breaker Errors

```rust
use rustok_core::{RedisCacheBackend, Error};

let cache = RedisCacheBackend::new(/* ... */)?;

match cache.get("key").await {
    Ok(Some(value)) => {
        // Cache hit
        println!("Found: {:?}", value);
    }
    Ok(None) => {
        // Cache miss
        println!("Not in cache");
    }
    Err(Error::Cache(msg)) if msg.contains("circuit breaker open") => {
        // Circuit is open, use fallback
        tracing::warn!("Redis unavailable, using fallback");
        // Return default or query database
    }
    Err(e) => {
        // Other cache error
        tracing::error!("Cache error: {}", e);
    }
}
```

### Fallback Strategy

```rust
async fn get_user_cached(
    id: Uuid,
    cache: &RedisCacheBackend,
    db: &DatabaseConnection,
) -> Result<User, Error> {
    // Try cache first
    let cache_key = format!("user:{}", id);
    
    match cache.get(&cache_key).await {
        Ok(Some(bytes)) => {
            // Cache hit
            serde_json::from_slice(&bytes)
                .map_err(|e| Error::Cache(e.to_string()))
        }
        Ok(None) => {
            // Cache miss - query database
            let user = query_user_from_db(id, db).await?;
            
            // Try to cache (ignore errors)
            let _ = cache.set(
                cache_key,
                serde_json::to_vec(&user)?,
            ).await;
            
            Ok(user)
        }
        Err(Error::Cache(msg)) if msg.contains("circuit breaker open") => {
            // Circuit open - skip cache, go directly to database
            tracing::warn!("Redis circuit open, querying database directly");
            query_user_from_db(id, db).await
        }
        Err(e) => {
            // Other error - also fallback to database
            tracing::error!("Cache error: {}, using database", e);
            query_user_from_db(id, db).await
        }
    }
}
```

---

## Configuration Tuning

### Production (Conservative)

```rust
CircuitBreakerConfig {
    failure_threshold: 5,      // Tolerate some transient errors
    success_threshold: 2,      // Need proof of recovery
    timeout: Duration::from_secs(60),
    half_open_max_requests: 3,
}
```

**Use when:**
- Redis is generally reliable
- Occasional network blips
- Want to avoid premature circuit opening

### High Availability (Aggressive)

```rust
CircuitBreakerConfig {
    failure_threshold: 3,      // Fast failure detection
    success_threshold: 3,      // Require strong proof
    timeout: Duration::from_secs(30),
    half_open_max_requests: 1,
}
```

**Use when:**
- Need fast failover
- Have good fallback mechanisms
- Want to minimize impact of Redis issues

### Development (Tolerant)

```rust
CircuitBreakerConfig {
    failure_threshold: 10,     // Very tolerant
    success_threshold: 1,      // Quick recovery
    timeout: Duration::from_secs(120),
    half_open_max_requests: 5,
}
```

**Use when:**
- Redis is flaky in dev environment
- Don't want circuit breaking during development
- Testing fallback mechanisms

---

## Monitoring

### Logging

The circuit breaker automatically logs state changes:

```
WARN Redis cache circuit breaker is OPEN
INFO Circuit breaker transitioning to HALF_OPEN state
INFO Circuit breaker closed (service recovered)
```

### Metrics

Export circuit breaker metrics to Prometheus:

```rust
use metrics::{gauge, counter};

// Periodically export metrics
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(10));
    loop {
        interval.tick().await;
        
        let breaker = cache.circuit_breaker();
        let state = breaker.get_state();
        
        gauge!("redis_circuit_breaker_state", state as u32 as f64);
        gauge!("redis_circuit_breaker_failures", breaker.failure_count() as f64);
        gauge!("redis_circuit_breaker_successes", breaker.success_count() as f64);
        
        if state == State::Open {
            counter!("redis_circuit_breaker_open_total", 1);
        }
    }
});
```

### Health Checks

Include circuit breaker state in health endpoint:

```rust
#[derive(Serialize)]
struct HealthResponse {
    status: String,
    redis_status: String,
    redis_circuit_state: String,
}

async fn health(cache: Arc<RedisCacheBackend>) -> Json<HealthResponse> {
    let breaker = cache.circuit_breaker();
    let state = breaker.get_state();
    
    let redis_status = match state {
        State::Closed => "healthy",
        State::Open => "unavailable",
        State::HalfOpen => "recovering",
    };
    
    Json(HealthResponse {
        status: "ok".to_string(),
        redis_status: redis_status.to_string(),
        redis_circuit_state: format!("{:?}", state),
    })
}
```

---

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_redis_cache_with_invalid_url() {
    let cache = RedisCacheBackend::with_circuit_breaker(
        "redis://invalid:9999",
        "test",
        Duration::from_secs(300),
        CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        },
    ).unwrap();
    
    // First failure
    let result1 = cache.get("key1").await;
    assert!(result1.is_err());
    
    // Second failure - circuit trips
    let result2 = cache.get("key2").await;
    assert!(result2.is_err());
    
    // Check state
    assert_eq!(cache.circuit_breaker().failure_count(), 2);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_redis_circuit_breaker_with_real_redis() {
    // Requires Redis running
    let cache = RedisCacheBackend::new(
        "redis://127.0.0.1:6379",
        "test",
        Duration::from_secs(300),
    ).unwrap();
    
    // Test normal operation
    cache.set("key".to_string(), b"value".to_vec()).await.unwrap();
    let value = cache.get("key").await.unwrap();
    assert_eq!(value, Some(b"value".to_vec()));
    
    // Circuit should remain closed
    assert_eq!(cache.circuit_breaker().failure_count(), 0);
}
```

---

## Migration

### Before (No Circuit Breaker)

```rust
// Old code - no protection
let cache = RedisCacheBackend::new(
    "redis://localhost:6379",
    "myapp",
    Duration::from_secs(300),
)?;

// If Redis is down, this hangs or times out
let value = cache.get("key").await?;
```

### After (With Circuit Breaker)

```rust
// New code - automatic protection
let cache = RedisCacheBackend::new(
    "redis://localhost:6379",
    "myapp",
    Duration::from_secs(300),
)?;

// If Redis is down, fails fast after threshold
match cache.get("key").await {
    Ok(value) => /* use value */,
    Err(e) if e.to_string().contains("circuit breaker") => {
        // Use fallback
    },
    Err(e) => /* handle error */,
}
```

**No API changes required!** The circuit breaker is integrated transparently.

---

## Best Practices

### 1. Always Implement Fallbacks

```rust
// ❌ Bad - no fallback
let value = cache.get("key").await?;

// ✅ Good - fallback to database
let value = match cache.get("key").await {
    Ok(Some(v)) => v,
    _ => query_from_db("key").await?,
};
```

### 2. Log Circuit State Changes

```rust
// Monitor circuit state
if cache.circuit_breaker().get_state() == State::Open {
    tracing::error!(
        "Redis circuit breaker is OPEN - using fallbacks",
        failures = cache.circuit_breaker().failure_count()
    );
}
```

### 3. Tune for Your Environment

```rust
// Production: conservative
let prod_config = CircuitBreakerConfig {
    failure_threshold: 5,
    timeout: Duration::from_secs(60),
    ..Default::default()
};

// Development: tolerant
let dev_config = CircuitBreakerConfig {
    failure_threshold: 10,
    timeout: Duration::from_secs(10),
    ..Default::default()
};
```

### 4. Monitor and Alert

Set up alerts for:
- Circuit state changes (CLOSED → OPEN)
- High failure counts
- Extended time in OPEN state

---

## Troubleshooting

### Circuit Opens Too Quickly

**Problem:** Circuit opens with only a few failures.

**Solutions:**
- Increase `failure_threshold` (e.g., from 3 to 5)
- Check if errors are truly Redis failures
- Review Redis connection stability

### Circuit Stays Open Too Long

**Problem:** Redis recovered but circuit still open.

**Solutions:**
- Decrease `timeout` (e.g., from 60s to 30s)
- Increase `half_open_max_requests` for more testing
- Check logs for half-open failures

### High Latency During Recovery

**Problem:** Requests slow during half-open state.

**Solutions:**
- Adjust `half_open_max_requests` (fewer = faster failover)
- Ensure Redis health check is fast
- Consider separate circuit breakers for read/write

---

## Related Documentation

- [CIRCUIT_BREAKER_GUIDE.md](./CIRCUIT_BREAKER_GUIDE.md) - General circuit breaker guide
- [REFACTORING_ROADMAP.md](./REFACTORING_ROADMAP.md) - Sprint 2, Task 2.2
- [IMPLEMENTATION_PROGRESS.md](./IMPLEMENTATION_PROGRESS.md) - Progress tracking

---

**Last Updated:** 2026-02-12  
**Sprint 2 Task:** Circuit Breaker Integration with Redis Cache  
**Component:** `rustok-core::cache::RedisCacheBackend`
