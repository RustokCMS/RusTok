/// Rate Limiting Middleware for RusToK
///
/// Implements a sliding window rate limiter to protect endpoints from abuse.
/// Supports per-IP and per-user rate limiting with configurable limits.

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

/// Configuration for rate limiting
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: usize,
    /// Time window duration
    pub window: Duration,
    /// Whether to enable rate limiting (can be disabled in dev)
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window: Duration::from_secs(60), // 100 requests per minute
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_secs),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        Self {
            max_requests: 0,
            window: Duration::from_secs(0),
            enabled: false,
        }
    }
}

/// Tracks request counts for a specific client
#[derive(Clone, Debug)]
struct RequestCounter {
    count: usize,
    window_start: Instant,
}

/// Rate limiter implementation using sliding window algorithm
#[derive(Clone)]
pub struct RateLimiter {
    /// Map of client ID to request counter
    requests: Arc<RwLock<HashMap<String, RequestCounter>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Check if a request should be rate limited
    /// 
    /// FIXED: Improved locking strategy - use read lock first, write lock only when needed
    pub async fn check_rate_limit(&self, key: &str) -> Result<RateLimitInfo, StatusCode> {
        if !self.config.enabled {
            return Ok(RateLimitInfo::unlimited());
        }

        let now = Instant::now();
        
        // First, try to check with read lock (allows concurrent reads)
        {
            let requests = self.requests.read().await;
            if let Some(counter) = requests.get(key) {
                // Check if window is still valid
                if now.duration_since(counter.window_start) <= self.config.window {
                    // Check if limit exceeded
                    if counter.count >= self.config.max_requests {
                        let retry_after = self.config.window
                            .saturating_sub(now.duration_since(counter.window_start))
                            .as_secs();

                        warn!(
                            key = %key,
                            count = counter.count,
                            limit = self.config.max_requests,
                            retry_after = retry_after,
                            "Rate limit exceeded"
                        );

                        return Err(StatusCode::TOO_MANY_REQUESTS);
                    }
                }
            }
        }

        // Now acquire write lock to increment or create counter
        let mut requests = self.requests.write().await;
        let counter = requests.entry(key.to_string()).or_insert_with(|| RequestCounter {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(counter.window_start) > self.config.window {
            counter.count = 0;
            counter.window_start = now;
        }

        // Double-check limit (race condition protection)
        if counter.count >= self.config.max_requests {
            let retry_after = self.config.window
                .saturating_sub(now.duration_since(counter.window_start))
                .as_secs();

            warn!(
                key = %key,
                count = counter.count,
                limit = self.config.max_requests,
                retry_after = retry_after,
                "Rate limit exceeded (race condition check)"
            );

            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        counter.count += 1;

        // FIXED: Correct calculation of reset time
        let reset_at = counter.window_start + self.config.window;
        let reset_secs = reset_at
            .saturating_duration_since(now)
            .as_secs();

        Ok(RateLimitInfo {
            limit: self.config.max_requests,
            remaining: self.config.max_requests.saturating_sub(counter.count),
            reset: reset_secs,
        })
    }

    /// Clean up old entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        requests.retain(|_, counter| {
            now.duration_since(counter.window_start) <= self.config.window
        });

        debug!(retained = requests.len(), "Cleaned up expired rate limit entries");
    }
    
    /// Get current statistics (useful for monitoring)
    pub async fn get_stats(&self) -> RateLimitStats {
        let requests = self.requests.read().await;
        RateLimitStats {
            active_clients: requests.len(),
            total_entries: requests.len(),
        }
    }
}

/// Statistics about rate limiter
#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub active_clients: usize,
    pub total_entries: usize,
}

/// Information about current rate limit status
#[derive(Debug)]
pub struct RateLimitInfo {
    pub limit: usize,
    pub remaining: usize,
    pub reset: u64,
}

impl RateLimitInfo {
    fn unlimited() -> Self {
        Self {
            limit: usize::MAX,
            remaining: usize::MAX,
            reset: 0,
        }
    }
}

/// Extract client identifier from request
fn extract_client_id(headers: &HeaderMap) -> String {
    // Priority:
    // 1. User ID from JWT (if authenticated)
    // 2. X-Forwarded-For (behind proxy)
    // 3. X-Real-IP (behind nginx)
    // 4. Remote IP from connection

    // Try to get authenticated user ID from headers
    // (This would be set by auth middleware)
    if let Some(user_id) = headers.get("x-user-id") {
        if let Ok(user_id_str) = user_id.to_str() {
            if let Ok(uuid) = Uuid::parse_str(user_id_str) {
                return format!("user:{}", uuid);
            }
        }
    }

    // Try X-Forwarded-For
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take first IP if multiple
            if let Some(first_ip) = forwarded_str.split(',').next() {
                let ip = first_ip.trim();
                if IpAddr::from_str(ip).is_ok() {
                    return format!("ip:{}", ip);
                }
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if IpAddr::from_str(ip_str).is_ok() {
                return format!("ip:{}", ip_str);
            }
        }
    }

    // Fallback to unknown (will be rate limited as single client)
    "ip:unknown".to_string()
}

/// Axum middleware for rate limiting
pub async fn rate_limit_middleware(
    State(limiter): State<Arc<RateLimiter>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    let client_id = extract_client_id(&headers);

    debug!(client_id = %client_id, "Checking rate limit");

    match limiter.check_rate_limit(&client_id).await {
        Ok(info) => {
            let mut response = next.run(request).await;

            // Add rate limit headers
            let headers = response.headers_mut();
            headers.insert(
                "x-ratelimit-limit",
                info.limit.to_string().parse().unwrap(),
            );
            headers.insert(
                "x-ratelimit-remaining",
                info.remaining.to_string().parse().unwrap(),
            );
            headers.insert(
                "x-ratelimit-reset",
                info.reset.to_string().parse().unwrap(),
            );

            Ok(response)
        }
        Err(status) => {
            // Return 429 Too Many Requests
            let mut response = Response::new("Rate limit exceeded".into());
            *response.status_mut() = status;

            let headers = response.headers_mut();
            headers.insert(
                "retry-after",
                limiter
                    .config
                    .window
                    .as_secs()
                    .to_string()
                    .parse()
                    .unwrap(),
            );

            Err(response)
        }
    }
}

/// Background task to periodically clean up expired rate limit entries
pub async fn cleanup_task(limiter: Arc<RateLimiter>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes

    loop {
        interval.tick().await;
        limiter.cleanup_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_allows_requests_within_limit() {
        let config = RateLimitConfig::new(5, 60);
        let limiter = RateLimiter::new(config);

        for i in 1..=5 {
            let result = limiter.check_rate_limit("test-client").await;
            assert!(result.is_ok(), "Request {} should be allowed", i);

            let info = result.unwrap();
            assert_eq!(info.remaining, 5 - i);
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess_requests() {
        let config = RateLimitConfig::new(3, 60);
        let limiter = RateLimiter::new(config);

        // First 3 should succeed
        for _ in 0..3 {
            assert!(limiter.check_rate_limit("test-client").await.is_ok());
        }

        // 4th should fail
        let result = limiter.check_rate_limit("test-client").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_rate_limit_resets_after_window() {
        let config = RateLimitConfig::new(2, 1); // 2 requests per second
        let limiter = RateLimiter::new(config);

        // Use up limit
        assert!(limiter.check_rate_limit("test-client").await.is_ok());
        assert!(limiter.check_rate_limit("test-client").await.is_ok());
        assert!(limiter.check_rate_limit("test-client").await.is_err());

        // Wait for window to expire
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Should be allowed again
        assert!(limiter.check_rate_limit("test-client").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_separate_clients() {
        let config = RateLimitConfig::new(2, 60);
        let limiter = RateLimiter::new(config);

        // Client A uses limit
        assert!(limiter.check_rate_limit("client-a").await.is_ok());
        assert!(limiter.check_rate_limit("client-a").await.is_ok());
        assert!(limiter.check_rate_limit("client-a").await.is_err());

        // Client B should still have full limit
        assert!(limiter.check_rate_limit("client-b").await.is_ok());
        assert!(limiter.check_rate_limit("client-b").await.is_ok());
    }

    #[tokio::test]
    async fn test_disabled_rate_limiter() {
        let config = RateLimitConfig::disabled();
        let limiter = RateLimiter::new(config);

        // Should allow unlimited requests
        for _ in 0..1000 {
            assert!(limiter.check_rate_limit("test-client").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = RateLimitConfig::new(10, 1); // 1 second window
        let limiter = RateLimiter::new(config);

        // Create some entries
        limiter.check_rate_limit("client-1").await.ok();
        limiter.check_rate_limit("client-2").await.ok();
        limiter.check_rate_limit("client-3").await.ok();

        {
            let requests = limiter.requests.read().await;
            assert_eq!(requests.len(), 3);
        }

        // Wait for expiration
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Cleanup
        limiter.cleanup_expired().await;

        {
            let requests = limiter.requests.read().await;
            assert_eq!(requests.len(), 0);
        }
    }
    
    #[tokio::test]
    async fn test_concurrent_requests() {
        use tokio::task::JoinSet;
        
        let config = RateLimitConfig::new(100, 60);
        let limiter = Arc::new(RateLimiter::new(config));
        
        let mut tasks = JoinSet::new();
        
        // Spawn 50 concurrent requests
        for i in 0..50 {
            let limiter = limiter.clone();
            tasks.spawn(async move {
                limiter.check_rate_limit(&format!("client-{}", i)).await
            });
        }
        
        // All should succeed (different clients)
        while let Some(result) = tasks.join_next().await {
            assert!(result.unwrap().is_ok());
        }
    }
}
