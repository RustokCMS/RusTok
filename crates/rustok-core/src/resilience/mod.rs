/// Resilience patterns for fault-tolerant systems
/// 
/// This module provides:
/// - Circuit Breaker: Prevent cascading failures
/// - Retry: Automatic retry with backoff
/// - Timeout: Enforce operation deadlines
/// - Bulkhead: Isolate resources

pub mod circuit_breaker;
pub mod retry;
pub mod timeout;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitState};
pub use retry::{RetryPolicy, RetryStrategy};
pub use timeout::with_timeout;
