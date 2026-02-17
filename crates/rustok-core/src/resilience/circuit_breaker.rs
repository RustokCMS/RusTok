/// Circuit Breaker Pattern Implementation
///
/// Prevents cascading failures by failing fast when a service is unavailable.
///
/// States:
/// - **Closed**: Normal operation, requests pass through
/// - **Open**: Service is down, requests fail immediately (fail-fast)
/// - **HalfOpen**: Testing if service recovered, limited requests pass through
///
/// Benefits:
/// - Prevents cascade failures
/// - Reduces latency during outages (fail-fast instead of timeout)
/// - Automatic recovery detection
/// - Resource protection (connections, threads)
///
/// Example:
/// ```rust
/// let breaker = CircuitBreaker::new(CircuitBreakerConfig {
///     failure_threshold: 5,
///     success_threshold: 2,
///     timeout: Duration::from_secs(60),
/// });
///
/// match breaker.call(|| external_service.call()).await {
///     Ok(result) => // Handle success
///     Err(CircuitBreakerError::Open) => // Service unavailable, fail-fast
///     Err(CircuitBreakerError::Execution(e)) => // Actual error
/// }
/// ```
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening circuit
    pub failure_threshold: u32,

    /// Number of consecutive successes to close circuit from half-open
    pub success_threshold: u32,

    /// Time to wait before transitioning from open to half-open
    pub timeout: Duration,

    /// Optional: Maximum requests allowed in half-open state
    pub half_open_max_requests: Option<u32>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_requests: Some(3),
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, requests pass through normally
    Closed,

    /// Circuit is open, requests fail immediately
    Open,

    /// Circuit is testing if service recovered
    HalfOpen,
}

impl CircuitState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CircuitState::Closed => "closed",
            CircuitState::Open => "open",
            CircuitState::HalfOpen => "half_open",
        }
    }

    /// Get numeric representation for metrics (0=closed, 1=open, 2=half_open)
    pub fn as_u8(&self) -> u8 {
        match self {
            CircuitState::Closed => 0,
            CircuitState::Open => 1,
            CircuitState::HalfOpen => 2,
        }
    }
}

/// Circuit breaker error
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E = String> {
    #[error("Circuit breaker is open, requests blocked")]
    Open,

    #[error("Upstream error: {0}")]
    Upstream(E),
}

/// Internal state tracking
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    half_open_requests: u32,
}

impl CircuitBreakerState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            half_open_requests: 0,
        }
    }
}

/// Circuit breaker implementation
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitBreakerState>>,

    // Metrics (atomic for lock-free reads)
    total_requests: AtomicU64,
    total_successes: AtomicU64,
    total_failures: AtomicU64,
    total_rejected: AtomicU64,
    state_transitions: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(CircuitBreakerState::new())),
            total_requests: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            total_rejected: AtomicU64::new(0),
            state_transitions: AtomicU64::new(0),
        }
    }

    /// Execute a fallible operation with circuit breaker protection
    pub async fn call<F, Fut, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        self.total_requests.fetch_add(1, Ordering::Relaxed);

        // Check if we can execute the request
        if !self.can_execute().await {
            self.total_rejected.fetch_add(1, Ordering::Relaxed);

            tracing::warn!(
                state = self.get_state().await.as_str(),
                "Circuit breaker rejected request"
            );

            return Err(CircuitBreakerError::Open);
        }

        // Execute the operation
        let start = Instant::now();
        let result = f().await;
        let duration = start.elapsed();

        // Record the result
        match result {
            Ok(value) => {
                self.record_success().await;
                self.total_successes.fetch_add(1, Ordering::Relaxed);

                tracing::debug!(
                    duration_ms = duration.as_millis(),
                    state = self.get_state().await.as_str(),
                    "Circuit breaker: success"
                );

                Ok(value)
            }
            Err(err) => {
                self.record_failure().await;
                self.total_failures.fetch_add(1, Ordering::Relaxed);

                tracing::warn!(
                    duration_ms = duration.as_millis(),
                    state = self.get_state().await.as_str(),
                    error = %err,
                    "Circuit breaker: failure"
                );

                Err(CircuitBreakerError::Upstream(err))
            }
        }
    }

    /// Check if we can execute a request
    async fn can_execute(&self) -> bool {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => true,

            CircuitState::Open => {
                // Check if timeout expired, transition to half-open
                if let Some(last_failure) = state.last_failure_time {
                    if last_failure.elapsed() >= self.config.timeout {
                        state.state = CircuitState::HalfOpen;
                        state.half_open_requests = 0;
                        self.state_transitions.fetch_add(1, Ordering::Relaxed);

                        tracing::info!("Circuit breaker: Open -> HalfOpen");

                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }

            CircuitState::HalfOpen => {
                // Check if we can allow another request in half-open state
                if let Some(max_requests) = self.config.half_open_max_requests {
                    if state.half_open_requests >= max_requests {
                        return false;
                    }
                }

                state.half_open_requests += 1;
                true
            }
        }
    }

    /// Record a successful execution
    async fn record_success(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }

            CircuitState::HalfOpen => {
                state.success_count += 1;

                // Check if we've reached success threshold
                if state.success_count >= self.config.success_threshold {
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.half_open_requests = 0;
                    self.state_transitions.fetch_add(1, Ordering::Relaxed);

                    tracing::info!("Circuit breaker: HalfOpen -> Closed");
                }
            }

            CircuitState::Open => {
                // Should not happen, but reset if it does
                tracing::warn!("Circuit breaker: Unexpected success in Open state");
            }
        }
    }

    /// Record a failed execution
    async fn record_failure(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::Closed => {
                state.failure_count += 1;

                // Check if we've reached failure threshold
                if state.failure_count >= self.config.failure_threshold {
                    state.state = CircuitState::Open;
                    state.last_failure_time = Some(Instant::now());
                    state.success_count = 0;
                    self.state_transitions.fetch_add(1, Ordering::Relaxed);

                    tracing::error!(
                        failure_count = state.failure_count,
                        "Circuit breaker: Closed -> Open"
                    );
                }
            }

            CircuitState::HalfOpen => {
                // Any failure in half-open returns to open
                state.state = CircuitState::Open;
                state.last_failure_time = Some(Instant::now());
                state.failure_count = 0;
                state.success_count = 0;
                state.half_open_requests = 0;
                self.state_transitions.fetch_add(1, Ordering::Relaxed);

                tracing::warn!("Circuit breaker: HalfOpen -> Open");
            }

            CircuitState::Open => {
                // Update last failure time
                state.last_failure_time = Some(Instant::now());
            }
        }
    }

    /// Get current state
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.state
    }

    /// Get circuit breaker statistics
    pub async fn stats(&self) -> CircuitBreakerStats {
        let state = self.state.read().await;

        CircuitBreakerStats {
            state: state.state,
            total_requests: self.total_requests.load(Ordering::Relaxed),
            total_successes: self.total_successes.load(Ordering::Relaxed),
            total_failures: self.total_failures.load(Ordering::Relaxed),
            total_rejected: self.total_rejected.load(Ordering::Relaxed),
            state_transitions: self.state_transitions.load(Ordering::Relaxed),
            failure_count: state.failure_count,
            success_count: state.success_count,
        }
    }

    /// Export metrics in Prometheus exposition format
    ///
    /// # Example
    /// ```
    /// let metrics = breaker.export_prometheus_metrics("redis_cache").await;
    /// println!("{}", metrics);
    /// ```
    pub async fn export_prometheus_metrics(&self, name: &str) -> String {
        let stats = self.stats().await;
        let state_value = stats.state.as_u8();

        format!(
            r#"# HELP circuit_breaker_state Current state of the circuit breaker (0=closed, 1=open, 2=half_open)
# TYPE circuit_breaker_state gauge
circuit_breaker_state{{name="{name}"}} {state}
# HELP circuit_breaker_requests_total Total number of requests
# TYPE circuit_breaker_requests_total counter
circuit_breaker_requests_total{{name="{name}"}} {requests}
# HELP circuit_breaker_successes_total Total number of successful requests
# TYPE circuit_breaker_successes_total counter
circuit_breaker_successes_total{{name="{name}"}} {successes}
# HELP circuit_breaker_failures_total Total number of failed requests
# TYPE circuit_breaker_failures_total counter
circuit_breaker_failures_total{{name="{name}"}} {failures}
# HELP circuit_breaker_rejected_total Total number of rejected requests (circuit open)
# TYPE circuit_breaker_rejected_total counter
circuit_breaker_rejected_total{{name="{name}"}} {rejected}
# HELP circuit_breaker_state_transitions_total Total number of state transitions
# TYPE circuit_breaker_state_transitions_total counter
circuit_breaker_state_transitions_total{{name="{name}"}} {transitions}
# HELP circuit_breaker_success_rate Current success rate (0.0 - 1.0)
# TYPE circuit_breaker_success_rate gauge
circuit_breaker_success_rate{{name="{name}"}} {success_rate}
# HELP circuit_breaker_rejection_rate Current rejection rate (0.0 - 1.0)
# TYPE circuit_breaker_rejection_rate gauge
circuit_breaker_rejection_rate{{name="{name}"}} {rejection_rate}
"#,
            name = name,
            state = state_value,
            requests = stats.total_requests,
            successes = stats.total_successes,
            failures = stats.total_failures,
            rejected = stats.total_rejected,
            transitions = stats.state_transitions,
            success_rate = stats.success_rate(),
            rejection_rate = stats.rejection_rate(),
        )
    }

    /// Force circuit to open (manual control)
    pub async fn open(&self) {
        let mut state = self.state.write().await;
        if state.state != CircuitState::Open {
            state.state = CircuitState::Open;
            state.last_failure_time = Some(Instant::now());
            self.state_transitions.fetch_add(1, Ordering::Relaxed);

            tracing::warn!("Circuit breaker: Manually opened");
        }
    }

    /// Force circuit to close (manual control)
    pub async fn close(&self) {
        let mut state = self.state.write().await;
        if state.state != CircuitState::Closed {
            state.state = CircuitState::Closed;
            state.failure_count = 0;
            state.success_count = 0;
            self.state_transitions.fetch_add(1, Ordering::Relaxed);

            tracing::info!("Circuit breaker: Manually closed");
        }
    }

    /// Reset all counters
    pub async fn reset(&self) {
        let mut state = self.state.write().await;
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.last_failure_time = None;
        state.half_open_requests = 0;

        self.total_requests.store(0, Ordering::Relaxed);
        self.total_successes.store(0, Ordering::Relaxed);
        self.total_failures.store(0, Ordering::Relaxed);
        self.total_rejected.store(0, Ordering::Relaxed);
        self.state_transitions.store(0, Ordering::Relaxed);

        tracing::info!("Circuit breaker: Reset");
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone, Copy)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub total_requests: u64,
    pub total_successes: u64,
    pub total_failures: u64,
    pub total_rejected: u64,
    pub state_transitions: u64,
    pub failure_count: u32,
    pub success_count: u32,
}

impl CircuitBreakerStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 1.0;
        }

        self.total_successes as f64 / self.total_requests as f64
    }

    pub fn rejection_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }

        self.total_rejected as f64 / self.total_requests as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;

    #[tokio::test]
    async fn test_closed_state_success() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        let result = breaker.call(|| async { Ok::<_, String>(42) }).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_closed_to_open_on_failures() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        });

        // First 2 failures: stay closed
        for _ in 0..2 {
            let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;
            assert_eq!(breaker.get_state().await, CircuitState::Closed);
        }

        // 3rd failure: open circuit
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_open_state_rejects_requests() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            ..Default::default()
        });

        // Trigger open state
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Next request should be rejected
        let result = breaker.call(|| async { Ok::<_, String>(42) }).await;

        assert!(matches!(result, Err(CircuitBreakerError::Open)));

        let stats = breaker.stats().await;
        assert_eq!(stats.total_rejected, 1);
    }

    #[tokio::test]
    async fn test_open_to_halfopen_after_timeout() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            ..Default::default()
        });

        // Trigger open state
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Next request should transition to half-open
        let _ = breaker.call(|| async { Ok::<_, String>(42) }).await;

        // Check state (might be closed if success threshold is 1)
        let state = breaker.get_state().await;
        assert!(state == CircuitState::HalfOpen || state == CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_halfopen_to_closed_on_success() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            half_open_max_requests: None,
        });

        // Trigger open state
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // First success in half-open
        let _ = breaker.call(|| async { Ok::<_, String>(1) }).await;

        // Second success should close circuit
        let _ = breaker.call(|| async { Ok::<_, String>(2) }).await;

        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_halfopen_to_open_on_failure() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 1,
            timeout: Duration::from_millis(100),
            ..Default::default()
        });

        // Trigger open state
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;

        // Wait for timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Failure in half-open should reopen circuit
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;

        assert_eq!(breaker.get_state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_statistics() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 10,
            ..Default::default()
        });

        // Execute some operations
        let _ = breaker.call(|| async { Ok::<_, String>(1) }).await;
        let _ = breaker.call(|| async { Ok::<_, String>(2) }).await;
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;

        let stats = breaker.stats().await;

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.total_successes, 2);
        assert_eq!(stats.total_failures, 1);
        assert_eq!(stats.success_rate(), 2.0 / 3.0);
    }

    #[tokio::test]
    async fn test_manual_control() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Manually open
        breaker.open().await;
        assert_eq!(breaker.get_state().await, CircuitState::Open);

        // Manually close
        breaker.close().await;
        assert_eq!(breaker.get_state().await, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_reset() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Execute some operations
        let _ = breaker.call(|| async { Ok::<_, String>(1) }).await;
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;

        // Reset
        breaker.reset().await;

        let stats = breaker.stats().await;
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_prometheus_metrics_export() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Execute some operations
        let _ = breaker.call(|| async { Ok::<_, String>(1) }).await;
        let _ = breaker.call(|| async { Ok::<_, String>(2) }).await;
        let _ = breaker.call(|| async { Err::<i32, _>("error") }).await;

        // Export metrics
        let metrics = breaker.export_prometheus_metrics("test_circuit").await;

        // Verify metrics format
        assert!(metrics.contains("circuit_breaker_state{name=\"test_circuit\"}"));
        assert!(metrics.contains("circuit_breaker_requests_total{name=\"test_circuit\"} 3"));
        assert!(metrics.contains("circuit_breaker_successes_total{name=\"test_circuit\"} 2"));
        assert!(metrics.contains("circuit_breaker_failures_total{name=\"test_circuit\"} 1"));
        assert!(metrics.contains("# HELP circuit_breaker_state"));
        assert!(metrics.contains("# TYPE circuit_breaker_requests_total counter"));
    }

    #[tokio::test]
    async fn test_circuit_state_as_u8() {
        assert_eq!(CircuitState::Closed.as_u8(), 0);
        assert_eq!(CircuitState::Open.as_u8(), 1);
        assert_eq!(CircuitState::HalfOpen.as_u8(), 2);
    }
}
