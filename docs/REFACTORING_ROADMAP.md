# RusToK — Refactoring Roadmap

> **Дата:** 2026-02-12  
> **Версия:** 1.0  
> **Связанный документ:** [ARCHITECTURE_REVIEW_2026-02-12.md](./ARCHITECTURE_REVIEW_2026-02-12.md)

Этот документ содержит пошаговый план рефакторинга с конкретными файлами и изменениями.

---

## Sprint 1: Critical Fixes (Week 1)

### Task 1.1: Event Validation Framework

**Создать новый файл:** `crates/rustok-core/src/events/validation.rs`

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventValidationError {
    #[error("Field '{0}' cannot be empty")]
    EmptyField(&'static str),
    
    #[error("Field '{0}' exceeds maximum length of {1}")]
    FieldTooLong(&'static str, usize),
    
    #[error("UUID field '{0}' cannot be nil")]
    NilUuid(&'static str),
    
    #[error("Invalid value for field '{0}': {1}")]
    InvalidValue(&'static str, String),
    
    #[error("Missing required field '{0}'")]
    MissingField(&'static str),
}

pub trait ValidateEvent {
    fn validate(&self) -> Result<(), EventValidationError>;
}
```

**Обновить:** `crates/rustok-core/src/events/types.rs`

```rust
use super::validation::{ValidateEvent, EventValidationError};

impl ValidateEvent for DomainEvent {
    fn validate(&self) -> Result<(), EventValidationError> {
        match self {
            Self::NodeCreated { node_id, kind, author_id } => {
                if node_id.is_nil() {
                    return Err(EventValidationError::NilUuid("node_id"));
                }
                if kind.is_empty() {
                    return Err(EventValidationError::EmptyField("kind"));
                }
                if kind.len() > 64 {
                    return Err(EventValidationError::FieldTooLong("kind", 64));
                }
                if let Some(id) = author_id {
                    if id.is_nil() {
                        return Err(EventValidationError::NilUuid("author_id"));
                    }
                }
                Ok(())
            }
            
            Self::ProductCreated { product_id } => {
                if product_id.is_nil() {
                    return Err(EventValidationError::NilUuid("product_id"));
                }
                Ok(())
            }
            
            Self::OrderPlaced { order_id, customer_id, total, currency } => {
                if order_id.is_nil() {
                    return Err(EventValidationError::NilUuid("order_id"));
                }
                if let Some(id) = customer_id {
                    if id.is_nil() {
                        return Err(EventValidationError::NilUuid("customer_id"));
                    }
                }
                if *total < 0 {
                    return Err(EventValidationError::InvalidValue(
                        "total",
                        "cannot be negative".to_string()
                    ));
                }
                if currency.len() != 3 {
                    return Err(EventValidationError::InvalidValue(
                        "currency",
                        "must be 3-letter ISO code".to_string()
                    ));
                }
                Ok(())
            }
            
            // Добавить валидацию для всех остальных вариантов...
            _ => Ok(()),  // Temporary: implement for all variants
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    
    #[test]
    fn test_node_created_validation() {
        let valid = DomainEvent::NodeCreated {
            node_id: Uuid::new_v4(),
            kind: "post".to_string(),
            author_id: Some(Uuid::new_v4()),
        };
        assert!(valid.validate().is_ok());
        
        let nil_id = DomainEvent::NodeCreated {
            node_id: Uuid::nil(),
            kind: "post".to_string(),
            author_id: None,
        };
        assert!(matches!(
            nil_id.validate(),
            Err(EventValidationError::NilUuid("node_id"))
        ));
        
        let empty_kind = DomainEvent::NodeCreated {
            node_id: Uuid::new_v4(),
            kind: "".to_string(),
            author_id: None,
        };
        assert!(matches!(
            empty_kind.validate(),
            Err(EventValidationError::EmptyField("kind"))
        ));
    }
}
```

**Обновить:** `crates/rustok-outbox/src/transactional.rs`

```rust
pub async fn publish_in_tx<C: ConnectionTrait>(
    &self,
    conn: &C,
    tenant_id: Uuid,
    actor_id: Option<Uuid>,
    event: DomainEvent,
) -> Result<(), Error> {
    // НОВОЕ: Валидация перед публикацией
    event.validate().map_err(|e| Error::EventValidation(e.to_string()))?;
    
    let envelope = EventEnvelope::new(
        generate_id(),
        tenant_id,
        actor_id,
        event,
        Utc::now(),
    );
    
    // Existing serialization logic...
}
```

---

### Task 1.2: Tenant Identifier Sanitization

**Создать новый файл:** `crates/rustok-core/src/tenant_validation.rs`

```rust
use regex::Regex;
use once_cell::sync::Lazy;
use thiserror::Error;

static VALID_SLUG_PATTERN: Lazy<Regex> = 
    Lazy::new(|| Regex::new(r"^[a-z0-9][a-z0-9-]{0,62}$").unwrap());

static VALID_UUID_PATTERN: Lazy<Regex> = 
    Lazy::new(|| Regex::new(
        r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$"
    ).unwrap());

const RESERVED_SLUGS: &[&str] = &[
    "api",
    "admin",
    "www",
    "app",
    "localhost",
    "cdn",
    "assets",
    "static",
    "health",
    "metrics",
];

#[derive(Debug, Error)]
pub enum TenantValidationError {
    #[error("Identifier is too long (max 64 characters)")]
    TooLong,
    
    #[error("Identifier contains invalid characters")]
    InvalidCharacters,
    
    #[error("Identifier '{0}' is reserved")]
    Reserved(String),
    
    #[error("Invalid UUID format")]
    InvalidUuid,
    
    #[error("Identifier cannot be empty")]
    Empty,
}

pub struct TenantIdentifierValidator;

impl TenantIdentifierValidator {
    pub fn validate_slug(raw: &str) -> Result<String, TenantValidationError> {
        let sanitized = raw.trim().to_lowercase();
        
        if sanitized.is_empty() {
            return Err(TenantValidationError::Empty);
        }
        
        if sanitized.len() > 64 {
            return Err(TenantValidationError::TooLong);
        }
        
        if !VALID_SLUG_PATTERN.is_match(&sanitized) {
            return Err(TenantValidationError::InvalidCharacters);
        }
        
        if RESERVED_SLUGS.contains(&sanitized.as_str()) {
            return Err(TenantValidationError::Reserved(sanitized));
        }
        
        Ok(sanitized)
    }
    
    pub fn validate_uuid(raw: &str) -> Result<uuid::Uuid, TenantValidationError> {
        let sanitized = raw.trim().to_lowercase();
        
        if !VALID_UUID_PATTERN.is_match(&sanitized) {
            return Err(TenantValidationError::InvalidUuid);
        }
        
        sanitized.parse::<uuid::Uuid>()
            .map_err(|_| TenantValidationError::InvalidUuid)
    }
    
    pub fn validate_host(raw: &str) -> Result<String, TenantValidationError> {
        let sanitized = raw.trim().to_lowercase();
        
        if sanitized.is_empty() {
            return Err(TenantValidationError::Empty);
        }
        
        // Basic hostname validation (simplified)
        if sanitized.len() > 253 {
            return Err(TenantValidationError::TooLong);
        }
        
        // Check for invalid characters in hostname
        if !sanitized.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
            return Err(TenantValidationError::InvalidCharacters);
        }
        
        Ok(sanitized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_slug_validation() {
        assert_eq!(
            TenantIdentifierValidator::validate_slug("my-tenant"),
            Ok("my-tenant".to_string())
        );
        
        assert_eq!(
            TenantIdentifierValidator::validate_slug("  MyTenant123  "),
            Ok("mytenant123".to_string())
        );
        
        assert!(TenantIdentifierValidator::validate_slug("").is_err());
        assert!(TenantIdentifierValidator::validate_slug("admin").is_err());
        assert!(TenantIdentifierValidator::validate_slug("tenant@123").is_err());
        
        let long = "a".repeat(65);
        assert!(TenantIdentifierValidator::validate_slug(&long).is_err());
    }
}
```

**Обновить:** `apps/server/src/middleware/tenant.rs`

```rust
use rustok_core::tenant_validation::TenantIdentifierValidator;

fn extract_tenant_identifier(
    req: &Request<Body>,
    settings: &TenantSettings,
) -> Result<String, TenantError> {
    let raw = match settings.resolution.as_str() {
        "subdomain" => {
            let host = req.headers()
                .get(HOST)
                .ok_or(TenantError::MissingHost)?
                .to_str()
                .map_err(|_| TenantError::InvalidHost)?;
            
            host.split('.')
                .next()
                .ok_or(TenantError::InvalidHost)?
                .to_string()
        }
        "header" => {
            req.headers()
                .get(&settings.header_name)
                .ok_or(TenantError::MissingHeader)?
                .to_str()
                .map_err(|_| TenantError::InvalidHeader)?
                .to_string()
        }
        "path" => {
            req.uri()
                .path()
                .split('/')
                .nth(1)
                .ok_or(TenantError::InvalidPath)?
                .to_string()
        }
        _ => return Err(TenantError::InvalidResolution),
    };
    
    // НОВОЕ: Валидация и санитизация
    match settings.resolution.as_str() {
        "subdomain" | "header" | "path" => {
            TenantIdentifierValidator::validate_slug(&raw)
                .map_err(|e| TenantError::InvalidIdentifier(e.to_string()))
        }
        _ => Ok(raw),
    }
}
```

---

### Task 1.3: EventDispatcher Rate Limiting

**Создать новый файл:** `crates/rustok-core/src/events/backpressure.rs`

```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackpressureError {
    #[error("Event queue is full (capacity: {capacity}, current: {current})")]
    QueueFull { capacity: usize, current: usize },
    
    #[error("Event dispatcher is shutting down")]
    ShuttingDown,
}

pub struct BackpressureConfig {
    pub max_queue_depth: usize,
    pub high_watermark: f32,  // 0.8 = 80% capacity
    pub low_watermark: f32,   // 0.5 = 50% capacity
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_queue_depth: 10_000,
            high_watermark: 0.8,
            low_watermark: 0.5,
        }
    }
}

#[derive(Clone)]
pub struct BackpressureController {
    queue_depth: Arc<AtomicUsize>,
    config: BackpressureConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackpressureState {
    Normal,
    Warning,
    Critical,
}

impl BackpressureController {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            queue_depth: Arc::new(AtomicUsize::new(0)),
            config,
        }
    }
    
    pub fn increment(&self) {
        self.queue_depth.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn decrement(&self) {
        self.queue_depth.fetch_sub(1, Ordering::Relaxed);
    }
    
    pub fn current_depth(&self) -> usize {
        self.queue_depth.load(Ordering::Relaxed)
    }
    
    pub fn state(&self) -> BackpressureState {
        let depth = self.current_depth();
        let high_threshold = (self.config.max_queue_depth as f32 
                              * self.config.high_watermark) as usize;
        let low_threshold = (self.config.max_queue_depth as f32 
                             * self.config.low_watermark) as usize;
        
        if depth >= high_threshold {
            BackpressureState::Critical
        } else if depth >= low_threshold {
            BackpressureState::Warning
        } else {
            BackpressureState::Normal
        }
    }
    
    pub fn check_capacity(&self) -> Result<(), BackpressureError> {
        let depth = self.current_depth();
        
        if depth >= self.config.max_queue_depth {
            return Err(BackpressureError::QueueFull {
                capacity: self.config.max_queue_depth,
                current: depth,
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_backpressure_states() {
        let config = BackpressureConfig {
            max_queue_depth: 100,
            high_watermark: 0.8,
            low_watermark: 0.5,
        };
        let controller = BackpressureController::new(config);
        
        // Normal state
        for _ in 0..40 {
            controller.increment();
        }
        assert_eq!(controller.state(), BackpressureState::Normal);
        
        // Warning state
        for _ in 0..20 {
            controller.increment();
        }
        assert_eq!(controller.state(), BackpressureState::Warning);
        
        // Critical state
        for _ in 0..25 {
            controller.increment();
        }
        assert_eq!(controller.state(), BackpressureState::Critical);
        
        // Check capacity
        for _ in 0..15 {
            controller.increment();
        }
        assert!(controller.check_capacity().is_err());
    }
}
```

**Обновить:** `crates/rustok-core/src/events/handler.rs`

```rust
use super::backpressure::{BackpressureConfig, BackpressureController, BackpressureError};

pub struct DispatcherConfig {
    pub fail_fast: bool,
    pub max_concurrent: usize,
    pub retry_count: usize,
    pub retry_delay_ms: u64,
    pub backpressure: BackpressureConfig,
}

impl Default for DispatcherConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            max_concurrent: 100,
            retry_count: 3,
            retry_delay_ms: 1000,
            backpressure: BackpressureConfig::default(),
        }
    }
}

pub struct EventDispatcher {
    handlers: Vec<Arc<dyn EventHandler>>,
    config: DispatcherConfig,
    backpressure: BackpressureController,
}

impl EventDispatcher {
    pub fn new(config: DispatcherConfig) -> Self {
        let backpressure = BackpressureController::new(config.backpressure.clone());
        
        Self {
            handlers: Vec::new(),
            config,
            backpressure,
        }
    }
    
    async fn dispatch_with_backpressure(
        &self,
        envelope: EventEnvelope,
    ) -> Result<(), HandlerError> {
        // Проверка backpressure
        self.backpressure.check_capacity()
            .map_err(|e| HandlerError::Backpressure(e.to_string()))?;
        
        // Логирование при критическом состоянии
        match self.backpressure.state() {
            BackpressureState::Critical => {
                tracing::error!(
                    queue_depth = self.backpressure.current_depth(),
                    "Event dispatcher in critical state, rejecting new events"
                );
            }
            BackpressureState::Warning => {
                tracing::warn!(
                    queue_depth = self.backpressure.current_depth(),
                    "Event dispatcher approaching capacity"
                );
            }
            _ => {}
        }
        
        // Increment счётчика
        self.backpressure.increment();
        
        // Dispatch event
        let result = self.dispatch_internal(envelope).await;
        
        // Decrement счётчика
        self.backpressure.decrement();
        
        result
    }
}
```

---

## Sprint 2: Simplification (Week 2-3)

### Task 2.1: Simplified Tenant Resolver with Moka

**Создать новый файл:** `apps/server/src/middleware/tenant_v2.rs`

```rust
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};

use crate::models::tenants;
use crate::context::TenantContext;

#[derive(Hash, Eq, PartialEq, Clone)]
pub enum TenantKey {
    Uuid(Uuid),
    Slug(String),
    Host(String),
}

pub struct TenantResolver {
    cache: Cache<TenantKey, Arc<TenantContext>>,
    db: DatabaseConnection,
}

impl TenantResolver {
    pub fn new(db: DatabaseConnection) -> Self {
        let cache = Cache::builder()
            .max_capacity(1_000)
            .time_to_live(Duration::from_secs(300))
            .time_to_idle(Duration::from_secs(60))
            .build();
        
        Self { cache, db }
    }
    
    pub async fn resolve(&self, key: TenantKey) -> Result<Arc<TenantContext>, TenantError> {
        // moka automatically handles cache stampede!
        self.cache
            .try_get_with(key.clone(), async {
                self.load_from_db(&key).await
            })
            .await
            .map_err(|e| TenantError::LoadFailed(e.to_string()))
    }
    
    async fn load_from_db(&self, key: &TenantKey) -> Result<Arc<TenantContext>, TenantError> {
        let tenant = match key {
            TenantKey::Uuid(id) => {
                tenants::Entity::find_by_id(*id)
                    .one(&self.db)
                    .await?
            }
            TenantKey::Slug(slug) => {
                tenants::Entity::find()
                    .filter(tenants::Column::Slug.eq(slug))
                    .one(&self.db)
                    .await?
            }
            TenantKey::Host(host) => {
                // Custom host resolution logic
                tenants::Entity::find()
                    .filter(tenants::Column::Settings.contains(&format!(r#"{{"domain":"{}"}}"#, host)))
                    .one(&self.db)
                    .await?
            }
        };
        
        tenant
            .ok_or(TenantError::NotFound)
            .map(|t| Arc::new(TenantContext::from_model(t)))
    }
    
    pub async fn invalidate(&self, tenant_id: Uuid) {
        self.cache.invalidate(&TenantKey::Uuid(tenant_id)).await;
    }
    
    pub async fn invalidate_all(&self) {
        self.cache.invalidate_all();
    }
}
```

---

### Task 2.2: Circuit Breaker для Redis

**Создать новый файл:** `crates/rustok-core/src/circuit_breaker.rs`

```rust
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    Open,
    
    #[error("Upstream error: {0}")]
    Upstream(E),
}

#[derive(Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub timeout: Duration,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

pub struct CircuitBreaker {
    state: Arc<AtomicU32>,
    failure_count: Arc<AtomicU32>,
    success_count: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(AtomicU32::new(State::Closed as u32)),
            failure_count: Arc::new(AtomicU32::new(0)),
            success_count: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            config,
        }
    }
    
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
            State::HalfOpen => self.execute_half_open(f).await,
        }
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
                self.trip();
                Err(CircuitBreakerError::Upstream(e))
            }
        }
    }
    
    fn get_state(&self) -> State {
        match self.state.load(Ordering::Acquire) {
            0 => State::Closed,
            1 => State::Open,
            2 => State::HalfOpen,
            _ => unreachable!(),
        }
    }
    
    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Release);
    }
    
    async fn on_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if failures >= self.config.failure_threshold {
            self.trip();
            *self.last_failure_time.lock().await = Some(Instant::now());
        }
    }
    
    fn on_half_open_success(&self) {
        let successes = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;
        
        if successes >= self.config.success_threshold {
            self.reset();
        }
    }
    
    fn trip(&self) {
        tracing::warn!("Circuit breaker tripped to OPEN state");
        self.state.store(State::Open as u32, Ordering::Release);
    }
    
    fn reset(&self) {
        tracing::info!("Circuit breaker reset to CLOSED state");
        self.state.store(State::Closed as u32, Ordering::Release);
        self.failure_count.store(0, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
    }
    
    fn transition_to_half_open(&self) {
        tracing::info!("Circuit breaker transitioning to HALF_OPEN state");
        self.state.store(State::HalfOpen as u32, Ordering::Release);
        self.success_count.store(0, Ordering::Release);
    }
    
    async fn should_attempt_reset(&self) -> bool {
        if let Some(last_failure) = *self.last_failure_time.lock().await {
            last_failure.elapsed() >= self.config.timeout
        } else {
            false
        }
    }
}
```

---

## Sprint 3: Observability & Testing (Week 4)

### Task 3.1: OpenTelemetry Integration

**Обновить:** `crates/rustok-telemetry/src/tracing.rs`

```rust
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_telemetry(service_name: &str, otlp_endpoint: Option<String>) -> anyhow::Result<()> {
    let tracer = if let Some(endpoint) = otlp_endpoint {
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(1.0))))
                    .with_id_generator(RandomIdGenerator::default())
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", service_name.to_string()),
                    ]))
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?
    } else {
        // Fallback to stdout/noop
        opentelemetry_sdk::trace::TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .build()
    };
    
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer.tracer(service_name));
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .with(telemetry)
        .init();
    
    Ok(())
}
```

---

### Task 3.2: Integration Tests для Tenant Isolation

**Создать:** `apps/server/tests/integration/tenant_isolation_test.rs`

```rust
use rustok_server::*;
use uuid::Uuid;

#[tokio::test]
async fn test_tenant_cannot_access_other_tenant_products() {
    let app = setup_test_app().await;
    
    // Setup
    let tenant1 = create_test_tenant(&app, "tenant1").await;
    let tenant2 = create_test_tenant(&app, "tenant2").await;
    
    let product1 = create_product(&app, tenant1.id, "Product 1").await;
    
    // Attempt to access product1 with tenant2 credentials
    let response = app
        .get(format!("/api/v1/products/{}", product1.id))
        .header("X-Tenant-Id", tenant2.id.to_string())
        .send()
        .await;
    
    assert_eq!(response.status(), 404);
    assert!(response.json::<ErrorResponse>().await.unwrap().message.contains("not found"));
}

#[tokio::test]
async fn test_tenant_cannot_list_other_tenant_products() {
    let app = setup_test_app().await;
    
    let tenant1 = create_test_tenant(&app, "tenant1").await;
    let tenant2 = create_test_tenant(&app, "tenant2").await;
    
    create_product(&app, tenant1.id, "Product 1").await;
    create_product(&app, tenant1.id, "Product 2").await;
    create_product(&app, tenant2.id, "Product 3").await;
    
    // List products as tenant2
    let response = app
        .get("/api/v1/products")
        .header("X-Tenant-Id", tenant2.id.to_string())
        .send()
        .await;
    
    let products: Vec<Product> = response.json().await.unwrap();
    assert_eq!(products.len(), 1);
    assert_eq!(products[0].title, "Product 3");
}
```

---

## Метрики прогресса

После каждого спринта замерять:

1. **Code Metrics:**
   - Lines of code (должен уменьшаться)
   - Cyclomatic complexity (должен уменьшаться)
   - Test coverage (должен расти)

2. **Performance Metrics:**
   - P95 latency для tenant resolution
   - Event dispatch throughput
   - Memory usage

3. **Quality Metrics:**
   - Number of TODO/FIXME comments
   - Clippy warnings
   - Security audit findings

---

## Чеклист завершения

- [ ] Все P0 issues закрыты
- [ ] Test coverage > 40%
- [ ] Zero clippy warnings
- [ ] Security audit пройден
- [ ] Performance benchmarks > baseline
- [ ] Documentation updated
- [ ] Migration guide написан

---

*Этот roadmap является живым документом и должен обновляться после каждого спринта.*
