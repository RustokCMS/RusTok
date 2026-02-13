use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use sea_orm::DatabaseConnection;
use thiserror::Error;

use crate::context::TenantContext;
use crate::models::tenants;

#[derive(Debug, Error)]
pub enum TenantError {
    #[error("Tenant not found")]
    NotFound,
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),
    
    #[error("Failed to load tenant: {0}")]
    LoadFailed(String),
}

/// Tenant key for caching - supports multiple lookup methods
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum TenantKey {
    /// Lookup by UUID (primary key)
    Uuid(Uuid),
    
    /// Lookup by slug (unique identifier)
    Slug(String),
    
    /// Lookup by hostname (for multi-tenant domains)
    Host(String),
}

/// Simplified tenant resolver using moka cache
/// 
/// This replaces the complex manual caching infrastructure with moka's
/// built-in cache stampede protection and TTL management.
pub struct TenantResolver {
    cache: Cache<TenantKey, Arc<TenantContext>>,
    db: DatabaseConnection,
}

impl TenantResolver {
    /// Create a new tenant resolver with default cache settings
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_config(db, TenantResolverConfig::default())
    }
    
    /// Create a new tenant resolver with custom cache settings
    pub fn with_config(db: DatabaseConnection, config: TenantResolverConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.time_to_live)
            .time_to_idle(config.time_to_idle)
            .build();
        
        Self { cache, db }
    }
    
    /// Resolve a tenant by key (with automatic caching and cache stampede protection)
    /// 
    /// Moka's `try_get_with` automatically:
    /// - Checks the cache first
    /// - Coalesces concurrent requests for the same key
    /// - Loads from DB if needed
    /// - Caches the result
    pub async fn resolve(&self, key: TenantKey) -> Result<Arc<TenantContext>, TenantError> {
        self.cache
            .try_get_with(key.clone(), async {
                self.load_from_db(&key).await
            })
            .await
            .map_err(|e| TenantError::LoadFailed(e.to_string()))
    }
    
    /// Load tenant from database (called by cache on miss)
    async fn load_from_db(&self, key: &TenantKey) -> Result<Arc<TenantContext>, TenantError> {
        tracing::debug!(key = ?key, "Loading tenant from database");
        
        let tenant = match key {
            TenantKey::Uuid(id) => {
                tenants::Entity::find_by_id(&self.db, *id)
                    .await?
            }
            TenantKey::Slug(slug) => {
                tenants::Entity::find_by_slug(&self.db, slug)
                    .await?
            }
            TenantKey::Host(host) => {
                tenants::Entity::find_by_domain(&self.db, host)
                    .await?
            }
        };
        
        tenant
            .ok_or(TenantError::NotFound)
            .map(|t| {
                tracing::debug!(tenant_id = %t.id, "Tenant loaded from database");
                Arc::new(TenantContext::from_model(&t))
            })
    }
    
    /// Invalidate a specific tenant by UUID
    pub async fn invalidate_by_uuid(&self, tenant_id: Uuid) {
        self.cache.invalidate(&TenantKey::Uuid(tenant_id)).await;
        tracing::debug!(tenant_id = %tenant_id, "Invalidated tenant cache by UUID");
    }
    
    /// Invalidate a specific tenant by slug
    pub async fn invalidate_by_slug(&self, slug: &str) {
        self.cache.invalidate(&TenantKey::Slug(slug.to_string())).await;
        tracing::debug!(slug = %slug, "Invalidated tenant cache by slug");
    }
    
    /// Invalidate a specific tenant by host
    pub async fn invalidate_by_host(&self, host: &str) {
        self.cache.invalidate(&TenantKey::Host(host.to_string())).await;
        tracing::debug!(host = %host, "Invalidated tenant cache by host");
    }
    
    /// Invalidate all cached tenants
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
        tracing::info!("Invalidated all tenant caches");
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> TenantCacheStats {
        TenantCacheStats {
            entries: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Configuration for tenant resolver cache
#[derive(Debug, Clone)]
pub struct TenantResolverConfig {
    /// Maximum number of tenants to cache (default: 1,000)
    pub max_capacity: u64,
    
    /// Time-to-live: how long an entry stays in cache (default: 5 minutes)
    pub time_to_live: Duration,
    
    /// Time-to-idle: evict if not accessed for this long (default: 1 minute)
    pub time_to_idle: Duration,
}

impl Default for TenantResolverConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1_000,
            time_to_live: Duration::from_secs(300),  // 5 minutes
            time_to_idle: Duration::from_secs(60),   // 1 minute
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct TenantCacheStats {
    /// Number of entries currently in cache
    pub entries: u64,
    
    /// Total weighted size of cached entries
    pub weighted_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{Database, DatabaseBackend, MockDatabase, MockExecResult};
    use sea_orm::entity::prelude::*;
    
    fn mock_tenant() -> tenants::Model {
        tenants::Model {
            id: Uuid::new_v4(),
            slug: "test-tenant".to_string(),
            name: "Test Tenant".to_string(),
            status: "active".to_string(),
            settings: serde_json::json!({"domain": "test.example.com"}),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }
    
    #[tokio::test]
    async fn test_tenant_resolver_caches_results() {
        let tenant = mock_tenant();
        let tenant_id = tenant.id;
        
        // Mock database that returns tenant once, then panics if called again
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![tenant.clone()]])
            .into_connection();
        
        let resolver = TenantResolver::new(db);
        
        // First call - should hit database
        let result1 = resolver.resolve(TenantKey::Uuid(tenant_id)).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap().id, tenant_id);
        
        // Second call - should hit cache (database mock would panic)
        let result2 = resolver.resolve(TenantKey::Uuid(tenant_id)).await;
        assert!(result2.is_ok());
    }
    
    #[tokio::test]
    async fn test_tenant_resolver_invalidation() {
        let tenant = mock_tenant();
        let tenant_id = tenant.id;
        
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![
                vec![tenant.clone()],
                vec![tenant.clone()],
            ])
            .into_connection();
        
        let resolver = TenantResolver::new(db);
        
        // Load into cache
        let _ = resolver.resolve(TenantKey::Uuid(tenant_id)).await;
        
        // Invalidate
        resolver.invalidate_by_uuid(tenant_id).await;
        
        // Should load from database again (second query result)
        let result = resolver.resolve(TenantKey::Uuid(tenant_id)).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_tenant_resolver_not_found() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results::<tenants::Model, Vec<tenants::Model>, _>(vec![
                vec![]  // Empty result
            ])
            .into_connection();
        
        let resolver = TenantResolver::new(db);
        
        let result = resolver.resolve(TenantKey::Uuid(Uuid::new_v4())).await;
        assert!(matches!(result, Err(TenantError::NotFound)));
    }
    
    #[tokio::test]
    async fn test_cache_stats() {
        let tenant = mock_tenant();
        
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![tenant.clone()]])
            .into_connection();
        
        let resolver = TenantResolver::new(db);
        
        // Initially empty
        let stats = resolver.stats();
        assert_eq!(stats.entries, 0);
        
        // Load one tenant
        let _ = resolver.resolve(TenantKey::Uuid(tenant.id)).await;
        
        // Should have one entry
        let stats = resolver.stats();
        assert_eq!(stats.entries, 1);
    }
}
