use axum::{
    body::Body,
    extract::State,
    http::{header::HOST, Request, StatusCode},
    middleware::Next,
    response::Response,
};
use loco_rs::app::AppContext;
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

use crate::common::settings::RustokSettings;
use crate::context::{TenantContext, TenantContextExtension};
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
            .try_get_with(key.clone(), async { self.load_from_db(&key).await })
            .await
            .map_err(|e| TenantError::LoadFailed(e.to_string()))
    }

    /// Load tenant from database (called by cache on miss)
    async fn load_from_db(&self, key: &TenantKey) -> Result<Arc<TenantContext>, TenantError> {
        tracing::debug!(key = ?key, "Loading tenant from database");

        let tenant = match key {
            TenantKey::Uuid(id) => tenants::Entity::find_by_id(&self.db, *id).await?,
            TenantKey::Slug(slug) => tenants::Entity::find_by_slug(&self.db, slug).await?,
            TenantKey::Host(host) => tenants::Entity::find_by_domain(&self.db, host).await?,
        };

        tenant.ok_or(TenantError::NotFound).map(|t| {
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
        self.cache
            .invalidate(&TenantKey::Slug(slug.to_string()))
            .await;
        tracing::debug!(slug = %slug, "Invalidated tenant cache by slug");
    }

    /// Invalidate a specific tenant by host
    pub async fn invalidate_by_host(&self, host: &str) {
        self.cache
            .invalidate(&TenantKey::Host(host.to_string()))
            .await;
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
            time_to_live: Duration::from_secs(300), // 5 minutes
            time_to_idle: Duration::from_secs(60),  // 1 minute
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

/// Extract tenant identifier from request
fn resolve_identifier(
    req: &Request<Body>,
    settings: &RustokSettings,
) -> Result<TenantKey, StatusCode> {
    // Check for configured tenant header first (from settings)
    if let Ok(tenant_id) = req
        .headers()
        .get(&settings.tenant.header_name)
        .ok_or(())
        .and_then(|h| h.to_str().map_err(|_| ()))
        .and_then(|s| Uuid::parse_str(s).map_err(|_| ()))
    {
        return Ok(TenantKey::Uuid(tenant_id));
    }

    // Check for X-Tenant-ID header (standard header)
    if let Ok(tenant_id) = req
        .headers()
        .get("X-Tenant-ID")
        .ok_or(())
        .and_then(|h| h.to_str().map_err(|_| ()))
        .and_then(|s| Uuid::parse_str(s).map_err(|_| ()))
    {
        return Ok(TenantKey::Uuid(tenant_id));
    }

    // Check for X-Tenant-Slug header
    if let Ok(slug) = req
        .headers()
        .get("X-Tenant-Slug")
        .ok_or(())
        .and_then(|h| h.to_str().map_err(|_| ()))
    {
        if !slug.is_empty() {
            return Ok(TenantKey::Slug(slug.to_string()));
        }
    }

    // Fall back to Host header
    if let Ok(host) = req
        .headers()
        .get(HOST)
        .ok_or(())
        .and_then(|h| h.to_str().map_err(|_| ()))
    {
        if !host.is_empty() {
            return Ok(TenantKey::Host(host.to_string()));
        }
    }

    Err(StatusCode::BAD_REQUEST)
}

/// Global tenant resolver instance
static TENANT_RESOLVER: OnceLock<Arc<TenantResolver>> = OnceLock::new();

/// Initialize tenant resolver in app context
pub async fn init_tenant_resolver(ctx: &AppContext) {
    if TENANT_RESOLVER.get().is_none() {
        let resolver = Arc::new(TenantResolver::new(ctx.db.clone()));
        let _ = TENANT_RESOLVER.set(resolver);
    }
}

/// Get tenant resolver from global instance
fn get_resolver() -> Option<Arc<TenantResolver>> {
    TENANT_RESOLVER.get().cloned()
}

/// Axum middleware for tenant resolution using moka cache
///
/// This middleware resolves the tenant from the request and injects
/// it into the request extensions for downstream handlers.
///
/// # Example
/// ```rust
/// use axum::{Router, middleware::from_fn};
/// use rustok_server::middleware::tenant_v2;
///
/// let app = Router::new()
///     .layer(from_fn(tenant_v2::resolve));
/// ```
pub async fn resolve(
    State(ctx): State<AppContext>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let settings = RustokSettings::from_settings(&ctx.config.settings)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let key = resolve_identifier(&req, &settings)?;

    let resolver = get_resolver().unwrap_or_else(|| {
        // Fallback: create resolver on demand if not initialized
        Arc::new(TenantResolver::new(ctx.db.clone()))
    });

    match resolver.resolve(key).await {
        Ok(context) => {
            req.extensions_mut()
                .insert(TenantContextExtension((*context).clone()));
            Ok(next.run(req).await)
        }
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tenant_key_is_hashable_and_distinct() {
        let id = Uuid::new_v4();
        let by_id = TenantKey::Uuid(id);
        let by_slug = TenantKey::Slug("acme".to_string());
        let by_host = TenantKey::Host("acme.example.com".to_string());

        assert_ne!(by_id, by_slug);
        assert_ne!(by_slug, by_host);
    }

    #[test]
    fn default_config_is_sane() {
        let cfg = TenantResolverConfig::default();
        assert!(cfg.max_capacity >= 1_000);
        assert!(cfg.time_to_live.as_secs() >= cfg.time_to_idle.as_secs());
    }
}
