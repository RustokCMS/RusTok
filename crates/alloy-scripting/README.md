# alloy-scripting

Скриптовый движок на базе [Rhai](https://rhai.rs/) для пользовательской автоматизации в RusToK.

## Назначение

`alloy-scripting` предоставляет возможность написания пользовательских скриптов для:
- **Валидации данных** — проверка условий перед сохранением
- **Модификации данных** — автоматическое изменение полей
- **Бизнес-логики** — custom rules и calculations
- **Интеграций** — webhooks, notifications (on_commit hooks)
- **Scheduled tasks** — cron-based автоматизация

## Features

- **Rhai scripting language** — безопасный, быстрый, Rust-like синтаксис
- **Resource limits** — защита от бесконечных циклов и DoS
- **Cache invalidation** — автоматическая инвалидация при изменении кода
- **Multi-trigger support** — events, cron, API endpoints, manual
- **EntityProxy** — удобный доступ к данным сущности с отслеживанием изменений
- **Phase-specific helpers** — validation, DB, external services
- **Auto-disable on errors** — скрипт отключается после 3 ошибок подряд

## Quick Start

```rust
use alloy_scripting::*;
use std::sync::Arc;

// Create engine and storage
let storage = Arc::new(InMemoryStorage::new());
let orchestrator = create_orchestrator(storage.clone());

// Define a validation script
let mut script = Script::new(
    "validate_order",
    r#"
        if entity["total"] < 100 {
            abort("Minimum order is $100");
        }
        if entity["total"] > 10000 {
            entity["requires_approval"] = true;
        }
    "#,
    ScriptTrigger::Event {
        entity_type: "order".into(),
        event: EventType::BeforeCreate,
    },
);
script.activate();
storage.save(script).await?;

// Execute in domain service
let order_data = HashMap::from([
    ("total".into(), 5000i64.into()),
]);
let entity = EntityProxy::new("order-1", "order", order_data);

match orchestrator.run_before("order", EventType::BeforeCreate, entity, None).await {
    HookOutcome::Continue { changes } => {
        println!("Proceeding with changes: {:?}", changes);
    }
    HookOutcome::Rejected { reason } => {
        println!("Validation failed: {}", reason);
    }
    HookOutcome::Error { error } => {
        eprintln!("Script error: {}", error);
    }
}
```

## Script Syntax

Скрипты используют [Rhai syntax](https://rhai.rs/book/):

```js
// Access entity fields
let amount = entity["amount"];
entity["status"] = "processed";

// Validation
if entity["email"] != "" && !validate_email(entity["email"]) {
    abort("Invalid email format");
}

// Helpers
log("Processing order: " + entity["id"]);
let now = now_timestamp();

// Conditional logic
if entity["priority"] == "high" {
    entity["assigned_to"] = "senior_manager";
}
```

## Available Functions

### Core Functions

| Function | Description |
|----------|-------------|
| `log(msg)` | Info-level logging |
| `log_warn(msg)` | Warning-level logging |
| `log_error(msg)` | Error-level logging |
| `abort(msg)` | Abort execution with message |
| `now()` | Current timestamp (ISO 8601) |
| `now_unix()` | Current Unix timestamp |

### Validation Helpers (Before phase)

| Function | Description |
|----------|-------------|
| `validate_email(email)` | Basic email validation |
| `validate_required(value)` | Check non-empty string |
| `validate_min_length(value, min)` | Minimum string length |
| `validate_max_length(value, max)` | Maximum string length |
| `validate_range(value, min, max)` | Numeric range check |

### Utility Functions

| Function | Description |
|----------|-------------|
| `format_money(amount)` | Format number with spaces |
| `is_empty(value)` | Check for empty/unit values |
| `coalesce(value, default)` | Return default if empty |

## EntityProxy API

```js
entity["field"]           // Get field value
entity["field"] = value   // Set field value
entity.id                 // Entity ID
entity.type               // Entity type
is_changed(entity, "field")  // Check if field was modified
has_changes(entity)       // Check any modifications
snapshot(entity)          // Get current state as map
```

## Triggers

### Event Trigger

```rust
ScriptTrigger::Event {
    entity_type: "order".into(),
    event: EventType::BeforeCreate, // or AfterCreate, BeforeUpdate, etc.
}
```

### Cron Trigger

```rust
ScriptTrigger::Cron {
    expression: "0 0 * * * *".into(), // Every hour
}
```

### API Trigger

```rust
ScriptTrigger::Api {
    path: "/scripts/cleanup".into(),
    method: HttpMethod::POST,
}
```

### Manual Trigger

```rust
ScriptTrigger::Manual // Only executable via API call
```

## Resource Limits

Default limits (configurable):

| Limit | Default | Description |
|-------|---------|-------------|
| `max_operations` | 50,000 | Max AST operations |
| `timeout` | 100ms | Execution timeout (warning) |
| `max_call_depth` | 16 | Max nested calls |
| `max_string_size` | 64KB | Max string length |
| `max_array_size` | 10,000 | Max array elements |

## Configuration

```rust
// Strict mode (faster, less memory)
let config = EngineConfig::strict();

// Relaxed mode (for complex scripts)
let config = EngineConfig::relaxed();

// Custom configuration
let config = EngineConfig {
    max_operations: 100_000,
    timeout: Duration::from_millis(500),
    ..Default::default()
};
let engine = create_engine_with_config(config);
```

## API Reference

See [implementation-plan.md](./docs/implementation-plan.md) for detailed architecture.

## Integration with Domain Modules

```rust
use alloy_scripting::{HookExecutor, ScriptableEntity};

// Implement ScriptableEntity for your entity
impl ScriptableEntity for Order {
    fn entity_type(&self) -> &'static str { "order" }
    fn id(&self) -> String { self.id.clone() }
    fn to_dynamic_map(&self) -> HashMap<String, Dynamic> {
        // Convert to map
    }
    fn apply_changes(&mut self, changes: HashMap<String, Dynamic>) {
        // Apply script modifications
    }
}

// Use in service
impl OrderService {
    async fn create(&self, mut order: Order) -> Result<Order, Error> {
        let proxy = order.to_entity_proxy();
        
        match self.hook_executor.run_before(
            "order", EventType::BeforeCreate, &order.id, proxy.into(), None
        ).await? {
            BeforeHookResult::Continue(changes) => {
                order.apply_changes(changes);
            }
            BeforeHookResult::Rejected(reason) => {
                return Err(Error::Validation(reason));
            }
        }
        
        // Save to DB...
        
        self.hook_executor.run_after(/* ... */).await;
        self.hook_executor.run_on_commit(/* ... */).await;
        
        Ok(order)
    }
}
```

## Documentation

- [Alloy Concept](../../docs/alloy-concept.md) — стратегическое видение Alloy: Self-Evolving Integration Runtime
- [Implementation Plan](./docs/implementation-plan.md) — архитектура и детали реализации
- [Rhai Book](https://rhai.rs/book/) — документация по языку Rhai

## Status

✅ Core engine with Rhai  
✅ Event triggers (before/after/on_commit)  
✅ Cron scheduler  
✅ API endpoints  
✅ Cache invalidation  
✅ Pagination  
✅ Validation helpers  

🚧 Audit log (planned)  
🚧 Prometheus metrics (planned)  
🚧 HTTP bridge (planned)  
