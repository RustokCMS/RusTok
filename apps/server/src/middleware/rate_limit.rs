/// Rate Limiting Middleware for RusToK
///
/// Implements a sliding window rate limiter to protect endpoints from abuse.
/// Supports per-IP rate limiting with configurable limits.
use axum::{
    body::Body,
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
            window: Duration::from_secs(60),
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

#[derive(Clone, Debug)]
struct RequestCounter {
    count: usize,
    window_start: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
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

    pub async fn check_rate_limit(&self, key: &str) -> Result<RateLimitInfo, StatusCode> {
        if !self.config.enabled {
            return Ok(RateLimitInfo::unlimited());
        }

        let now = Instant::now();

        {
            let requests = self.requests.read().await;
            if let Some(counter) = requests.get(key).filter(|counter| {
                now.duration_since(counter.window_start) <= self.config.window
                    && counter.count >= self.config.max_requests
            }) {
                let retry_after = self
                    .config
                    .window
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

        let mut requests = self.requests.write().await;
        let counter = requests
            .entry(key.to_string())
            .or_insert_with(|| RequestCounter {
                count: 0,
                window_start: now,
            });

        if now.duration_since(counter.window_start) > self.config.window {
            counter.count = 0;
            counter.window_start = now;
        }

        if counter.count >= self.config.max_requests {
            let retry_after = self
                .config
                .window
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

        let reset_at = counter.window_start + self.config.window;
        let reset_secs = reset_at.saturating_duration_since(now).as_secs();

        Ok(RateLimitInfo {
            limit: self.config.max_requests,
            remaining: self.config.max_requests.saturating_sub(counter.count),
            reset: reset_secs,
        })
    }

    pub async fn cleanup_expired(&self) {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        requests
            .retain(|_, counter| now.duration_since(counter.window_start) <= self.config.window);

        debug!(
            retained = requests.len(),
            "Cleaned up expired rate limit entries"
        );
    }

    pub async fn get_stats(&self) -> RateLimitStats {
        let requests = self.requests.read().await;
        RateLimitStats {
            active_clients: requests.len(),
            total_entries: requests.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitStats {
    pub active_clients: usize,
    pub total_entries: usize,
}

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

/// Extract client identifier from the request.
///
/// Priority:
/// 1. X-Forwarded-For — first IP in the list (behind a proxy)
/// 2. X-Real-IP (behind nginx)
/// 3. "ip:unknown" fallback
///
/// Security note: user identity MUST NOT be sourced from client-supplied headers
/// such as X-User-ID.  Any client can set an arbitrary value, which would allow
/// them to exhaust another user's rate-limit bucket or bypass their own.
/// User-scoped rate limiting must be implemented after JWT verification using
/// the verified claims from a trusted middleware layer.
fn extract_client_id(headers: &HeaderMap) -> String {
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                let ip = first_ip.trim();
                if IpAddr::from_str(ip).is_ok() {
                    return format!("ip:{}", ip);
                }
            }
        }
    }

    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if IpAddr::from_str(ip_str).is_ok() {
                return format!("ip:{}", ip_str);
            }
        }
    }

    "ip:unknown".to_string()
}

fn insert_header_if_valid(headers: &mut axum::http::HeaderMap, key: &'static str, value: String) {
    match axum::http::HeaderValue::from_str(&value) {
        Ok(header_value) => {
            headers.insert(key, header_value);
        }
        Err(error) => {
            tracing::warn!(%key, %value, %error, "Skipping invalid rate limit header value");
        }
    }
}

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

            let headers = response.headers_mut();
            insert_header_if_valid(headers, "x-ratelimit-limit", info.limit.to_string());
            insert_header_if_valid(headers, "x-ratelimit-remaining", info.remaining.to_string());
            insert_header_if_valid(headers, "x-ratelimit-reset", info.reset.to_string());

            Ok(response)
        }
        Err(status) => {
            let mut response = Response::new(Body::from("Rate limit exceeded"));
            *response.status_mut() = status;

            let headers = response.headers_mut();
            insert_header_if_valid(
                headers,
                "retry-after",
                limiter.config.window.as_secs().to_string(),
            );

            Err(response)
        }
    }
}

pub async fn cleanup_task(limiter: Arc<RateLimiter>) {
    let mut interval = tokio::time::interval(Duration::from_secs(300));

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

        for _ in 0..3 {
            assert!(limiter.check_rate_limit("test-client").await.is_ok());
        }

        let result = limiter.check_rate_limit("test-client").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn test_rate_limit_resets_after_window() {
        let config = RateLimitConfig::new(2, 1);
        let limiter = RateLimiter::new(config);

        assert!(limiter.check_rate_limit("test-client").await.is_ok());
        assert!(limiter.check_rate_limit("test-client").await.is_ok());
        assert!(limiter.check_rate_limit("test-client").await.is_err());

        tokio::time::sleep(Duration::from_secs(2)).await;

        assert!(limiter.check_rate_limit("test-client").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_separate_clients() {
        let config = RateLimitConfig::new(2, 60);
        let limiter = RateLimiter::new(config);

        assert!(limiter.check_rate_limit("client-a").await.is_ok());
        assert!(limiter.check_rate_limit("client-a").await.is_ok());
        assert!(limiter.check_rate_limit("client-a").await.is_err());

        assert!(limiter.check_rate_limit("client-b").await.is_ok());
        assert!(limiter.check_rate_limit("client-b").await.is_ok());
    }

    #[tokio::test]
    async fn test_disabled_rate_limiter() {
        let config = RateLimitConfig::disabled();
        let limiter = RateLimiter::new(config);

        for _ in 0..1000 {
            assert!(limiter.check_rate_limit("test-client").await.is_ok());
        }
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let config = RateLimitConfig::new(10, 1);
        let limiter = RateLimiter::new(config);

        limiter.check_rate_limit("client-1").await.ok();
        limiter.check_rate_limit("client-2").await.ok();
        limiter.check_rate_limit("client-3").await.ok();

        {
            let requests = limiter.requests.read().await;
            assert_eq!(requests.len(), 3);
        }

        tokio::time::sleep(Duration::from_secs(2)).await;

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

        for i in 0..50 {
            let limiter = limiter.clone();
            tasks.spawn(async move { limiter.check_rate_limit(&format!("client-{}", i)).await });
        }

        while let Some(result) = tasks.join_next().await {
            assert!(result.unwrap().is_ok());
        }
    }

    #[test]
    fn extract_client_id_does_not_use_x_user_id() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-user-id",
            "550e8400-e29b-41d4-a716-446655440000".parse().unwrap(),
        );
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());

        let id = extract_client_id(&headers);
        assert_eq!(id, "ip:1.2.3.4", "must use IP, not x-user-id");
    }

    #[test]
    fn extract_client_id_falls_back_to_unknown() {
        let headers = HeaderMap::new();
        let id = extract_client_id(&headers);
        assert_eq!(id, "ip:unknown");
    }
}
