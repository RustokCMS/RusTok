/// Simplified Tenant Cache using Moka
///
/// This is a simplified version of the tenant caching system that uses moka's built-in
/// features instead of manual implementations:
/// - Automatic stampede protection via `try_get_with`
/// - Built-in TTL and eviction
/// - Thread-safe without manual locking
/// - Simpler codebase (~200 lines vs ~700 lines)
///
/// Trade-offs:
/// - Simpler implementation, easier to test and maintain
/// - Built-in stampede protection (request coalescing)
/// - Automatic TTL management
/// - Less granular metrics (moka provides basic stats)
/// - Local cache only (no Redis layer in v1)
use axum::{
    body::Body,
    extract::State,
    http::{
        header::{FORWARDED, HOST},
        Request, StatusCode,
    },
    middleware::Next,
    response::Response,
};
use loco_rs::app::AppContext;
use moka::future::Cache;
use rustok_core::tenant_validation::TenantIdentifierValidator;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::common::settings::RustokSettings;
use crate::context::{TenantContext, TenantContextExtension};
use crate::models::tenants;

/// Cache configuration
const TENANT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const TENANT_CACHE_IDLE: Duration = Duration::from_secs(180); // 3 minutes idle
const TENANT_CACHE_MAX_CAPACITY: u64 = 10_000; // Max 10k tenants in cache

/// Tenant identifier types for cache key generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TenantIdentifierKind {
    Uuid,
    Slug,
    Host,
}

impl TenantIdentifierKind {
    fn as_str(&self) -> &'static str {
        match self {
            TenantIdentifierKind::Uuid => "uuid",
            TenantIdentifierKind::Slug => "slug",
            TenantIdentifierKind::Host => "host",
        }
    }
}

/// Resolved tenant identifier with validation
#[derive(Debug, Clone)]
struct ResolvedTenantIdentifier {
    value: String,
    kind: TenantIdentifierKind,
    uuid: Uuid,
}

/// Cache entry - can be either a positive (found) or negative (not found) result
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CachedTenant {
    Found(TenantContext),
    NotFound, // Negative cache to prevent repeated DB lookups
}

/// Simplified tenant cache using moka
#[derive(Clone)]
pub struct SimplifiedTenantCache {
    /// Single cache for both positive and negative entries
    cache: Cache<String, Arc<CachedTenant>>,
    /// Database connection for loading tenants
    db: sea_orm::DatabaseConnection,
}

impl SimplifiedTenantCache {
    /// Create a new simplified tenant cache
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        let cache = Cache::builder()
            .max_capacity(TENANT_CACHE_MAX_CAPACITY)
            .time_to_live(TENANT_CACHE_TTL)
            .time_to_idle(TENANT_CACHE_IDLE)
            // Moka automatically handles request coalescing (stampede protection)
            .build();

        Self { cache, db }
    }

    /// Get or load a tenant by identifier
    ///
    /// This method automatically handles stampede protection via moka's `try_get_with`.
    /// Multiple concurrent requests for the same tenant will be coalesced into a single DB query.
    async fn get_or_load(
        &self,
        identifier: &ResolvedTenantIdentifier,
    ) -> Result<TenantContext, StatusCode> {
        let cache_key = self.build_cache_key(identifier);

        // Moka's try_get_with automatically handles stampede protection!
        // If multiple requests come in for the same key, only one will execute the loader
        let cached = self
            .cache
            .try_get_with(cache_key, async { self.load_from_db(identifier).await })
            .await
            .map_err(|e| {
                tracing::error!(
                    error = %e,
                    identifier_kind = ?identifier.kind,
                    identifier_value = %identifier.value,
                    "Failed to load tenant from database"
                );
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        match cached.as_ref() {
            CachedTenant::Found(context) => Ok(context.clone()),
            CachedTenant::NotFound => {
                tracing::debug!(
                    identifier_kind = ?identifier.kind,
                    identifier_value = %identifier.value,
                    "Tenant not found (cached negative result)"
                );
                Err(StatusCode::NOT_FOUND)
            }
        }
    }

    /// Load tenant from database
    async fn load_from_db(
        &self,
        identifier: &ResolvedTenantIdentifier,
    ) -> Result<Arc<CachedTenant>, sea_orm::DbErr> {
        tracing::debug!(
            identifier_kind = ?identifier.kind,
            identifier_value = %identifier.value,
            "Loading tenant from database"
        );

        let tenant = match identifier.kind {
            TenantIdentifierKind::Uuid => {
                tenants::Entity::find_by_id(&self.db, identifier.uuid).await?
            }
            TenantIdentifierKind::Slug => {
                tenants::Entity::find_by_slug(&self.db, &identifier.value).await?
            }
            TenantIdentifierKind::Host => {
                tenants::Entity::find_by_domain(&self.db, &identifier.value).await?
            }
        };

        let cached = match tenant {
            Some(tenant) => {
                tracing::info!(
                    tenant_id = %tenant.id,
                    tenant_slug = %tenant.slug,
                    identifier_kind = ?identifier.kind,
                    "Tenant loaded and cached"
                );
                CachedTenant::Found(TenantContext::from_model(&tenant))
            }
            None => {
                tracing::debug!(
                    identifier_kind = ?identifier.kind,
                    identifier_value = %identifier.value,
                    "Tenant not found, caching negative result"
                );
                CachedTenant::NotFound
            }
        };

        Ok(Arc::new(cached))
    }

    /// Build cache key from identifier
    fn build_cache_key(&self, identifier: &ResolvedTenantIdentifier) -> String {
        format!(
            "tenant_v2:{}:{}",
            identifier.kind.as_str(),
            match identifier.kind {
                TenantIdentifierKind::Host => identifier.value.to_lowercase(),
                _ => identifier.value.clone(),
            }
        )
    }

    /// Invalidate cached tenant
    async fn invalidate(&self, identifier: &ResolvedTenantIdentifier) {
        let cache_key = self.build_cache_key(identifier);
        self.cache.invalidate(&cache_key).await;

        tracing::info!(
            identifier_kind = ?identifier.kind,
            identifier_value = %identifier.value,
            "Tenant cache invalidated"
        );
    }

    /// Invalidate all cache entries
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
        tracing::info!("All tenant cache entries invalidated");
    }

    /// Get cache statistics
    pub fn stats(&self) -> SimplifiedTenantCacheStats {
        SimplifiedTenantCacheStats {
            entry_count: self.cache.entry_count(),
            weighted_size: self.cache.weighted_size(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct SimplifiedTenantCacheStats {
    pub entry_count: u64,
    pub weighted_size: u64,
}

/// Initialize simplified tenant cache
pub async fn init_simplified_tenant_cache(ctx: &AppContext) {
    if ctx.shared_store.contains::<Arc<SimplifiedTenantCache>>() {
        return;
    }

    let cache = Arc::new(SimplifiedTenantCache::new(ctx.db.clone()));
    ctx.shared_store.insert(cache);

    tracing::info!("Simplified tenant cache initialized");
}

/// Get tenant cache from context
fn tenant_cache(ctx: &AppContext) -> Option<Arc<SimplifiedTenantCache>> {
    ctx.shared_store.get::<Arc<SimplifiedTenantCache>>()
}

/// Tenant resolution middleware (simplified version)
pub async fn resolve_v2(
    State(ctx): State<AppContext>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let settings = RustokSettings::from_settings(&ctx.config.settings)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let identifier = resolve_identifier(&req, &settings)?;

    let Some(cache) = tenant_cache(&ctx) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let context = cache.get_or_load(&identifier).await?;

    req.extensions_mut().insert(TenantContextExtension(context));
    Ok(next.run(req).await)
}

/// Resolve tenant identifier from request
fn resolve_identifier(
    req: &Request<Body>,
    settings: &RustokSettings,
) -> Result<ResolvedTenantIdentifier, StatusCode> {
    if !settings.tenant.enabled {
        return Ok(ResolvedTenantIdentifier {
            value: settings.tenant.default_id.to_string(),
            kind: TenantIdentifierKind::Uuid,
            uuid: settings.tenant.default_id,
        });
    }

    match settings.tenant.resolution.as_str() {
        "header" => {
            let header_value = req
                .headers()
                .get(&settings.tenant.header_name)
                .and_then(|value| value.to_str().ok());

            let identifier = header_value
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| settings.tenant.default_id.to_string());

            classify_and_validate_identifier(&identifier).map_err(|e| {
                tracing::warn!(
                    identifier = %identifier,
                    error = %e,
                    "Invalid tenant identifier from header"
                );
                StatusCode::BAD_REQUEST
            })
        }
        "host" | "domain" | "subdomain" => {
            let host = extract_host(req.headers()).ok_or(StatusCode::BAD_REQUEST)?;
            let host_without_port = host.split(':').next().unwrap_or(host);

            let validated_host = TenantIdentifierValidator::validate_host(host_without_port)
                .map_err(|e| {
                    tracing::warn!(
                        host = %host_without_port,
                        error = %e,
                        "Invalid tenant hostname"
                    );
                    StatusCode::BAD_REQUEST
                })?;

            Ok(ResolvedTenantIdentifier {
                value: validated_host,
                kind: TenantIdentifierKind::Host,
                uuid: settings.tenant.default_id,
            })
        }
        _ => Ok(ResolvedTenantIdentifier {
            value: settings.tenant.default_id.to_string(),
            kind: TenantIdentifierKind::Uuid,
            uuid: settings.tenant.default_id,
        }),
    }
}

/// Extract host from headers
fn extract_host(headers: &axum::http::HeaderMap) -> Option<&str> {
    if let Some(host) = headers
        .get("x-forwarded-host")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
    {
        return Some(host.trim());
    }

    if let Some(forwarded) = headers.get(FORWARDED).and_then(|value| value.to_str().ok()) {
        if let Some(host) = parse_forwarded_host(forwarded) {
            return Some(host);
        }
    }

    headers.get(HOST).and_then(|value| value.to_str().ok())
}

/// Parse host from Forwarded header
fn parse_forwarded_host(forwarded: &str) -> Option<&str> {
    forwarded
        .split(',')
        .next()
        .and_then(|entry| {
            entry
                .split(';')
                .find(|part| part.trim_start().starts_with("host="))
        })
        .and_then(|part| part.trim_start().strip_prefix("host="))
        .map(|host| host.trim_matches('"').trim())
}

/// Classify and validate tenant identifier
fn classify_and_validate_identifier(
    value: &str,
) -> Result<ResolvedTenantIdentifier, rustok_core::tenant_validation::TenantValidationError> {
    // Try UUID first
    if let Ok(uuid) = TenantIdentifierValidator::validate_uuid(value) {
        return Ok(ResolvedTenantIdentifier {
            value: uuid.to_string(),
            kind: TenantIdentifierKind::Uuid,
            uuid,
        });
    }

    // Try slug with security validation
    let validated_slug = TenantIdentifierValidator::validate_slug(value)?;

    Ok(ResolvedTenantIdentifier {
        value: validated_slug,
        kind: TenantIdentifierKind::Slug,
        uuid: Uuid::nil(),
    })
}

/// Invalidate cached tenant (public API)
pub async fn invalidate_tenant_cache_v2(ctx: &AppContext, identifier_value: &str) {
    let Some(cache) = tenant_cache(ctx) else {
        return;
    };

    // Try to classify the identifier and invalidate
    if let Ok(uuid) = TenantIdentifierValidator::validate_uuid(identifier_value) {
        let identifier = ResolvedTenantIdentifier {
            value: uuid.to_string(),
            kind: TenantIdentifierKind::Uuid,
            uuid,
        };
        cache.invalidate(&identifier).await;
    } else if let Ok(slug) = TenantIdentifierValidator::validate_slug(identifier_value) {
        let identifier = ResolvedTenantIdentifier {
            value: slug,
            kind: TenantIdentifierKind::Slug,
            uuid: Uuid::nil(),
        };
        cache.invalidate(&identifier).await;
    }
}

/// Get cache statistics
pub fn tenant_cache_stats_v2(ctx: &AppContext) -> Option<SimplifiedTenantCacheStats> {
    tenant_cache(ctx).map(|cache| cache.stats())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use uuid::Uuid;

    #[test]
    fn test_identifier_kind_as_str() {
        assert_eq!(TenantIdentifierKind::Uuid.as_str(), "uuid");
        assert_eq!(TenantIdentifierKind::Slug.as_str(), "slug");
        assert_eq!(TenantIdentifierKind::Host.as_str(), "host");
    }

    #[test]
    fn test_parse_forwarded_host() {
        let header = "for=192.0.2.60;proto=https;host=tenant.example.com";
        assert_eq!(parse_forwarded_host(header), Some("tenant.example.com"));
    }

    #[test]
    fn test_extract_host_prefers_x_forwarded_host() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-host", "tenant-a.example.com".parse().unwrap());
        headers.insert(HOST, "fallback.example.com".parse().unwrap());

        assert_eq!(extract_host(&headers), Some("tenant-a.example.com"));
    }

    #[test]
    fn test_classify_and_validate_identifier_slug() {
        let result = classify_and_validate_identifier("demo-tenant").unwrap();
        assert_eq!(result.kind, TenantIdentifierKind::Slug);
        assert_eq!(result.value, "demo-tenant");
        assert_eq!(result.uuid, Uuid::nil());
    }

    #[test]
    fn test_classify_and_validate_identifier_uuid() {
        let raw = "00000000-0000-0000-0000-000000000001";
        let result = classify_and_validate_identifier(raw).unwrap();
        assert_eq!(result.kind, TenantIdentifierKind::Uuid);
        assert_eq!(result.value, raw);
        assert_eq!(result.uuid, Uuid::parse_str(raw).unwrap());
    }

    #[test]
    fn test_classify_and_validate_identifier_rejects_invalid_slug() {
        let result = classify_and_validate_identifier("../../etc/passwd");
        assert!(result.is_err());
    }
}
