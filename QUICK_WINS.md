# RusToK — Quick Wins: Быстрые улучшения

Этот документ содержит конкретные примеры кода для самых важных и простых в реализации улучшений из основного файла рекомендаций.

---

## 1. Добавление базовых тестов (1-2 дня)

### 1.1 Unit Test Template для Services

Создайте файл `crates/rustok-content/tests/node_service_test.rs`:

```rust
use rustok_content::*;
use rustok_core::{EventBus, SecurityContext, UserRole, PermissionScope};
use sea_orm::{Database, DatabaseConnection};
use uuid::Uuid;

async fn setup_test_db() -> DatabaseConnection {
    let db = Database::connect("sqlite::memory:").await.unwrap();
    // Run migrations
    migration::Migrator::up(&db, None).await.unwrap();
    db
}

fn mock_security_context() -> SecurityContext {
    SecurityContext::new(UserRole::Admin, Some(Uuid::new_v4()))
}

#[tokio::test]
async fn test_create_node_success() {
    let db = setup_test_db().await;
    let event_bus = EventBus::new(100);
    let service = NodeService::new(db, event_bus);
    
    let tenant_id = Uuid::new_v4();
    let security = mock_security_context();
    
    let input = CreateNodeInput {
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
    };
    
    let result = service.create_node(tenant_id, security, input).await;
    assert!(result.is_ok());
    
    let node = result.unwrap();
    assert_eq!(node.kind, "post");
    assert_eq!(node.translations.len(), 1);
}

#[tokio::test]
async fn test_create_node_requires_translations() {
    let db = setup_test_db().await;
    let event_bus = EventBus::new(100);
    let service = NodeService::new(db, event_bus);
    
    let tenant_id = Uuid::new_v4();
    let security = mock_security_context();
    
    let input = CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![],  // Empty!
        bodies: vec![],
        status: None,
        parent_id: None,
        author_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    };
    
    let result = service.create_node(tenant_id, security, input).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ContentError::Validation(_)));
}

#[tokio::test]
async fn test_rbac_own_scope_enforcement() {
    let db = setup_test_db().await;
    let event_bus = EventBus::new(100);
    let service = NodeService::new(db, event_bus);
    
    let tenant_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let security = SecurityContext::new(UserRole::Customer, Some(user_id));
    
    // Customer with "Own" scope should have author_id forced
    let input = CreateNodeInput {
        kind: "post".to_string(),
        translations: vec![
            NodeTranslationInput {
                locale: "en".to_string(),
                title: Some("My Post".to_string()),
                slug: None,
                excerpt: None,
            }
        ],
        bodies: vec![],
        author_id: None,  // Not set
        status: None,
        parent_id: None,
        category_id: None,
        position: None,
        depth: None,
        reply_count: None,
        metadata: serde_json::json!({}),
    };
    
    let node = service.create_node(tenant_id, security, input).await.unwrap();
    assert_eq!(node.author_id, Some(user_id));  // Should be forced to user_id
}
```

---

## 2. Добавление Validation с validator (1 день)

### 2.1 Обновление DTO с валидацией

`crates/rustok-content/src/dto.rs`:

```rust
use validator::{Validate, ValidationError};

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateNodeInput {
    #[validate(length(min = 1, max = 64, message = "Kind must be 1-64 characters"))]
    pub kind: String,
    
    #[validate(length(min = 1, message = "At least one translation required"))]
    pub translations: Vec<NodeTranslationInput>,
    
    #[validate]
    pub bodies: Vec<BodyInput>,
    
    pub status: Option<ContentStatus>,
    pub parent_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub category_id: Option<Uuid>,
    
    #[validate(range(min = 0, max = 10000, message = "Position must be 0-10000"))]
    pub position: Option<i32>,
    
    #[validate(range(min = 0, max = 100, message = "Depth must be 0-100"))]
    pub depth: Option<i32>,
    
    #[validate(range(min = 0, message = "Reply count cannot be negative"))]
    pub reply_count: Option<i32>,
    
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NodeTranslationInput {
    #[validate(length(min = 2, max = 10, message = "Locale must be 2-10 characters (e.g., 'en', 'en-US')"))]
    pub locale: String,
    
    #[validate(length(min = 1, max = 255, message = "Title must be 1-255 characters"))]
    pub title: Option<String>,
    
    #[validate(length(max = 255, message = "Slug must be max 255 characters"))]
    pub slug: Option<String>,
    
    #[validate(length(max = 1000, message = "Excerpt must be max 1000 characters"))]
    pub excerpt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct BodyInput {
    #[validate(length(min = 2, max = 10))]
    pub locale: String,
    
    #[validate(length(max = 1_000_000, message = "Body too large (max 1MB)"))]
    pub body: Option<String>,
    
    #[validate(custom = "validate_body_format")]
    pub format: Option<String>,
}

fn validate_body_format(format: &str) -> Result<(), ValidationError> {
    match format {
        "markdown" | "html" | "plain" => Ok(()),
        _ => Err(ValidationError::new("invalid_format")),
    }
}

// Использование в сервисе:
impl NodeService {
    pub async fn create_node(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateNodeInput,
    ) -> ContentResult<NodeResponse> {
        // Валидация в начале функции
        input.validate()
            .map_err(|e| ContentError::Validation(format!("Invalid input: {}", e)))?;
        
        // Остальная логика...
    }
}
```

---

## 3. Rate Limiting Middleware (1 день)

### 3.1 Создание middleware

`apps/server/src/middleware/rate_limit.rs`:

```rust
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use loco_rs::app::AppContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, RequestCounter>>>,
    max_requests: usize,
    window: Duration,
}

struct RequestCounter {
    count: usize,
    window_start: Instant,
}

impl RateLimiter {
    fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }
    
    async fn check_rate_limit(&self, key: &str) -> Result<(), StatusCode> {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        
        let counter = requests.entry(key.to_string()).or_insert(RequestCounter {
            count: 0,
            window_start: now,
        });
        
        // Reset window if expired
        if now.duration_since(counter.window_start) > self.window {
            counter.count = 0;
            counter.window_start = now;
        }
        
        // Check limit
        if counter.count >= self.max_requests {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }
        
        counter.count += 1;
        Ok(())
    }
}

pub async fn rate_limit_middleware<B>(
    State(ctx): State<AppContext>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Extract client identifier (IP or user ID)
    let client_id = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    // Get rate limiter from context
    let rate_limiter = ctx
        .shared_store
        .get::<RateLimiter>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Check rate limit
    rate_limiter.check_rate_limit(client_id).await?;
    
    Ok(next.run(req).await)
}

// Добавить в app.rs:
pub fn init_rate_limiter(ctx: &AppContext, max_requests: usize, window: Duration) {
    let limiter = RateLimiter::new(max_requests, window);
    ctx.shared_store.insert(limiter);
}

// В after_routes:
async fn after_routes(router: AxumRouter, ctx: &AppContext) -> Result<AxumRouter> {
    init_rate_limiter(ctx, 100, Duration::from_secs(60)); // 100 req/min
    
    Ok(router.layer(axum_middleware::from_fn_with_state(
        ctx.clone(),
        rate_limit_middleware,
    )))
}
```

---

## 4. Structured Logging (0.5 дня)

### 4.1 Добавление tracing spans

`crates/rustok-content/src/services/node_service.rs`:

```rust
use tracing::{info, warn, error, instrument, Span};

impl NodeService {
    #[instrument(
        name = "node_service.create",
        skip(self, input),
        fields(
            tenant_id = %tenant_id,
            user_id = ?security.user_id,
            kind = %input.kind,
            translation_count = input.translations.len(),
        )
    )]
    pub async fn create_node(
        &self,
        tenant_id: Uuid,
        security: SecurityContext,
        input: CreateNodeInput,
    ) -> ContentResult<NodeResponse> {
        info!("Creating new node");
        
        // Validation
        input.validate()
            .map_err(|e| {
                warn!("Validation failed: {}", e);
                ContentError::Validation(e.to_string())
            })?;
        
        // ... остальная логика ...
        
        let node_id = rustok_core::generate_id();
        Span::current().record("node_id", &node_id.to_string());
        
        match self.create_node_impl(tenant_id, security, input).await {
            Ok(response) => {
                info!(node_id = %response.id, "Node created successfully");
                Ok(response)
            }
            Err(e) => {
                error!(error = ?e, "Failed to create node");
                Err(e)
            }
        }
    }
    
    #[instrument(
        name = "node_service.update",
        skip(self, update),
        fields(
            node_id = %node_id,
            user_id = ?security.user_id,
        )
    )]
    pub async fn update_node(
        &self,
        node_id: Uuid,
        security: SecurityContext,
        update: UpdateNodeInput,
    ) -> ContentResult<NodeResponse> {
        info!("Updating node");
        
        // ... логика ...
        
        info!("Node updated successfully");
        Ok(response)
    }
}
```

---

## 5. Metrics по модулям (1 день)

### 5.1 Prometheus метрики

`crates/rustok-content/src/metrics.rs`:

```rust
use prometheus::{
    Counter, Histogram, IntGauge, Registry, 
    register_counter_vec, register_histogram_vec, register_int_gauge,
};
use once_cell::sync::Lazy;

pub static CONTENT_OPERATIONS: Lazy<prometheus::CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "rustok_content_operations_total",
        "Total content operations",
        &["operation", "kind", "status"]
    ).unwrap()
});

pub static CONTENT_OPERATION_DURATION: Lazy<prometheus::HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "rustok_content_operation_duration_seconds",
        "Duration of content operations",
        &["operation", "kind"]
    ).unwrap()
});

pub static CONTENT_NODES_TOTAL: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "rustok_content_nodes_total",
        "Total number of content nodes"
    ).unwrap()
});

// Использование:
impl NodeService {
    pub async fn create_node(...) -> ContentResult<NodeResponse> {
        let timer = CONTENT_OPERATION_DURATION
            .with_label_values(&["create", &input.kind])
            .start_timer();
        
        let result = self.create_node_impl(...).await;
        
        let status = if result.is_ok() { "success" } else { "error" };
        CONTENT_OPERATIONS
            .with_label_values(&["create", &input.kind, status])
            .inc();
        
        drop(timer);  // Record duration
        
        if result.is_ok() {
            CONTENT_NODES_TOTAL.inc();
        }
        
        result
    }
}
```

### 5.2 Экспорт метрик

`apps/server/src/controllers/metrics.rs`:

```rust
use axum::{response::IntoResponse, http::StatusCode};
use prometheus::{Encoder, TextEncoder};

pub async fn prometheus_metrics() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metric_families, &mut buffer) {
        tracing::error!("Failed to encode metrics: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to encode metrics".to_string());
    }
    
    match String::from_utf8(buffer) {
        Ok(s) => (StatusCode::OK, s),
        Err(e) => {
            tracing::error!("Failed to convert metrics to UTF-8: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to convert metrics".to_string())
        }
    }
}
```

---

## 6. Pre-commit Hook (0.5 дня)

### 6.1 Установка pre-commit

Создайте `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "🔍 Running pre-commit checks..."

# 1. Format check
echo "📝 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code is not formatted. Run: cargo fmt --all"
    exit 1
fi

# 2. Clippy
echo "🔧 Running clippy..."
if ! cargo clippy --workspace --all-targets -- -D warnings; then
    echo "❌ Clippy found issues"
    exit 1
fi

# 3. Tests (только быстрые unit тесты)
echo "🧪 Running tests..."
if ! cargo test --workspace --lib; then
    echo "❌ Tests failed"
    exit 1
fi

echo "✅ All pre-commit checks passed!"
```

Сделайте executable:

```bash
chmod +x .git/hooks/pre-commit
```

---

## 7. GraphQL DataLoader (2 дня)

### 7.1 Создание DataLoader

`apps/server/src/graphql/loaders/node_loader.rs`:

```rust
use async_graphql::dataloader::*;
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, EntityTrait, ColumnTrait, QueryFilter};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::nodes::{Entity as NodeEntity, Model as Node};

pub struct NodeLoader {
    db: DatabaseConnection,
}

impl NodeLoader {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl Loader<Uuid> for NodeLoader {
    type Value = Node;
    type Error = Arc<sea_orm::DbErr>;

    async fn load(&self, keys: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let nodes = NodeEntity::find()
            .filter(rustok_content::entities::node::Column::Id.is_in(keys.to_vec()))
            .all(&self.db)
            .await
            .map_err(Arc::new)?;
        
        Ok(nodes.into_iter().map(|node| (node.id, node)).collect())
    }
}

// Аналогично для translations
pub struct NodeTranslationLoader {
    db: DatabaseConnection,
}

#[async_trait]
impl Loader<Uuid> for NodeTranslationLoader {
    type Value = Vec<NodeTranslation>;
    type Error = Arc<sea_orm::DbErr>;

    async fn load(&self, node_ids: &[Uuid]) -> Result<HashMap<Uuid, Self::Value>, Self::Error> {
        let translations = NodeTranslationEntity::find()
            .filter(rustok_content::entities::node_translation::Column::NodeId.is_in(node_ids.to_vec()))
            .all(&self.db)
            .await
            .map_err(Arc::new)?;
        
        let mut map: HashMap<Uuid, Vec<NodeTranslation>> = HashMap::new();
        for translation in translations {
            map.entry(translation.node_id)
                .or_insert_with(Vec::new)
                .push(translation);
        }
        
        Ok(map)
    }
}

// Использование в resolver:
#[Object]
impl NodeObject {
    async fn translations(&self, ctx: &Context<'_>) -> Result<Vec<NodeTranslation>> {
        let loader = ctx.data_unchecked::<DataLoader<NodeTranslationLoader>>();
        Ok(loader.load_one(self.id).await?.unwrap_or_default())
    }
    
    async fn author(&self, ctx: &Context<'_>) -> Result<Option<User>> {
        let Some(author_id) = self.author_id else {
            return Ok(None);
        };
        
        let loader = ctx.data_unchecked::<DataLoader<UserLoader>>();
        Ok(loader.load_one(author_id).await?)
    }
}
```

### 7.2 Регистрация DataLoader

`apps/server/src/graphql/schema.rs`:

```rust
use async_graphql::dataloader::DataLoader;
use crate::graphql::loaders::*;

pub fn build_schema(db: DatabaseConnection) -> AppSchema {
    let node_loader = DataLoader::new(
        NodeLoader::new(db.clone()),
        tokio::spawn,
    );
    
    let translation_loader = DataLoader::new(
        NodeTranslationLoader::new(db.clone()),
        tokio::spawn,
    );
    
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(node_loader)
        .data(translation_loader)
        .finish()
}
```

---

## 8. Error Context (0.5 дня)

### 8.1 Добавление контекста к ошибкам

```rust
use anyhow::{Context, Result};

// В контроллерах:
pub async fn create_node_handler(
    State(ctx): State<AppContext>,
    Extension(tenant): Extension<TenantContext>,
    Json(input): Json<CreateNodeInput>,
) -> Result<Json<NodeResponse>, AppError> {
    let service = NodeService::new(ctx.db.clone(), ctx.event_bus.clone());
    
    let result = service
        .create_node(tenant.id, security, input)
        .await
        .context("Failed to create node")
        .map_err(|e| {
            tracing::error!("Create node error: {:?}", e);
            AppError::from(e)
        })?;
    
    Ok(Json(result))
}

// В сервисах:
impl NodeService {
    pub async fn create_node(...) -> ContentResult<NodeResponse> {
        let node_model = node::ActiveModel { ... }
            .insert(&txn)
            .await
            .context("Failed to insert node into database")?;
        
        // ...
    }
}
```

---

## 9. Health Check Improvements (1 день)

### 9.1 Добавление custom health checks

`apps/server/src/controllers/health.rs`:

```rust
use std::time::Duration;

async fn check_database_health(db: &DatabaseConnection) -> HealthCheckResult {
    let start = Instant::now();
    
    match tokio::time::timeout(
        Duration::from_secs(2),
        db.execute_unprepared("SELECT 1")
    ).await {
        Ok(Ok(_)) => HealthCheckResult {
            name: "database".to_string(),
            status: HealthStatus::Healthy,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            message: None,
        },
        Ok(Err(e)) => HealthCheckResult {
            name: "database".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: Some(start.elapsed().as_millis() as u64),
            message: Some(format!("Database error: {}", e)),
        },
        Err(_) => HealthCheckResult {
            name: "database".to_string(),
            status: HealthStatus::Unhealthy,
            latency_ms: Some(2000),
            message: Some("Database timeout".to_string()),
        },
    }
}

async fn check_cache_health(cache: &Arc<dyn CacheBackend>) -> HealthCheckResult {
    let start = Instant::now();
    let test_key = format!("health_check_{}", Uuid::new_v4());
    
    match cache.set(test_key.clone(), vec![1, 2, 3]).await {
        Ok(_) => {
            cache.invalidate(&test_key).await.ok();
            HealthCheckResult {
                name: "cache".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(start.elapsed().as_millis() as u64),
                message: None,
            }
        }
        Err(e) => HealthCheckResult {
            name: "cache".to_string(),
            status: HealthStatus::Degraded,  // Cache is not critical
            latency_ms: Some(start.elapsed().as_millis() as u64),
            message: Some(format!("Cache error: {}", e)),
        },
    }
}
```

---

## 10. Простые Cargo Aliases (0.1 день)

### 10.1 .cargo/config.toml

```toml
[alias]
# Development
dev = "watch -x 'loco start'"
dev-admin = "watch -x 'run -p rustok-admin'"

# Testing
test-all = "test --workspace --all-features"
test-fast = "test --workspace --lib"
test-integration = "test --workspace --test '*'"

# Quality
lint = "clippy --workspace --all-targets -- -D warnings"
lint-fix = "clippy --workspace --all-targets --fix"
fmt-check = "fmt --all -- --check"
fmt-fix = "fmt --all"

# Database
db-reset = "loco db reset"
db-migrate = "loco db migrate"
db-status = "loco db status"

# Build
build-release = "build --release --workspace"
build-admin = "build -p rustok-admin --target wasm32-unknown-unknown"

# Audit
audit-all = "audit && deny check"
outdated-check = "outdated -R"

# Complete check (CI equivalent)
ci = "fmt --all -- --check && clippy --workspace --all-targets -- -D warnings && test --workspace"
```

Использование:

```bash
cargo dev              # Start dev server with auto-reload
cargo test-fast        # Quick unit tests
cargo lint             # Run clippy
cargo ci               # Run all CI checks locally
```

---

## Заключение

Эти "quick wins" можно реализовать за **5-7 дней** работы одного разработчика и они дадут:

✅ **Базовое покрытие тестами** — foundation для TDD  
✅ **Rate limiting** — защита от abuse  
✅ **Structured logging** — упрощение debugging  
✅ **Metrics** — visibility в production  
✅ **Pre-commit hooks** — качество кода  
✅ **DataLoader** — решение N+1 проблемы  
✅ **Validation** — защита от невалидных данных  
✅ **Health checks** — K8s readiness  

Начните с тестов и validation — это даст максимальный ROI для долгосрочной maintainability проекта.

---

**Автор:** AI Code Review System  
**Дата:** 11 февраля 2026
