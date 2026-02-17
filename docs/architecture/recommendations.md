# RusToK — Рекомендации по улучшению архитектуры

> **Дата:** 2026-02-12  
> **Версия анализа:** 1.0  
> **Автор:** AI Code Review

---

## 📋 Executive Summary

Кодовая база RusToK демонстрирует **зрелую архитектуру** с хорошим разделением ответственности, продуманной модульной структурой и качественной реализацией CQRS/Event-Driven паттернов. Однако выявлены области для улучшения: стандартизация сервисного слоя, улучшение безопасности, упрощение некоторых абстракций и устранение технического долга.

**Общая оценка:** 8/10 ⭐

---

## 🔴 Критические рекомендации (P0)

### 1. Несогласованность транспорта событий между модулями

**Проблема:**
- `rustok-content` использует `TransactionalEventBus` из `rustok-outbox` (✅ правильно)
- `rustok-commerce` использует `EventBus` напрямую из `rustok-core` (❌ нарушает надежность)

**Места в коде:**
```rust
// crates/rustok-content/src/services/node_service.rs - ПРАВИЛЬНО
pub struct NodeService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,  // ✅
}

// crates/rustok-commerce/src/services/catalog.rs - НЕПРАВИЛЬНО
pub struct CatalogService {
    db: DatabaseConnection,
    event_bus: EventBus,  // ❌ должно быть TransactionalEventBus
}
```

**Почему это важно:**
- Без `TransactionalEventBus` события могут быть потеряны при падении транзакции
- Нарушается консистентность данных между write-моделью и read-моделью (CQRS)

**Исправление:**
```rust
// crates/rustok-commerce/src/services/catalog.rs
use rustok_outbox::TransactionalEventBus;

pub struct CatalogService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
}

// Использовать publish_in_tx внутри транзакций:
self.event_bus
    .publish_in_tx(&txn, tenant_id, Some(actor_id), DomainEvent::ProductCreated { product_id })
    .await?;
```

---

### 2. Отсутствие валидации событий при публикации

**Проблема:** `DomainEvent` публикуется без проверки соответствия схеме.

**Текущий код:**
```rust
// crates/rustok-core/src/events/types.rs
impl DomainEvent {
    pub fn event_type(&self) -> &'static str { ... }
    pub fn schema_version(&self) -> u16 { ... }
    // Нет метода валидации!
}
```

**Рекомендация:**
```rust
impl DomainEvent {
    pub fn validate(&self) -> Result<(), ValidationError> {
        match self {
            Self::NodeCreated { node_id, kind, author_id } => {
                if kind.is_empty() {
                    return Err(ValidationError::EmptyField("kind".to_string()));
                }
                // ...
            }
            // ...
        }
    }
}
```

---

### 3. Уязвимость в slugify (потенциальные инъекции)

**Проблема:** Ручная реализация slugify в `CatalogService` может быть ненадежной:

```rust
// crates/rustok-commerce/src/services/catalog.rs:552-561
fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
```

**Рекомендация:** Использовать проверенную библиотеку `slug` или санитизировать Unicode:
```rust
use slug::slugify;  // crate: slug = "0.1"

fn slugify(text: &str) -> String {
    let slug = slug::slugify(text);
    if slug.is_empty() {
        generate_fallback_slug()
    } else {
        slug
    }
}
```

---

## 🟡 Важные рекомендации (P1)

### 4. Упрощение архитектуры кэширования тенантов

**Проблема:** Слишком сложная система кэширования с множеством уровней абстракции (580+ строк в `tenant.rs`).

**Текущая структура:**
```
TenantCacheInfrastructure
├── TenantCacheKeyBuilder
├── TenantCacheMetricsStore
├── TenantInvalidationPublisher
├── tenant_cache (dyn CacheBackend)
└── tenant_negative_cache (dyn CacheBackend)
```

**Рекомендация:** Рассмотреть использование `moka` crate (уже есть в зависимостях!) для упрощения:

```rust
use moka::future::Cache;

pub struct TenantResolver {
    cache: Cache<String, TenantContext>,
    negative_cache: Cache<String, ()>,  // Для негативного кэширования
    db: DatabaseConnection,
}

impl TenantResolver {
    pub async fn resolve(&self, identifier: &str) -> Result<TenantContext, Error> {
        if let Some(tenant) = self.cache.get(identifier).await {
            return Ok(tenant);
        }
        
        if self.negative_cache.get(identifier).await.is_some() {
            return Err(Error::NotFound("Tenant not found".into()));
        }
        
        // Загрузка из БД...
    }
}
```

---

### 5. Отсутствие rate limiting в событийной системе

**Проблема:** `EventDispatcher` может быть перегружен большим потоком событий.

**Текущий код:**
```rust
// crates/rustok-core/src/events/handler.rs:179-188
for handler in matching_handlers {
    let envelope = envelope.clone();
    let config = config.clone();
    let permit = semaphore.clone().acquire_owned().await;
    
    tokio::spawn(async move {
        let _permit = permit;
        let _ = Self::handle_with_retry(handler, envelope, &config).await;
    });
}
```

**Рекомендация:** Добавить backpressure и circuit breaker:
```rust
pub struct DispatcherConfig {
    pub fail_fast: bool,
    pub max_concurrent: usize,
    pub retry_count: usize,
    pub retry_delay_ms: u64,
    // Добавить:
    pub max_queue_depth: usize,      // Лимит очереди
    pub circuit_breaker_threshold: usize,  // Порог для circuit breaker
    pub event_rate_limit: u32,       // Событий в секунду
}
```

---

### 6. Неполная интеграция Alloy Scripting

**Проблема:** `ScriptingContext` инициализируется, но интеграция с обработчиками событий не завершена.

**Текущий код:**
```rust
// crates/rustok-core/src/context.rs
pub struct AppContext {
    pub db: Arc<DatabaseConnection>,
    pub events: Arc<dyn EventTransport>,
    pub cache: Arc<dyn CacheBackend>,
    pub search: Arc<dyn SearchBackend>,
    pub scripting: Arc<ScriptingContext>,  // ✅ Существует
}
```

**Рекомендация:** Добавить hook в EventDispatcher:
```rust
pub struct EventDispatcher {
    bus: EventBus,
    handlers: Vec<Arc<dyn EventHandler>>,
    config: DispatcherConfig,
    scripting_hooks: Vec<ScriptingHook>,  // Добавить
}

// Позволит пользователям писать скрипты на Rhai для обработки событий
```

---

### 7. Отсутствие graceful shutdown

**Проблема:** Нет механизма корректного завершения работы.

**Рекомендация:**
```rust
// apps/server/src/app.rs
pub async fn shutdown(ctx: &AppContext) {
    // 1. Остановить прием новых запросов
    // 2. Дождаться завершения текущих транзакций
    // 3. Остановить EventDispatcher
    // 4. Завершить OutboxRelay
    // 5. Закрыть соединения с БД
}
```

---

## 🟢 Улучшения качества кода (P2)

### 8. Стандартизация DTO

**Проблема:** Нет единого подхода к DTO между модулями.

**Рекомендация:** Создать макрос или derive:
```rust
// crates/rustok-core/src/dto.rs
#[derive(Dto)]
#[dto(crate = "content")]
pub struct NodeDto {
    #[dto(required)]
    pub id: Uuid,
    pub title: Option<String>,
    #[dto(validate = "not_empty")]
    pub kind: String,
}
```

---

### 9. Улучшение ошибок

**Текущие ошибки:**
```rust
// crates/rustok-core/src/error.rs
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid ID format: {0}")]
    InvalidIdFormat(String),
    // ...
}
```

**Рекомендация:** Добавить коды ошибок для API:
```rust
#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid ID format: {0}")]
    #[error_code("INVALID_ID")]
    #[status_code(400)]
    InvalidIdFormat(String),
    // ...
}
```

---

### 10. Улучшение observability

**Рекомендация:** Добавить метрики в стиле Prometheus:
```rust
// crates/rustok-core/src/metrics.rs
lazy_static! {
    pub static ref EVENTS_PUBLISHED: Counter = register_counter!(
        "rustok_events_published_total",
        "Total events published"
    ).unwrap();
    
    pub static ref DB_QUERY_DURATION: Histogram = register_histogram!(
        "rustok_db_query_duration_seconds",
        "Database query duration"
    ).unwrap();
}
```

---

## 📊 Архитектурные паттерны (Что сохранить)

### ✅ Отличные решения:

1. **CQRS-lite с Event-Driven синхронизацией** — правильный баланс сложности
2. **Module Registry** — чистое разделение доменов
3. **TransactionalEventBus** — надежная доставка событий
4. **Tenant Resolution Middleware** — гибкая мульти-тенантность
5. **Wrapper Module Pattern** — Blog/Forum как надстройки над Content

### ⚠️ Требуют внимания:

1. **Event Schema Versioning** — хорошая основа, нужна валидация
2. **Cache Abstraction** — хороший trait, но реализация сложная
3. **RBAC Integration** — правильное направление, нужно завершить

---

## 🔧 Технический долг

| Приоритет | Задача | Оценка |
|-----------|--------|--------|
| P0 | Миграция commerce на TransactionalEventBus | 2-3 дня |
| P0 | Валидация событий | 1-2 дня |
| P1 | Упрощение кэширования тенантов | 3-5 дней |
| P1 | Graceful shutdown | 1-2 дня |
| P2 | Стандартизация DTO | 2-3 дня |
| P2 | Метрики Prometheus | 2-3 дня |

---

## 📚 Рекомендации по организации кода

### Структура crate (текущая — хорошая):
```
crates/rustok-[name]/
├── src/
│   ├── entities/       # ✅ SeaORM модели
│   ├── dto/            # ✅ Request/Response
│   ├── services/       # ✅ Бизнес-логика
│   ├── error.rs        # ✅ Ошибки модуля
│   └── lib.rs          # ✅ Регистрация модуля
```

### Что добавить:
```
crates/rustok-[name]/
├── src/
│   ├── validators/     # 🆕 Валидация входных данных
│   ├── policies/       # 🆕 RBAC политики
│   └── tests/
│       ├── unit/       # 🆕 Модульные тесты
│       └── integration/# 🆕 Интеграционные тесты
└── benches/            # 🆕 Бенчмарки
```

---

## 🎯 Рекомендации по тестированию

### Покрытие:
| Компонент | Целевое покрытие | Текущее |
|-----------|------------------|---------|
| rustok-core | 80% | ~31% |
| rustok-content | 70% | ? |
| rustok-commerce | 70% | ? |
| Event System | 90% | ? |

### Critical paths для тестирования:
1. Tenant resolution с кэшированием
2. Event publishing → Outbox → Delivery
3. Transaction rollback с событиями
4. RBAC permission checks

---

## 🚀 Долгосрочные рекомендации

### 1. Рассмотреть gRPC для межсервисного взаимодействия
При переходе к микросервисам, gRPC даст:
- Строгие контракты
- Бинарная сериализация
- Streaming

### 2. Внедрить OpenTelemetry
Для распределенной трассировки:
```rust
#[tracing::instrument(fields(tenant_id = %tenant_id))]
pub async fn create_node(...) -> Result<...> {
    // Автоматическое распространение trace_id
}
```

### 3. Рассмотреть Materialize или Flink
Для сложных CQRS read models при высокой нагрузке.

---

## ✅ Чеклист приемки

- [ ] Все сервисы используют TransactionalEventBus
- [ ] События валидируются при публикации
- [ ] Graceful shutdown работает корректно
- [ ] Метрики доступны в `/metrics`
- [ ] Код проходит `cargo audit`
- [ ] Тестовое покрытие > 70% для критических путей

---

**Заключение:** RusToK имеет отличную архитектурную основу. Основные усилия должны быть направлены на устранение несогласованностей между модулями и упрощение сложных компонентов.
