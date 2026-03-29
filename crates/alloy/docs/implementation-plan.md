# Implementation Plan: alloy

## Overview

`alloy` вЂ” РјРѕРґСѓР»СЊ СЃРєСЂРёРїС‚РѕРІРѕРіРѕ РґРІРёР¶РєР° РЅР° Р±Р°Р·Рµ Rhai, РїСЂРµРґРѕСЃС‚Р°РІР»СЏСЋС‰РёР№ РІРѕР·РјРѕР¶РЅРѕСЃС‚СЊ РЅР°РїРёСЃР°РЅРёСЏ РїРѕР»СЊР·РѕРІР°С‚РµР»СЊСЃРєРёС… СЃРєСЂРёРїС‚РѕРІ РґР»СЏ Р°РІС‚РѕРјР°С‚РёР·Р°С†РёРё Р±РёР·РЅРµСЃ-Р»РѕРіРёРєРё, РІР°Р»РёРґР°С†РёРё Рё РёРЅС‚РµРіСЂР°С†РёР№.

## Architecture

```
в”Њв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”ђ
в”‚                        alloy                          в”‚
в”њв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”¤
в”‚  API Layer (axum)                                               в”‚
в”‚  в”њв”Ђв”Ђ CRUD: create/read/update/delete scripts                   в”‚
в”‚  в”њв”Ђв”Ђ Execution: run scripts manually or by name                в”‚
в”‚  в””в”Ђв”Ђ Validation: validate script syntax                        в”‚
в”њв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”¤
в”‚  Runner Layer                                                   в”‚
в”‚  в”њв”Ђв”Ђ ScriptOrchestrator вЂ” РєРѕРѕСЂРґРёРЅР°С†РёСЏ РІС‹РїРѕР»РЅРµРЅРёСЏ               в”‚
в”‚  в”њв”Ђв”Ђ ScriptExecutor вЂ” РЅРёР·РєРѕСѓСЂРѕРІРЅРµРІРѕРµ РёСЃРїРѕР»РЅРµРЅРёРµ                в”‚
в”‚  в””в”Ђв”Ђ Scheduler вЂ” cron-based Р·Р°РїСѓСЃРє                              в”‚
в”њв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”¤
в”‚  Engine Layer                                                   в”‚
в”‚  в”њв”Ђв”Ђ ScriptEngine вЂ” РѕР±С‘СЂС‚РєР° РЅР°Рґ Rhai                           в”‚
в”‚  в”њв”Ђв”Ђ EngineConfig вЂ” Р»РёРјРёС‚С‹ Рё С‚Р°Р№РјР°СѓС‚С‹                          в”‚
в”‚  в””в”Ђв”Ђ Bridge вЂ” С„Р°Р·РѕР·Р°РІРёСЃРёРјС‹Рµ helper-С„СѓРЅРєС†РёРё                     в”‚
в”њв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”¤
в”‚  Storage Layer                                                  в”‚
в”‚  в”њв”Ђв”Ђ ScriptRegistry trait вЂ” РёРЅС‚РµСЂС„РµР№СЃ С…СЂР°РЅРµРЅРёСЏ                 в”‚
в”‚  в”њв”Ђв”Ђ InMemoryStorage вЂ” РґР»СЏ С‚РµСЃС‚РѕРІ                               в”‚
в”‚  в””в”Ђв”Ђ SeaOrmStorage вЂ” PostgreSQL                                 в”‚
в”њв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”¤
в”‚  Integration Layer                                              в”‚
в”‚  в”њв”Ђв”Ђ HookExecutor вЂ” РёРЅС‚РµРіСЂР°С†РёСЏ СЃ РґРѕРјРµРЅРЅС‹РјРё РјРѕРґСѓР»СЏРјРё            в”‚
в”‚  в””в”Ђв”Ђ ScriptableEntity вЂ” trait РґР»СЏ РєРѕРЅРІРµСЂС‚Р°С†РёРё СЃСѓС‰РЅРѕСЃС‚РµР№        в”‚
в””в”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”Ђв”
```

## Core Components

### 1. ScriptEngine (`engine/runtime.rs`)

Rhai engine wrapper СЃ:
- **Compilation caching** вЂ” AST РєСЌС€РёСЂСѓРµС‚СЃСЏ СЃ hash-based РёРЅРІР°Р»РёРґР°С†РёРµР№
- **Resource limits** вЂ” max_operations, max_call_depth, timeout
- **Custom types** вЂ” EntityProxy РґР»СЏ СЂР°Р±РѕС‚С‹ СЃ СЃСѓС‰РЅРѕСЃС‚СЏРјРё

### 2. ScriptOrchestrator (`runner/orchestrator.rs`)

РљРѕРѕСЂРґРёРЅРёСЂСѓРµС‚ РІС‹РїРѕР»РЅРµРЅРёРµ СЃРєСЂРёРїС‚РѕРІ:
- `run_before` вЂ” РґР»СЏ РІР°Р»РёРґР°С†РёРё Рё РјРѕРґРёС„РёРєР°С†РёРё РґР°РЅРЅС‹С… РґРѕ СЃРѕС…СЂР°РЅРµРЅРёСЏ
- `run_after` вЂ” РґР»СЏ side effects РїРѕСЃР»Рµ СЃРѕС…СЂР°РЅРµРЅРёСЏ
- `run_on_commit` вЂ” РґР»СЏ С„РёРЅР°Р»СЊРЅС‹С… РґРµР№СЃС‚РІРёР№ (notifications, webhooks)
- `run_manual` вЂ” СЂСѓС‡РЅРѕР№ Р·Р°РїСѓСЃРє С‡РµСЂРµР· API

### 3. EntityProxy (`model/proxy.rs`)

Proxy-РѕР±СЉРµРєС‚ РґР»СЏ РґРѕСЃС‚СѓРїР° Рє РґР°РЅРЅС‹Рј СЃСѓС‰РЅРѕСЃС‚Рё РІ СЃРєСЂРёРїС‚Р°С…:
- РћС‚СЃР»РµР¶РёРІР°РЅРёРµ РёР·РјРµРЅРµРЅРёР№ (changes tracking)
- Immutable РѕСЂРёРіРёРЅР°Р»СЊРЅС‹Рµ РґР°РЅРЅС‹Рµ
- РџРѕРґРґРµСЂР¶РєР° РёРЅРґРµРєСЃРЅРѕРіРѕ РґРѕСЃС‚СѓРїР°: `entity["field"]`

### 4. ScriptTrigger (`model/trigger.rs`)

РўРёРїС‹ С‚СЂРёРіРіРµСЂРѕРІ:
- **Event** вЂ” РїСЂРёРІСЏР·РєР° Рє СЃРѕР±С‹С‚РёСЏРј СЃСѓС‰РЅРѕСЃС‚Рё (before_create, after_update, etc.)
- **Cron** вЂ” scheduled РІС‹РїРѕР»РЅРµРЅРёРµ
- **Manual** вЂ” С‚РѕР»СЊРєРѕ СЂСѓС‡РЅРѕР№ Р·Р°РїСѓСЃРє
- **Api** вЂ” HTTP endpoint

### 5. Bridge (`bridge/mod.rs`)

Р РµРіРёСЃС‚СЂР°С†РёСЏ helper-С„СѓРЅРєС†РёР№ РІ Р·Р°РІРёСЃРёРјРѕСЃС‚Рё РѕС‚ С„Р°Р·С‹:
- **Before**: validation helpers (validate_email, validate_required, etc.)
- **After**: DB services (placeholder)
- **OnCommit**: external services (placeholder)
- **Manual/Scheduled**: РїРѕР»РЅС‹Р№ РЅР°Р±РѕСЂ

## Execution Flow

### Before Hook

```
Domain Service в†’ HookExecutor.run_before()
                     в†“
              Find scripts by (entity_type, BeforeCreate)
                     в†“
              For each script:
                     в†“
              ScriptExecutor.execute()
                     в†“
              Check outcome:
                - Success: apply changes to entity
                - Aborted: reject operation
                - Failed: log error, disable after 3 failures
                     в†“
              Return HookOutcome::Continue or Rejected
```

### After Hook

```
Domain Service в†’ HookExecutor.run_after()
                     в†“
              Find scripts by (entity_type, AfterCreate)
                     в†“
              Execute with entity_before context
                     в†“
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

- **3 consecutive errors** в†’ script auto-disabled
- **Manual reset** required to re-enable
- **Error logging** with timestamps

### Sandboxing

Rhai engine configured with:
- `strict_variables` вЂ” РЅРµС‚ РґРѕСЃС‚СѓРїР° Рє РЅРµРѕРїСЂРµРґРµР»С‘РЅРЅС‹Рј РїРµСЂРµРјРµРЅРЅС‹Рј
- `allow_shadowing` вЂ” СЂР°Р·СЂРµС€РµРЅРѕ РїРµСЂРµРѕРїСЂРµРґРµР»РµРЅРёРµ РїРµСЂРµРјРµРЅРЅС‹С…
- No filesystem access (default)
- HTTP access via `http_get` / `http_post` / `http_request` (OnCommit/Manual/Scheduled phases only)

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/scripts` | List scripts (paginated) |
| POST | `/scripts` | Create script |
| POST | `/scripts/validate` | Validate script syntax |
| GET | `/scripts/{id}` | Get script by ID |
| PUT | `/scripts/{id}` | Update script |
| DELETE | `/scripts/{id}` | Delete script |
| POST | `/scripts/{id}/run` | Execute script by ID |
| POST | `/scripts/name/{name}/run` | Execute script by name |

## Usage Example

### Creating a validation script

```rust
use alloy::*;

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
- Script lifecycle (create в†’ active в†’ disabled)

## Recent Improvements

### v1.3 (Current)

1. **Audit Log** вЂ” `script_executions` table + `SeaOrmExecutionLog` РґР»СЏ С…СЂР°РЅРµРЅРёСЏ РёСЃС‚РѕСЂРёРё РІС‹РїРѕР»РЅРµРЅРёР№; `run_script` GraphQL mutation Р»РѕРіРёСЂСѓРµС‚ СЂРµР·СѓР»СЊС‚Р°С‚ СЃ `user_id` Рё `tenant_id`
2. **HTTP Bridge** вЂ” `http_get(url)`, `http_post(url, body)`, `http_request(method, url, body, headers)` РґРѕСЃС‚СѓРїРЅС‹ РІ OnCommit/After/Manual/Scheduled С„Р°Р·Р°С…
3. **Tenant isolation** вЂ” `SeaOrmStorage::with_tenant(db, tenant_id)` Рё `.for_tenant(tenant_id)` РґР»СЏ СЃРѕР·РґР°РЅРёСЏ РёР·РѕР»РёСЂРѕРІР°РЅРЅРѕРіРѕ registry; РІСЃРµ queries С„РёР»СЊС‚СЂСѓСЋС‚ РїРѕ `tenant_id` РєРѕРіРґР° Р·Р°РґР°РЅ
4. **ExecutionResult.phase** вЂ” РґРѕР±Р°РІР»РµРЅРѕ РїРѕР»Рµ `phase: ExecutionPhase` РІ `ExecutionResult` РґР»СЏ С…СЂР°РЅРµРЅРёСЏ С„Р°Р·С‹ РІС‹РїРѕР»РЅРµРЅРёСЏ

### v1.2

1. **Observability** вЂ” `ScriptExecutor.execute()` wrapped in `tracing::info_span!` СЃ OTel-СЃРѕРІРјРµСЃС‚РёРјС‹РјРё span fields
2. **DB-level pagination** вЂ” `ScriptRegistry::find_paginated(query, offset, limit) -> ScriptPage` СЃ `COUNT` + `LIMIT/OFFSET`
3. **Improved log targets** вЂ” target `alloy::script` РґР»СЏ РІСЃРµС… script-generated logs
4. **MCP integration** вЂ” 9 MCP-РёРЅСЃС‚СЂСѓРјРµРЅС‚РѕРІ РґР»СЏ СѓРїСЂР°РІР»РµРЅРёСЏ СЃРєСЂРёРїС‚Р°РјРё С‡РµСЂРµР· AI (СЃРј. `crates/rustok-mcp`)
5. **Email validation** вЂ” RFC 5321-compliant РїСЂРѕРІРµСЂРєР°

## Future Improvements

### Phase 2 (Planned)

1. **Database Bridge** вЂ” controlled DB queries РёР· СЃРєСЂРёРїС‚РѕРІ
2. **Execution metrics** вЂ” СЃС‡С‘С‚С‡РёРєРё Рё РіРёСЃС‚РѕРіСЂР°РјРјС‹ РІС‹РїРѕР»РЅРµРЅРёР№ РїРѕ script_id/phase
3. **REST audit endpoint** вЂ” `GET /scripts/{id}/executions` РґР»СЏ РїСЂРѕСЃРјРѕС‚СЂР° РёСЃС‚РѕСЂРёРё

### Phase 3 (Future)

1. **Script versioning** вЂ” РёСЃС‚РѕСЂРёСЏ РёР·РјРµРЅРµРЅРёР№ СЃ rollback
2. **Script marketplace** вЂ” РіРѕС‚РѕРІС‹Рµ С€Р°Р±Р»РѕРЅС‹
3. **Debug mode** вЂ” РїРѕС€Р°РіРѕРІРѕРµ РІС‹РїРѕР»РЅРµРЅРёРµ
4. **Hot reload** вЂ” РѕР±РЅРѕРІР»РµРЅРёРµ Р±РµР· СЂРµСЃС‚Р°СЂС‚Р°

## Migration History

- **v1** вЂ” Initial implementation with basic CRUD and execution
- **v1.1** вЂ” Added cache invalidation, pagination, validation helpers, REST router, Scheduler startup, health check
- **v1.2** вЂ” DB-level pagination, OTel spans on executor, improved log targets, email validation, MCP tools
- **v1.3** вЂ” Audit log (`script_executions`), HTTP bridge, tenant isolation in SeaOrmStorage, ExecutionResult.phase

