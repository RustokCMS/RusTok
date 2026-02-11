# RusToK — Рекомендации по улучшению архитектуры и кода

> **Дата анализа:** 11 февраля 2026  
> **Версия проекта:** 0.1.0 (Alpha)  
> **Объём кодовой базы:** ~32,500 строк Rust кода (339 файлов)

---

## Резюме

RusToK — это амбициозный и архитектурно продуманный проект, демонстрирующий зрелый подход к построению модульной headless CMS/commerce платформы на Rust. Проект следует лучшим практикам event-driven architecture, CQRS, и модульного монолита.

**Общая оценка архитектуры:** 8/10

**Сильные стороны:**
- ✅ Чистая модульная архитектура с четкими границами
- ✅ Event-driven коммуникация между модулями
- ✅ CQRS-lite с денормализованными read моделями
- ✅ Multi-tenancy как first-class citizen
- ✅ Эволюционируемый event transport (L0→L1→L2)
- ✅ Хорошая документация и манифесты

**Области для улучшения:**
- ⚠️ Недостаточное покрытие тестами
- ⚠️ Отсутствие type-state pattern для критичных бизнес-процессов
- ⚠️ Производительность кэширования tenant resolver
- ⚠️ Error handling в некоторых местах можно улучшить
- ⚠️ Отсутствие observability трассировки по модулям

---

## 1. Архитектурные рекомендации

### 1.1 Event Schema Versioning ⭐ КРИТИЧНО

**Проблема:** В текущей реализации `DomainEvent` использует `#[serde(tag = "type", content = "data")]`, но нет явного поля версии схемы. Это создаст проблемы при эволюции событий.

**Рекомендация:**

```rust
// crates/rustok-core/src/events/types.rs

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VersionedEventEnvelope {
    pub id: Uuid,
    pub schema_version: u16,  // ⬅️ ДОБАВИТЬ
    pub event_type: String,   // ⬅️ ДОБАВИТЬ для быстрой фильтрации
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub trace_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub event: DomainEvent,
    pub retry_count: u32,
}

impl DomainEvent {
    pub fn schema_version(&self) -> u16 {
        match self {
            Self::NodeCreated { .. } => 1,
            Self::NodeUpdated { .. } => 1,
            Self::ProductCreated { .. } => 2,  // Если меняли
            // ...
        }
    }
}
```

**Обоснование:**
- Outbox и Iggy должны хранить версию для replay
- При breaking changes старые обработчики должны игнорировать новые события
- Упрощает миграцию событий в production

**Приоритет:** HIGH  
**Трудоёмкость:** Medium (2-3 дня)

---

### 1.2 Transaction Boundary Pattern ⭐ ВАЖНО

**Проблема:** В `NodeService::create_node` и других методах транзакции используются корректно, но нет явного паттерна для обеспечения атомарности "запись + событие".

**Текущий код:**
```rust
// Сначала commit
txn.commit().await?;

// Потом событие (может упасть!)
self.event_bus.publish(tenant_id, security.user_id, event)?;
```

**Риск:** Если `publish` падает, данные записаны, но индекс не обновится.

**Рекомендация:** Использовать Outbox Pattern для критичных операций:

```rust
pub struct TransactionalEventBus {
    outbox: Arc<OutboxTransport>,
}

impl TransactionalEventBus {
    pub async fn publish_in_tx<C>(
        &self,
        txn: &C,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<()>
    where
        C: ConnectionTrait,
    {
        let envelope = EventEnvelope::new(tenant_id, actor_id, event);
        // Записываем в sys_events внутри той же транзакции
        self.outbox.write_to_outbox(txn, envelope).await
    }
}

// В NodeService:
async fn create_node(...) -> Result<NodeResponse> {
    let txn = self.db.begin().await?;
    
    // ... вставка данных ...
    
    // Событие в рамках той же транзакции
    self.transactional_bus
        .publish_in_tx(&txn, tenant_id, security.user_id, event)
        .await?;
    
    txn.commit().await?;  // Атомарно: данные + событие
    
    Ok(response)
}
```

**Обоснование:**
- Гарантия exactly-once delivery для критичных событий
- Consistency между write model и event bus
- Защита от потери событий при сбоях

**Приоритет:** HIGH  
**Трудоёмкость:** High (5-7 дней, требует рефакторинга сервисов)

---

### 1.3 Type-State Pattern для Order Flow ⭐ ВАЖНО

**Проблема:** В `rustok-commerce` статус заказа представлен как `String` или enum без type-safety для transitions.

**Рекомендация:**

```rust
// crates/rustok-commerce/src/order/states.rs

pub struct OrderPending {
    pub cart_items: Vec<CartItem>,
}

pub struct OrderConfirmed {
    pub order_id: Uuid,
    pub payment_intent: String,
}

pub struct OrderPaid {
    pub order_id: Uuid,
    pub payment_id: String,
}

pub struct OrderCancelled {
    pub order_id: Uuid,
    pub reason: String,
}

pub struct Order<State> {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub total: i64,
    pub currency: String,
    pub state: State,
}

impl Order<OrderPending> {
    pub fn confirm(self, payment_intent: String) -> Order<OrderConfirmed> {
        Order {
            id: self.id,
            tenant_id: self.tenant_id,
            customer_id: self.customer_id,
            total: self.total,
            currency: self.currency,
            state: OrderConfirmed {
                order_id: self.id,
                payment_intent,
            },
        }
    }
}

impl Order<OrderConfirmed> {
    pub fn mark_paid(self, payment_id: String) -> Order<OrderPaid> {
        // ...
    }
    
    pub fn cancel(self, reason: String) -> Order<OrderCancelled> {
        // ...
    }
}

// Компилятор не позволит вызвать mark_paid на OrderPending!
```

**Обоснование:**
- Невозможные состояния становятся непредставимыми
- Transitions валидируются на compile-time
- Упрощает тестирование и reasoning о бизнес-логике

**Приоритет:** MEDIUM  
**Трудоёмкость:** Medium (3-4 дня)

---

### 1.4 Tenant Cache Stampede Protection ⭐ КРИТИЧНО

**Проблема:** В `apps/server/src/middleware/tenant.rs` при cache miss все параллельные запросы пойдут в БД:

```rust
// Текущий код:
if let Some(cached_context) = infra.get_cached_tenant(&cache_key).await? {
    // OK
} else {
    // Проблема: 1000 одновременных запросов = 1000 SELECT к БД
    let tenant = tenants::Entity::find_by_id(&ctx.db, identifier.uuid).await?;
}
```

**Рекомендация:** Implement Singleflight/Coalescing pattern:

```rust
use tokio::sync::Mutex;
use std::collections::HashMap;

pub struct TenantResolver {
    cache: Arc<dyn CacheBackend>,
    in_flight: Arc<Mutex<HashMap<String, Arc<Notify>>>>,
}

impl TenantResolver {
    async fn get_or_load(&self, key: String, loader: impl Future<Output = Result<TenantContext>>) 
        -> Result<TenantContext> 
    {
        // Check cache
        if let Some(cached) = self.cache.get(&key).await? {
            return Ok(cached);
        }
        
        // Check if someone is already loading
        let notify = {
            let mut in_flight = self.in_flight.lock().await;
            if let Some(existing) = in_flight.get(&key) {
                existing.clone()
            } else {
                let notify = Arc::new(Notify::new());
                in_flight.insert(key.clone(), notify.clone());
                notify
            }
        };
        
        // Wait if someone else is loading
        if Arc::strong_count(&notify) > 1 {
            notify.notified().await;
            return self.cache.get(&key).await?.ok_or(Error::NotFound);
        }
        
        // Load
        let result = loader.await?;
        self.cache.set(key.clone(), &result).await?;
        
        // Notify waiters
        {
            let mut in_flight = self.in_flight.lock().await;
            in_flight.remove(&key);
        }
        notify.notify_waiters();
        
        Ok(result)
    }
}
```

**Обоснование:**
- Защита БД от stampede при cache invalidation
- Критично для multi-tenant систем с тысячами тенантов

**Приоритет:** HIGH  
**Трудоёмкость:** Medium (2-3 дня)

---

### 1.5 Event Handler Error Handling ⚠️ ВАЖНО

**Проблема:** В `EventDispatcher` нет retry механизма и dead letter queue.

**Рекомендация:**

```rust
// crates/rustok-core/src/events/dispatcher.rs

pub struct EventDispatcherConfig {
    pub max_retries: u32,
    pub retry_backoff_ms: u64,
    pub dlq_enabled: bool,
}

impl EventDispatcher {
    async fn dispatch_with_retry(&self, envelope: EventEnvelope) -> HandlerResult {
        let mut attempts = 0;
        let mut last_error = None;
        
        while attempts < self.config.max_retries {
            match self.dispatch_once(&envelope).await {
                Ok(()) => return Ok(()),
                Err(e) if e.is_transient() => {
                    attempts += 1;
                    last_error = Some(e);
                    tokio::time::sleep(
                        Duration::from_millis(self.config.retry_backoff_ms * 2_u64.pow(attempts))
                    ).await;
                }
                Err(e) => {
                    // Permanent error - send to DLQ immediately
                    self.send_to_dlq(envelope, e).await?;
                    return Err(e);
                }
            }
        }
        
        // Max retries exceeded - send to DLQ
        if let Some(err) = last_error {
            self.send_to_dlq(envelope, err).await?;
        }
        
        Ok(())
    }
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Medium (3-4 дня)

---

## 2. Рекомендации по коду

### 2.1 Error Handling Consistency

**Проблема:** Смешение `anyhow::Error`, `thiserror::Error`, и custom Result types.

**Текущее состояние:**
- `rustok-core` использует `thiserror`
- Некоторые модули используют `anyhow`
- Нет единого стандарта

**Рекомендация:**

1. **Библиотечные crates** (rustok-*): используйте `thiserror` с четкими error types
2. **Application layer** (apps/server): используйте `anyhow` для convenience
3. **Никогда** не используйте `.unwrap()` или `.expect()` в production коде

```rust
// В модулях:
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),
    
    #[error("Validation failed: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
}

// В контроллерах:
async fn handler() -> Result<Response> {
    let result = service.do_something()
        .await
        .context("Failed to process request")?;
    Ok(Json(result))
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Low (1-2 дня для стандартизации)

---

### 2.2 Database Query Performance

**Проблема:** В `NodeService::list_nodes` делается N+1 запрос для translations:

```rust
// Сначала загружаем nodes
let nodes = paginator.fetch_page(filter.page.saturating_sub(1)).await?;

// Потом загружаем translations для всех nodes одним запросом ✅
let translations = node_translation::Entity::find()
    .filter(node_translation::Column::NodeId.is_in(node_ids))
    .filter(node_translation::Column::Locale.eq(locale))
    .all(&self.db)
    .await?;
```

**Хорошо!** Но есть потенциал для оптимизации:

**Рекомендация:** Добавить eager loading через JOIN когда это возможно:

```rust
use sea_orm::JoinType;

let nodes_with_translations = node::Entity::find()
    .filter(node::Column::TenantId.eq(tenant_id))
    .join(JoinType::LeftJoin, node::Relation::Translation.def())
    .filter(node_translation::Column::Locale.eq(locale))
    .all(&self.db)
    .await?;
```

**Приоритет:** LOW  
**Трудоёмкость:** Low (1 день)

---

### 2.3 Input Validation

**Проблема:** Валидация данных происходит в разных местах, нет централизованного подхода.

**Рекомендация:** Использовать `validator` crate с derive макросами:

```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateNodeInput {
    #[validate(length(min = 1, max = 64))]
    pub kind: String,
    
    #[validate(length(min = 1))]
    pub translations: Vec<NodeTranslationInput>,
    
    #[validate(range(min = 0, max = 1000))]
    pub position: Option<i32>,
}

impl NodeService {
    pub async fn create_node(&self, input: CreateNodeInput) -> ContentResult<NodeResponse> {
        input.validate()
            .map_err(|e| ContentError::Validation(e.to_string()))?;
        
        // ...
    }
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Medium (2-3 дня)

---

### 2.4 Async Boundary Optimization

**Проблема:** В некоторых местах используются ненужные async вызовы:

```rust
// НЕ ОПТИМАЛЬНО:
async fn some_pure_function(x: i32) -> i32 {
    x + 1  // Не делает I/O, не нужен async
}

// ЛУЧШЕ:
fn some_pure_function(x: i32) -> i32 {
    x + 1
}
```

**Рекомендация:** Аудит всех async функций и удаление лишних:

```bash
# Найти подозрительные async функции
rg "async fn \w+.*\{\s*\n.*[^await]" -A 5
```

**Приоритет:** LOW  
**Трудоёмкость:** Low (0.5 дня)

---

## 3. Тестирование

### 3.1 Test Coverage ⭐ КРИТИЧНО

**Проблема:** Практически отсутствуют unit и integration тесты.

**Текущее состояние:**
```bash
$ find . -name "*test*.rs" | wc -l
1  # ⚠️ Всего 1 тестовый файл!
```

**Рекомендация:** Добавить тесты по уровням:

#### Level 1: Unit Tests для Services

```rust
// crates/rustok-content/src/services/node_service.rs

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};
    
    #[tokio::test]
    async fn test_create_node_requires_translations() {
        let db = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
        let event_bus = EventBus::new(100);
        let service = NodeService::new(db, event_bus);
        
        let input = CreateNodeInput {
            kind: "post".to_string(),
            translations: vec![],  // Empty!
            ..Default::default()
        };
        
        let result = service.create_node(Uuid::new_v4(), security_ctx, input).await;
        assert!(matches!(result, Err(ContentError::Validation(_))));
    }
}
```

#### Level 2: Integration Tests

```rust
// crates/rustok-content/tests/integration_test.rs

#[tokio::test]
async fn test_node_crud_workflow() {
    let db = setup_test_db().await;
    let event_bus = EventBus::new(100);
    let service = NodeService::new(db, event_bus);
    
    // Create
    let node = service.create_node(...).await.unwrap();
    assert_eq!(node.kind, "post");
    
    // Read
    let fetched = service.get_node(node.id).await.unwrap();
    assert_eq!(fetched.id, node.id);
    
    // Update
    let updated = service.update_node(node.id, update_input).await.unwrap();
    assert_eq!(updated.status, ContentStatus::Published);
    
    // Delete
    service.delete_node(node.id).await.unwrap();
    assert!(service.get_node(node.id).await.is_err());
}
```

#### Level 3: Event Integration Tests

```rust
#[tokio::test]
async fn test_node_creation_triggers_indexing() {
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    let event_bus = EventBus::with_channel(event_tx);
    
    let service = NodeService::new(db, event_bus);
    let _node = service.create_node(...).await.unwrap();
    
    // Verify event published
    let envelope = event_rx.recv().await.unwrap();
    assert!(matches!(envelope.event, DomainEvent::NodeCreated { .. }));
}
```

**Приоритет:** CRITICAL  
**Трудоёмкость:** High (10+ дней для базового покрытия)

---

### 3.2 Test Utilities

**Рекомендация:** Создать `rustok-test-utils` crate:

```rust
// crates/rustok-test-utils/src/lib.rs

pub mod fixtures {
    pub fn sample_tenant() -> Tenant { ... }
    pub fn sample_user() -> User { ... }
    pub fn sample_node() -> Node { ... }
}

pub mod db {
    pub async fn setup_test_db() -> DatabaseConnection {
        let db = Database::connect("sqlite::memory:").await.unwrap();
        // Run migrations
        Migrator::up(&db, None).await.unwrap();
        db
    }
}

pub mod events {
    pub fn mock_event_bus() -> EventBus {
        EventBus::new(100)
    }
}
```

**Приоритет:** HIGH  
**Трудоёмкость:** Medium (2-3 дня)

---

## 4. Performance & Scalability

### 4.1 Database Connection Pooling

**Текущее состояние:** Loco.rs управляет connection pool, но нет явных настроек.

**Рекомендация:** Добавить в config:

```yaml
# apps/server/config/production.yaml
database:
  uri: ${DATABASE_URL}
  max_connections: 50  # ⬅️ Добавить
  min_connections: 10
  connect_timeout: 5
  idle_timeout: 300
  enable_logging: false
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Low (0.5 дня)

---

### 4.2 Index Rebuild Performance

**Проблема:** В `rustok-index` нет механизма инкрементальной индексации.

**Рекомендация:**

```rust
// crates/rustok-index/src/rebuild.rs

pub struct IndexRebuildStrategy {
    pub batch_size: usize,
    pub parallel_workers: usize,
    pub checkpoint_enabled: bool,
}

impl ContentIndexer {
    pub async fn rebuild_with_checkpoint(&self, tenant_id: Uuid) -> IndexResult<()> {
        let checkpoint = self.load_checkpoint(tenant_id).await?;
        let last_processed_id = checkpoint.last_node_id;
        
        let mut stream = node::Entity::find()
            .filter(node::Column::TenantId.eq(tenant_id))
            .filter(node::Column::Id.gt(last_processed_id))
            .order_by_asc(node::Column::Id)
            .stream(&self.db)
            .await?;
        
        let mut batch = Vec::new();
        while let Some(node) = stream.try_next().await? {
            batch.push(node);
            
            if batch.len() >= self.config.batch_size {
                self.index_batch(&batch).await?;
                self.save_checkpoint(tenant_id, batch.last().unwrap().id).await?;
                batch.clear();
            }
        }
        
        if !batch.is_empty() {
            self.index_batch(&batch).await?;
        }
        
        Ok(())
    }
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** High (4-5 дней)

---

### 4.3 GraphQL N+1 Queries

**Проблема:** В GraphQL resolvers потенциально есть N+1 queries.

**Рекомендация:** Использовать DataLoader pattern:

```rust
// apps/server/src/graphql/loaders/mod.rs

use async_graphql::dataloader::*;

pub struct NodeLoader {
    db: DatabaseConnection,
}

#[async_trait::async_trait]
impl Loader<Uuid> for NodeLoader {
    type Value = Node;
    type Error = Arc<sea_orm::DbErr>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let nodes = node::Entity::find()
            .filter(node::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await?;
        
        Ok(nodes.into_iter().map(|node| (node.id, node)).collect())
    }
}

// В resolver:
async fn author(&self, ctx: &Context<'_>) -> Result<Option<User>> {
    let loader = ctx.data_unchecked::<DataLoader<UserLoader>>();
    Ok(loader.load_one(self.author_id?).await?)
}
```

**Приоритет:** HIGH (для production)  
**Трудоёмкость:** Medium (3-4 дня)

---

## 5. Security

### 5.1 SQL Injection Protection ✅

**Статус:** SeaORM использует prepared statements, защита есть.

---

### 5.2 Rate Limiting

**Проблема:** Отсутствует rate limiting на API endpoints.

**Рекомендация:**

```rust
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;

// apps/server/src/middleware/rate_limit.rs

pub fn rate_limit_layer(ctx: &AppContext) -> RateLimitLayer {
    let settings = &ctx.config.rustok.rate_limit;
    
    RateLimitLayer::new(
        settings.requests_per_minute / 60,  // requests per second
        Duration::from_secs(1),
    )
}

// В app.rs:
router.layer(ServiceBuilder::new()
    .layer(rate_limit_layer(&ctx))
    .layer(tenant_middleware))
```

**Приоритет:** HIGH (для production)  
**Трудоёмкость:** Low (1 день)

---

### 5.3 RBAC Enforcement Audit

**Проблема:** Не все endpoints проверяют permissions.

**Рекомендация:** Добавить middleware:

```rust
// apps/server/src/middleware/rbac.rs

pub async fn enforce_permission(
    State(ctx): State<AppContext>,
    Extension(tenant): Extension<TenantContext>,
    Extension(user): Extension<User>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = req.uri().path();
    let method = req.method();
    
    let required_permission = match (method, path) {
        (&Method::POST, "/api/products") => Permission::new(Resource::Products, Action::Create),
        (&Method::GET, "/api/products") => Permission::new(Resource::Products, Action::List),
        // ...
        _ => return Ok(next.run(req).await),
    };
    
    let security = SecurityContext::new(user.role, Some(user.id));
    if !Rbac::has_permission(&security, &required_permission) {
        return Err(StatusCode::FORBIDDEN);
    }
    
    Ok(next.run(req).await)
}
```

**Приоритет:** CRITICAL (для production)  
**Трудоёмкость:** Medium (3-4 дня)

---

## 6. Observability

### 6.1 Structured Logging

**Проблема:** Логи не всегда структурированы и содержат tenant_id/trace_id.

**Рекомендация:**

```rust
use tracing::{info, instrument};

#[instrument(
    name = "node_service.create",
    skip(self, input),
    fields(
        tenant_id = %tenant_id,
        user_id = ?security.user_id,
        kind = %input.kind,
    )
)]
pub async fn create_node(&self, tenant_id: Uuid, security: SecurityContext, input: CreateNodeInput) 
    -> ContentResult<NodeResponse> 
{
    info!("Creating node");
    // ...
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Medium (2-3 дня)

---

### 6.2 Metrics по модулям

**Проблема:** `/metrics` endpoint есть, но нет метрик по модулям.

**Рекомендация:**

```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref NODE_OPERATIONS: Counter = register_counter!(
        "rustok_content_node_operations_total",
        "Total node operations"
    ).unwrap();
    
    static ref NODE_OPERATION_DURATION: Histogram = register_histogram!(
        "rustok_content_node_operation_duration_seconds",
        "Duration of node operations"
    ).unwrap();
}

impl NodeService {
    pub async fn create_node(...) -> ContentResult<NodeResponse> {
        let _timer = NODE_OPERATION_DURATION.start_timer();
        
        let result = self.create_node_impl(...).await;
        
        NODE_OPERATIONS.inc();
        result
    }
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Medium (2-3 дня)

---

## 7. DevEx & Maintainability

### 7.1 CLI Tools для разработки

**Рекомендация:** Добавить Cargo scripts:

```toml
# .cargo/config.toml

[alias]
dev = "watch -x 'loco start'"
test-all = "test --workspace --all-features"
lint = "clippy --workspace --all-targets -- -D warnings"
fmt-check = "fmt --all -- --check"
db-reset = "loco db reset && loco db migrate"
generate-schema = "run -p schema-gen"
```

**Приоритет:** LOW  
**Трудоёмкость:** Low (0.5 дня)

---

### 7.2 Pre-commit Hooks

**Рекомендация:**

```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check || {
    echo "❌ Code is not formatted. Run: cargo fmt --all"
    exit 1
}

# Clippy
cargo clippy --workspace --all-targets -- -D warnings || {
    echo "❌ Clippy found issues"
    exit 1
}

# Tests
cargo test --workspace || {
    echo "❌ Tests failed"
    exit 1
}

echo "✅ All checks passed"
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Low (0.5 дня)

---

## 8. Документация

### 8.1 API Documentation

**Рекомендация:** Добавить примеры в OpenAPI/GraphQL docs:

```rust
/// Create a new node
///
/// # Examples
///
/// ```json
/// {
///   "kind": "post",
///   "translations": [{
///     "locale": "en",
///     "title": "Hello World",
///     "excerpt": "My first post"
///   }],
///   "bodies": [{
///     "locale": "en",
///     "body": "## Hello\nWorld content",
///     "format": "markdown"
///   }]
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/content/nodes",
    request_body = CreateNodeInput,
    responses(
        (status = 201, description = "Node created", body = NodeResponse),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Forbidden"),
    ),
    tag = "content"
)]
pub async fn create_node(...) -> Result<Json<NodeResponse>> {
    // ...
}
```

**Приоритет:** MEDIUM  
**Трудоёмкость:** Low (1-2 дня)

---

### 8.2 Architecture Decision Records

**Рекомендация:** Создать `docs/adr/` для записи архитектурных решений:

```markdown
# ADR-001: Event Transport Evolution Strategy

## Status
Accepted

## Context
We need a flexible event transport that can evolve from in-memory to distributed.

## Decision
Implement L0 (memory) → L1 (outbox) → L2 (streaming) evolution through EventTransport trait.

## Consequences
- Pros: Flexibility, testability, gradual scaling
- Cons: Additional abstraction layer

## Date
2025-01-30
```

**Приоритет:** LOW  
**Трудоёмкость:** Low (ongoing)

---

## 9. Приоритизированный Roadmap

### Phase 1: Critical Fixes (2-3 недели)

1. ✅ Event Schema Versioning
2. ✅ Transaction Boundary Pattern
3. ✅ Tenant Cache Stampede Protection
4. ✅ RBAC Enforcement Audit
5. ✅ Unit Tests (базовое покрытие)

### Phase 2: Stability & Performance (3-4 недели)

1. ✅ Event Handler Error Handling & DLQ
2. ✅ GraphQL DataLoader
3. ✅ Index Rebuild with Checkpoints
4. ✅ Rate Limiting
5. ✅ Integration Tests

### Phase 3: Production Readiness (2-3 недели)

1. ✅ Observability (metrics по модулям)
2. ✅ Structured Logging
3. ✅ Error Handling Standardization
4. ✅ API Documentation
5. ✅ Pre-commit Hooks

### Phase 4: Advanced Features (4+ недели)

1. ✅ Type-State Pattern для Order Flow
2. ✅ Advanced RBAC (Attribute-Based)
3. ✅ Flex Module (если нужен)
4. ✅ Performance Optimization
5. ✅ Advanced Testing (E2E, Load)

---

## 10. Заключение

RusToK демонстрирует отличную архитектурную базу. Основные области для улучшения:

1. **Тестирование** — самый большой пробел
2. **Transaction safety** — критично для data consistency
3. **Performance optimizations** — для production scale
4. **Observability** — для мониторинга и debugging

Проект имеет все шансы стать production-ready системой при условии реализации критических рекомендаций из Phase 1.

**Общий вердикт:** 👍 Продолжайте в том же духе, фокусируйтесь на тестах и production hardening.

---

## Приложение A: Инструменты для контроля качества

```bash
# Установка инструментов
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-outdated
cargo install cargo-watch
cargo install cargo-llvm-cov  # Coverage

# Запуск проверок
cargo audit                    # Security vulnerabilities
cargo deny check              # License compliance
cargo outdated -R             # Outdated dependencies
cargo llvm-cov --html         # Test coverage report
```

---

## Приложение B: Метрики для мониторинга

Рекомендуемые метрики для Prometheus:

```
# HTTP Metrics
http_requests_total{method, path, status}
http_request_duration_seconds{method, path}

# Database Metrics
db_connections_active
db_connections_idle
db_query_duration_seconds{query_type}

# Tenant Metrics
tenant_cache_hits_total
tenant_cache_misses_total
tenant_resolution_duration_seconds

# Event Metrics
events_published_total{event_type}
events_processed_total{event_type, handler}
events_failed_total{event_type, handler}
event_processing_duration_seconds{event_type}

# Module Metrics
module_health{module, status}
module_operation_duration_seconds{module, operation}
```

---

**Автор рекомендаций:** AI Architecture Review System  
**Дата:** 11 февраля 2026  
**Версия документа:** 1.0
