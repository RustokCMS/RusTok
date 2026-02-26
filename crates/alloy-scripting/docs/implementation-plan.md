# Implementation Plan: alloy-scripting

## Overview

`alloy-scripting` — модуль скриптового движка на базе Rhai, предоставляющий возможность написания пользовательских скриптов для автоматизации бизнес-логики, валидации и интеграций.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        alloy-scripting                          │
├─────────────────────────────────────────────────────────────────┤
│  API Layer (axum)                                               │
│  ├── CRUD: create/read/update/delete scripts                   │
│  ├── Execution: run scripts manually or by name                │
│  └── Validation: validate script syntax                        │
├─────────────────────────────────────────────────────────────────┤
│  Runner Layer                                                   │
│  ├── ScriptOrchestrator — координация выполнения               │
│  ├── ScriptExecutor — низкоуровневое исполнение                │
│  └── Scheduler — cron-based запуск                              │
├─────────────────────────────────────────────────────────────────┤
│  Engine Layer                                                   │
│  ├── ScriptEngine — обёртка над Rhai                           │
│  ├── EngineConfig — лимиты и таймауты                          │
│  └── Bridge — фазозависимые helper-функции                     │
├─────────────────────────────────────────────────────────────────┤
│  Storage Layer                                                  │
│  ├── ScriptRegistry trait — интерфейс хранения                 │
│  ├── InMemoryStorage — для тестов                               │
│  └── SeaOrmStorage — PostgreSQL                                 │
├─────────────────────────────────────────────────────────────────┤
│  Integration Layer                                              │
│  ├── HookExecutor — интеграция с доменными модулями            │
│  └── ScriptableEntity — trait для конвертации сущностей        │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. ScriptEngine (`engine/runtime.rs`)

Rhai engine wrapper с:
- **Compilation caching** — AST кэшируется с hash-based инвалидацией
- **Resource limits** — max_operations, max_call_depth, timeout
- **Custom types** — EntityProxy для работы с сущностями

### 2. ScriptOrchestrator (`runner/orchestrator.rs`)

Координирует выполнение скриптов:
- `run_before` — для валидации и модификации данных до сохранения
- `run_after` — для side effects после сохранения
- `run_on_commit` — для финальных действий (notifications, webhooks)
- `run_manual` — ручной запуск через API

### 3. EntityProxy (`model/proxy.rs`)

Proxy-объект для доступа к данным сущности в скриптах:
- Отслеживание изменений (changes tracking)
- Immutable оригинальные данные
- Поддержка индексного доступа: `entity["field"]`

### 4. ScriptTrigger (`model/trigger.rs`)

Типы триггеров:
- **Event** — привязка к событиям сущности (before_create, after_update, etc.)
- **Cron** — scheduled выполнение
- **Manual** — только ручной запуск
- **Api** — HTTP endpoint

### 5. Bridge (`bridge/mod.rs`)

Регистрация helper-функций в зависимости от фазы:
- **Before**: validation helpers (validate_email, validate_required, etc.)
- **After**: DB services (placeholder)
- **OnCommit**: external services (placeholder)
- **Manual/Scheduled**: полный набор

## Execution Flow

### Before Hook

```
Domain Service → HookExecutor.run_before()
                     ↓
              Find scripts by (entity_type, BeforeCreate)
                     ↓
              For each script:
                     ↓
              ScriptExecutor.execute()
                     ↓
              Check outcome:
                - Success: apply changes to entity
                - Aborted: reject operation
                - Failed: log error, disable after 3 failures
                     ↓
              Return HookOutcome::Continue or Rejected
```

### After Hook

```
Domain Service → HookExecutor.run_after()
                     ↓
              Find scripts by (entity_type, AfterCreate)
                     ↓
              Execute with entity_before context
                     ↓
              Return HookOutcome
```

## Security Model

### Resource Limits

```rust
EngineConfig {
    max_operations: 50_000,    // Maximum AST operations
    timeout: 100ms,            // Execution timeout (warning only)
    max_call_depth: 16,        // Maximum function call depth
    max_string_size: 64KB,     // Maximum string length
    max_array_size: 10_000,    // Maximum array elements
}
```

### Error Handling

- **3 consecutive errors** → script auto-disabled
- **Manual reset** required to re-enable
- **Error logging** with timestamps

### Sandboxing

Rhai engine configured with:
- `strict_variables` — нет доступа к неопределённым переменным
- `allow_shadowing` — разрешено переопределение переменных
- No filesystem access (default)
- HTTP access via `http_get` / `http_post` / `http_request` (OnCommit/Manual/Scheduled phases only)

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/scripts` | List scripts (paginated) |
| POST | `/scripts` | Create script |
| POST | `/scripts/validate` | Validate script syntax |
| GET | `/scripts/:id` | Get script by ID |
| PUT | `/scripts/:id` | Update script |
| DELETE | `/scripts/:id` | Delete script |
| POST | `/scripts/:id/run` | Execute script by ID |
| POST | `/scripts/name/:name/run` | Execute script by name |

## Usage Example

### Creating a validation script

```rust
use alloy_scripting::*;

let engine = create_default_engine();
let storage = Arc::new(InMemoryStorage::new());
let orchestrator = create_orchestrator(storage.clone());

// Create validation script
let mut script = Script::new(
    "validate_deal",
    r#"
        if entity["amount"] < 100 {
            abort("Minimum deal amount is 100");
        }
        if entity["amount"] > 100000 {
            entity["status"] = "needs_approval";
        }
    "#,
    ScriptTrigger::Event {
        entity_type: "deal".into(),
        event: EventType::BeforeCreate,
    },
);
script.activate();
storage.save(script).await?;

// Execute before hook
let deal_data = HashMap::from([
    ("amount".into(), 50000i64.into()),
]);
let entity = EntityProxy::new("1", "deal", deal_data);

match orchestrator.run_before("deal", EventType::BeforeCreate, entity, None).await {
    HookOutcome::Continue { changes } => {
        // Apply changes and proceed
    }
    HookOutcome::Rejected { reason } => {
        // Validation failed
    }
    HookOutcome::Error { error } => {
        // Script error
    }
}
```

## Testing Strategy

### Unit Tests

- Script compilation and execution
- Error handling (abort, timeout, limits)
- EntityProxy changes tracking
- Cache invalidation

### Integration Tests

- Full hook execution flow
- Storage operations (InMemoryStorage)
- Script lifecycle (create → active → disabled)

## Recent Improvements

### v1.3 (Current)

1. **Audit Log** — `script_executions` table + `SeaOrmExecutionLog` для хранения истории выполнений; `run_script` GraphQL mutation логирует результат с `user_id` и `tenant_id`
2. **HTTP Bridge** — `http_get(url)`, `http_post(url, body)`, `http_request(method, url, body, headers)` доступны в OnCommit/After/Manual/Scheduled фазах
3. **Tenant isolation** — `SeaOrmStorage::with_tenant(db, tenant_id)` и `.for_tenant(tenant_id)` для создания изолированного registry; все queries фильтруют по `tenant_id` когда задан
4. **ExecutionResult.phase** — добавлено поле `phase: ExecutionPhase` в `ExecutionResult` для хранения фазы выполнения

### v1.2

1. **Observability** — `ScriptExecutor.execute()` wrapped in `tracing::info_span!` с OTel-совместимыми span fields
2. **DB-level pagination** — `ScriptRegistry::find_paginated(query, offset, limit) -> ScriptPage` с `COUNT` + `LIMIT/OFFSET`
3. **Improved log targets** — target `alloy::script` для всех script-generated logs
4. **MCP integration** — 9 MCP-инструментов для управления скриптами через AI (см. `crates/rustok-mcp`)
5. **Email validation** — RFC 5321-compliant проверка

## Future Improvements

### Phase 2 (Planned)

1. **Database Bridge** — controlled DB queries из скриптов
2. **Execution metrics** — счётчики и гистограммы выполнений по script_id/phase
3. **REST audit endpoint** — `GET /scripts/:id/executions` для просмотра истории

### Phase 3 (Future)

1. **Script versioning** — история изменений с rollback
2. **Script marketplace** — готовые шаблоны
3. **Debug mode** — пошаговое выполнение
4. **Hot reload** — обновление без рестарта

## Migration History

- **v1** — Initial implementation with basic CRUD and execution
- **v1.1** — Added cache invalidation, pagination, validation helpers, REST router, Scheduler startup, health check
- **v1.2** — DB-level pagination, OTel spans on executor, improved log targets, email validation, MCP tools
- **v1.3** — Audit log (`script_executions`), HTTP bridge, tenant isolation in SeaOrmStorage, ExecutionResult.phase
