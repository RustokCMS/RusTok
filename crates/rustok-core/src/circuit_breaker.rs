use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use thiserror::Error;

/// Error types for circuit breaker operations
#[derive(Debug, Error)]
pub enum CircuitBreakerError<E> {
    /// Circuit breaker is open, rejecting requests
    #[error("Circuit breaker is open")]
    Open,
    
    /// Upstream service returned an error
    #[error("Upstream error: {0}")]
    Upstream(E),
}

/// Configuration for circuit breaker behavior
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures before opening the circuit
    pub failure_threshold: u32,
    
    /// Number of consecutive successes in half-open state before closing
    pub success_threshold: u32,
    
    /// Time to wait before attempting to close an open circuit
    pub timeout: Duration,
    
    /// Maximum number of requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            timeout: Duration::from_secs(60),
            half_open_max_requests: 3,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum State {
    /// Circuit is closed, requests flow normally
    Closed = 0,
    
    /// Circuit is open, requests are rejected
    Open = 1,
    
    /// Circuit is half-open, testing if service recovered
    HalfOpen = 2,
}

impl State {
    fn from_u32(value: u32) -> Self {
        match value {
            0 => State::Closed,
            1 => State::Open,
            2 => State::HalfOpen,
            _ => unreachable!("Invalid state value"),
        }
    }
}

/// Circuit breaker implementation
/// 
/// Protects services from cascading failures by temporarily blocking requests
/// when a failure threshold is reached. The circuit breaker has three states:
/// 
/// - **Closed**: Requests flow normally. Failures are counted.
/// - **Open**: Requests are immediately rejected. No calls to upstream service.
/// - **Half-Open**: Limited requests are allowed to test if service recovered.
/// 
/// # Example
/// 
/// ```rust,no_run
/// use rustok_core::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
/// use std::time::Duration;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CircuitBreakerConfig {
///     failure_threshold: 3,
///     success_threshold: 2,
///     timeout: Duration::from_secs(30),
///     half_open_max_requests: 2,
/// };
/// 
/// let breaker = CircuitBreaker::new(config);
/// 
/// // Call a potentially failing service
/// let result = breaker.call(async {
///     // Some operation that might fail
///     Ok::<_, String>("success")
/// }).await;
/// # Ok(())
/// # }
/// ```
pub struct CircuitBreaker {
    state: Arc<AtomicU32>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    half_open_requests: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(AtomicU32::new(State::Closed as u32)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            half_open_requests: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            config,
        }
    }
    
    /// Execute a future through the circuit breaker
    /// 
    /// If the circuit is open, returns `CircuitBreakerError::Open` immediately.
    /// If the circuit is closed or half-open, executes the future and tracks
    /// success/failure for state transitions.
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let state = self.get_state();
        
        match state {
            State::Closed => self.execute_closed(f).await,
            State::Open => {
                if self.should_attempt_reset().await {
                    self.transition_to_half_open();
                    self.execute_half_open(f).await
                } else {
                    Err(CircuitBreakerError::Open)
                }
            }
            State::HalfOpen => {
                // Check if we've exceeded half-open request limit
                let current = self.half_open_requests.load(Ordering::Acquire);
                if current >= self.config.half_open_max_requests {
                    return Err(CircuitBreakerError::Open);
                }
                
                self.half_open_requests.fetch_add(1, Ordering::AcqRel);
                let result = self.execute_half_open(f).await;
                self.half_open_requests.fetch_sub(1, Ordering::AcqRel);
                result
            }
        }
    }
    
    /// Get the current state of the circuit breaker
    pub fn get_state(&self) -> State {
        State::from_u32(self.state.load(Ordering::Acquire))
    }
    
    /// Get current failure count
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Acquire)
    }
    
    /// Get current success count (in half-open state)
    pub fn success_count(&self) -> u32 {
        self.success_count.load(Ordering::Acquire)
    }
    
    /// Manually reset the circuit breaker to closed state
    pub fn reset(&self) {
        tracing::info!("Circuit breaker manually reset to CLOSED state");
        self.state.store(State::Closed as u32, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
        self.half_open_requests.store(0, Ordering::Release);
    }
    
    async fn execute_closed<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        match f.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure().await;
                Err(CircuitBreakerError::Upstream(e))
            }
        }
    }
    
    async fn execute_half_open<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        match f.await {
            Ok(result) => {
                self.on_half_open_success();
                Ok(result)
            }
            Err(e) => {
                self.trip().await;
                Err(CircuitBreakerError::Upstream(e))
            }
        }
    }
    
    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Release);
    }
    
    async fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if failures >= self.config.failure_threshold {
            self.trip().await;
        }
    }
    
    fn on_half_open_success(&self) {
        let successes = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if successes >= self.config.success_threshold {
            self.close();
        }
    }
    
    async fn trip(&self) {
        tracing::warn!("Circuit breaker tripped to OPEN state");
        self.state.store(State::Open as u32, Ordering::Release);
        *self.last_failure_time.lock().await = Some(Instant::now());
    }
    
    fn close(&self) {
        tracing::info!("Circuit breaker closed (service recovered)");
        self.state.store(State::Closed as u32, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
        self.half_open_requests.store(0, Ordering::Release);
    }
    
    fn transition_to_half_open(&self) {
        tracing::info!("Circuit breaker transitioning to HALF_OPEN state");
        self.state.store(State::HalfOpen as u32, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
        self.half_open_requests.store(0, Ordering::Release);
    }
    
    async fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = *self.last_failure_time.lock().await {
            last_failure.elapsed() >= self.config.timeout
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicU32;
    use tokio::time::sleep;
    
    #[tokio::test]
    async fn test_circuit_starts_closed() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        assert_eq!(breaker.get_state(), State::Closed);
    }
    
    #[tokio::test]
    async fn test_successful_calls_keep_circuit_closed() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        
        for _ in 0..10 {
            let result = breaker.call(async { Ok::<_, String>("success") }).await;
            assert!(result.is_ok());
        }
        
        assert_eq!(breaker.get_state(), State::Closed);
        assert_eq!(breaker.failure_count(), 0);
    }
    
    #[tokio::test]
    async fn test_circuit_opens_after_threshold_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Fail 3 times
        for _ in 0..3 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(breaker.get_state(), State::Open);
    }
    
    #[tokio::test]
    async fn test_open_circuit_rejects_requests() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_secs(1),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Trip the circuit
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(breaker.get_state(), State::Open);
        
        // Next call should be rejected immediately
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));
    }
    
    #[tokio::test]
    async fn test_circuit_transitions_to_half_open_after_timeout() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Trip the circuit
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(breaker.get_state(), State::Open);
        
        // Wait for timeout
        sleep(Duration::from_millis(150)).await;
        
        // Next call should transition to half-open
        let result = breaker.call(async { Ok::<_, String>("success") }).await;
        assert!(result.is_ok());
        assert_eq!(breaker.get_state(), State::HalfOpen);
    }
    
    #[tokio::test]
    async fn test_half_open_closes_after_success_threshold() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Trip the circuit
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        // Wait and transition to half-open
        sleep(Duration::from_millis(150)).await;
        
        // First success (enters half-open)
        let _ = breaker.call(async { Ok::<_, String>("success") }).await;
        assert_eq!(breaker.get_state(), State::HalfOpen);
        
        // Second success (should close)
        let _ = breaker.call(async { Ok::<_, String>("success") }).await;
        assert_eq!(breaker.get_state(), State::Closed);
    }
    
    #[tokio::test]
    async fn test_half_open_reopens_on_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Trip the circuit
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        // Wait and transition to half-open
        sleep(Duration::from_millis(150)).await;
        let _ = breaker.call(async { Ok::<_, String>("success") }).await;
        assert_eq!(breaker.get_state(), State::HalfOpen);
        
        // Failure in half-open should reopen circuit
        let _ = breaker.call(async { Err::<String, _>("error") }).await;
        assert_eq!(breaker.get_state(), State::Open);
    }
    
    #[tokio::test]
    async fn test_half_open_request_limit() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            timeout: Duration::from_millis(100),
            half_open_max_requests: 2,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Trip the circuit
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        // Wait and transition to half-open
        sleep(Duration::from_millis(150)).await;
        
        // First request should work
        let _ = breaker.call(async { Ok::<_, String>("success") }).await;
        
        // Track concurrent requests
        let breaker_clone = Arc::new(breaker);
        let counter = Arc::new(AtomicU32::new(0));
        
        // Spawn multiple concurrent requests
        let mut handles = vec![];
        for _ in 0..5 {
            let b = breaker_clone.clone();
            let c = counter.clone();
            let handle = tokio::spawn(async move {
                sleep(Duration::from_millis(10)).await;
                let result = b.call(async { 
                    c.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, String>("success") 
                }).await;
                result
            });
            handles.push(handle);
        }
        
        let results: Vec<_> = futures::future::join_all(handles).await;
        
        // Some requests should be rejected due to half-open limit
        let rejected = results.iter().filter(|r| {
            matches!(r, Ok(Err(CircuitBreakerError::Open)))
        }).count();
        
        assert!(rejected > 0, "Some requests should be rejected in half-open state");
    }
    
    #[tokio::test]
    async fn test_manual_reset() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Trip the circuit
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(breaker.get_state(), State::Open);
        
        // Manual reset
        breaker.reset();
        
        assert_eq!(breaker.get_state(), State::Closed);
        assert_eq!(breaker.failure_count(), 0);
    }
    
    #[tokio::test]
    async fn test_success_resets_failure_count() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            ..Default::default()
        };
        let breaker = CircuitBreaker::new(config);
        
        // Fail twice
        for _ in 0..2 {
            let _ = breaker.call(async { Err::<String, _>("error") }).await;
        }
        
        assert_eq!(breaker.failure_count(), 2);
        
        // Success should reset counter
        let _ = breaker.call(async { Ok::<_, String>("success") }).await;
        assert_eq!(breaker.failure_count(), 0);
        assert_eq!(breaker.get_state(), State::Closed);
    }
}
