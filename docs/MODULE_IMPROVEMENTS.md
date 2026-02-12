# Module-Specific Improvement Recommendations

> Детальные рекомендации по улучшению каждого модуля RusToK

---

## rustok-core

### Текущие проблемы

1. **Слишком много ответственности** - core содержит и инфраструктуру, и domain types
2. **Зависимость от SeaORM** - даже модули без БД тянут SeaORM
3. **Отсутствие feature flags** для опциональных зависимостей

### Рекомендации

#### 1. Разбить на подмодули

```toml
# Cargo.toml
[package]
name = "rustok-core"

[features]
default = ["events", "cache"]
events = []
cache = []
database = ["dep:sea-orm", "dep:sea-orm-migration"]
redis-cache = ["cache", "dep:redis"]
scripting = ["dep:rhai"]

[dependencies]
# Core (всегда)
uuid = { workspace = true }
ulid = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
async-trait = { workspace = true }

# Optional
sea-orm = { workspace = true, optional = true }
sea-orm-migration = { workspace = true, optional = true }
redis = { version = "0.24", optional = true }
rhai = { workspace = true, optional = true }
```

#### 2. Создать sub-crates

```
crates/rustok-core/
├── rustok-core-types/      # Базовые типы (UUID, Error, Result)
├── rustok-core-events/     # Event system
├── rustok-core-cache/      # Cache abstractions
└── rustok-core/            # Общий re-export
```

#### 3. Улучшить Error Handling

```rust
// crates/rustok-core/src/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("Event error: {0}")]
    Event(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },
    
    #[error("Tenant error: {0}")]
    Tenant(#[from] TenantError),
    
    // Добавить context для лучшего debugging
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
        backtrace: Option<std::backtrace::Backtrace>,
    },
}

impl Error {
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            source: None,
            backtrace: Some(std::backtrace::Backtrace::capture()),
        }
    }
    
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::Internal {
            message: message.into(),
            source: Some(Box::new(source)),
            backtrace: Some(std::backtrace::Backtrace::capture()),
        }
    }
}
```

---

## rustok-commerce

### Текущие проблемы

1. **CatalogService слишком большой** (700+ строк)
2. **Нет разделения между Product Catalog и Order Management**
3. **Отсутствие domain events для важных операций** (price change, stock update)

### Рекомендации

#### 1. Разбить CatalogService

```
crates/rustok-commerce/src/services/
├── catalog/
│   ├── mod.rs                  # Re-exports
│   ├── product_service.rs      # Product CRUD
│   ├── variant_service.rs      # Variant management
│   ├── price_service.rs        # Pricing logic
│   └── inventory_service.rs    # Stock management
└── orders/
    ├── mod.rs
    ├── order_service.rs
    └── payment_service.rs
```

**product_service.rs:**
```rust
pub struct ProductService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
    variant_service: VariantService,
}

impl ProductService {
    pub async fn create_product(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        input: CreateProductInput,
    ) -> CommerceResult<ProductResponse> {
        // Only product creation logic
        // Delegate variants to VariantService
        let txn = self.db.begin().await?;
        
        let product_id = self.create_product_entity(&txn, tenant_id, input).await?;
        
        for variant_input in input.variants {
            self.variant_service
                .create_variant(&txn, product_id, variant_input)
                .await?;
        }
        
        self.event_bus
            .publish_in_tx(&txn, tenant_id, Some(actor_id), 
                DomainEvent::ProductCreated { product_id })
            .await?;
        
        txn.commit().await?;
        
        self.get_product(tenant_id, product_id).await
    }
}
```

#### 2. Добавить Domain Events для важных операций

```rust
// crates/rustok-core/src/events/types.rs
pub enum DomainEvent {
    // Existing...
    
    // NEW: More granular commerce events
    ProductPriceChanged {
        product_id: Uuid,
        variant_id: Uuid,
        old_price: i64,
        new_price: i64,
        currency: String,
    },
    
    ProductStockUpdated {
        product_id: Uuid,
        variant_id: Uuid,
        old_quantity: i32,
        new_quantity: i32,
        reason: StockUpdateReason,
    },
    
    ProductPublished {
        product_id: Uuid,
        published_at: DateTime<Utc>,
    },
    
    ProductUnpublished {
        product_id: Uuid,
        reason: Option<String>,
    },
}

pub enum StockUpdateReason {
    Purchase,
    Return,
    Adjustment,
    Recount,
}
```

#### 3. Добавить Aggregate Root pattern для Product

```rust
// crates/rustok-commerce/src/domain/product.rs
pub struct ProductAggregate {
    id: Uuid,
    tenant_id: Uuid,
    status: ProductStatus,
    variants: Vec<VariantAggregate>,
    uncommitted_events: Vec<DomainEvent>,
}

impl ProductAggregate {
    pub fn create(
        tenant_id: Uuid,
        title: String,
        variants: Vec<CreateVariantInput>,
    ) -> Result<Self, CommerceError> {
        // Business rules validation
        if variants.is_empty() {
            return Err(CommerceError::NoVariants);
        }
        
        let id = generate_id();
        let event = DomainEvent::ProductCreated { product_id: id };
        
        let mut aggregate = Self {
            id,
            tenant_id,
            status: ProductStatus::Draft,
            variants: Vec::new(),
            uncommitted_events: vec![event],
        };
        
        for variant_input in variants {
            aggregate.add_variant(variant_input)?;
        }
        
        Ok(aggregate)
    }
    
    pub fn publish(&mut self) -> Result<(), CommerceError> {
        if self.status == ProductStatus::Active {
            return Err(CommerceError::AlreadyPublished);
        }
        
        self.status = ProductStatus::Active;
        self.uncommitted_events.push(DomainEvent::ProductPublished {
            product_id: self.id,
            published_at: Utc::now(),
        });
        
        Ok(())
    }
    
    pub fn take_events(&mut self) -> Vec<DomainEvent> {
        std::mem::take(&mut self.uncommitted_events)
    }
}
```

---

## rustok-content

### Текущие проблемы

1. **NodeService смешивает разные типы контента** (posts, pages, comments)
2. **Нет явного separation of concerns** между разными kinds
3. **Body storage может быть оптимизирован** для больших документов

### Рекомендации

#### 1. Type-safe Node kinds

```rust
// crates/rustok-content/src/domain/node_kind.rs
use std::marker::PhantomData;

pub trait NodeKind: Send + Sync {
    fn kind_name() -> &'static str;
    fn validate(node: &Node) -> Result<(), ContentError>;
}

pub struct Post;
impl NodeKind for Post {
    fn kind_name() -> &'static str { "post" }
    
    fn validate(node: &Node) -> Result<(), ContentError> {
        if node.title.is_none() {
            return Err(ContentError::Validation("Post must have title".into()));
        }
        Ok(())
    }
}

pub struct Page;
impl NodeKind for Page {
    fn kind_name() -> &'static str { "page" }
    
    fn validate(node: &Node) -> Result<(), ContentError> {
        if node.slug.is_none() {
            return Err(ContentError::Validation("Page must have slug".into()));
        }
        Ok(())
    }
}

pub struct Comment;
impl NodeKind for Comment {
    fn kind_name() -> &'static str { "comment" }
    
    fn validate(node: &Node) -> Result<(), ContentError> {
        if node.parent_id.is_none() {
            return Err(ContentError::Validation("Comment must have parent".into()));
        }
        Ok(())
    }
}

// Type-safe operations
pub struct TypedNode<K: NodeKind> {
    inner: Node,
    _kind: PhantomData<K>,
}

impl<K: NodeKind> TypedNode<K> {
    pub fn new(node: Node) -> Result<Self, ContentError> {
        if node.kind != K::kind_name() {
            return Err(ContentError::InvalidKind);
        }
        K::validate(&node)?;
        Ok(Self {
            inner: node,
            _kind: PhantomData,
        })
    }
}

// Usage:
let post_node = TypedNode::<Post>::new(node)?;
// Compile-time guarantee это Post!
```

#### 2. Separate Body Storage для больших документов

```rust
// crates/rustok-content/src/storage/body_storage.rs
#[async_trait]
pub trait BodyStorage: Send + Sync {
    async fn store(&self, node_id: Uuid, body: &str) -> Result<StorageRef, ContentError>;
    async fn retrieve(&self, reference: &StorageRef) -> Result<String, ContentError>;
    async fn delete(&self, reference: &StorageRef) -> Result<(), ContentError>;
}

pub enum StorageRef {
    Database { node_id: Uuid },
    ObjectStore { key: String },
    Compressed { key: String },
}

pub struct SmartBodyStorage {
    db: DatabaseConnection,
    object_store: Option<Arc<dyn ObjectStore>>,
    size_threshold: usize,  // 100KB
}

impl SmartBodyStorage {
    async fn store(&self, node_id: Uuid, body: &str) -> Result<StorageRef, ContentError> {
        if body.len() > self.size_threshold {
            if let Some(store) = &self.object_store {
                let key = format!("bodies/{}/{}", node_id, Uuid::new_v4());
                store.put(&key, body.as_bytes()).await?;
                return Ok(StorageRef::ObjectStore { key });
            }
        }
        
        // Fallback to database
        let entity = bodies::ActiveModel {
            node_id: Set(node_id),
            body: Set(Some(body.to_string())),
            ..Default::default()
        };
        entity.insert(&self.db).await?;
        
        Ok(StorageRef::Database { node_id })
    }
}
```

---

## rustok-index

### Текущие проблемы

1. **Нет graceful handling при падении indexer**
2. **Отсутствие механизма re-indexing**
3. **Не оптимизирован для high-throughput**

### Рекомендации

#### 1. Добавить Index Queue с batching

```rust
// crates/rustok-index/src/queue.rs
pub struct IndexQueue {
    queue: Arc<Mutex<VecDeque<IndexTask>>>,
    batch_size: usize,
    flush_interval: Duration,
}

pub struct IndexTask {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub tenant_id: Uuid,
    pub operation: IndexOperation,
    pub retry_count: usize,
}

pub enum IndexOperation {
    Create,
    Update,
    Delete,
}

impl IndexQueue {
    pub async fn enqueue(&self, task: IndexTask) -> Result<()> {
        let mut queue = self.queue.lock().await;
        queue.push_back(task);
        
        if queue.len() >= self.batch_size {
            self.flush_batch().await?;
        }
        
        Ok(())
    }
    
    pub async fn flush_batch(&self) -> Result<()> {
        let mut queue = self.queue.lock().await;
        let batch: Vec<_> = queue.drain(..self.batch_size.min(queue.len())).collect();
        
        if batch.is_empty() {
            return Ok(());
        }
        
        // Группируем по типу для эффективной обработки
        let mut by_type: HashMap<String, Vec<IndexTask>> = HashMap::new();
        for task in batch {
            by_type.entry(task.entity_type.clone()).or_default().push(task);
        }
        
        for (entity_type, tasks) in by_type {
            self.process_batch(&entity_type, tasks).await?;
        }
        
        Ok(())
    }
}
```

#### 2. Добавить Re-indexing mechanism

```rust
// crates/rustok-index/src/reindex.rs
pub struct ReindexManager {
    db: DatabaseConnection,
    indexers: HashMap<String, Box<dyn Indexer>>,
}

impl ReindexManager {
    pub async fn reindex_all(&self, tenant_id: Option<Uuid>) -> Result<ReindexReport> {
        let mut report = ReindexReport::default();
        
        for (entity_type, indexer) in &self.indexers {
            tracing::info!("Reindexing {}", entity_type);
            
            let result = indexer.reindex_full(tenant_id).await;
            
            match result {
                Ok(count) => {
                    report.success.insert(entity_type.clone(), count);
                }
                Err(e) => {
                    report.failures.insert(entity_type.clone(), e.to_string());
                }
            }
        }
        
        Ok(report)
    }
    
    pub async fn reindex_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> Result<()> {
        let indexer = self.indexers
            .get(entity_type)
            .ok_or(IndexError::UnknownEntityType)?;
        
        indexer.reindex_one(entity_id).await
    }
}

#[async_trait]
pub trait Indexer: Send + Sync {
    async fn reindex_full(&self, tenant_id: Option<Uuid>) -> Result<usize>;
    async fn reindex_one(&self, entity_id: Uuid) -> Result<()>;
}
```

---

## rustok-blog, rustok-forum, rustok-pages (Wrapper Modules)

### Текущие проблемы

1. **Minimal functionality** - в основном просто re-export
2. **Нет специфичной бизнес-логики**
3. **Не добавляют value поверх rustok-content**

### Рекомендации

#### 1. Добавить domain-specific logic

**rustok-blog:**
```rust
// crates/rustok-blog/src/services/post_service.rs
pub struct PostService {
    node_service: NodeService,
    event_bus: TransactionalEventBus,
}

impl PostService {
    pub async fn create_post(
        &self,
        tenant_id: Uuid,
        author_id: Uuid,
        input: CreatePostInput,
    ) -> Result<PostResponse> {
        // Blog-specific validation
        self.validate_post_input(&input)?;
        
        // Set default category if not provided
        let category_id = input.category_id
            .or_else(|| self.get_default_blog_category(tenant_id));
        
        // Create node with blog-specific defaults
        let node_input = CreateNodeInput {
            kind: "post".to_string(),
            title: Some(input.title),
            category_id,
            metadata: json!({
                "featured_image": input.featured_image,
                "excerpt": input.excerpt,
                "reading_time": self.calculate_reading_time(&input.body),
            }),
            ..input.into()
        };
        
        self.node_service
            .create_node(tenant_id, author_id, node_input)
            .await
    }
    
    fn calculate_reading_time(&self, body: &str) -> u32 {
        let words = body.split_whitespace().count();
        (words / 200).max(1) as u32  // 200 words per minute
    }
    
    pub async fn schedule_post(
        &self,
        tenant_id: Uuid,
        post_id: Uuid,
        publish_at: DateTime<Utc>,
    ) -> Result<()> {
        // Blog-specific: schedule publishing
        // Store in metadata, worker will publish at specified time
        todo!()
    }
}
```

**rustok-forum:**
```rust
// crates/rustok-forum/src/services/forum_service.rs
pub struct ForumService {
    node_service: NodeService,
}

impl ForumService {
    pub async fn create_topic(
        &self,
        tenant_id: Uuid,
        author_id: Uuid,
        category_id: Uuid,
        input: CreateTopicInput,
    ) -> Result<TopicResponse> {
        // Forum-specific validation
        if input.title.len() < 10 {
            return Err(ForumError::TitleTooShort);
        }
        
        let node_input = CreateNodeInput {
            kind: "topic".to_string(),
            category_id: Some(category_id),
            metadata: json!({
                "sticky": false,
                "locked": false,
                "view_count": 0,
            }),
            ..input.into()
        };
        
        self.node_service.create_node(tenant_id, author_id, node_input).await
    }
    
    pub async fn reply_to_topic(
        &self,
        tenant_id: Uuid,
        author_id: Uuid,
        topic_id: Uuid,
        body: String,
    ) -> Result<ReplyResponse> {
        // Forum-specific: increment reply count on parent
        let reply_node = CreateNodeInput {
            kind: "reply".to_string(),
            parent_id: Some(topic_id),
            body: Some(body),
            ..Default::default()
        };
        
        let reply = self.node_service
            .create_node(tenant_id, author_id, reply_node)
            .await?;
        
        // Update topic reply count
        self.node_service
            .increment_reply_count(tenant_id, topic_id)
            .await?;
        
        Ok(reply.into())
    }
}
```

---

## rustok-outbox

### Текущие проблемы

1. **Нет monitoring relay worker performance**
2. **Отсутствие dead letter queue**
3. **Нет механизма recovery при сбоях**

### Рекомендации

#### 1. Добавить Dead Letter Queue

```rust
// crates/rustok-outbox/src/dlq.rs
pub struct DeadLetterQueue {
    db: DatabaseConnection,
}

impl DeadLetterQueue {
    pub async fn move_to_dlq(
        &self,
        event_id: Uuid,
        reason: &str,
        failure_count: u32,
    ) -> Result<()> {
        let event = sys_events::Entity::find_by_id(event_id)
            .one(&self.db)
            .await?
            .ok_or(OutboxError::EventNotFound)?;
        
        let dlq_entry = dead_letter_events::ActiveModel {
            id: Set(generate_id()),
            original_event_id: Set(event_id),
            payload: Set(event.payload),
            tenant_id: Set(event.tenant_id),
            failure_reason: Set(reason.to_string()),
            failure_count: Set(failure_count),
            moved_at: Set(Utc::now().into()),
            ..Default::default()
        };
        
        dlq_entry.insert(&self.db).await?;
        
        // Delete from sys_events
        sys_events::Entity::delete_by_id(event_id)
            .exec(&self.db)
            .await?;
        
        Ok(())
    }
    
    pub async fn retry_dlq_event(&self, dlq_id: Uuid) -> Result<()> {
        // Move back to sys_events for retry
        todo!()
    }
}
```

#### 2. Добавить Relay Metrics

```rust
// crates/rustok-outbox/src/relay.rs
#[derive(Clone)]
pub struct RelayMetrics {
    pub events_processed: Arc<AtomicU64>,
    pub events_failed: Arc<AtomicU64>,
    pub batch_count: Arc<AtomicU64>,
    pub last_batch_size: Arc<AtomicU64>,
    pub processing_time_ms: Arc<AtomicU64>,
}

impl RelayMetrics {
    pub fn snapshot(&self) -> RelayMetricsSnapshot {
        RelayMetricsSnapshot {
            events_processed: self.events_processed.load(Ordering::Relaxed),
            events_failed: self.events_failed.load(Ordering::Relaxed),
            batch_count: self.batch_count.load(Ordering::Relaxed),
            last_batch_size: self.last_batch_size.load(Ordering::Relaxed),
            processing_time_ms: self.processing_time_ms.load(Ordering::Relaxed),
        }
    }
}

// Expose metrics via Prometheus
pub fn register_prometheus_metrics(metrics: &RelayMetrics) {
    // Register with prometheus crate
    todo!()
}
```

---

## Заключение

Эти рекомендации направлены на:

1. ✅ **Упрощение** - меньше кода, проще понимать
2. ✅ **Безопасность** - больше compile-time проверок
3. ✅ **Производительность** - batching, caching, optimizations
4. ✅ **Observability** - metrics, tracing, monitoring
5. ✅ **Maintainability** - четкое разделение ответственности

Приоритизируйте изменения по критичности и начинайте с P0 issues.
