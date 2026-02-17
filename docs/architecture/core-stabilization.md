# Рекомендации по Стабилизации и Улучшению Ядра RusToK

## Executive Summary

На основе детального анализа кодовой базы (413 Rust-файлов, ~32K LOC в crates) разработаны комплексные рекомендации по доведению архитектуры до идеального состояния.

**Текущая оценка:** 9.6/10 (отличная база)  
**Целевая оценка:** 9.9/10 (production-grade perfection)

---

## 1. Критические Приоритеты (Немедленные Действия)

### 1.1 Разделение Монолитных Файлов

**Проблема:** Несколько файлов превышают разумные пределы размера:

| Файл | Строки | Статус | Рекомендация |
|------|--------|--------|--------------|
| `tenant_validation.rs` | 17,295 | 🔴 Критично | Разделить на подмодули |
| `circuit_breaker.rs` | 15,910 | 🔴 Критично | Разделить на core/tests/metrics |
| `i18n.rs` | 13,000 | 🟡 Высоко | Модульная структура по локалям |

**Действие:**
```rust
// Было: tenant_validation.rs (17K lines)
// Стало:
tenant_validation/
  ├── mod.rs           # Публичный API
  ├── validators/      # Конкретные валидаторы
  │   ├── domain.rs
  │   ├── email.rs
  │   └── phone.rs
  ├── sanitizers/      # Очистка входных данных
  └── rules/           # Бизнес-правила
```

### 1.2 Покрытие Тестами (Критические Пробелы)

**Краты с 0 тестами:**
- `leptos-*` (8 crates) - UI компоненты без тестов
- `rustok-iggy` - Message broker интеграция
- `rustok-mcp` - MCP сервер
- `rustok-outbox` - Outbox pattern

**Рекомендуемые тесты:**

```rust
// Для leptos-компонентов
#[cfg(test)]
mod tests {
    use leptos::*;
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    fn test_auth_component_render() {
        mount_to_body(|| view! { <AuthForm /> });
        // Assert DOM structure
    }
}

// Для iggy/mcp/outbox - интеграционные тесты
#[tokio::test]
async fn test_outbox_event_delivery() {
    // Arrange: Create outbox with test transport
    // Act: Publish events, trigger relay
    // Assert: Events delivered in order
}
```

---

## 2. Архитектурные Улучшения

### 2.1 Единый Паттерн Обработки Ошибок

**Текущее состояние:** Хорошее, но можно улучшить

**Рекомендация - Typed Errors:**

```rust
// Добавить в rustok-core/src/error/typed.rs

pub trait DomainError: std::error::Error + Send + Sync + 'static {
    fn error_code(&self) -> ErrorCode;
    fn severity(&self) -> Severity;
    fn retryable(&self) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    // Auth errors (1xxx)
    InvalidCredentials = 1001,
    TokenExpired = 1002,
    
    // Validation errors (2xxx)
    InvalidInput = 2001,
    DuplicateEntry = 2002,
    
    // Business errors (3xxx)
    InsufficientFunds = 3001,
    OrderAlreadyProcessed = 3002,
    
    // System errors (5xxx)
    DatabaseUnavailable = 5001,
    ExternalServiceTimeout = 5002,
}

// Макрос для автоматической генерации
#[macro_export]
macro_rules! define_domain_error {
    ($name:ident { $($variant:ident($code:expr, $msg:expr)),* }) => {
        #[derive(Debug, thiserror::Error)]
        pub enum $name {
            $(
                #[error($msg)]
                $variant,
            )*
        }
        
        impl DomainError for $name {
            fn error_code(&self) -> ErrorCode {
                match self {
                    $(Self::$variant => ErrorCode::$variant,)*
                }
            }
            // ...
        }
    };
}
```

### 2.2 Унификация Async Паттернов

**Проблема:** Непоследовательное использование `async_trait`

**Стандарт:**

```rust
// Всегда использовать async_trait для публичных трейтов
#[async_trait]
pub trait Repository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Entity>>;
    async fn save(&self, entity: &Entity) -> Result<()>;
}

// Для внутренних trait - RPITIT (Rust 1.75+)
pub trait InternalProcessor {
    fn process(&self, input: Input) -> impl Future<Output = Result<Output>> + Send;
}
```

### 2.3 Улучшение API Безопасности

**Текущее состояние:** Отличная база в `security/`

**Улучшения:**

```rust
// Добавить в security/mod.rs

/// Zero-trust middleware для Axum
pub async fn zero_trust_middleware<B>(
    State(config): State<SecurityConfig>,
    headers: HeaderMap,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, SecurityError> {
    // 1. Validate request signature
    // 2. Check rate limit
    // 3. Verify tenant context
    // 4. Audit log
    // 5. Proceed or reject
}

/// Resource-level authorization
#[derive(Debug)]
pub struct ResourceGuard<R: Resource> {
    resource: R,
    permissions: PermissionSet,
}

impl<R: Resource> ResourceGuard<R> {
    pub fn ensure(&self, action: Action) -> Result<&R, AuthError> {
        if self.permissions.allows(action) {
            Ok(&self.resource)
        } else {
            Err(AuthError::InsufficientPermissions)
        }
    }
}
```

---

## 3. Производительность и Масштабируемость

### 3.1 Connection Pool Management

```rust
// Новый модуль: rustok-core/src/pool.rs

use deadpool::managed::{Pool, Manager, RecycleResult};

pub struct PooledConnection<M: Manager> {
    inner: Option<Object<M>>,
    metrics: Arc<PoolMetrics>,
}

#[derive(Debug)]
pub struct PoolConfig {
    pub max_size: usize,
    pub min_idle: usize,
    pub max_lifetime: Duration,
    pub connection_timeout: Duration,
    pub health_check_interval: Duration,
}

pub trait PooledResource: Send + Sync + 'static {
    type Error: std::error::Error;
    
    async fn create() -> Result<Self, Self::Error>;
    async fn recycle(&self) -> RecycleResult<Self::Error>;
    async fn health_check(&self) -> Result<(), Self::Error>;
}
```

### 3.2 Event Sourcing Optimization

**Текущее:** EventBus с backpressure

**Улучшение:**

```rust
// Добавить в events/

pub struct EventStore {
    writer: EventWriter,
    reader: EventReader,
    projections: Vec<Box<dyn Projection>>,
}

#[async_trait]
pub trait Projection: Send + Sync {
    fn name(&self) -> &str;
    fn handles(&self, event_type: &str) -> bool;
    async fn handle(&mut self, event: &DomainEvent) -> Result<()>;
    
    // Снапшоты для быстрого восстановления
    async fn snapshot(&self) -> Result<Snapshot>;
    async fn restore(&mut self, snapshot: Snapshot) -> Result<()>;
}

// CQRS read model
pub struct ReadModel<T> {
    cache: Arc<dyn CacheBackend>,
    projection: Box<dyn Projection>,
    _phantom: PhantomData<T>,
}
```

### 3.3 Memory Optimization

```rust
// Для больших структур данных

#[derive(Debug)]
pub struct CompactString {
    inner: compact_str::CompactString,
}

#[derive(Debug)]
pub struct IdSet {
    // Использовать roaring bitmap для больших наборов
    inner: roaring::RoaringBitmap,
}

// Arc для иммутабельных данных
pub struct SharedConfig {
    data: Arc<ConfigData>,
}

impl Clone for SharedConfig {
    fn clone(&self) -> Self {
        Self { data: Arc::clone(&self.data) }
    }
}
```

---

## 4. Надежность и Отказоустойчивость

### 4.1 Graceful Degradation

```rust
// Новый модуль: resilience/degradation.rs

pub enum ServiceMode {
    Full,       // 100% функциональности
    Degraded,   // Основные функции
    Minimal,    // Только чтение
    Offline,    // Сервис недоступен
}

pub struct DegradationController {
    mode: AtomicCell<ServiceMode>,
    thresholds: DegradationThresholds,
}

impl DegradationController {
    pub fn check_operation(&self, op: Operation) -> Result<(), DegradationError> {
        match (self.mode.load(), op.criticality()) {
            (ServiceMode::Full, _) => Ok(()),
            (ServiceMode::Degraded, Criticality::Critical) => Ok(()),
            (ServiceMode::Minimal, Criticality::Critical) if op.is_read() => Ok(()),
            _ => Err(DegradationError::ServiceUnavailable),
        }
    }
}
```

### 4.2 Distributed Tracing Enhancement

```rust
// Расширение tracing модуля

#[derive(Debug)]
pub struct TraceContext {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span: Option<SpanId>,
    pub baggage: HashMap<String, String>,
    pub sampled: bool,
}

impl TraceContext {
    /// Propagate context across service boundaries
    pub fn inject(&self, headers: &mut HeaderMap) {
        headers.insert(
            "x-trace-id",
            self.trace_id.to_string().parse().unwrap(),
        );
        // W3C Trace Context format
    }
    
    /// Extract context from incoming request
    pub fn extract(headers: &HeaderMap) -> Option<Self> {
        // Parse W3C Trace Context
    }
}

// Автоматическое создание спанов для трейтов
#[async_trait]
pub trait TracedRepository: Repository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Entity>> {
        let span = tracing::info_span!("repository.find_by_id", entity_id = %id);
        let _enter = span.enter();
        
        let start = Instant::now();
        let result = self.inner_find_by_id(id).await;
        
        span.record("duration_ms", start.elapsed().as_millis() as i64);
        
        result
    }
}
```

---

## 5. Управление Данными

### 5.1 Database Layer Abstraction

```rust
// Новый модуль: db/

#[async_trait]
pub trait UnitOfWork: Send + Sync {
    async fn begin(&mut self) -> Result<()>;
    async fn commit(&mut self) -> Result<()>;
    async fn rollback(&mut self) -> Result<()>;
    
    fn repository<T: Entity>(&self) -> Box<dyn Repository<T>>;
}

pub struct Transactional<T> {
    inner: T,
    uow: Box<dyn UnitOfWork>,
}

impl<T> Transactional<T> {
    pub async fn execute<F, R>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(&mut T, &dyn UnitOfWork) -> Result<R>,
    {
        self.uow.begin().await?;
        
        match f(&mut self.inner, &*self.uow) {
            Ok(result) => {
                self.uow.commit().await?;
                Ok(result)
            }
            Err(e) => {
                self.uow.rollback().await?;
                Err(e)
            }
        }
    }
}
```

### 5.2 Multi-Tenant Data Isolation

```rust
// Расширение tenant_validation.rs

pub struct TenantIsolation {
    strategy: IsolationStrategy,
    validator: TenantValidator,
}

pub enum IsolationStrategy {
    /// Separate databases per tenant
    DatabasePerTenant,
    
    /// Schema separation
    SchemaPerTenant,
    
    /// Row-level security
    RowLevelSecurity,
    
    /// Discriminator column
    DiscriminatorColumn(String),
}

impl TenantIsolation {
    pub async fn with_tenant_scope<F, R>(
        &self,
        tenant_id: TenantId,
        f: F,
    ) -> Result<R>
    where
        F: AsyncFnOnce() -> Result<R>,
    {
        // Установить tenant context
        let _guard = TenantContext::set_current(tenant_id);
        
        // Применить RLS политики
        self.apply_rls(tenant_id).await?;
        
        f().await
    }
}
```

---

## 6. Observability

### 6.1 Structured Logging

```rust
// Унифицированный формат логов

use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::layer::SubscriberExt;

pub fn init_logging(config: &LoggingConfig) {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true);
    
    let json_layer = fmt_layer
        .event_format(
            tracing_subscriber::fmt::format()
                .with_file(true)
                .with_line_number(true)
        );
    
    tracing_subscriber::registry()
        .with(json_layer)
        .with(MetricsLayer::new())
        .init();
}

// Structured log macro
#[macro_export]
macro_rules! log_event {
    ($level:ident, $event:expr, $($key:ident = $value:expr),*) => {
        tracing::$level!(
            event = $event,
            $( $key = %$value, )*
            timestamp = %chrono::Utc::now().to_rfc3339(),
        )
    };
}
```

### 6.2 Health Checks

```rust
// Расширение module.rs

#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthStatus;
    fn interval(&self) -> Duration;
}

pub struct HealthRegistry {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthRegistry {
    pub async fn run_checks(&self) -> Vec<HealthResult> {
        futures::future::join_all(
            self.checks.iter().map(|c| async move {
                HealthResult {
                    name: c.name().to_string(),
                    status: c.check().await,
                    timestamp: Utc::now(),
                }
            })
        ).await
    }
}
```

---

## 7. Разработка и Тестирование

### 7.1 Test Fixtures и Factories

```rust
// В rustok-test-utils

pub struct TenantFactory;

impl TenantFactory {
    pub fn build() -> TenantBuilder {
        TenantBuilder::default()
    }
}

pub struct TenantBuilder {
    name: String,
    slug: String,
    config: TenantConfig,
}

impl TenantBuilder {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
    
    pub async fn create(self, db: &DatabaseConnection) -> Tenant {
        let tenant = tenant::ActiveModel {
            name: Set(self.name),
            slug: Set(self.slug),
            ..Default::default()
        };
        
        tenant.insert(db).await.unwrap()
    }
}

// Использование в тестах
#[tokio::test]
async fn test_with_tenant() {
    let tenant = TenantFactory::build()
        .with_name("Test Tenant")
        .create(&db)
        .await;
    
    // Test logic...
}
```

### 7.2 Contract Testing

```rust
// Тесты контрактов между сервисами

#[tokio::test]
async fn test_event_contract() {
    // Arrange
    let event = OrderCreated::default();
    
    // Act
    let json = serde_json::to_string(&event).unwrap();
    
    // Assert - проверить что структура соответствует контракту
    let schema = schemars::schema_for!(OrderCreated);
    let validator = jsonschema::JSONSchema::compile(&schema).unwrap();
    
    assert!(validator.is_valid(&json.parse().unwrap()));
}
```

---

## 8. Документация

### 8.1 Architecture Decision Records (ADR)

Создать директорию `docs/adr/`:

```
docs/adr/
├── 001-event-driven-architecture.md
├── 002-why-sea-orm.md
├── 003-circuit-breaker-pattern.md
├── 004-multi-tenancy-strategy.md
└── 005-state-machine-type-safety.md
```

### 8.2 API Documentation

```rust
/// Полный пример документированного API
///
/// # Purpose
/// Создает новый заказ в системе
///
/// # Arguments
/// * `input` - Данные для создания заказа
/// * `context` - Контекст выполнения с tenant_id и user_id
///
/// # Returns
/// * `Ok(Order)` - Созданный заказ
/// * `Err(OrderError::InvalidProduct)` - Если продукт не существует
/// * `Err(OrderError::InsufficientInventory)` - Если недостаточно товара
///
/// # Examples
/// ```rust
/// let order = service.create_order(
///     CreateOrderInput {
///         product_id: product.id,
///         quantity: 2,
///     },
///     &context,
/// ).await?;
/// ```
///
/// # Errors
/// Возвращает `OrderError` если:
/// - Продукт не найден
/// - Недостаточно инвентаря
/// - Пользователь не имеет прав
///
/// # Performance
/// - O(1) для проверки прав
/// - O(n) для резервирования инвентаря где n = quantity
///
/// # Security
/// Требует `order:create` permission
#[instrument(skip(self, input))]
pub async fn create_order(
    &self,
    input: CreateOrderInput,
    context: &ExecutionContext,
) -> Result<Order, OrderError> {
    // Implementation
}
```

---

## 9. Инфраструктура и DevOps

### 9.1 Feature Flags

```rust
// Новый модуль: feature_flags.rs

use unleash_client::{Context, Unleash};

pub struct FeatureFlags {
    client: Unleash,
}

impl FeatureFlags {
    pub fn is_enabled(&self, flag: &str, context: &FeatureContext) -> bool {
        self.client.is_enabled(flag, Some(context))
    }
}

// Использование
if feature_flags.is_enabled("new-checkout-flow", &user_context) {
    new_checkout().await
} else {
    legacy_checkout().await
}
```

### 9.2 Configuration Management

```rust
// Улучшенная конфигурация

#[derive(Debug, Deserialize, Validate)]
pub struct AppConfig {
    #[validate(nested)]
    pub database: DatabaseConfig,
    
    #[validate(nested)]
    pub cache: CacheConfig,
    
    #[validate(nested)]
    pub security: SecurityConfig,
    
    /// Feature flags
    pub features: FeatureConfig,
}

impl AppConfig {
    /// Load from multiple sources with precedence:
    /// 1. Environment variables (highest)
    /// 2. Config files
    /// 3. Defaults (lowest)
    pub fn load() -> Result<Self, ConfigError> {
        config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name("config/local").required(false))
            .add_source(config::Environment::with_prefix("RUSTOK"))
            .build()?
            .try_deserialize()
            .map_err(ConfigError::from)
    }
}
```

---

## 10. План Внедрения

### Фаза 1: Критические Исправления (2 недели)
- [ ] Разделение `tenant_validation.rs`
- [ ] Разделение `circuit_breaker.rs`
- [ ] Базовые тесты для leptos-компонентов
- [ ] Тесты для iggy/outbox/mcp

### Фаза 2: Архитектурные Улучшения (3 недели)
- [ ] Typed errors
- [ ] Unified async patterns
- [ ] Connection pool management
- [ ] Enhanced distributed tracing

### Фаза 3: Надежность (2 недели)
- [ ] Graceful degradation
- [ ] Advanced health checks
- [ ] Multi-tenant data isolation
- [ ] Transaction management

### Фаза 4: Observability (2 недели)
- [ ] Structured logging
- [ ] Metrics collection
- [ ] ADR documentation
- [ ] API documentation coverage

### Фаза 5: Инфраструктура (2 недели)
- [ ] Feature flags
- [ ] Configuration management
- [ ] Contract tests
- [ ] Performance benchmarks

---

## Метрики Успеха

| Метрика | Текущее | Цель |
|---------|---------|------|
| Test Coverage | 80% | 90% |
| Documentation Coverage | 60% | 95% |
| Max File Size | 17K lines | <3K lines |
| CI Build Time | ? | <10 min |
| API Response Time (p99) | ? | <100ms |
| Error Rate | ? | <0.1% |

---

## Заключение

RusToK имеет отличную архитектурную основу с правильными абстракциями. Основные усилия должны быть направлены на:

1. **Модульность** - разделение больших файлов
2. **Тестирование** - покрытие UI и инфраструктурных кратов
3. **Наблюдаемость** - полный стек мониторинга
4. **Документация** - ADR и полное API покрытие

Приоритет: Стабильность > Производительность > Новые фичи
