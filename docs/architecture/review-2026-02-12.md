# RusToK — Comprehensive Architecture Review & Recommendations

> **Дата:** 2026-02-12  
> **Версия:** 1.0  
> **Автор:** AI Architecture Review  
> **Scope:** Полный анализ кодовой базы, манифеста, зависимостей и архитектурных решений

---

## 📊 Executive Summary

**Общая оценка архитектуры: 8.5/10** ⭐⭐⭐⭐⭐

RusToK демонстрирует **зрелую и хорошо продуманную архитектуру** с правильным применением enterprise-паттернов:
- ✅ Правильная реализация CQRS-lite
- ✅ Event-driven decoupling между модулями
- ✅ Надежная транзакционная публикация событий (Outbox pattern)
- ✅ Четкое разделение ответственности (модульный монолит)
- ✅ Использование Loco.rs для инфраструктуры (не изобретаем велосипед)

**Однако выявлены области для улучшения:**
- 🟡 Несогласованность в применении паттернов между модулями
- 🟡 Излишняя сложность в некоторых абстракциях
- 🟡 Недостаточная типизация и валидация в критических местах
- 🟠 Потенциальные проблемы с производительностью при scale
- 🔴 Некоторые бреши в безопасности

---

## 🎯 Ключевые находки

### ✅ Что сделано отлично

1. **Модульная архитектура (Modular Monolith)**
   - Четкое разделение на Core, Domain, Wrapper и Infrastructure модули
   - Модули взаимодействуют только через события
   - Нет прямых зависимостей между доменными модулями

2. **Event System**
   - Трехуровневая архитектура транспорта (L0→L1→L2)
   - Правильная реализация Outbox Pattern
   - Версионирование схем событий
   - Correlation и causation IDs для трейсинга

3. **CQRS Implementation**
   - Разделение Write Model (normalized) и Read Model (denormalized)
   - Index модуль для быстрых запросов
   - Event-driven синхронизация

4. **Multi-tenancy**
   - Tenant isolation на уровне БД
   - Кэширование с stampede protection
   - Redis pub/sub для распределенной инвалидации

### 🟡 Что требует внимания

1. **Несогласованность в использовании транзакционных событий**
2. **Избыточная сложность tenant resolver**
3. **Отсутствие rate limiting и backpressure**
4. **Недостаточная валидация доменных событий**
5. **Потенциальные уязвимости безопасности**

---

## 🔴 Критические рекомендации (P0 — исправить немедленно)

### 1. Несогласованное использование EventBus vs TransactionalEventBus

**Проблема:** Модули используют разные абстракции для публикации событий, что приводит к потенциальной потере событий.

**Текущее состояние:**
- ✅ `rustok-content` → использует `TransactionalEventBus` (правильно)
- ✅ `rustok-blog` → использует `TransactionalEventBus` (правильно)
- ✅ `rustok-forum` → использует `TransactionalEventBus` (правильно)
- ✅ `rustok-pages` → использует `TransactionalEventBus` (правильно)
- ✅ `rustok-commerce` → использует `TransactionalEventBus` (правильно, после миграции 2026-02-11)

**Рекомендация:** Провести аудит всех сервисов и убедиться, что везде используется `TransactionalEventBus`.

**Action items:**
```rust
// ❌ НЕПРАВИЛЬНО (legacy):
pub struct SomeService {
    db: DatabaseConnection,
    event_bus: EventBus,  // Может потерять события!
}

// ✅ ПРАВИЛЬНО:
pub struct SomeService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
}

// Внутри транзакций:
self.event_bus
    .publish_in_tx(&txn, tenant_id, Some(actor_id), event)
    .await?;
```

---

### 2. Отсутствие валидации DomainEvent при публикации

**Проблема:** События публикуются без проверки корректности данных, что может привести к:
- Invalid data в event store
- Сложности при replay events
- Проблемы с миграцией схем

**Текущий код:**
```rust
// crates/rustok-core/src/events/types.rs
pub enum DomainEvent {
    NodeCreated { node_id: Uuid, kind: String, author_id: Option<Uuid> },
    // ...
}
// Нет валидации!
```

**Рекомендация:** Добавить trait `ValidateEvent` и проверять при публикации:

```rust
// crates/rustok-core/src/events/validation.rs
pub trait ValidateEvent {
    fn validate(&self) -> Result<(), EventValidationError>;
}

impl ValidateEvent for DomainEvent {
    fn validate(&self) -> Result<(), EventValidationError> {
        match self {
            Self::NodeCreated { kind, .. } => {
                if kind.is_empty() {
                    return Err(EventValidationError::EmptyField("kind"));
                }
                if kind.len() > 64 {
                    return Err(EventValidationError::FieldTooLong("kind", 64));
                }
                Ok(())
            }
            Self::ProductCreated { product_id } => {
                if product_id.is_nil() {
                    return Err(EventValidationError::NilUuid("product_id"));
                }
                Ok(())
            }
            // Валидация для всех вариантов...
        }
    }
}

// В TransactionalEventBus::publish_in_tx:
pub async fn publish_in_tx<C: ConnectionTrait>(
    &self,
    conn: &C,
    tenant_id: Uuid,
    actor_id: Option<Uuid>,
    event: DomainEvent,
) -> Result<(), Error> {
    event.validate()?;  // Валидация перед сохранением!
    // ...
}
```

---

### 3. Уязвимость в tenant resolution — потенциальная инъекция

**Проблема:** Tenant identifier извлекается из различных источников (host, header, query param) без достаточной санитизации.

**Места риска:**
```rust
// apps/server/src/middleware/tenant.rs
fn extract_tenant_identifier(req: &Request<Body>, settings: &TenantSettings) -> Option<String> {
    match settings.resolution.as_str() {
        "subdomain" => {
            let host = req.headers().get(HOST)?.to_str().ok()?;
            // Нет валидации host!
            Some(host.split('.').next()?.to_string())
        }
        "header" => {
            let header = req.headers().get(&settings.header_name)?.to_str().ok()?;
            // Нет валидации header value!
            Some(header.to_string())
        }
        // ...
    }
}
```

**Рекомендация:** Добавить whitelist валидацию:

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static VALID_SLUG_PATTERN: Lazy<Regex> = 
    Lazy::new(|| Regex::new(r"^[a-z0-9][a-z0-9-]{0,62}$").unwrap());

fn sanitize_tenant_identifier(raw: &str) -> Result<String, TenantError> {
    let sanitized = raw.trim().to_lowercase();
    
    // Защита от очень длинных строк
    if sanitized.len() > 64 {
        return Err(TenantError::InvalidIdentifier("too long"));
    }
    
    // Защита от инъекций
    if !VALID_SLUG_PATTERN.is_match(&sanitized) {
        return Err(TenantError::InvalidIdentifier("invalid characters"));
    }
    
    // Защита от reserved names
    const RESERVED: &[&str] = &["api", "admin", "www", "app", "localhost"];
    if RESERVED.contains(&sanitized.as_str()) {
        return Err(TenantError::ReservedIdentifier);
    }
    
    Ok(sanitized)
}
```

---

### 4. Missing rate limiting в EventDispatcher

**Проблема:** `EventDispatcher` может быть перегружен при burst of events, что приведет к:
- OOM (out of memory)
- Деградации производительности
- Потере событий при отключении сервиса

**Текущий код:**
```rust
// crates/rustok-core/src/events/handler.rs
for handler in matching_handlers {
    let envelope = envelope.clone();
    let permit = semaphore.clone().acquire_owned().await;
    
    tokio::spawn(async move {
        let _permit = permit;
        let _ = Self::handle_with_retry(handler, envelope, &config).await;
    });
}
```

**Рекомендация:** Добавить bounded channel и backpressure:

```rust
pub struct DispatcherConfig {
    pub fail_fast: bool,
    pub max_concurrent: usize,
    pub retry_count: usize,
    pub retry_delay_ms: u64,
    // NEW:
    pub max_queue_depth: usize,        // Лимит очереди
    pub backpressure_threshold: f32,   // 0.8 = сбросить 80% загруженности
}

pub struct EventDispatcher {
    handlers: Vec<Arc<dyn EventHandler>>,
    config: DispatcherConfig,
    // NEW:
    queue_tx: mpsc::Sender<EnvelopeTask>,
    queue_rx: Mutex<mpsc::Receiver<EnvelopeTask>>,
    queue_depth: Arc<AtomicUsize>,
}

impl EventDispatcher {
    pub fn new(config: DispatcherConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.max_queue_depth);
        Self {
            handlers: Vec::new(),
            config,
            queue_tx: tx,
            queue_rx: Mutex::new(rx),
            queue_depth: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub async fn dispatch(&self, envelope: EventEnvelope) -> Result<()> {
        let current_depth = self.queue_depth.load(Ordering::Relaxed);
        let threshold = (self.config.max_queue_depth as f32 
                        * self.config.backpressure_threshold) as usize;
        
        if current_depth > threshold {
            tracing::warn!(
                current_depth,
                threshold,
                "Event queue approaching capacity, applying backpressure"
            );
            // Опционально: circuit breaker или reject
            return Err(Error::Backpressure);
        }
        
        self.queue_tx.send(envelope).await
            .map_err(|_| Error::QueueClosed)?;
        
        self.queue_depth.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
```

---

## 🟠 Важные рекомендации (P1 — запланировать на ближайший спринт)

### 5. Упростить архитектуру tenant caching

**Проблема:** Текущая реализация в `apps/server/src/middleware/tenant.rs` избыточно сложная:
- 580+ строк кода
- Множество уровней абстракции
- Сложность тестирования
- Трудность понимания для новых разработчиков

**Структура:**
```
TenantCacheInfrastructure
├── TenantCacheKeyBuilder (66 строк)
├── TenantCacheMetricsStore (120+ строк)
├── TenantInvalidationPublisher (40+ строк)
├── tenant_cache (dyn CacheBackend)
└── tenant_negative_cache (dyn CacheBackend)
```

**Рекомендация:** Использовать `moka` crate (уже есть в зависимостях!) для упрощения:

```rust
use moka::future::Cache;
use std::time::Duration;

pub struct SimplifiedTenantResolver {
    // Один кэш с встроенной логикой
    cache: Cache<TenantKey, TenantContext>,
    db: DatabaseConnection,
    metrics: Arc<TenantMetrics>,
}

#[derive(Hash, Eq, PartialEq, Clone)]
enum TenantKey {
    Uuid(Uuid),
    Slug(String),
    Host(String),
}

impl SimplifiedTenantResolver {
    pub fn new(db: DatabaseConnection) -> Self {
        let cache = Cache::builder()
            .max_capacity(1_000)
            .time_to_live(Duration::from_secs(300))
            .time_to_idle(Duration::from_secs(60))
            .build();
        
        Self {
            cache,
            db,
            metrics: Arc::new(TenantMetrics::new()),
        }
    }
    
    pub async fn resolve(&self, key: TenantKey) -> Result<TenantContext, TenantError> {
        // moka автоматически обрабатывает stampede protection!
        self.cache
            .try_get_with(key.clone(), async {
                self.load_from_db(&key).await
            })
            .await
            .map_err(|e| TenantError::LoadFailed(e.to_string()))
    }
    
    async fn load_from_db(&self, key: &TenantKey) -> Result<TenantContext, TenantError> {
        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
        
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
                self.resolve_by_host(host).await?
            }
        };
        
        tenant
            .ok_or(TenantError::NotFound)
            .map(|t| TenantContext::from_model(t))
    }
    
    pub async fn invalidate(&self, tenant_id: Uuid) {
        // Инвалидация всех вариантов ключей
        self.cache.invalidate(&TenantKey::Uuid(tenant_id)).await;
        // Для slug и host нужен reverse lookup или отдельный mapping
    }
}
```

**Преимущества:**
- ✅ Сокращение с 580 до ~150 строк
- ✅ Встроенная защита от cache stampede в moka
- ✅ Автоматическое управление TTL/TTI
- ✅ Проще тестировать
- ✅ Меньше кастомной логики = меньше багов

---

### 6. Добавить Circuit Breaker для внешних зависимостей

**Проблема:** Нет защиты от cascading failures при деградации внешних сервисов (Redis, Iggy, etc).

**Рекомендация:** Использовать crate `failsafe` или реализовать простой circuit breaker:

```rust
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct CircuitBreaker {
    state: Arc<AtomicU32>,  // 0=Closed, 1=Open, 2=HalfOpen
    failure_count: Arc<AtomicU32>,
    last_failure_time: Arc<Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub timeout: Duration,
    pub half_open_max_requests: u32,
}

impl CircuitBreaker {
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        match self.state.load(Ordering::Acquire) {
            0 => {  // Closed
                match f.await {
                    Ok(result) => {
                        self.on_success();
                        Ok(result)
                    }
                    Err(e) => {
                        self.on_failure();
                        Err(CircuitBreakerError::Upstream(e))
                    }
                }
            }
            1 => {  // Open
                if self.should_attempt_reset() {
                    self.state.store(2, Ordering::Release);  // HalfOpen
                    self.call(f).await
                } else {
                    Err(CircuitBreakerError::Open)
                }
            }
            2 => {  // HalfOpen
                match f.await {
                    Ok(result) => {
                        self.reset();
                        Ok(result)
                    }
                    Err(e) => {
                        self.trip();
                        Err(CircuitBreakerError::Upstream(e))
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

// Использование:
let breaker = CircuitBreaker::new(config);

// Оборачиваем Redis вызовы
let result = breaker.call(async {
    redis_client.get("key").await
}).await;

match result {
    Ok(val) => { /* use val */ }
    Err(CircuitBreakerError::Open) => {
        // Fallback to in-memory cache
        tracing::warn!("Redis circuit breaker open, using fallback");
    }
    Err(CircuitBreakerError::Upstream(e)) => {
        // Handle redis error
    }
}
```

---

### 7. Улучшить type safety для статусов и переходов

**Проблема:** Статусы хранятся как строки или enums, но валидация переходов происходит в runtime.

**Текущий код:**
```rust
// crates/rustok-commerce/src/entities/product.rs
pub enum ProductStatus {
    Draft,
    Active,
    Archived,
}

// В сервисе:
pub async fn update_status(&self, product_id: Uuid, new_status: ProductStatus) -> Result<()> {
    // Проверка допустимых переходов в runtime
    if !self.is_valid_transition(current, new_status) {
        return Err(Error::InvalidTransition);
    }
    // ...
}
```

**Рекомендация:** Использовать typestate pattern для compile-time проверки:

```rust
// State machine для продукта
pub struct Draft;
pub struct Active;
pub struct Archived;

pub struct Product<S> {
    id: Uuid,
    data: ProductData,
    _state: PhantomData<S>,
}

impl Product<Draft> {
    pub fn publish(self) -> Product<Active> {
        // Переход возможен только из Draft в Active
        Product {
            id: self.id,
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Product<Active> {
    pub fn archive(self) -> Product<Archived> {
        Product {
            id: self.id,
            data: self.data,
            _state: PhantomData,
        }
    }
    
    pub fn unpublish(self) -> Product<Draft> {
        Product {
            id: self.id,
            data: self.data,
            _state: PhantomData,
        }
    }
}

// Невозможно скомпилировать:
// let product: Product<Draft> = ...;
// product.archive();  // ❌ Compile error! 
//                     // archive() доступен только для Product<Active>
```

**Примечание:** Typestate pattern усложняет сериализацию, поэтому применять с осторожностью. Рассмотреть для критических state machines (Order, Payment flow).

---

### 8. Формализовать политику обработки ошибок

**Проблема:** Нет единой стратегии для:
- Логирования ошибок
- Retry логики
- Fallback behaviour
- Error recovery

**Рекомендация:** Создать `ErrorPolicy` trait и стандартные имплементации:

```rust
// crates/rustok-core/src/error_policy.rs
#[async_trait]
pub trait ErrorPolicy: Send + Sync {
    async fn handle_error(&self, error: &Error, context: &ErrorContext) -> ErrorAction;
}

pub enum ErrorAction {
    Retry { after: Duration, max_attempts: usize },
    Fallback { handler: Box<dyn FallbackHandler> },
    Fail { should_alert: bool },
    Ignore,
}

pub struct ErrorContext {
    pub operation: &'static str,
    pub tenant_id: Option<Uuid>,
    pub correlation_id: Uuid,
    pub attempt: usize,
}

// Предопределенные политики
pub struct DefaultErrorPolicy;

impl ErrorPolicy for DefaultErrorPolicy {
    async fn handle_error(&self, error: &Error, context: &ErrorContext) -> ErrorAction {
        match error {
            Error::Database(_) => {
                if context.attempt < 3 {
                    ErrorAction::Retry {
                        after: Duration::from_millis(100 * 2_u64.pow(context.attempt as u32)),
                        max_attempts: 3,
                    }
                } else {
                    ErrorAction::Fail { should_alert: true }
                }
            }
            Error::CacheMiss => ErrorAction::Fallback {
                handler: Box::new(LoadFromDatabase),
            },
            Error::NotFound => ErrorAction::Fail { should_alert: false },
            _ => ErrorAction::Fail { should_alert: true },
        }
    }
}
```

---

## 🟡 Рекомендации по улучшению (P2 — добавить в backlog)

### 9. Оптимизировать структуру событий для сериализации

**Текущий подход:** События сериализуются в JSON для хранения в `sys_events`.

**Проблема:**
- JSON занимает много места
- Медленная сериализация/десериализация
- Нет binary backwards compatibility

**Рекомендация:** Рассмотреть переход на Protocol Buffers или Cap'n Proto:

```rust
// Преимущества Protobuf:
// 1. Компактнее JSON (2-10x меньше)
// 2. Быстрее парсинг
// 3. Четкая схема с версионированием
// 4. Backwards/forwards compatibility

// events.proto
syntax = "proto3";

message DomainEvent {
  string event_type = 1;
  uint32 schema_version = 2;
  
  oneof payload {
    NodeCreated node_created = 10;
    ProductCreated product_created = 11;
    // ...
  }
}

message NodeCreated {
  bytes node_id = 1;  // UUID as bytes
  string kind = 2;
  optional bytes author_id = 3;
}
```

**Альтернатива:** Остаться на JSON, но добавить компрессию (zstd/lz4) для больших событий.

---

### 10. Добавить observability для event flows

**Проблема:** Сложно отследить путь события через систему.

**Рекомендация:** Добавить distributed tracing с OpenTelemetry:

```rust
use tracing::{instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

impl TransactionalEventBus {
    #[instrument(
        skip(self, conn, event),
        fields(
            event.type = %event.event_type(),
            event.version = %event.schema_version(),
            tenant.id = %tenant_id,
        )
    )]
    pub async fn publish_in_tx<C: ConnectionTrait>(
        &self,
        conn: &C,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<(), Error> {
        let span = Span::current();
        
        // Записываем trace context в event envelope
        let trace_id = span.context().span().span_context().trace_id();
        
        let envelope = EventEnvelope {
            // ...
            metadata: json!({
                "trace_id": trace_id.to_string(),
                "span_id": span.context().span().span_context().span_id().to_string(),
            }),
        };
        
        // ...
    }
}
```

---

### 11. Реализовать graceful degradation для модулей

**Проблема:** Если один модуль падает, может упасть весь сервер.

**Рекомендация:** Добавить healthcheck и изоляцию:

```rust
// crates/rustok-core/src/module.rs
pub trait RusToKModule {
    // ...
    
    async fn health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
    
    // NEW:
    fn is_critical(&self) -> bool {
        false  // По умолчанию модули не критичны
    }
    
    fn fallback_mode(&self) -> FallbackMode {
        FallbackMode::ReadOnly  // Режим деградации
    }
}

pub enum FallbackMode {
    ReadOnly,        // Чтение работает, запись отключена
    CachedData,      // Отдаем кэшированные данные
    Disabled,        // Модуль полностью отключен
}

// В registry:
impl ModuleRegistry {
    pub async fn check_health(&self) -> SystemHealth {
        let mut health = SystemHealth::Healthy;
        
        for (slug, module) in &self.modules {
            match module.health().await {
                HealthStatus::Unhealthy if module.is_critical() => {
                    health = SystemHealth::Critical;
                    break;
                }
                HealthStatus::Unhealthy => {
                    health = SystemHealth::Degraded;
                    // Активируем fallback mode
                    self.activate_fallback(slug, module.fallback_mode()).await;
                }
                _ => {}
            }
        }
        
        health
    }
}
```

---

### 12. Добавить feature flags для постепенного rollout

**Проблема:** Новые фичи сразу активируются для всех tenants.

**Рекомендация:** Добавить систему feature flags:

```rust
// crates/rustok-core/src/feature_flags.rs
pub trait FeatureFlagProvider: Send + Sync {
    async fn is_enabled(&self, flag: &str, context: &FlagContext) -> bool;
}

pub struct FlagContext {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub environment: String,
}

pub struct DatabaseFeatureFlags {
    db: DatabaseConnection,
    cache: Cache<(Uuid, String), bool>,
}

impl FeatureFlagProvider for DatabaseFeatureFlags {
    async fn is_enabled(&self, flag: &str, context: &FlagContext) -> bool {
        let cache_key = (context.tenant_id, flag.to_string());
        
        if let Some(enabled) = self.cache.get(&cache_key).await {
            return enabled;
        }
        
        let enabled = self.load_from_db(flag, context).await
            .unwrap_or(false);
        
        self.cache.insert(cache_key, enabled).await;
        enabled
    }
}

// Использование в сервисах:
pub async fn create_product(&self, input: CreateProductInput) -> Result<ProductResponse> {
    // Проверка feature flag
    if !self.flags.is_enabled("commerce.variants_v2", &context).await {
        return self.create_product_v1(input).await;
    }
    
    // Новая логика с variants_v2
    self.create_product_v2(input).await
}
```

---

## 📐 Архитектурные паттерны — рекомендации

### 13. Domain-Driven Design границы

**Текущее состояние:** Модули хорошо разделены, но нет явных Bounded Contexts.

**Рекомендация:** Формализовать DDD boundaries:

```
Bounded Contexts:

┌─────────────────────────────────────────┐
│  Content Management (rustok-content)    │
│  - Node                                 │
│  - Body                                 │
│  - Category                             │
└─────────────────────────────────────────┘
         ↓ (events)
┌─────────────────────────────────────────┐
│  Catalog (rustok-commerce)              │
│  - Product (Aggregate Root)             │
│  - Variant                              │
│  - Price                                │
└─────────────────────────────────────────┘
         ↓ (events)
┌─────────────────────────────────────────┐
│  Order Management (future)              │
│  - Order (Aggregate Root)               │
│  - OrderLine                            │
│  - Payment                              │
└─────────────────────────────────────────┘
```

**Правила:**
1. Aggregate Root — единственная точка входа в агрегат
2. Внешние модули не могут менять internals агрегата
3. Связи между контекстами только через события или API

---

### 14. Event Sourcing (опционально)

**Текущее состояние:** CQRS-lite без Event Sourcing.

**Рекомендация для критичных модулей (Orders, Payments):**
Рассмотреть полноценный Event Sourcing для полного audit trail.

```rust
// Example: Order Aggregate с Event Sourcing
pub struct Order {
    id: Uuid,
    version: u64,
    state: OrderState,
    // Не храним uncommitted events в агрегате
}

impl Order {
    pub fn place(
        tenant_id: Uuid,
        customer_id: Uuid,
        items: Vec<OrderItem>,
    ) -> (Self, OrderPlaced) {
        let id = generate_id();
        let event = OrderPlaced { id, tenant_id, customer_id, items, timestamp: Utc::now() };
        let order = Self::apply_event(Self::empty(id), &event);
        (order, event)
    }
    
    pub fn apply_event(mut self, event: &OrderEvent) -> Self {
        match event {
            OrderEvent::OrderPlaced { id, customer_id, items, .. } => {
                self.id = *id;
                self.state = OrderState::Placed { customer_id: *customer_id, items: items.clone() };
                self.version += 1;
            }
            // ...
        }
        self
    }
    
    pub fn rebuild_from_history(events: Vec<OrderEvent>) -> Self {
        events.into_iter().fold(Self::empty(Uuid::nil()), |order, event| {
            order.apply_event(&event)
        })
    }
}
```

**Преимущества:**
- Полный audit trail
- Возможность replay для debugging
- Temporal queries ("как выглядел заказ 3 месяца назад?")

**Недостатки:**
- Сложность реализации
- Требуется snapshot mechanism для производительности
- Миграция схемы событий сложнее

---

## 🔧 Технический долг

### 15. Dependency graph cleanup

**Проблема:** Некоторые зависимости дублируются или неоптимальны.

**Audit результаты:**

```toml
# Проблемные зависимости:

# 1. anyhow vs thiserror
# Рекомендация: Использовать thiserror для library crates,
#               anyhow только для applications

# 2. Версии tokio
# Везде используется workspace dependency - ХОРОШО ✅

# 3. sea-orm включен даже где не нужен
# Пример: rustok-iggy не использует БД, но зависит от sea-orm через rustok-core
# Рекомендация: Вынести DB-специфичные типы в отдельный crate rustok-db-types

# 4. Много feature flags не используется
# Рекомендация: Провести аудит и отключить неиспользуемые features
```

**Action plan:**

```toml
# Создать rustok-db-types для изоляции SeaORM
[workspace.dependencies]
rustok-db-types = { path = "crates/rustok-db-types" }

# В rustok-core:
[dependencies]
# sea-orm = { workspace = true }  # Убрать!
rustok-db-types = { workspace = true, optional = true }

[features]
database = ["rustok-db-types"]

# Модули, которым нужна БД:
rustok-core = { workspace = true, features = ["database"] }
```

---

### 16. Test coverage gaps

**Текущее покрытие:** 31% (хороший старт!)

**Приоритетные области для тестирования:**

1. **Event validation** (P0)
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_event_validation() {
           let event = DomainEvent::NodeCreated {
               node_id: Uuid::nil(),
               kind: "".to_string(),  // Invalid!
               author_id: None,
           };
           assert!(event.validate().is_err());
       }
   }
   ```

2. **Tenant isolation** (P0)
   ```rust
   #[tokio::test]
   async fn test_tenant_isolation() {
       let tenant1 = create_test_tenant().await;
       let tenant2 = create_test_tenant().await;
       
       let product1 = create_product(tenant1.id, "product1").await;
       
       // Попытка получить product1 с tenant2 credentials
       let result = get_product(tenant2.id, product1.id).await;
       assert!(matches!(result, Err(Error::NotFound)));  // Не должны видеть!
   }
   ```

3. **Circuit breaker behavior** (P1)
4. **Cache stampede protection** (P1)
5. **Event ordering and idempotency** (P1)

---

### 17. Documentation gaps

**Что хорошо документировано:**
- ✅ Архитектурные решения (MANIFEST, ARCHITECTURE_GUIDE)
- ✅ Модульная структура (MODULE_MATRIX)
- ✅ Event system (transactional_event_publishing)

**Что требует документации:**

1. **Operational Runbook**
   - Процедуры deployment
   - Rollback strategy
   - Emergency procedures
   - Monitoring и alerting

2. **API versioning strategy**
   - Как добавлять breaking changes
   - Deprecation policy
   - Migration guides

3. **Performance tuning guide**
   - Database indexing strategy
   - Cache configuration
   - Connection pooling
   - Query optimization

4. **Security practices**
   - Authentication flow
   - Authorization checks
   - Tenant isolation verification
   - Input validation checklist

---

## 🎯 Prioritization Matrix

| Рекомендация | Влияние | Сложность | Приоритет |
|--------------|---------|-----------|-----------|
| Event validation | High | Low | **P0** |
| Tenant identifier sanitization | Critical | Low | **P0** |
| EventDispatcher rate limiting | High | Medium | **P0** |
| EventBus consistency audit | Medium | Low | **P0** |
| Simplify tenant caching | Medium | High | P1 |
| Circuit breaker | High | Medium | P1 |
| Type-safe state machines | Medium | High | P2 |
| Error policy formalization | Medium | Medium | P2 |
| Event serialization optimization | Low | High | P3 |
| Feature flags system | Medium | High | P3 |
| Full Event Sourcing | Low | Very High | P3 |

---

## 📋 Action Items Summary

### Немедленно (эта неделя):

1. [ ] Добавить валидацию `DomainEvent::validate()`
2. [ ] Добавить sanitization для tenant identifiers
3. [ ] Провести аудит использования EventBus vs TransactionalEventBus
4. [ ] Добавить rate limiting в EventDispatcher

### Ближайший спринт:

5. [ ] Рефакторинг tenant resolver с moka
6. [ ] Добавить circuit breaker для внешних зависимостей
7. [ ] Формализовать error handling policy
8. [ ] Увеличить test coverage до 40%

### Backlog:

9. [ ] Рассмотреть protobuf для event serialization
10. [ ] Добавить OpenTelemetry distributed tracing
11. [ ] Реализовать graceful degradation
12. [ ] Система feature flags
13. [ ] Cleanup dependency graph

---

## 🎓 Заключение

RusToK — это **хорошо спроектированная система** с правильным применением enterprise паттернов. Основные архитектурные решения верны:

✅ Modular monolith  
✅ CQRS-lite  
✅ Event-driven architecture  
✅ Proper use of Loco.rs  
✅ Transaction-safe event publishing  

**Главные риски:**
- Безопасность tenant resolution
- Отсутствие backpressure механизмов
- Недостаточная валидация критичных данных

**Следующие шаги:**
1. Закрыть критичные P0 issues (безопасность + валидация)
2. Упростить сложные абстракции (tenant caching)
3. Улучшить observability и testing
4. Документировать operational procedures

**Оценка готовности к production:** 75%

Для запуска в production рекомендуется закрыть все P0 issues и хотя бы 50% P1 issues.

---

*Этот документ является живым и должен обновляться по мере развития проекта.*
