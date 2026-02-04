use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;

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
}

#[cfg(feature = "redis-cache")]
impl RedisCacheBackend {
    pub fn new(url: &str, prefix: impl Into<String>, ttl: Duration) -> Result<Self> {
        let client =
            redis::Client::open(url).map_err(|err| crate::Error::Cache(err.to_string()))?;
        Ok(Self {
            client,
            prefix: prefix.into(),
            ttl,
        })
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
        let mut conn = self
            .client
            .get_async_connection()
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
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| crate::Error::Cache(err.to_string()))?;
        let value: Option<Vec<u8>> = redis::cmd("GET")
            .arg(self.key(key))
            .query_async(&mut conn)
            .await
            .map_err(|err| crate::Error::Cache(err.to_string()))?;
        Ok(value)
    }

    async fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| crate::Error::Cache(err.to_string()))?;
        redis::cmd("SET")
            .arg(self.key(&key))
            .arg(value)
            .arg("EX")
            .arg(self.ttl.as_secs())
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|err| crate::Error::Cache(err.to_string()))?;
        Ok(())
    }

    async fn invalidate(&self, key: &str) -> Result<()> {
        let mut conn = self
            .client
            .get_async_connection()
            .await
            .map_err(|err| crate::Error::Cache(err.to_string()))?;
        redis::cmd("DEL")
            .arg(self.key(key))
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(|err| crate::Error::Cache(err.to_string()))?;
        Ok(())
    }

    fn stats(&self) -> CacheStats {
        CacheStats::default()
    }
}
