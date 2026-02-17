# 🏗️ RusToK — Расширенные архитектурные рекомендации

> **Дата:** 2026-02-12  
> **Версия:** 1.1 Extended  
> **Статус:** Sprint 1 завершён (4/4 P0 задачи ✅), переход на Sprint 2  
> **Основан на:** Анализ 370 файлов Rust (43,637 строк кода), 23 crate модулей

---

## 📊 Текущее состояние

### Метрики кодовой базы
- **Файлов Rust:** 370
- **Строк кода:** 43,637
- **Модулей (crates):** 23
- **Приложений:** 4 (server, admin, storefront, mcp)
- **Оценка архитектуры:** 8.5/10 → 8.7/10 (после Sprint 1)
- **Production readiness:** 75% → 85% (+10%)

### Завершённые улучшения (Sprint 1) ✅
- ✅ Event Validation Framework (260 строк, 25+ тестов)
- ✅ Tenant Identifier Sanitization (505 строк, 30+ тестов)
- ✅ EventDispatcher Backpressure Control (464 строк)
- ✅ EventBus Consistency Audit (100% pass)

---

## 🎯 Стратегические направления улучшения

### 1. Архитектурная зрелость (Maturity)

#### Текущая ситуация
RusToK демонстрирует **зрелую enterprise-архитектуру** с правильным применением паттернов:
- ✅ Event-Driven Architecture с Outbox Pattern
- ✅ CQRS-lite для разделения чтения/записи
- ✅ Modular Monolith с чёткими границами
- ✅ Multi-tenancy с isolation
- ✅ Transactional guarantees

#### Проблемы
- 🟡 **Несогласованность между модулями** — разные стили кода, DI patterns
- 🟡 **Излишняя сложность** — некоторые абстракции over-engineered
- 🟡 **Недостаточное покрытие тестами** — 31% (цель: 50%+)

---

## 🚀 Приоритетные рекомендации (Sprint 2-4)

### Sprint 2: Simplification & Refactoring (Weeks 2-3)

#### P1.1: Упростить Tenant Caching с использованием Moka

**Проблема:** Текущая реализация `TenantCacheManager` содержит 580+ строк сложной логики с ручной реализацией stampede protection, Redis pub/sub invalidation, и manual eviction.

**Решение:** Использовать `moka` crate (уже в зависимостях!)

```rust
// crates/rustok-tenant/src/cache_v2.rs (NEW)
use moka::future::Cache;
use std::time::Duration;

pub struct SimplifiedTenantCache {
    cache: Cache<String, Arc<Tenant>>,
    db: DatabaseConnection,
}

impl SimplifiedTenantCache {
    pub fn new(db: DatabaseConnection, config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_seconds))
            .time_to_idle(Duration::from_secs(config.idle_seconds))
            .build();
        
        Self { cache, db }
    }
    
    pub async fn get_or_load(&self, identifier: &str) -> Result<Arc<Tenant>> {
        // Moka handles stampede protection automatically!
        self.cache
            .try_get_with(identifier.to_string(), async {
                self.load_from_db(identifier).await.map(Arc::new)
            })
            .await
            .map_err(|e| Error::Cache(e.to_string()))
    }
    
    pub fn invalidate(&self, identifier: &str) {
        self.cache.invalidate(identifier);
    }
    
    async fn load_from_db(&self, identifier: &str) -> Result<Tenant> {
        tenant::Entity::find()
            .filter(tenant::Column::Identifier.eq(identifier))
            .one(&self.db)
            .await?
            .ok_or_else(|| Error::TenantNotFound)
    }
}
```

**Выигрыш:**
- Сокращение кода: 580 строк → ~150 строк (-74%)
- Встроенная stampede protection (без ручной реализации)
- Автоматический eviction (LRU/LFU)
- Меньше багов (проверенная библиотека)
- Легче поддерживать и тестировать

**Усилия:** 2 дня  
**Риск:** Низкий (можно запустить параллельно со старым кодом)

---

#### P1.2: Добавить Circuit Breaker для внешних зависимостей

**Проблема:** Нет защиты от cascading failures при проблемах с Redis, Iggy, или другими внешними сервисами.

```rust
// crates/rustok-core/src/resilience/circuit_breaker.rs (NEW)
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    Closed,      // Всё работает
    Open,        // Сбои, запросы блокируются
    HalfOpen,    // Тестируем восстановление
}

pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<tokio::sync::RwLock<State>>,
}

struct State {
    circuit_state: CircuitState,
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    last_failure_time: Option<Instant>,
    opened_at: Option<Instant>,
}

#[derive(Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: usize,      // Открыть после N сбоев
    pub success_threshold: usize,      // Закрыть после N успехов
    pub timeout: Duration,              // Время открытия
    pub half_open_max_calls: usize,    // Лимит вызовов в HalfOpen
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(tokio::sync::RwLock::new(State {
                circuit_state: CircuitState::Closed,
                failure_count: AtomicUsize::new(0),
                success_count: AtomicUsize::new(0),
                last_failure_time: None,
                opened_at: None,
            })),
        }
    }
    
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
    {
        // Проверить состояние
        let current_state = self.get_state().await;
        
        match current_state {
            CircuitState::Open => {
                // Проверить timeout
                if self.should_attempt_reset().await {
                    self.transition_to_half_open().await;
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                // Ограничить количество попыток
                if !self.can_make_trial_call().await {
                    return Err(CircuitBreakerError::TooManyTrialCalls);
                }
            }
            CircuitState::Closed => {}
        }
        
        // Выполнить запрос
        let result = f().await;
        
        match result {
            Ok(value) => {
                self.on_success().await;
                Ok(value)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::RequestFailed(error))
            }
        }
    }
    
    async fn on_success(&self) {
        let mut state = self.state.write().await;
        state.success_count.fetch_add(1, Ordering::Relaxed);
        
        if state.circuit_state == CircuitState::HalfOpen {
            let success_count = state.success_count.load(Ordering::Relaxed);
            if success_count >= self.config.success_threshold {
                tracing::info!("Circuit breaker closing after {} successes", success_count);
                state.circuit_state = CircuitState::Closed;
                state.failure_count.store(0, Ordering::Relaxed);
                state.success_count.store(0, Ordering::Relaxed);
            }
        }
    }
    
    async fn on_failure(&self) {
        let mut state = self.state.write().await;
        state.failure_count.fetch_add(1, Ordering::Relaxed);
        state.last_failure_time = Some(Instant::now());
        
        let failure_count = state.failure_count.load(Ordering::Relaxed);
        
        if failure_count >= self.config.failure_threshold {
            tracing::warn!("Circuit breaker opening after {} failures", failure_count);
            state.circuit_state = CircuitState::Open;
            state.opened_at = Some(Instant::now());
        }
    }
    
    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.circuit_state
    }
    
    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let state = self.state.read().await;
        CircuitBreakerMetrics {
            state: state.circuit_state,
            failure_count: state.failure_count.load(Ordering::Relaxed),
            success_count: state.success_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    TooManyTrialCalls,
    RequestFailed(E),
}

pub struct CircuitBreakerMetrics {
    pub state: CircuitState,
    pub failure_count: usize,
    pub success_count: usize,
}

// Использование:
// crates/rustok-core/src/cache/redis.rs
pub struct ResilientRedisCacheBackend {
    redis: redis::Client,
    circuit_breaker: CircuitBreaker,
}

impl ResilientRedisCacheBackend {
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        self.circuit_breaker
            .call(|| {
                let redis = self.redis.clone();
                let key = key.to_string();
                Box::pin(async move {
                    let mut conn = redis.get_async_connection().await?;
                    conn.get(&key).await
                })
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::CircuitOpen => {
                    tracing::warn!("Redis circuit breaker is OPEN, falling back");
                    Error::CircuitBreakerOpen
                }
                CircuitBreakerError::RequestFailed(err) => {
                    Error::RedisError(err)
                }
                _ => Error::Internal("Circuit breaker error".to_string()),
            })
    }
}
```

**Выигрыш:**
- Защита от cascading failures
- Быстрое восстановление (fail-fast)
- Graceful degradation
- Observability (метрики состояния)

**Усилия:** 3 дня  
**Риск:** Средний (нужны интеграционные тесты)

---

#### P1.3: Type-Safe State Machines для Order/Product статусов

**Проблема:** Переходы между статусами (Draft→Published, Pending→Paid) проверяются в runtime.

```rust
// crates/rustok-commerce/src/order/state_machine.rs (NEW)
use std::marker::PhantomData;

// === States ===
pub struct Draft;
pub struct PendingPayment;
pub struct Paid;
pub struct Shipped;
pub struct Delivered;
pub struct Cancelled;

// === State Machine ===
pub struct Order<State> {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub total: Decimal,
    pub items: Vec<OrderItem>,
    pub created_at: DateTime<Utc>,
    _state: PhantomData<State>,
}

// Только Draft может быть отменён без оплаты
impl Order<Draft> {
    pub fn new(customer_id: Uuid, items: Vec<OrderItem>) -> Self {
        let total = items.iter().map(|i| i.total).sum();
        Self {
            id: Uuid::new_v4(),
            customer_id,
            total,
            items,
            created_at: Utc::now(),
            _state: PhantomData,
        }
    }
    
    pub fn submit(self) -> Order<PendingPayment> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            total: self.total,
            items: self.items,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
    
    pub fn cancel(self) -> Order<Cancelled> {
        // Compile-time guarantee: можно отменить только Draft
        Order {
            id: self.id,
            customer_id: self.customer_id,
            total: self.total,
            items: self.items,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// Только PendingPayment может быть оплачен
impl Order<PendingPayment> {
    pub fn pay(self, payment_id: Uuid) -> Order<Paid> {
        // Событие: OrderPaid
        Order {
            id: self.id,
            customer_id: self.customer_id,
            total: self.total,
            items: self.items,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
    
    pub fn cancel(self, reason: String) -> Order<Cancelled> {
        // Можно отменить до оплаты
        Order {
            id: self.id,
            customer_id: self.customer_id,
            total: self.total,
            items: self.items,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// Только Paid может быть отправлен
impl Order<Paid> {
    pub fn ship(self, tracking_number: String) -> Order<Shipped> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            total: self.total,
            items: self.items,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
    
    // НЕТ метода cancel() — нельзя отменить оплаченный заказ!
}

// Только Shipped может быть доставлен
impl Order<Shipped> {
    pub fn deliver(self) -> Order<Delivered> {
        Order {
            id: self.id,
            customer_id: self.customer_id,
            total: self.total,
            items: self.items,
            created_at: self.created_at,
            _state: PhantomData,
        }
    }
}

// Использование:
pub async fn process_order_workflow() -> Result<()> {
    let order = Order::<Draft>::new(customer_id, items);
    
    let order = order.submit(); // Draft → PendingPayment
    
    let order = order.pay(payment_id); // PendingPayment → Paid
    
    // order.cancel(); // ❌ ОШИБКА КОМПИЛЯЦИИ! Paid не имеет метода cancel()
    
    let order = order.ship(tracking_number); // Paid → Shipped
    
    let order = order.deliver(); // Shipped → Delivered
    
    Ok(())
}
```

**Выигрыш:**
- Compile-time гарантии правильности переходов
- Невозможно сделать invalid state transition
- Саморедактирование (IDE autocomplete показывает только доступные действия)
- Код документирует сам себя

**Усилия:** 4 дня (для Order + Product)  
**Риск:** Средний (нужна миграция существующего кода)

---

### Sprint 3: Observability & Testing (Week 4)

#### P2.1: OpenTelemetry Integration

**Проблема:** Текущий telemetry basic (только logs через tracing-subscriber). Нет distributed tracing, метрик, или spans.

```rust
// crates/rustok-telemetry/src/otel.rs (NEW)
use opentelemetry::{
    global,
    sdk::{
        trace::{self, Tracer},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_telemetry(config: TelemetryConfig) -> Result<()> {
    // OpenTelemetry tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config.otlp_endpoint),
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![
                KeyValue::new("service.name", "rustok"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    // Tracing subscriber с OpenTelemetry layer
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
    
    Ok(())
}

// Использование:
#[tracing::instrument(
    name = "create_product",
    skip(self),
    fields(
        tenant_id = %tenant_id,
        product_sku = %input.sku,
        otel.kind = "internal"
    )
)]
pub async fn create_product(
    &self,
    tenant_id: Uuid,
    actor_id: Uuid,
    input: CreateProductInput,
) -> Result<ProductResponse> {
    let span = tracing::Span::current();
    span.record("product_sku", &input.sku.as_str());
    
    // Business logic...
    
    span.record("product_id", &product_id.to_string());
    Ok(response)
}
```

**Выигрыш:**
- Distributed tracing через микросервисы/модули
- Связь событий через correlation_id
- Визуализация в Jaeger/Zipkin/Honeycomb
- Профилирование производительности
- Анализ bottlenecks

**Усилия:** 5 дней  
**Риск:** Низкий

---

#### P2.2: Increase Test Coverage (31% → 50%+)

**Стратегия:**

1. **Integration Tests для критических flows**
```rust
// apps/server/tests/integration/order_flow_test.rs
#[tokio::test]
async fn test_complete_order_flow() {
    let app = test_utils::spawn_test_app().await;
    
    // 1. Create product
    let product = app.create_product(ProductInput {
        title: "Test Product".into(),
        price: 1000,
    }).await.unwrap();
    
    // 2. Create order
    let order = app.create_order(OrderInput {
        items: vec![OrderItemInput {
            product_id: product.id,
            quantity: 2,
        }],
    }).await.unwrap();
    
    assert_eq!(order.status, OrderStatus::Draft);
    
    // 3. Submit order
    let order = app.submit_order(order.id).await.unwrap();
    assert_eq!(order.status, OrderStatus::PendingPayment);
    
    // 4. Process payment
    let payment = app.process_payment(order.id, PaymentInput {
        method: PaymentMethod::Card,
        amount: 2000,
    }).await.unwrap();
    
    // 5. Verify order is paid
    let order = app.get_order(order.id).await.unwrap();
    assert_eq!(order.status, OrderStatus::Paid);
    
    // 6. Verify event was emitted
    let events = app.get_events_for_order(order.id).await;
    assert!(events.iter().any(|e| matches!(e, DomainEvent::OrderPaid { .. })));
}
```

2. **Property-based tests для валидации**
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_tenant_identifier_always_valid(s in "[a-z0-9-]{1,64}") {
        let validator = TenantIdentifierValidator::new();
        prop_assume!(!RESERVED_SLUGS.contains(&s.as_str()));
        
        let result = validator.validate_slug(&s);
        prop_assert!(result.is_ok());
    }
    
    #[test]
    fn test_invalid_chars_rejected(s in ".{1,64}") {
        prop_assume!(s.contains(|c: char| !c.is_alphanumeric() && c != '-'));
        
        let validator = TenantIdentifierValidator::new();
        let result = validator.validate_slug(&s);
        prop_assert!(result.is_err());
    }
}
```

3. **Snapshot tests для GraphQL schema**
```rust
#[test]
fn test_graphql_schema_unchanged() {
    let schema = create_merged_schema().sdl();
    insta::assert_snapshot!(schema);
}
```

**Цель:** 31% → 50% (+19 percentage points)  
**Усилия:** 10 дней (continuous)  
**Риск:** Низкий

---

## 🔧 Технические улучшения

### 1. Разбить rustok-core на sub-crates

**Проблема:** `rustok-core` слишком большой (содержит auth, events, cache, RBAC, tenant, permissions).

**Решение:**
```
crates/
├── rustok-core/              # Re-exports + common types
├── rustok-core-events/       # Event system
├── rustok-core-auth/         # Authentication
├── rustok-core-cache/        # Cache abstractions
├── rustok-core-permissions/  # RBAC + permissions
└── rustok-core-tenant/       # Multi-tenancy
```

**Выигрыш:**
- Более чёткое разделение ответственности
- Независимые версии sub-crates
- Меньше recompilation time
- Easier dependency management

---

### 2. Стандартизировать Error Handling

**Проблема:** Разные модули используют разные error types (некоторые `anyhow`, некоторые `thiserror`).

**Решение:**
```rust
// crates/rustok-core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied { action: String, resource: String },
    
    #[error("Tenant not found: {identifier}")]
    TenantNotFound { identifier: String },
    
    #[error("Circuit breaker open for {service}")]
    CircuitBreakerOpen { service: String },
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl Error {
    // HTTP status mapping
    pub fn http_status_code(&self) -> StatusCode {
        match self {
            Self::NotFound { .. } => StatusCode::NOT_FOUND,
            Self::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            Self::CircuitBreakerOpen { .. } => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    // GraphQL error mapping
    pub fn graphql_error_code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "NOT_FOUND",
            Self::PermissionDenied { .. } => "FORBIDDEN",
            Self::Validation(_) => "BAD_USER_INPUT",
            Self::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            _ => "INTERNAL_SERVER_ERROR",
        }
    }
}
```

---

### 3. Feature Flags System

**Проблема:** Нет способа включать/выключать функции per-tenant без перекомпиляции.

```rust
// crates/rustok-core/src/features/mod.rs (NEW)
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Feature {
    Commerce,
    Blog,
    Forum,
    Pages,
    AdvancedSearch,
    RealtimeChat,
    VideoStreaming,
    CustomScripting,
}

pub struct FeatureFlags {
    enabled: HashSet<Feature>,
}

impl FeatureFlags {
    pub async fn load_for_tenant(db: &DatabaseConnection, tenant_id: Uuid) -> Result<Self> {
        let flags: Vec<String> = tenant_features::Entity::find()
            .filter(tenant_features::Column::TenantId.eq(tenant_id))
            .filter(tenant_features::Column::Enabled.eq(true))
            .all(db)
            .await?
            .into_iter()
            .map(|f| f.feature_name)
            .collect();
        
        let enabled = flags
            .iter()
            .filter_map(|s| Feature::from_str(s).ok())
            .collect();
        
        Ok(Self { enabled })
    }
    
    pub fn is_enabled(&self, feature: Feature) -> bool {
        self.enabled.contains(&feature)
    }
    
    pub fn require(&self, feature: Feature) -> Result<()> {
        if self.is_enabled(feature) {
            Ok(())
        } else {
            Err(Error::FeatureNotEnabled { feature })
        }
    }
}

// Использование:
#[graphql(guard = "FeatureGuard::new(Feature::Commerce)")]
pub async fn create_product(&self, ctx: &Context<'_>, input: CreateProductInput) -> Result<Product> {
    let features = ctx.data::<FeatureFlags>()?;
    features.require(Feature::Commerce)?;
    
    // Business logic...
}
```

---

## 📈 Roadmap улучшений

### Sprint 2 (Weeks 2-3) — Simplification
- [ ] **Task 2.1:** Упростить tenant caching с moka (2 дня)
- [ ] **Task 2.2:** Добавить circuit breaker (3 дня)
- [ ] **Task 2.3:** Type-safe state machines (4 дня)
- [ ] **Task 2.4:** Стандартизировать error handling (2 дня)

### Sprint 3 (Week 4) — Observability
- [ ] **Task 3.1:** OpenTelemetry integration (5 дней)
- [ ] **Task 3.2:** Distributed tracing (3 дня)
- [ ] **Task 3.3:** Metrics dashboard (2 дня)

### Sprint 4 (Week 5-6) — Testing & Quality
- [ ] **Task 4.1:** Integration tests (5 дней)
- [ ] **Task 4.2:** Property-based tests (3 дня)
- [ ] **Task 4.3:** Performance benchmarks (2 дня)
- [ ] **Task 4.4:** Security audit (5 дней)

---

## 🎓 Дополнительные архитектурные улучшения

### 1. Saga Pattern для распределённых транзакций

Для complex workflows (например, order + payment + inventory + shipping):

```rust
// crates/rustok-core/src/saga/mod.rs
pub trait SagaStep: Send + Sync {
    type Input: Send;
    type Output: Send;
    
    async fn execute(&self, input: Self::Input) -> Result<Self::Output>;
    async fn compensate(&self, output: Self::Output) -> Result<()>;
}

pub struct SagaOrchestrator {
    steps: Vec<Box<dyn SagaStep>>,
}

impl SagaOrchestrator {
    pub async fn execute(&self) -> Result<()> {
        let mut completed_steps = Vec::new();
        
        for step in &self.steps {
            match step.execute(()).await {
                Ok(output) => {
                    completed_steps.push((step, output));
                }
                Err(e) => {
                    // Compensate in reverse order
                    for (step, output) in completed_steps.into_iter().rev() {
                        if let Err(comp_err) = step.compensate(output).await {
                            tracing::error!("Compensation failed: {}", comp_err);
                        }
                    }
                    return Err(e);
                }
            }
        }
        
        Ok(())
    }
}
```

---

### 2. Command/Query Separation в GraphQL

```rust
// apps/server/src/graphql/schema.rs
pub struct Query {
    // Read-only operations (используют Index)
}

pub struct Mutation {
    // Write operations (используют Domain services)
}

pub struct Subscription {
    // Real-time updates
}
```

---

### 3. Event Sourcing для критичных aggregates

Для Order, Payment — хранить все изменения как события:

```rust
pub struct OrderAggregate {
    id: Uuid,
    events: Vec<OrderEvent>,
}

impl OrderAggregate {
    pub fn apply(&mut self, event: OrderEvent) {
        self.events.push(event.clone());
        match event {
            OrderEvent::Created { .. } => { /* update state */ }
            OrderEvent::ItemAdded { .. } => { /* update state */ }
            OrderEvent::Paid { .. } => { /* update state */ }
        }
    }
    
    pub fn rebuild_from_events(events: Vec<OrderEvent>) -> Self {
        let mut aggregate = Self::new();
        for event in events {
            aggregate.apply(event);
        }
        aggregate
    }
}
```

---

## 🎯 Заключение

### Текущая оценка: 8.7/10

**Сильные стороны:**
- ✅ Зрелая event-driven архитектура
- ✅ Правильное применение CQRS
- ✅ Хорошая модульность
- ✅ Безопасность (после Sprint 1)

**Области улучшения:**
- 🟡 Упростить сложные абстракции (tenant cache)
- 🟡 Добавить resilience patterns (circuit breaker)
- 🟡 Увеличить test coverage (31% → 50%+)
- 🟡 Улучшить observability (OpenTelemetry)

### Целевая оценка: 9.5/10

**Достигается через:**
- Sprint 2: Simplification (2-3 weeks)
- Sprint 3: Observability (1 week)
- Sprint 4: Testing & Quality (2 weeks)

**Total time to 9.5/10:** 5-6 недель при полной фокусировке

---

## 📊 Метрики прогресса

| Метрика | Текущее | После Sprint 2 | Цель |
|---------|---------|----------------|------|
| Architecture Score | 8.7/10 | 9.0/10 | 9.5/10 |
| Test Coverage | 31% | 40% | 50%+ |
| Code Complexity | Medium | Low-Medium | Low |
| Production Readiness | 85% | 90% | 100% |
| Maintainability | Good | Excellent | Excellent |
| Scalability | Good | Excellent | Excellent |
| Security Score | 85% | 90% | 95%+ |

---

**Автор:** AI Architecture Review Team  
**Дата:** 2026-02-12  
**Версия:** 1.1 Extended  
**Следующий review:** 2026-03-12
