# RusToK — Детальный план устранения проблем

> **Дата:** 11 февраля 2026  
> **Статус:** Ready to implement  
> **Приоритет:** Critical issues first

---

## 🎯 Цель этого документа

Этот документ содержит **конкретные изменения кода** для устранения критичных проблем, найденных в архитектурном review.

В отличие от ARCHITECTURE_RECOMMENDATIONS.md (который объясняет "что" и "почему"), этот документ фокусируется на **"как именно"** с ready-to-apply code changes.

---

## 📋 План работы

### Week 1: Transaction Safety + Event Versioning
- **Day 1-2:** Event schema versioning
- **Day 3-5:** Transactional event publishing

### Week 2: Testing Foundation
- **Day 1-3:** Test utilities crate
- **Day 4-5:** Basic unit tests for NodeService

### Week 3: Cache & Security
- **Day 1-2:** Cache stampede protection
- **Day 3-5:** RBAC enforcement middleware

---

## 🔴 Issue #1: Event Schema Versioning

### Проблема
События не имеют версии схемы, что создаст проблемы при эволюции.

### Решение: Добавить версионирование

#### Step 1: Обновить EventEnvelope

**Файл:** `crates/rustok-core/src/events/types.rs`

```rust
// БЫЛО:
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub trace_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub event: DomainEvent,
    pub retry_count: u32,
}

// СТАЛО:
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    
    // ⬇️ НОВЫЕ ПОЛЯ
    pub event_type: String,        // Для быстрой фильтрации
    pub schema_version: u16,       // Версия схемы события
    
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub tenant_id: Uuid,
    pub trace_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub actor_id: Option<Uuid>,
    pub event: DomainEvent,
    pub retry_count: u32,
}

impl EventEnvelope {
    pub fn new(tenant_id: Uuid, actor_id: Option<Uuid>, event: DomainEvent) -> Self {
        let id = crate::id::generate_id();
        Self {
            id,
            event_type: event.event_type().to_string(),  // ⬅️ НОВОЕ
            schema_version: event.schema_version(),      // ⬅️ НОВОЕ
            correlation_id: id,
            causation_id: None,
            tenant_id,
            trace_id: rustok_telemetry::current_trace_id(),
            timestamp: Utc::now(),
            actor_id,
            event,
            retry_count: 0,
        }
    }
}
```

#### Step 2: Добавить schema_version() к DomainEvent

**Файл:** `crates/rustok-core/src/events/types.rs`

```rust
impl DomainEvent {
    pub fn event_type(&self) -> &'static str {
        // ... существующий код ...
    }
    
    // ⬇️ НОВЫЙ МЕТОД
    /// Returns the schema version for this event type.
    /// Increment this version when making breaking changes to the event structure.
    pub fn schema_version(&self) -> u16 {
        match self {
            // Content events (v1)
            Self::NodeCreated { .. } => 1,
            Self::NodeUpdated { .. } => 1,
            Self::NodeTranslationUpdated { .. } => 1,
            Self::NodePublished { .. } => 1,
            Self::NodeUnpublished { .. } => 1,
            Self::NodeDeleted { .. } => 1,
            Self::BodyUpdated { .. } => 1,
            
            // Category events (v1)
            Self::CategoryCreated { .. } => 1,
            Self::CategoryUpdated { .. } => 1,
            Self::CategoryDeleted { .. } => 1,
            
            // Tag events (v1)
            Self::TagCreated { .. } => 1,
            Self::TagAttached { .. } => 1,
            Self::TagDetached { .. } => 1,
            
            // Media events (v1)
            Self::MediaUploaded { .. } => 1,
            Self::MediaDeleted { .. } => 1,
            
            // User events (v1)
            Self::UserRegistered { .. } => 1,
            Self::UserLoggedIn { .. } => 1,
            Self::UserUpdated { .. } => 1,
            Self::UserDeleted { .. } => 1,
            
            // Commerce events (v1)
            Self::ProductCreated { .. } => 1,
            Self::ProductUpdated { .. } => 1,
            Self::ProductPublished { .. } => 1,
            Self::ProductDeleted { .. } => 1,
            Self::VariantCreated { .. } => 1,
            Self::VariantUpdated { .. } => 1,
            Self::VariantDeleted { .. } => 1,
            Self::InventoryUpdated { .. } => 1,
            Self::InventoryLow { .. } => 1,
            Self::PriceUpdated { .. } => 1,
            Self::OrderPlaced { .. } => 1,
            Self::OrderStatusChanged { .. } => 1,
            Self::OrderCompleted { .. } => 1,
            Self::OrderCancelled { .. } => 1,
            
            // Index events (v1)
            Self::ReindexRequested { .. } => 1,
            Self::IndexUpdated { .. } => 1,
            
            // Tenant events (v1)
            Self::TenantCreated { .. } => 1,
            Self::TenantUpdated { .. } => 1,
            Self::LocaleEnabled { .. } => 1,
            Self::LocaleDisabled { .. } => 1,
        }
    }
    
    pub fn affects_index(&self) -> bool {
        // ... существующий код ...
    }
}
```

#### Step 3: Обновить Outbox migration для хранения версии

**Файл:** `crates/rustok-outbox/src/migration.rs`

```rust
// Добавить в миграцию:
.col(
    ColumnDef::new(SysEvents::EventType)
        .string()
        .not_null()
)
.col(
    ColumnDef::new(SysEvents::SchemaVersion)
        .small_integer()
        .not_null()
        .default(1)
)
```

#### Step 4: Обновить Outbox Entity

**Файл:** `crates/rustok-outbox/src/entity.rs`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sys_events")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    
    pub event_type: String,        // ⬅️ НОВОЕ
    pub schema_version: i16,       // ⬅️ НОВОЕ
    
    pub payload: Json,
    pub status: EventStatus,
    pub tenant_id: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub dispatched_at: Option<DateTimeWithTimeZone>,
    pub error: Option<String>,
}
```

#### Step 5: Создать миграцию для обновления таблицы

**Создать файл:** `apps/server/migration/src/m20260211_000001_add_event_versioning.rs`

```rust
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SysEvents::Table)
                    .add_column(
                        ColumnDef::new(SysEvents::EventType)
                            .string()
                            .not_null()
                            .default("unknown")
                    )
                    .add_column(
                        ColumnDef::new(SysEvents::SchemaVersion)
                            .small_integer()
                            .not_null()
                            .default(1)
                    )
                    .to_owned(),
            )
            .await?;
        
        // Create index for filtering
        manager
            .create_index(
                Index::create()
                    .name("idx_sys_events_type_version")
                    .table(SysEvents::Table)
                    .col(SysEvents::EventType)
                    .col(SysEvents::SchemaVersion)
                    .to_owned(),
            )
            .await?;
        
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_sys_events_type_version").to_owned())
            .await?;
        
        manager
            .alter_table(
                Table::alter()
                    .table(SysEvents::Table)
                    .drop_column(SysEvents::SchemaVersion)
                    .drop_column(SysEvents::EventType)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum SysEvents {
    Table,
    EventType,
    SchemaVersion,
}
```

**Добавить в:** `apps/server/migration/src/lib.rs`

```rust
mod m20260211_000001_add_event_versioning;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // ... существующие миграции ...
            Box::new(m20260211_000001_add_event_versioning::Migration),
        ]
    }
}
```

#### Тестирование

```bash
# Запустить миграцию
cargo loco db migrate

# Проверить структуру таблицы
psql $DATABASE_URL -c "\d sys_events"

# Тест
cargo test -p rustok-core event_versioning
```

#### Checklist

- [ ] Обновить EventEnvelope
- [ ] Добавить schema_version() метод
- [ ] Обновить Outbox entity
- [ ] Создать миграцию
- [ ] Добавить миграцию в Migrator
- [ ] Запустить миграцию
- [ ] Добавить тесты
- [ ] Обновить документацию

**Трудоёмкость:** 1-2 дня  
**Приоритет:** 🔴 CRITICAL

---

## 🔴 Issue #2: Transactional Event Publishing

### Проблема
События публикуются после commit транзакции, что может привести к потере событий.

### Решение: Outbox Pattern с транзакционной записью

#### Step 1: Добавить write_to_outbox метод

**Файл:** `crates/rustok-outbox/src/transport.rs`

```rust
use sea_orm::{ConnectionTrait, Set};
use crate::entity::{self, EventStatus};

impl OutboxTransport {
    /// Write event to outbox within a transaction
    pub async fn write_to_outbox<C>(
        &self,
        txn: &C,
        envelope: EventEnvelope,
    ) -> Result<(), Error>
    where
        C: ConnectionTrait,
    {
        let payload = serde_json::to_value(&envelope.event)
            .map_err(|e| Error::Serialization(e.to_string()))?;
        
        let event_model = entity::ActiveModel {
            id: Set(envelope.id),
            event_type: Set(envelope.event_type),
            schema_version: Set(envelope.schema_version as i16),
            payload: Set(payload),
            status: Set(EventStatus::Pending),
            tenant_id: Set(envelope.tenant_id),
            created_at: Set(envelope.timestamp.into()),
            dispatched_at: Set(None),
            error: Set(None),
        };
        
        event_model.insert(txn).await
            .map_err(|e| Error::Database(e.to_string()))?;
        
        Ok(())
    }
    
    /// Publish event with transaction (legacy, non-transactional)
    pub async fn publish(&self, envelope: EventEnvelope) -> Result<(), Error> {
        // Use DB connection outside of transaction
        self.write_to_outbox(&self.db, envelope).await
    }
}
```

#### Step 2: Создать TransactionalEventBus

**Создать файл:** `crates/rustok-core/src/events/transactional.rs`

```rust
use crate::events::{DomainEvent, EventEnvelope, EventTransport};
use crate::{Error, Result};
use sea_orm::ConnectionTrait;
use std::sync::Arc;
use uuid::Uuid;

/// Event bus that ensures atomic write + event publishing
pub struct TransactionalEventBus {
    transport: Arc<dyn EventTransport>,
}

impl TransactionalEventBus {
    pub fn new(transport: Arc<dyn EventTransport>) -> Self {
        Self { transport }
    }
    
    /// Publish event within a transaction
    /// 
    /// This ensures that the event is only written if the transaction commits.
    /// If the transaction rolls back, the event will not be persisted.
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
        
        // If transport supports transactional writes, use it
        if let Some(outbox) = self.transport.as_any().downcast_ref::<rustok_outbox::OutboxTransport>() {
            outbox.write_to_outbox(txn, envelope).await
                .map_err(|e| Error::Event(e.to_string()))?;
        } else {
            // Fallback: publish after transaction (not atomic!)
            tracing::warn!(
                "EventTransport doesn't support transactional writes. \
                 Event may be lost if transaction fails."
            );
            self.transport.publish(envelope).await
                .map_err(|e| Error::Event(e.to_string()))?;
        }
        
        Ok(())
    }
    
    /// Publish event without transaction (legacy)
    pub async fn publish(
        &self,
        tenant_id: Uuid,
        actor_id: Option<Uuid>,
        event: DomainEvent,
    ) -> Result<()> {
        let envelope = EventEnvelope::new(tenant_id, actor_id, event);
        self.transport.publish(envelope).await
            .map_err(|e| Error::Event(e.to_string()))
    }
}
```

**Добавить в:** `crates/rustok-core/src/events/mod.rs`

```rust
pub mod transactional;
pub use transactional::TransactionalEventBus;
```

#### Step 3: Обновить EventTransport trait для downcast

**Файл:** `crates/rustok-core/src/events/transport.rs`

```rust
use std::any::Any;

#[async_trait]
pub trait EventTransport: Send + Sync {
    async fn publish(&self, envelope: EventEnvelope) -> Result<(), EventError>;
    async fn publish_batch(&self, events: Vec<EventEnvelope>) -> Result<(), EventError>;
    async fn acknowledge(&self, event_id: Uuid) -> Result<(), EventError>;
    fn reliability_level(&self) -> ReliabilityLevel;
    
    // ⬇️ НОВЫЙ МЕТОД для type-safe downcast
    fn as_any(&self) -> &dyn Any;
}
```

**Обновить:** `crates/rustok-outbox/src/transport.rs`

```rust
impl EventTransport for OutboxTransport {
    // ... существующие методы ...
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

#### Step 4: Обновить NodeService для использования транзакций

**Файл:** `crates/rustok-content/src/services/node_service.rs`

```rust
use rustok_core::TransactionalEventBus;

pub struct NodeService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,  // ⬅️ ИЗМЕНЕНО
}

impl NodeService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self { db, event_bus }
    }

    pub async fn create_node(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        mut input: CreateNodeInput,
    ) -> ContentResult<NodeResponse> {
        // ... валидация ...

        let txn = self.db.begin().await?;

        // Вставка данных
        let node_model = node::ActiveModel {
            // ... поля ...
        }
        .insert(&txn)
        .await?;

        // Вставка translations
        for translation in input.translations {
            // ...
        }

        // Вставка bodies
        for body_input in input.bodies {
            // ...
        }

        // ⬇️ ГЛАВНОЕ ИЗМЕНЕНИЕ: публикация в рамках транзакции
        self.event_bus.publish_in_tx(
            &txn,
            tenant_id,
            security.user_id,
            DomainEvent::NodeCreated {
                node_id,
                kind: input.kind.clone(),
                author_id: input.author_id,
            },
        ).await?;

        // Commit только после успешной записи события
        txn.commit().await?;

        let response = self.get_node(node_model.id).await?;
        Ok(response)
    }
}
```

#### Step 5: Обновить инициализацию в apps/server

**Файл:** `apps/server/src/app.rs`

```rust
use rustok_core::TransactionalEventBus;

async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
    let event_runtime = build_event_runtime(ctx).await?;
    
    // ⬇️ Создать TransactionalEventBus
    let transactional_bus = Arc::new(
        TransactionalEventBus::new(event_runtime.transport.clone())
    );
    
    ctx.shared_store.insert(transactional_bus);
    ctx.shared_store.insert(event_runtime.transport.clone());
    
    // ... остальной код ...
}
```

**Файл:** `apps/server/src/controllers/content.rs`

```rust
pub async fn create_node_handler(
    State(ctx): State<AppContext>,
    Extension(tenant): Extension<TenantContext>,
    Extension(user): Extension<User>,
    Json(input): Json<CreateNodeInput>,
) -> Result<Json<NodeResponse>, AppError> {
    let event_bus = ctx.shared_store
        .get::<Arc<TransactionalEventBus>>()
        .ok_or_else(|| AppError::InternalError("EventBus not initialized".into()))?;
    
    let service = NodeService::new(ctx.db.clone(), (*event_bus).clone());
    
    let security = SecurityContext::new(user.role, Some(user.id));
    
    let result = service
        .create_node(tenant.id, security, input)
        .await
        .map_err(AppError::from)?;
    
    Ok(Json(result))
}
```

#### Тестирование

**Создать:** `crates/rustok-content/tests/transactional_events_test.rs`

```rust
use rustok_content::*;
use rustok_core::{TransactionalEventBus, EventBus};
use sea_orm::{Database, DatabaseConnection};

#[tokio::test]
async fn test_event_published_only_on_commit() {
    let db = setup_test_db().await;
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let event_bus = create_test_event_bus(tx);
    let transactional_bus = TransactionalEventBus::new(Arc::new(event_bus));
    
    let service = NodeService::new(db.clone(), transactional_bus);
    
    // Test: Successful transaction
    {
        let input = create_test_input();
        let result = service.create_node(tenant_id, security, input).await;
        assert!(result.is_ok());
        
        // Event should be published
        let envelope = rx.recv().await.unwrap();
        assert!(matches!(envelope.event, DomainEvent::NodeCreated { .. }));
    }
    
    // Test: Failed transaction (validation error)
    {
        let invalid_input = CreateNodeInput {
            translations: vec![],  // Invalid
            ..Default::default()
        };
        
        let result = service.create_node(tenant_id, security, invalid_input).await;
        assert!(result.is_err());
        
        // Event should NOT be published
        tokio::time::timeout(
            Duration::from_millis(100),
            rx.recv()
        ).await.expect_err("No event should be published on transaction rollback");
    }
}
```

#### Checklist

- [ ] Добавить write_to_outbox метод
- [ ] Создать TransactionalEventBus
- [ ] Обновить EventTransport trait
- [ ] Обновить NodeService
- [ ] Обновить инициализацию в app.rs
- [ ] Обновить контроллеры
- [ ] Добавить тесты
- [ ] Обновить документацию

**Трудоёмкость:** 3-5 дней  
**Приоритет:** 🔴 CRITICAL

---

## 🔴 Issue #3: Test Utilities Crate

### Создать rustok-test-utils

**Создать:** `crates/rustok-test-utils/Cargo.toml`

```toml
[package]
name = "rustok-test-utils"
version = "0.1.0"
edition = "2021"

[dependencies]
rustok-core = { path = "../rustok-core" }
sea-orm = { workspace = true }
tokio = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
serde_json = { workspace = true }

[lib]
path = "src/lib.rs"
```

**Создать:** `crates/rustok-test-utils/src/lib.rs`

```rust
pub mod db;
pub mod events;
pub mod fixtures;

pub use db::setup_test_db;
pub use events::mock_event_bus;
pub use fixtures::*;
```

**Создать:** `crates/rustok-test-utils/src/db.rs`

```rust
use sea_orm::{Database, DatabaseConnection, DbErr};

pub async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("Failed to connect to test database");
    
    // Run migrations
    // NOTE: You'll need to run migrations here
    // migration::Migrator::up(&db, None).await.unwrap();
    
    db
}

pub async fn cleanup_test_db(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Drop all tables
    db.execute_unprepared("DROP TABLE IF EXISTS nodes CASCADE").await?;
    db.execute_unprepared("DROP TABLE IF EXISTS node_translations CASCADE").await?;
    db.execute_unprepared("DROP TABLE IF EXISTS bodies CASCADE").await?;
    Ok(())
}
```

**Создать:** `crates/rustok-test-utils/src/events.rs`

```rust
use rustok_core::{EventBus, TransactionalEventBus};
use tokio::sync::mpsc;
use std::sync::Arc;

pub fn mock_event_bus() -> (TransactionalEventBus, mpsc::UnboundedReceiver<EventEnvelope>) {
    let (tx, rx) = mpsc::unbounded_channel();
    let event_bus = EventBus::new_with_sender(tx);
    let transactional = TransactionalEventBus::new(Arc::new(event_bus));
    (transactional, rx)
}
```

**Создать:** `crates/rustok-test-utils/src/fixtures.rs`

```rust
use uuid::Uuid;
use rustok-core::{UserRole, SecurityContext};

pub fn sample_tenant_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}

pub fn sample_user_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
}

pub fn admin_security_context() -> SecurityContext {
    SecurityContext::new(UserRole::Admin, Some(sample_user_id()))
}

pub fn customer_security_context() -> SecurityContext {
    SecurityContext::new(UserRole::Customer, Some(sample_user_id()))
}

pub fn sample_node_input() -> CreateNodeInput {
    CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![
            NodeTranslationInput {
                locale: "en".to_string(),
                title: Some("Test Post".to_string()),
                slug: None,
                excerpt: Some("Test excerpt".to_string()),
            }
        ],
        bodies: vec![
            BodyInput {
                locale: "en".to_string(),
                body: Some("# Test Content".to_string()),
                format: Some("markdown".to_string()),
            }
        ],
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    }
}
```

#### Добавить в workspace

**Файл:** `Cargo.toml`

```toml
[workspace]
members = [
    # ... existing members ...
    "crates/rustok-test-utils",
]

[workspace.dependencies]
# ... existing deps ...
rustok-test-utils = { path = "crates/rustok-test-utils" }
```

**Трудоёмкость:** 1-2 дня  
**Приоритет:** 🔴 CRITICAL (blocks testing)

---

## 🔴 Issue #4: Cache Stampede Protection

### Решение: Singleflight Pattern

**Файл:** `apps/server/src/middleware/tenant.rs`

Добавить в начало файла:

```rust
use tokio::sync::{Mutex, Notify};
use std::collections::HashMap;

struct InflightRequest {
    notify: Arc<Notify>,
    result: Option<Result<TenantContext, StatusCode>>,
}

struct TenantCacheResolver {
    cache: Arc<dyn CacheBackend>,
    in_flight: Arc<Mutex<HashMap<String, Arc<InflightRequest>>>>,
}

impl TenantCacheResolver {
    fn new(cache: Arc<dyn CacheBackend>) -> Self {
        Self {
            cache,
            in_flight: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    async fn get_or_load<F, Fut>(
        &self,
        key: String,
        loader: F,
    ) -> Result<TenantContext, StatusCode>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<TenantContext, StatusCode>>,
    {
        // Check cache first
        if let Some(cached) = self.get_from_cache(&key).await? {
            return Ok(cached);
        }
        
        // Check if someone else is loading
        let (notify, should_load) = {
            let mut in_flight = self.in_flight.lock().await;
            
            if let Some(existing) = in_flight.get(&key) {
                // Someone else is loading, wait for them
                (existing.notify.clone(), false)
            } else {
                // We're the first, we'll load
                let notify = Arc::new(Notify::new());
                let request = Arc::new(InflightRequest {
                    notify: notify.clone(),
                    result: None,
                });
                in_flight.insert(key.clone(), request);
                (notify, true)
            }
        };
        
        if !should_load {
            // Wait for the loader to finish
            notify.notified().await;
            
            // Try cache again
            if let Some(cached) = self.get_from_cache(&key).await? {
                return Ok(cached);
            }
            
            // If still not in cache, something went wrong
            return Err(StatusCode::NOT_FOUND);
        }
        
        // We're the loader
        let result = loader().await;
        
        // Store in cache if successful
        if let Ok(ref context) = result {
            self.store_in_cache(&key, context).await.ok();
        }
        
        // Remove from in-flight and notify waiters
        {
            let mut in_flight = self.in_flight.lock().await;
            if let Some(request) = in_flight.remove(&key) {
                request.notify.notify_waiters();
            }
        }
        
        result
    }
    
    async fn get_from_cache(&self, key: &str) -> Result<Option<TenantContext>, StatusCode> {
        let bytes = self.cache.get(key).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        let Some(bytes) = bytes else {
            return Ok(None);
        };
        
        match serde_json::from_slice::<TenantContext>(&bytes) {
            Ok(context) => Ok(Some(context)),
            Err(_) => {
                self.cache.invalidate(key).await.ok();
                Ok(None)
            }
        }
    }
    
    async fn store_in_cache(&self, key: &str, context: &TenantContext) -> Result<(), StatusCode> {
        let bytes = serde_json::to_vec(context)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
        self.cache.set(key.to_string(), bytes).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
```

Обновить функцию `resolve`:

```rust
pub async fn resolve(
    State(ctx): State<AppContext>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let settings = RustokSettings::from_settings(&ctx.config.settings)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let identifier = resolve_identifier(&req, &settings)?;

    let Some(infra) = tenant_infra(&ctx) else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let cache_key = infra
        .key_builder
        .kind_key(identifier.kind, &identifier.value);

    // ⬇️ ИСПОЛЬЗУЕМ SINGLEFLIGHT
    let resolver = TenantCacheResolver::new(infra.tenant_cache.clone());
    
    let context = resolver.get_or_load(cache_key, || async {
        match identifier.kind {
            TenantIdentifierKind::Uuid => {
                tenants::Entity::find_by_id(&ctx.db, identifier.uuid)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .map(|t| TenantContext::from_model(&t))
                    .ok_or(StatusCode::NOT_FOUND)
            }
            TenantIdentifierKind::Slug => {
                tenants::Entity::find_by_slug(&ctx.db, &identifier.value)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .map(|t| TenantContext::from_model(&t))
                    .ok_or(StatusCode::NOT_FOUND)
            }
            TenantIdentifierKind::Host => {
                tenants::Entity::find_by_domain(&ctx.db, &identifier.value)
                    .await
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
                    .map(|t| TenantContext::from_model(&t))
                    .ok_or(StatusCode::NOT_FOUND)
            }
        }
    }).await?;

    req.extensions_mut().insert(TenantContextExtension(context));
    Ok(next.run(req).await)
}
```

#### Тестирование

**Создать:** `apps/server/tests/cache_stampede_test.rs`

```rust
#[tokio::test]
async fn test_cache_stampede_protection() {
    let ctx = setup_test_context().await;
    
    // Launch 1000 concurrent requests
    let mut handles = vec![];
    for _ in 0..1000 {
        let ctx_clone = ctx.clone();
        handles.push(tokio::spawn(async move {
            resolve_tenant(&ctx_clone, "test-tenant").await
        }));
    }
    
    // Wait for all
    let results: Vec<_> = futures::future::join_all(handles)
        .await
        .into_iter()
        .collect();
    
    // All should succeed
    for result in results {
        assert!(result.is_ok());
    }
    
    // Check database query count - should be 1, not 1000
    let query_count = ctx.db_query_count();
    assert_eq!(query_count, 1, "Should only query DB once, not {}", query_count);
}
```

**Трудоёмкость:** 2-3 дня  
**Приоритет:** 🔴 CRITICAL

---

## 📊 Timeline Summary

| Week | Tasks | Outcome |
|------|-------|---------|
| **Week 1** | Event versioning + Transactional publishing | Events are safe and versioned |
| **Week 2** | Test utils + Basic tests | Testing foundation ready |
| **Week 3** | Cache stampede + RBAC middleware | Production-safe multi-tenancy |

После этих 3 недель:
- ✅ События не теряются
- ✅ Версионирование готово для эволюции
- ✅ Базовые тесты покрывают critical paths
- ✅ Cache stampede защищен
- ✅ RBAC enforcement на всех endpoints

---

## 🔄 Следующие шаги

После завершения критичных issues:

1. **Week 4-5:** GraphQL DataLoaders + Rate Limiting
2. **Week 6-7:** Integration tests + Input validation
3. **Week 8-9:** Structured logging + Metrics
4. **Week 10:** Production deployment

---

## 📞 Поддержка

Если возникают вопросы по реализации:

1. **Проверить** этот документ сначала
2. **Проверить** QUICK_WINS.md для примеров
3. **Создать** GitHub Issue с вопросом
4. **Тег** `implementation-help`

---

**Версия:** 1.0  
**Обновлено:** 11 февраля 2026  
**Статус:** Ready to implement
