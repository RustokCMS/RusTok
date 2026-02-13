# Немедленные Действия для Стабилизации Ядра

## Критический Приоритет (Сделать в Первую Очередь)

### 1. 🔴 Разделение Монолитных Файлов

**tenant_validation.rs (17,295 строк)**
```bash
# Создать структуру
crates/rustok-core/src/tenant_validation/
  ├── mod.rs              # Публичные реэкспорты (~100 строк)
  ├── validators/
  │   ├── mod.rs
  │   ├── domain.rs       # Валидация доменов
  │   ├── email.rs        # Email валидация
  │   ├── phone.rs        # Телефонная валидация
  │   └── identifier.rs   # ID валидация
  ├── sanitizers/
  │   ├── mod.rs
  │   ├── input.rs        # Очистка входных данных
  │   └── sql.rs          # SQL-специфичная санитизация
  └── rules/
      ├── mod.rs
      └── business.rs     # Бизнес-правила
```

**circuit_breaker.rs (15,910 строк)**
```bash
crates/rustok-core/src/resilience/
  ├── mod.rs
  ├── circuit_breaker/
  │   ├── mod.rs          # Основная логика
  │   ├── config.rs       # Конфигурация
  │   ├── state.rs        # Состояния (Closed/Open/HalfOpen)
  │   └── metrics.rs      # Метрики
  └── retry/
      ├── mod.rs
      ├── policy.rs
      └── backoff.rs
```

### 2. 🔴 Тесты для Критических Компонентов

**Цель: 90% покрытие для core-компонентов**

```rust
// rustok-iggy/tests/integration_test.rs
#[tokio::test]
async fn test_message_produce_consume() {
    let iggy = IggyClient::new(test_config()).await.unwrap();
    
    // Produce
    iggy.produce("test-topic", b"test message").await.unwrap();
    
    // Consume
    let messages = iggy.consume("test-topic", 1).await.unwrap();
    assert_eq!(messages.len(), 1);
}

// rustok-outbox/tests/relay_test.rs
#[tokio::test]
async fn test_outbox_relay_delivery() {
    let (outbox, mut receiver) = Outbox::new(test_db()).await;
    
    // Store event
    let event = TestEvent::new();
    outbox.store(event.clone()).await.unwrap();
    
    // Trigger relay
    outbox.relay().await.unwrap();
    
    // Verify delivery
    let delivered = receiver.recv().await.unwrap();
    assert_eq!(delivered.id, event.id);
}

// rustok-mcp/tests/server_test.rs
#[tokio::test]
async fn test_mcp_server_handles_requests() {
    let server = McpServer::new(test_config()).await;
    
    let response = server
        .handle_request(json!({"method": "list_modules"}))
        .await;
    
    assert!(response.is_ok());
}
```

### 3. 🟡 Унификация Обработки Ошибок

**Создать `rustok-core/src/error/typed.rs`:**

```rust
//! Типизированные ошибки домена

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorCode(pub u16);

impl ErrorCode {
    // Auth (1xxx)
    pub const INVALID_CREDENTIALS: Self = Self(1001);
    pub const TOKEN_EXPIRED: Self = Self(1002);
    pub const INSUFFICIENT_PERMISSIONS: Self = Self(1003);
    
    // Validation (2xxx)
    pub const INVALID_INPUT: Self = Self(2001);
    pub const DUPLICATE_ENTRY: Self = Self(2002);
    pub const RESOURCE_NOT_FOUND: Self = Self(2003);
    
    // Business (3xxx)
    pub const INSUFFICIENT_FUNDS: Self = Self(3001);
    pub const ORDER_ALREADY_PROCESSED: Self = Self(3002);
    pub const INVENTORY_UNAVAILABLE: Self = Self(3003);
    
    // System (5xxx)
    pub const DATABASE_ERROR: Self = Self(5001);
    pub const EXTERNAL_SERVICE_TIMEOUT: Self = Self(5002);
    pub const CIRCUIT_OPEN: Self = Self(5003);
}

#[derive(Debug, Error)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub retryable: bool,
}

impl DomainError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            retryable: Self::is_retryable(code),
        }
    }
    
    fn is_retryable(code: ErrorCode) -> bool {
        matches!(code.0, 5002..=5999)
    }
}

// Макрос для удобства
#[macro_export]
macro_rules! err {
    ($code:expr, $msg:expr) => {
        Err(DomainError::new($code, $msg).into())
    };
    ($code:expr, $fmt:expr, $($arg:tt)*) => {
        Err(DomainError::new($code, format!($fmt, $($arg)*)).into())
    };
}
```

### 4. 🟡 Стандартизация Async Trait

**Все публичные трейты должны использовать async_trait:**

```rust
// Было (непоследовательно):
pub trait Repository {
    fn find(&self, id: Uuid) -> impl Future<Output = Result<Entity>>;
}

// Стало (стандартно):
#[async_trait]
pub trait Repository: Send + Sync {
    async fn find(&self, id: Uuid) -> Result<Entity>;
    async fn save(&self, entity: &Entity) -> Result<()>;
}
```

### 5. 🟢 Улучшение Логирования

**Создать `rustok-core/src/logging.rs`:**

```rust
//! Структурированное логирование

use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

pub fn init_logging(environment: &str) {
    let filter = match environment {
        "production" => "info,rustok_core=debug",
        "staging" => "debug,rustok_core=trace",
        _ => "trace",
    };
    
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json()
        .flatten_event(true);
    
    if environment == "production" {
        subscriber.init();
    } else {
        subscriber.pretty().init();
    }
}

// Хелперы для бизнес-логики
#[macro_export]
macro_rules! business_event {
    ($name:expr, $($key:ident = $value:expr),+) => {
        tracing::info!(
            event = $name,
            event_type = "business",
            $($key = %$value),+,
            timestamp = %chrono::Utc::now().to_rfc3339()
        )
    };
}

#[macro_export]
macro_rules! security_event {
    ($name:expr, severity = $sev:expr, $($key:ident = $value:expr),+) => {
        tracing::warn!(
            event = $name,
            event_type = "security",
            severity = %$sev,
            $($key = %$value),+,
            timestamp = %chrono::Utc::now().to_rfc3339()
        )
    };
}
```

### 6. 🟢 Health Checks

**Создать `rustok-core/src/health.rs`:**

```rust
//! Система health checks

use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[async_trait]
pub trait HealthCheck: Send + Sync {
    fn name(&self) -> &str;
    async fn check(&self) -> HealthResult;
}

pub struct HealthRegistry {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self { checks: vec![] }
    }
    
    pub fn register(&mut self, check: Box<dyn HealthCheck>) {
        self.checks.push(check);
    }
    
    pub async fn run_all(&self) -> OverallHealth {
        let results = futures::future::join_all(
            self.checks.iter().map(|c| c.check())
        ).await;
        
        OverallHealth {
            status: results.iter().map(|r| r.status).max().unwrap_or(HealthStatus::Healthy),
            checks: results,
        }
    }
}

// Предопределенные checks
pub struct DatabaseHealthCheck {
    pool: DatabaseConnection,
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    fn name(&self) -> &str {
        "database"
    }
    
    async fn check(&self) -> HealthResult {
        let start = Instant::now();
        
        match self.pool.execute("SELECT 1").await {
            Ok(_) => HealthResult {
                name: self.name().to_string(),
                status: HealthStatus::Healthy,
                latency_ms: start.elapsed().as_millis() as u64,
                message: None,
            },
            Err(e) => HealthResult {
                name: self.name().to_string(),
                status: HealthStatus::Unhealthy,
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some(e.to_string()),
            },
        }
    }
}
```

### 7. 🟢 Test Fixtures

**Создать `rustok-test-utils/src/factories.rs`:**

```rust
//! Фабрики для создания тестовых данных

use fake::{Fake, Faker};

pub struct TenantFactory;

impl TenantFactory {
    pub fn build() -> TenantBuilder {
        TenantBuilder::default()
    }
}

#[derive(Default)]
pub struct TenantBuilder {
    name: Option<String>,
    slug: Option<String>,
}

impl TenantBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub async fn create(self, db: &DatabaseConnection) -> tenant::Model {
        let name = self.name.unwrap_or_else(|| Faker.fake());
        let slug = self.slug.unwrap_or_else(|| name.to_lowercase().replace(' ', '-'));
        
        tenant::ActiveModel {
            name: Set(name),
            slug: Set(slug),
            ..Default::default()
        }
        .insert(db)
        .await
        .expect("Failed to create tenant")
    }
}

// Пример использования:
// let tenant = TenantFactory::build().name("Acme Corp").create(&db).await;
```

---

## Порядок Внедрения

### Неделя 1: Файловая Структура
1. Разделить `tenant_validation.rs`
2. Разделить `circuit_breaker.rs`
3. Разделить `i18n.rs`

### Неделя 2: Тесты
1. Базовые тесты для leptos-компонентов
2. Интеграционные тесты для iggy
3. Тесты для outbox relay
4. Тесты для mcp server

### Неделя 3: Инфраструктура
1. Typed errors
2. Унификация async traits
3. Структурированное логирование
4. Health checks

### Неделя 4: Утилиты
1. Test factories
2. Документация API
3. ADR записи

---

## Ожидаемые Результаты

После выполнения этих действий:

| Метрика | До | После |
|---------|-----|-------|
| Макс. размер файла | 17K строк | <3K строк |
| Покрытие тестами | 80% | 90%+ |
| CI время | ? | <10 мин |
| Время onboarding | ? | <30 мин |
