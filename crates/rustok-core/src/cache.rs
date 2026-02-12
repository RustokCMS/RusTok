use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;

use crate::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError};
use crate::context::CacheBackend;
use crate::Result;

#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub entries: u64,
}

pub struct InMemoryCacheBackend {
    cache: Cache<String, Vec<u8>>,
}

impl InMemoryCacheBackend {
    pub fn new(ttl: Duration, max_capacity: u64) -> Self {
        let cache = Cache::builder()
            .time_to_live(ttl)
            .max_capacity(max_capacity)
            .build();
        Self { cache }
    }
}

#[cfg(feature = "redis-cache")]
pub struct RedisCacheBackend {
    client: redis::Client,
    prefix: String,
    ttl: Duration,
    circuit_breaker: Arc<CircuitBreaker>,
}

#[cfg(feature = "redis-cache")]
impl RedisCacheBackend {
    pub fn new(url: &str, prefix: impl Into<String>, ttl: Duration) -> Result<Self> {
        Self::with_circuit_breaker(url, prefix, ttl, CircuitBreakerConfig::default())
    }

    pub fn with_circuit_breaker(
        url: &str,
        prefix: impl Into<String>,
        ttl: Duration,
        breaker_config: CircuitBreakerConfig,
    ) -> Result<Self> {
        let client =
            redis::Client::open(url).map_err(|err| crate::Error::Cache(err.to_string()))?;
        Ok(Self {
            client,
            prefix: prefix.into(),
            ttl,
            circuit_breaker: Arc::new(CircuitBreaker::new(breaker_config)),
        })
    }

    pub fn circuit_breaker(&self) -> &CircuitBreaker {
        &self.circuit_breaker
    }

    fn key(&self, key: &str) -> String {
        if self.prefix.is_empty() {
            key.to_string()
        } else {
            format!("{}:{key}", self.prefix)
        }
    }
}

#[async_trait]
impl CacheBackend for InMemoryCacheBackend {
    async fn health(&self) -> Result<()> {
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.cache.get(key).await)
    }

    async fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        self.cache.insert(key, value).await;
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<()> {
        self.cache.invalidate(key).await;
        Ok(())
    }

    fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.entry_count(),
            ..CacheStats::default()
        }
    }
}

#[cfg(feature = "redis-cache")]
#[async_trait]
impl CacheBackend for RedisCacheBackend {
    async fn health(&self) -> Result<()> {
        let client = self.client.clone();
        
        self.circuit_breaker
            .call(async move {
                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                let pong: String = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                if pong == "PONG" {
                    Ok(())
                } else {
                    Err(crate::Error::Cache(format!(
                        "unexpected Redis PING response: {pong}"
                    )))
                }
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Open => {
                    tracing::warn!("Redis cache circuit breaker is OPEN");
                    crate::Error::Cache("Redis unavailable (circuit breaker open)".to_string())
                }
                CircuitBreakerError::Upstream(err) => err,
            })
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let client = self.client.clone();
        let redis_key = self.key(key);
        
        self.circuit_breaker
            .call(async move {
                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                let value: Option<Vec<u8>> = redis::cmd("GET")
                    .arg(redis_key)
                    .query_async(&mut conn)
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                Ok(value)
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Open => {
                    tracing::debug!("Redis cache GET failed: circuit breaker open");
                    crate::Error::Cache("Redis unavailable (circuit breaker open)".to_string())
                }
                CircuitBreakerError::Upstream(err) => err,
            })
    }

    async fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let client = self.client.clone();
        let redis_key = self.key(&key);
        let ttl_secs = self.ttl.as_secs();
        
        self.circuit_breaker
            .call(async move {
                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                redis::cmd("SET")
                    .arg(redis_key)
                    .arg(value)
                    .arg("EX")
                    .arg(ttl_secs)
                    .query_async::<()>(&mut conn)
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Open => {
                    tracing::debug!("Redis cache SET failed: circuit breaker open");
                    crate::Error::Cache("Redis unavailable (circuit breaker open)".to_string())
                }
                CircuitBreakerError::Upstream(err) => err,
            })
    }

    async fn invalidate(&self, key: &str) -> Result<()> {
        let client = self.client.clone();
        let redis_key = self.key(key);
        
        self.circuit_breaker
            .call(async move {
                let mut conn = client
                    .get_multiplexed_async_connection()
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                redis::cmd("DEL")
                    .arg(redis_key)
                    .query_async::<()>(&mut conn)
                    .await
                    .map_err(|err| crate::Error::Cache(err.to_string()))?;
                Ok(())
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::Open => {
                    tracing::debug!("Redis cache DEL failed: circuit breaker open");
                    crate::Error::Cache("Redis unavailable (circuit breaker open)".to_string())
                }
                CircuitBreakerError::Upstream(err) => err,
            })
    }

    fn stats(&self) -> CacheStats {
        CacheStats::default()
    }
}

#[cfg(test)]
#[path = "cache_tests.rs"]
mod tests;
